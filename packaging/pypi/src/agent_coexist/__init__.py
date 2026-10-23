"""Launcher for the released agent-coexist executable.

The distribution carries no binary. On first use it downloads the archive the
Rust release workflow published for this same version, checks it against that
release's SHA256SUMS, caches the executable under the user cache directory,
and executes it. Arguments, streams and the exit code pass straight through,
because agent-coexist's exit codes carry meaning (0 ok or warn, 1 error,
2 usage, 3 lock refusal, 130 cancelled).

Nothing runs at install time, so `uvx agent-coexist` and `pipx run
agent-coexist` work without a build step or an install hook. The cache lives
outside the environment, so an ephemeral `uvx` run downloads once rather than
once per invocation.
"""

from __future__ import annotations

import hashlib
import os
import platform
import stat
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path

__version__ = "0.5.0"

REPO = "RatmmmhSquishyRat/agent-coexist"
BASE = f"https://github.com/{REPO}/releases/download/v{__version__}"

# Release archive naming, keyed by (system, machine). Linux uses the static
# musl builds, which run on glibc systems too.
TARGETS = {
    ("darwin", "arm64"): "agent-coexist-aarch64-apple-darwin.tar.gz",
    ("darwin", "x86_64"): "agent-coexist-x86_64-apple-darwin.tar.gz",
    ("linux", "arm64"): "agent-coexist-aarch64-unknown-linux-musl.tar.gz",
    ("linux", "x86_64"): "agent-coexist-x86_64-unknown-linux-musl.tar.gz",
    ("windows", "arm64"): "agent-coexist-aarch64-pc-windows-msvc.zip",
    ("windows", "x86_64"): "agent-coexist-x86_64-pc-windows-msvc.zip",
}

# platform.machine() spellings that mean the same architecture.
MACHINE_ALIASES = {
    "amd64": "x86_64",
    "x64": "x86_64",
    "x86_64": "x86_64",
    "aarch64": "arm64",
    "arm64": "arm64",
}

EXE = "agent-coexist.exe" if sys.platform == "win32" else "agent-coexist"


class LauncherError(Exception):
    """A failure the launcher expects and reports as one CLI error line:
    an unpublished platform, a failed download, a checksum mismatch, or a
    release archive that does not hold the executable. Anything else is a
    bug and is left to propagate with its traceback."""


class UnsupportedPlatform(LauncherError):
    """No binary is published for the running platform."""


def host() -> tuple[str, str]:
    """The (system, machine) pair used to pick a release archive."""
    system = platform.system().lower()
    raw = platform.machine().lower()
    return system, MACHINE_ALIASES.get(raw, raw)


def archive_name() -> str:
    """The release archive for the running platform."""
    key = host()
    name = TARGETS.get(key)
    if name is None:
        supported = ", ".join(f"{s}-{m}" for s, m in sorted(TARGETS))
        raise UnsupportedPlatform(
            f"agent-coexist has no published binary for {key[0]}-{key[1]}.\n"
            f"Published targets: {supported}.\n"
            f"Build it from source instead: cargo install agent-coexist --locked"
        )
    return name


def cache_dir() -> Path:
    """Where the executable is kept between runs, versioned so that an
    upgrade never reuses an older binary."""
    if sys.platform == "win32":
        base = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local"))
    elif sys.platform == "darwin":
        base = Path.home() / "Library" / "Caches"
    else:
        base = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    return base / "agent-coexist" / __version__


def binary_path() -> Path:
    return cache_dir() / EXE


def get(url: str) -> bytes:
    with urllib.request.urlopen(url) as response:  # noqa: S310 - fixed https host
        return response.read()


def digest_for(sums: bytes, name: str) -> str:
    """The expected digest of `name` from this release's SHA256SUMS."""
    for line in sums.decode("utf-8").splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[1].lstrip("*") == name:
            return fields[0]
    raise LauncherError(f"SHA256SUMS for v{__version__} has no entry for {name}")


def extract(archive: Path, name: str, entry: str) -> bytes:
    """The bytes of the one named entry, from either archive shape. Only that
    entry is read; the LICENSE and README beside it in the archive are left
    alone. A release that does not hold the executable is a launcher error,
    not a traceback."""
    try:
        if name.endswith(".zip"):
            with zipfile.ZipFile(archive) as zf:
                return zf.read(entry)
        with tarfile.open(archive, "r:gz") as tf:
            member = tf.extractfile(entry)
            if member is None:
                raise LauncherError(f"{name} does not contain '{entry}'")
            return member.read()
    except KeyError:
        raise LauncherError(f"{name} does not contain '{entry}'") from None


def download() -> Path:
    """Download the archive, verify it against the release's SHA256SUMS, and
    cache the executable. Returns its path."""
    name = archive_name()
    print(f"agent-coexist: fetching {name} (v{__version__})", file=sys.stderr)

    blob = get(f"{BASE}/{name}")
    expected = digest_for(get(f"{BASE}/SHA256SUMS"), name)
    actual = hashlib.sha256(blob).hexdigest()
    if actual != expected:
        raise LauncherError(
            f"checksum mismatch for {name}\n"
            f"  expected {expected}\n"
            f"  actual   {actual}\n"
            f"The download does not match the release's SHA256SUMS. "
            f"Nothing was installed."
        )

    target = cache_dir()
    target.mkdir(parents=True, exist_ok=True)

    with tempfile.NamedTemporaryFile(dir=target, delete=False) as holding:
        holding.write(blob)
        packed = Path(holding.name)
    try:
        payload = extract(packed, name, EXE)
    finally:
        packed.unlink()

    # Write to a unique name and replace, so concurrent first runs cannot
    # observe a partially written executable.
    staged = target / f".{EXE}.{os.getpid()}"
    staged.write_bytes(payload)
    staged.chmod(staged.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    final = binary_path()
    os.replace(staged, final)
    return final


def ensure_binary() -> Path:
    """The executable's path, downloading it if this is the first run."""
    bin_path = binary_path()
    return bin_path if bin_path.exists() else download()


def main() -> int:
    """Console entry point: run the executable with this process's arguments."""
    try:
        bin_path = ensure_binary()
    except (LauncherError, OSError) as error:
        # OSError covers urllib's URLError and the filesystem; a bug raises
        # something else and keeps its traceback.
        print(f"agent-coexist: {error}", file=sys.stderr)
        return 1
    return subprocess.run([str(bin_path), *sys.argv[1:]], check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
