#!/usr/bin/env python3
"""Package, authenticate, and smoke-test the six native release artifacts.

Run from the product root:
  release_artifacts.py pack TARGET VERSION
  release_artifacts.py verify DIR VERSION SOURCE_SHA SOURCE_REF [--write-checksums]
  release_artifacts.py smoke ARCHIVE TARGET VERSION

Archives contain only the executable, LICENSE, and README.md. Packaging streams
files with normalized metadata; SOURCE_DATE_EPOCH supplies the timestamp. Verify
authenticates each archive before unpacking, then authenticates its binary with
the same repository/workflow/source policy. Native runner selection belongs to
the workflow, not this helper. This module never builds or issues attestations.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import os
import shutil
import stat
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile
import zlib
from dataclasses import dataclass
from pathlib import Path

BIN = "agent-coexist"
SUMS_NAME = "SHA256SUMS"


@dataclass(frozen=True)
class Target:
    ext: str
    binary: str
    arch: str
    kind: str
    musl: bool = False


TARGETS: dict[str, Target] = {
    "x86_64-unknown-linux-musl": Target("tar.gz", BIN, "x86_64", "elf", True),
    "aarch64-unknown-linux-musl": Target("tar.gz", BIN, "aarch64", "elf", True),
    "x86_64-apple-darwin": Target("tar.gz", BIN, "x86_64", "macho"),
    "aarch64-apple-darwin": Target("tar.gz", BIN, "aarch64", "macho"),
    "x86_64-pc-windows-msvc": Target("zip", f"{BIN}.exe", "x86_64", "pe"),
    "aarch64-pc-windows-msvc": Target("zip", f"{BIN}.exe", "aarch64", "pe"),
}

__all__ = ["ArtifactError", "TARGETS", "pack", "verify", "smoke"]


class ArtifactError(Exception):
    """A release artifact failed a required check."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def inspect_binary(path: Path, target: str) -> None:
    """Read only executable headers; reject the wrong target or dynamic musl."""
    spec = TARGETS[target]
    with path.open("rb") as binary:
        header = binary.read(64)
        if len(header) < 64:
            raise ArtifactError(f"{path}: truncated executable header")
        if spec.kind == "elf":
            if header[:6] != b"\x7fELF\x02\x01":
                raise ArtifactError(f"{path}: expected little-endian ELF64 for {target}")
            file_type, machine = struct.unpack_from("<HH", header, 16)
            expected = 62 if spec.arch == "x86_64" else 183
            if file_type not in (2, 3) or machine != expected:
                raise ArtifactError(
                    f"{path}: wrong ELF type/machine {file_type}/{machine} for {target}"
                )
            offset = struct.unpack_from("<Q", header, 32)[0]
            entry_size, count = struct.unpack_from("<HH", header, 54)
            size = os.fstat(binary.fileno()).st_size
            if (offset < 64 or entry_size < 56 or count in (0, 0xFFFF)
                    or offset + entry_size * count > size):
                raise ArtifactError(f"{path}: invalid ELF64 program header table")
            for index in range(count):
                binary.seek(offset + index * entry_size)
                if spec.musl and binary.read(4) == b"\x03\x00\x00\x00":  # PT_INTERP
                    raise ArtifactError(f"{path}: musl binary has PT_INTERP; static linking is required")
        elif spec.kind == "macho":
            endian = {b"\xcf\xfa\xed\xfe": "<", b"\xfe\xed\xfa\xcf": ">"}.get(header[:4])
            if endian is None:
                raise ArtifactError(f"{path}: expected thin 64-bit Mach-O for {target}")
            machine, _, file_type = struct.unpack_from(endian + "III", header, 4)
            expected = 0x01000007 if spec.arch == "x86_64" else 0x0100000C
            if machine != expected or file_type != 2:  # MH_EXECUTE
                raise ArtifactError(
                    f"{path}: wrong Mach-O type/CPU {file_type}/0x{machine:x} for {target}"
                )
        else:
            if header[:2] != b"MZ":
                raise ArtifactError(f"{path}: expected PE executable for {target}")
            offset = struct.unpack_from("<I", header, 0x3C)[0]
            binary.seek(offset)
            pe = binary.read(26)
            if offset < 64 or len(pe) != 26 or pe[:4] != b"PE\x00\x00":
                raise ArtifactError(f"{path}: missing or truncated PE header")
            machine = struct.unpack_from("<H", pe, 4)[0]
            magic = struct.unpack_from("<H", pe, 24)[0]
            expected = 0x8664 if spec.arch == "x86_64" else 0xAA64
            if machine != expected or magic != 0x20B:  # PE32+
                raise ArtifactError(f"{path}: wrong PE machine/format 0x{machine:x}/0x{magic:x} for {target}")


def extract_and_validate(archive: Path, target: str, destination: Path) -> Path:
    """Stream only allowlisted regular members into an isolated directory."""
    spec = TARGETS[target]
    modes = {spec.binary: 0o755, "LICENSE": 0o644, "README.md": 0o644}
    seen: set[str] = set()
    destination.mkdir(parents=True, exist_ok=True)
    is_zip = spec.ext == "zip"
    try:
        with (zipfile.ZipFile(archive) if is_zip else tarfile.open(archive, "r|gz")) as bundle:
            entries = bundle.infolist() if is_zip else bundle
            for entry in entries:
                if is_zip:
                    name, size = entry.filename, entry.file_size
                    mode = entry.external_attr >> 16
                    regular = (stat.S_ISREG(mode) and not entry.is_dir()
                               and entry.orig_filename == name and not entry.flag_bits & 1)
                else:
                    name, size, mode = entry.name, entry.size, entry.mode
                    regular = entry.isreg()
                # Exact names reject traversal, separators, absolute paths, and aliases.
                if name not in modes or name in seen:
                    raise ArtifactError(f"{archive.name}: unexpected or duplicate member {name!r}")
                if not regular or size <= 0 or stat.S_IMODE(mode) != modes[name]:
                    raise ArtifactError(
                        f"{archive.name}: {name!r} must be a nonempty regular file with mode {modes[name]:04o}"
                    )
                output = destination / name
                source = bundle.open(entry) if is_zip else bundle.extractfile(entry)
                with source, output.open("xb") as sink:
                    shutil.copyfileobj(source, sink)
                    if sink.tell() != size:
                        raise ArtifactError(f"{archive.name}: truncated member {name!r}")
                output.chmod(modes[name])
                seen.add(name)
    except (OSError, EOFError, tarfile.TarError, zipfile.BadZipFile, zlib.error, NotImplementedError) as exc:
        raise ArtifactError(f"{archive}: cannot unpack archive: {exc}") from exc
    if seen != modes.keys():
        raise ArtifactError(f"{archive.name}: missing members {sorted(modes.keys() - seen)}")
    binary = destination / spec.binary
    inspect_binary(binary, target)
    return binary


def write_archive(path: Path, sources: list[tuple[Path, int]], epoch: int) -> None:
    """Write stable member order, timestamps, owners, and modes without buffering files."""
    partial = path.with_name(path.name + ".partial")
    if path.suffix == ".zip":
        stamp = time.gmtime(max(epoch, 315532800))[:6]  # ZIP timestamps start in 1980.
        with zipfile.ZipFile(partial, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
            for source, mode in sources:
                info = zipfile.ZipInfo(source.name, date_time=stamp)
                info.create_system = 3  # Store POSIX type/mode, even on Windows.
                info.external_attr = (stat.S_IFREG | mode) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                info.file_size = source.stat().st_size
                with source.open("rb") as data, bundle.open(info, "w") as member:
                    shutil.copyfileobj(data, member)
    else:
        with partial.open("wb") as raw:
            with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=epoch, compresslevel=9) as compressed:
                with tarfile.open(fileobj=compressed, mode="w|", format=tarfile.USTAR_FORMAT) as bundle:
                    for source, mode in sources:
                        info = tarfile.TarInfo(source.name)  # Regular, uid/gid 0, no owner names.
                        info.size = source.stat().st_size
                        info.mode, info.mtime = mode, epoch
                        with source.open("rb") as data:
                            bundle.addfile(info, data)
    partial.replace(path)


def run_command(
    argv: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None,
    expected: str | None = None, exact: bool = False, timeout: int = 60,
) -> str:
    """One error path for command failures, timeouts, and unexpected CLI output."""
    try:
        result = subprocess.run(
            argv, cwd=cwd, env=env, capture_output=True, text=True,
            encoding="utf-8", errors="replace", timeout=timeout,
        )
        stdout, stderr = result.stdout, result.stderr
        failure = f"exit {result.returncode}" if result.returncode else None
        if failure is None and expected is not None:
            matches = stdout == expected + "\n" if exact else stdout.partition("\n")[0] == expected
            if not matches:
                failure = f"expected {'exact stdout' if exact else 'first stdout line'} {expected!r}"
    except subprocess.TimeoutExpired as exc:
        failure = f"timed out after {timeout}s"
        stdout, stderr = exc.stdout or b"", exc.stderr or b""
    except OSError as exc:
        raise ArtifactError(f"cannot run {argv!r}: {exc}") from exc
    if failure:
        if isinstance(stdout, bytes):
            stdout = stdout.decode("utf-8", "replace")
        if isinstance(stderr, bytes):
            stderr = stderr.decode("utf-8", "replace")
        raise ArtifactError(f"{argv!r}: {failure}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    return stdout


def run_extracted_cli(binary: Path, version: str) -> None:
    with tempfile.TemporaryDirectory(prefix="ac-smoke-") as scratch:
        work = Path(scratch)
        (work / "docs").mkdir()
        env = dict(os.environ, AGENT_COEXIST_NOW="2026-09-17T10:00:00Z")
        commands = [
            (["--version"], f"{BIN} {version}"),
            (["post", "docs/standup", "--as", "alice", "-m", "release artifact smoke"], "ok  post  docs/standup"),
            (["check", "docs/standup"], "ok  check  docs/standup"),
            (["list", "."], "ok  list  ."),
        ]
        for args, expected in commands:
            run_command([str(binary.resolve()), *args], cwd=work, env=env,
                        expected=expected, exact=args == ["--version"])


def pack(target: str, version: str) -> tuple[Path, Path]:
    spec = TARGETS[target]
    try:
        epoch = int(os.environ.get("SOURCE_DATE_EPOCH", ""))
    except ValueError as exc:
        raise ArtifactError("pack: SOURCE_DATE_EPOCH is required as an integer timestamp") from exc
    if not 0 <= epoch <= 0xFFFFFFFF:
        raise ArtifactError("pack: SOURCE_DATE_EPOCH must fit an unsigned 32-bit gzip timestamp")
    staged = Path("dist") / "staging" / target / spec.binary
    staged.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(Path("target") / target / "release" / spec.binary, staged)
    staged.chmod(0o755)
    inspect_binary(staged, target)
    archive = Path("dist") / f"{BIN}-{target}.{spec.ext}"
    write_archive(archive, [(staged, 0o755), (Path("LICENSE"), 0o644), (Path("README.md"), 0o644)], epoch)
    with tempfile.TemporaryDirectory(prefix="ac-pack-") as scratch:
        binary = extract_and_validate(archive, target, Path(scratch))
        if sha256_file(binary) != sha256_file(staged):
            raise ArtifactError("pack: extracted binary differs from the staged attestation subject")
        run_extracted_cli(binary, version)
    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with open(output, "a", encoding="utf-8") as handle:
            for key, path in (("binary", staged), ("archive", archive)):
                value = str(path.resolve())
                if "\n" in value or "\r" in value:
                    raise ArtifactError(f"pack: cannot emit a path containing a newline: {value!r}")
                handle.write(f"{key}={value}\n")
    return staged, archive


def verify(
    directory: str | Path, version: str, source_sha: str, source_ref: str,
    *, write_checksums: bool = False,
) -> Path:
    """Authenticate a complete cohort; VERSION is exercised by native smoke jobs."""
    root = Path(directory)
    archives = {f"{BIN}-{target}.{spec.ext}": target for target, spec in TARGETS.items()}
    entries = {path.name: path for path in root.iterdir()}
    missing = archives.keys() - entries.keys()
    extra = entries.keys() - (archives.keys() | {SUMS_NAME})
    if missing or extra:
        raise ArtifactError(f"{root}: invalid release cohort; missing={sorted(missing)}, unexpected={sorted(extra)}")
    for path in entries.values():
        if not stat.S_ISREG(path.lstat().st_mode):
            raise ArtifactError(f"{path}: cohort entries must be regular files, not links or directories")
    repo = os.environ.get("GITHUB_REPOSITORY", "")
    # Empty gh filters would remove source restrictions, so never pass them through.
    if not repo or not source_sha or not source_ref:
        raise ArtifactError("verify: GITHUB_REPOSITORY, SOURCE_SHA, and SOURCE_REF are required")
    digests = {name: sha256_file(root / name) for name in archives}
    sums = root / SUMS_NAME
    if not write_checksums:
        if not sums.is_file():
            raise ArtifactError(f"{sums}: missing checksum manifest (new cohorts require --write-checksums)")
        recorded: dict[str, str] = {}
        for line in sums.read_text(encoding="ascii").splitlines():
            if len(line) < 67 or line[64:66] != "  ":
                raise ArtifactError(f"{sums}: expected '<sha256>  <filename>', got {line!r}")
            digest, name = line[:64].lower(), line[66:]
            if name not in digests or name in recorded:
                raise ArtifactError(f"{sums}: unexpected or duplicate entry {name!r}")
            recorded[name] = digest
        if recorded != digests:
            mismatches = [name for name in digests if recorded.get(name) != digests[name]]
            raise ArtifactError(f"{sums}: missing entries or SHA-256 mismatch for {mismatches}")
    policy = [
        "--repo", repo,
        "--signer-workflow", f"{repo}/.github/workflows/release.yml",
        "--source-digest", source_sha,
        "--source-ref", source_ref,
        "--signer-digest", source_sha,
        "--deny-self-hosted-runners",
    ]
    with tempfile.TemporaryDirectory(prefix="ac-verify-") as scratch:
        for name, target in archives.items():
            archive = root / name
            run_command(["gh", "attestation", "verify", str(archive), *policy], timeout=120)
            binary = extract_and_validate(archive, target, Path(scratch) / target)
            run_command(["gh", "attestation", "verify", str(binary), *policy], timeout=120)
    if write_checksums:
        partial = sums.with_name(sums.name + ".partial")
        partial.write_text("".join(f"{digest}  {name}\n" for name, digest in digests.items()),
                           encoding="ascii", newline="\n")
        partial.replace(sums)
    return sums


def smoke(archive: str | Path, target: str, version: str) -> None:
    with tempfile.TemporaryDirectory(prefix="ac-smoke-archive-") as scratch:
        binary = extract_and_validate(Path(archive), target, Path(scratch))
        run_extracted_cli(binary, version)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    pack_parser = commands.add_parser("pack", help="package and smoke one native target")
    pack_parser.add_argument("target", choices=TARGETS)
    pack_parser.add_argument("version")
    verify_parser = commands.add_parser("verify", help="authenticate a six-archive cohort")
    verify_parser.add_argument("directory")
    verify_parser.add_argument("version")
    verify_parser.add_argument("source_sha")
    verify_parser.add_argument("source_ref")
    verify_parser.add_argument("--write-checksums", action="store_true")
    smoke_parser = commands.add_parser("smoke", help="extract and exercise one native archive")
    smoke_parser.add_argument("archive")
    smoke_parser.add_argument("target", choices=TARGETS)
    smoke_parser.add_argument("version")
    args = parser.parse_args(argv)
    try:
        if args.command == "pack":
            pack(args.target, args.version)
        elif args.command == "verify":
            verify(args.directory, args.version, args.source_sha, args.source_ref,
                   write_checksums=args.write_checksums)
        else:
            smoke(args.archive, args.target, args.version)
    except (ArtifactError, OSError, UnicodeError) as exc:
        print(f"{args.command}: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
