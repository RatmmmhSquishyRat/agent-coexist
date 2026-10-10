#!/usr/bin/env python3
"""crates.io registry publication helper for the release workflow.

Subcommands (run from the clean tagged checkout; cargo on PATH):
  check --name N --version V     emit `published=true|false` (sparse index)
  publish --name N --version V   `cargo publish --locked` with the token from
                                 CARGO_REGISTRY_TOKEN, then wait for index
                                 visibility (idempotent: skips if published)
  verify --name N --version V    download the registry crate, verify its
                                 index checksum, compare its complete
                                 unpacked contents with the locally packaged
                                 tagged source, install that exact version
                                 into an isolated prefix, and exercise the
                                 binary (--version + real CLI operations)

Python 3 standard library only; temp work happens outside the source tree so
`cargo package`/`cargo publish` never need --allow-dirty. HTTP 404 from the
index means "never published"; other HTTP or network failures are hard
errors. Tokens are never printed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tarfile
import tempfile
import urllib.error
import urllib.request
from pathlib import Path

INDEX_URL = "https://index.crates.io"
STATIC_URL = "https://static.crates.io"
USER_AGENT = "agent-coexist-release"


def die(message: str) -> None:
    first = message.splitlines()[0] if message else "registry helper failure"
    print(f"::error::{first[:900]}", file=sys.stderr)
    print(message, file=sys.stderr)
    sys.exit(1)


def set_output(key: str, value: str) -> None:
    path = os.environ.get("GITHUB_OUTPUT")
    if path:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{key}={value}\n")


def http_get(url: str) -> tuple[int, bytes]:
    request = urllib.request.Request(
        url, headers={"User-Agent": USER_AGENT, "Accept": "*/*"}
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()
    except urllib.error.URLError as error:
        die(f"network failure calling {url}: {error.reason}")


def index_records(name: str) -> list[dict] | None:
    """Sparse-index records for the crate, or None when it does not exist."""
    lower = name.lower()
    if len(lower) == 1:
        segment = f"1/{lower}"
    elif len(lower) == 2:
        segment = f"2/{lower}"
    elif len(lower) == 3:
        segment = f"3/{lower[0]}/{lower}"
    else:
        segment = f"{lower[:2]}/{lower[2:4]}/{lower}"
    status, body = http_get(f"{INDEX_URL}/{segment}")
    if status == 404:
        return None
    if status >= 400:
        die(f"sparse index lookup for {name} failed with HTTP {status}: {body[:300]!r}")
    records = []
    for line in body.decode("utf-8", "replace").splitlines():
        line = line.strip()
        if line:
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                die(f"sparse index returned a malformed record for {name}")
    return records


def find_record(name: str, version: str) -> dict | None:
    records = index_records(name)
    if records is None:
        return None
    for record in records:
        if record.get("vers") == version:
            return record
    return None


def check_manifest(name: str, version: str) -> None:
    import tomllib

    try:
        with open("Cargo.toml", "rb") as handle:
            package = tomllib.load(handle)["package"]
    except (OSError, KeyError) as error:
        die(f"cannot read Cargo.toml package section: {error}")
    if package.get("name") != name or package.get("version") != version:
        die(
            f"checkout Cargo.toml is {package.get('name')} {package.get('version')}, "
            f"expected {name} {version}"
        )


def run_live(command: list[str], cwd: str | Path | None = None) -> None:
    result = subprocess.run(command, cwd=None if cwd is None else str(cwd))
    if result.returncode != 0:
        die(f"`{' '.join(command)}` failed with exit code {result.returncode}")


def run_capture(command: list[str], cwd: str | Path | None = None) -> str:
    result = subprocess.run(
        command, cwd=None if cwd is None else str(cwd), capture_output=True, text=True
    )
    if result.returncode != 0:
        die(
            f"`{' '.join(command)}` failed with exit code {result.returncode}:\n"
            f"{result.stdout}\n{result.stderr}"
        )
    return result.stdout


def first_line(text: str) -> str:
    lines = text.splitlines()
    return lines[0] if lines else ""


def cmd_check(args) -> None:
    records = index_records(args.name)
    first_publish = records is None
    published = any(record.get("vers") == args.version for record in records or [])
    set_output("published", "true" if published else "false")
    set_output("first_publish", "true" if first_publish else "false")
    print(
        f"{args.name} {args.version}: "
        + ("present in the sparse index" if published else "not yet published")
        + ("; crate does not exist yet (first publication)" if first_publish else "")
    )


def cmd_publish(args) -> None:
    check_manifest(args.name, args.version)
    if not os.environ.get("CARGO_REGISTRY_TOKEN"):
        die(
            "CARGO_REGISTRY_TOKEN is not set. The first publication of a "
            "crate REQUIRES the API token: crates.io trusted publishing only "
            "authenticates crates that already exist "
            "(https://crates.io/docs/trusted-publishing). Configure the "
            "repository secret CARGO_REGISTRY_TOKEN (crates.io API token with "
            "publish-new/publish-update); later versions may alternatively "
            "use trusted publishing for RatmmmhSquishyRat/agent-coexist with "
            "workflow .github/workflows/release.yml and environment release. "
            "Then re-run this job."
        )
    if find_record(args.name, args.version) is not None:
        print(f"{args.name} {args.version} is already visible in the index; nothing to publish")
        return
    # Cargo >=1.66 blocks until the new version is visible in the index;
    # the verify step fails closed if it still is not.
    run_live(["cargo", "publish", "--locked"])
    print(f"`cargo publish --locked` completed for {args.name} {args.version}")


def crate_url(name: str, version: str) -> str:
    return f"{STATIC_URL}/crates/{name}/{name}-{version}.crate"


def unpack_crate(path: Path, dest: Path, name: str, version: str) -> Path:
    dest.mkdir(parents=True, exist_ok=True)
    with tarfile.open(path, "r:gz") as archive:
        archive.extractall(dest, filter="data")
    root = dest / f"{name}-{version}"
    if not root.is_dir():
        die(f"{path} does not contain the expected top-level {name}-{version}/ directory")
    return root


def tree_files(root: Path) -> dict[str, Path]:
    files = {}
    for dirpath, _, filenames in os.walk(root):
        for filename in filenames:
            path = Path(dirpath) / filename
            files[path.relative_to(root).as_posix()] = path
    return files


def compare_packages(remote_root: Path, local_root: Path) -> None:
    remote, local = tree_files(remote_root), tree_files(local_root)
    only_remote = sorted(set(remote) - set(local))
    only_local = sorted(set(local) - set(remote))
    if only_remote or only_local:
        die(
            "registry package file set differs from the tagged package:\n"
            f"  only in registry: {only_remote[:10]}\n"
            f"  only in tag:      {only_local[:10]}"
        )
    differing = []
    for relative in sorted(remote):
        if remote[relative].read_bytes() != local[relative].read_bytes():
            differing.append(relative)
    if differing:
        die(
            "registry package content differs from the tagged package for: "
            f"{', '.join(differing[:10])} (compared {len(remote)} files)"
        )


def smoke_installed_binary(binary: Path, name: str, version: str, workdir: Path) -> None:
    workdir.mkdir(parents=True, exist_ok=True)
    output = run_capture([str(binary), "--version"])
    if output.strip() != f"{name} {version}":
        die(
            f"registry-installed --version printed {output.strip()!r}, "
            f"expected {name!s} {version}"
        )
    output = run_capture(
        [str(binary), "post", "docs/standup", "--as", "alice", "-m", "registry install verification"],
        cwd=workdir,
    )
    if first_line(output) != "ok  post  docs/standup":
        die(f"registry-installed post printed {first_line(output)!r}, expected 'ok  post  docs/standup'")
    output = run_capture([str(binary), "list", "."], cwd=workdir)
    if first_line(output) != "ok  list  .":
        die(f"registry-installed list printed {first_line(output)!r}, expected 'ok  list  .'")


def cmd_verify(args) -> None:
    check_manifest(args.name, args.version)
    record = find_record(args.name, args.version)
    if record is None:
        die(f"{args.name} {args.version} is not visible in the sparse index; cannot verify")
    expected_checksum = record.get("cksum")
    if not expected_checksum:
        die(f"sparse index record for {args.name} {args.version} has no cksum")

    status, blob = http_get(crate_url(args.name, args.version))
    if status != 200:
        die(f"downloading {crate_url(args.name, args.version)} failed with HTTP {status}")
    digest = hashlib.sha256(blob).hexdigest()
    if digest != expected_checksum:
        die(
            "registry crate checksum mismatch: sparse index "
            f"{expected_checksum} vs downloaded {digest}"
        )

    # Temp directories live outside the source tree; cargo steps below run
    # against the pristine checkout without --allow-dirty.
    with tempfile.TemporaryDirectory(prefix="registry-verify-") as tmp:
        root = Path(tmp)
        remote_crate = root / "registry.crate"
        remote_crate.write_bytes(blob)
        run_live(["cargo", "package", "--locked", "--no-verify"])
        local_crate = Path("target/package") / f"{args.name}-{args.version}.crate"
        if not local_crate.is_file():
            die(f"`cargo package` did not produce {local_crate}")
        remote_root = unpack_crate(remote_crate, root / "remote", args.name, args.version)
        local_root = unpack_crate(local_crate, root / "local", args.name, args.version)
        compare_packages(remote_root, local_root)

        prefix = root / "prefix"
        install_dir = root / "install"
        install_dir.mkdir()
        run_live(
            ["cargo", "install", args.name, "--version", args.version, "--locked", "--root", str(prefix)],
            cwd=install_dir,
        )
        binary = prefix / "bin" / args.name
        if not binary.is_file():
            die(f"cargo install did not place the binary at {binary}")
        smoke_installed_binary(binary, args.name, args.version, root / "work")

    print(
        f"registry verification OK: {args.name} {args.version} matches the "
        "tagged package byte-for-byte and the exact installed version "
        "passes the CLI smoke"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    check = commands.add_parser("check")
    check.add_argument("--name", required=True)
    check.add_argument("--version", required=True)
    check.set_defaults(func=cmd_check)

    publish = commands.add_parser("publish")
    publish.add_argument("--name", required=True)
    publish.add_argument("--version", required=True)
    publish.set_defaults(func=cmd_publish)

    verify = commands.add_parser("verify")
    verify.add_argument("--name", required=True)
    verify.add_argument("--version", required=True)
    verify.set_defaults(func=cmd_verify)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
