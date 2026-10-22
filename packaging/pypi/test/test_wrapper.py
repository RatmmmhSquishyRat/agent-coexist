#!/usr/bin/env python3
"""Tests for the Python wrapper.

The offline group checks target resolution, SHA256SUMS parsing and archive
extraction against fixtures built here. The online group, skipped with
AGENT_COEXIST_SKIP_NETWORK=1, performs the real download the wrapper would do
and runs the resulting executable.

Run with: python test/test_wrapper.py
"""

from __future__ import annotations

import io
import os
import re
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

import agent_coexist as ac  # noqa: E402

PASSED = 0
SKIPPED = 0


def test(name):
    def wrap(body):
        global PASSED
        body()
        PASSED += 1
        print(f"ok   {name}")
        return body

    return wrap


def skip(name, why):
    global SKIPPED
    SKIPPED += 1
    print(f"skip {name} ({why})")


def raises(kind, pattern, call):
    try:
        call()
    except kind as error:
        assert re.search(pattern, str(error), re.S), f"{error!r} does not match {pattern!r}"
        return
    raise AssertionError(f"expected {kind.__name__} matching {pattern!r}")


# ---------------------------------------------------------------- fixtures


def make_tar_gz(path: Path, name: str, body: bytes) -> None:
    with tarfile.open(path, "w:gz") as tf:
        info = tarfile.TarInfo(name)
        info.size = len(body)
        info.mode = 0o755
        tf.addfile(info, io.BytesIO(body))


def make_zip(path: Path, name: str, body: bytes) -> None:
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.writestr(name, body)


# ------------------------------------------------------------------ offline


@test("package version matches the crate version")
def _():
    manifest = (Path(__file__).resolve().parents[3] / "Cargo.toml").read_text("utf-8")
    crate = re.search(r'^version = "([^"]+)"', manifest, re.M).group(1)
    assert ac.__version__ == crate, (
        f"wrapper {ac.__version__} downloads its own release; crate is {crate}"
    )


@test("pyproject version matches the module version")
def _():
    pyproject = (Path(__file__).resolve().parents[1] / "pyproject.toml").read_text("utf-8")
    declared = re.search(r'^version = "([^"]+)"', pyproject, re.M).group(1)
    assert declared == ac.__version__


@test("every published release target is mapped")
def _():
    assert sorted(ac.TARGETS) == [
        ("darwin", "arm64"),
        ("darwin", "x86_64"),
        ("linux", "arm64"),
        ("linux", "x86_64"),
        ("windows", "arm64"),
        ("windows", "x86_64"),
    ]
    for archive in ac.TARGETS.values():
        assert re.match(r"^agent-coexist-.+\.(tar\.gz|zip)$", archive), archive


@test("machine aliases collapse to the two mapped architectures")
def _():
    assert set(ac.MACHINE_ALIASES.values()) == {"x86_64", "arm64"}
    for spelling in ("AMD64", "x86_64", "aarch64", "arm64"):
        assert ac.MACHINE_ALIASES[spelling.lower()] in {"x86_64", "arm64"}


@test("this platform resolves to a release archive")
def _():
    assert ac.archive_name().startswith("agent-coexist-")


@test("the cache directory is versioned")
def _():
    assert ac.cache_dir().name == ac.__version__


@test("SHA256SUMS parsing accepts the published binary-mode form")
def _():
    sums = (
        b"aaaa  agent-coexist-x86_64-apple-darwin.tar.gz\n"
        b"bbbb *agent-coexist-x86_64-pc-windows-msvc.zip\n"
    )
    assert ac.digest_for(sums, "agent-coexist-x86_64-apple-darwin.tar.gz") == "aaaa"
    assert ac.digest_for(sums, "agent-coexist-x86_64-pc-windows-msvc.zip") == "bbbb"
    raises(ac.LauncherError, "no entry for absent", lambda: ac.digest_for(sums, "absent.tar.gz"))


@test("tar.gz extraction returns the named entry byte for byte")
def _():
    body = os.urandom(5000)
    with tempfile.TemporaryDirectory() as work:
        archive = Path(work) / "a.tar.gz"
        make_tar_gz(archive, "agent-coexist", body)
        assert ac.extract(archive, "a.tar.gz", "agent-coexist") == body
        raises(
            ac.LauncherError,
            r"a\.tar\.gz does not contain 'absent'",
            lambda: ac.extract(archive, "a.tar.gz", "absent"),
        )


@test("zip extraction returns the named entry byte for byte")
def _():
    body = os.urandom(5000)
    with tempfile.TemporaryDirectory() as work:
        archive = Path(work) / "a.zip"
        make_zip(archive, "agent-coexist.exe", body)
        assert ac.extract(archive, "a.zip", "agent-coexist.exe") == body
        raises(
            ac.LauncherError,
            r"a\.zip does not contain 'absent'",
            lambda: ac.extract(archive, "a.zip", "absent"),
        )


@test("an unmapped platform names the supported set and the source build")
def _():
    original = ac.host
    ac.host = lambda: ("sunos", "sparc")
    try:
        raises(
            ac.UnsupportedPlatform,
            r"no published binary for sunos-sparc.*cargo install agent-coexist --locked",
            ac.archive_name,
        )
    finally:
        ac.host = original


@test("UnsupportedPlatform is a LauncherError, so main reports it as one line")
def _():
    assert issubclass(ac.UnsupportedPlatform, ac.LauncherError)


# ------------------------------------------------------------------- online

if os.environ.get("AGENT_COEXIST_SKIP_NETWORK") == "1":
    skip("real release download, verification and execution", "AGENT_COEXIST_SKIP_NETWORK=1")
else:
    env = {**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parents[1] / "src")}

    @test("the real release downloads, verifies and runs")
    def _():
        cached = ac.binary_path()
        cached.unlink(missing_ok=True)

        run = subprocess.run(
            [sys.executable, "-m", "agent_coexist", "--version"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            env=env,
            check=True,
        )
        assert cached.exists(), "the first run must leave the executable in the cache"
        assert run.stdout.strip() == f"agent-coexist {ac.__version__}", run.stdout

    @test("a second run reuses the cached executable without downloading")
    def _():
        run = subprocess.run(
            [sys.executable, "-m", "agent_coexist", "--version"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            env=env,
            check=True,
        )
        assert run.stderr == "", f"a cached run must print no fetch notice: {run.stderr!r}"

    @test("exit codes and streams pass through unchanged")
    def _():
        usage = subprocess.run(
            [sys.executable, "-m", "agent_coexist", "not-a-command"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            env=env,
        )
        assert usage.returncode == 2, usage.returncode
        assert usage.stdout == "", usage.stdout
        assert "error:" in usage.stderr

        guide = subprocess.run(
            [sys.executable, "-m", "agent_coexist", "guide"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            env=env,
        )
        assert guide.returncode == 0
        assert "agent-coexist" in guide.stdout

    @test("arguments reach the executable and produce a real record")
    def _():
        with tempfile.TemporaryDirectory() as work:
            run = subprocess.run(
                [
                    sys.executable, "-m", "agent_coexist",
                    "profile", "alice", "--section", "role", "-m", "Wrapper check.",
                ],
                capture_output=True,
                text=True,
                encoding="utf-8",
                cwd=work,
                env=env,
            )
            assert run.returncode == 0, run.stderr
            assert re.match(r"^ok {2}profile {2}alice", run.stdout), run.stdout

            written = Path(work) / "alice.agent-coexist.md"
            assert written.exists(), "the record must exist on disk"
            assert "# alice.agent-profile" in written.read_text("utf-8")


print(f"\n{PASSED} passed, {SKIPPED} skipped")
