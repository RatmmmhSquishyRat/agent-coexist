#!/usr/bin/env python3
"""GitHub Release state helper for the release workflow.

Subcommands (run from the repository root with GH_TOKEN in the environment):
  download --tag vX.Y.Z | --release-id ID [--tag vX.Y.Z] --dir DIST
                                  download the complete seven-asset cohort
                                  (drafts need --release-id; the by-tag
                                  endpoint is published-only)
  download-public --tag vX.Y.Z --dir DIST
                                  same, but fully unauthenticated through
                                  the public browser_download_urls
  stage --tag vX.Y.Z --version V --sha SHA --dir DIST
                                  create or resume the draft release
                                  (draft-aware discovery); never overwrite
                                  or delete existing assets; emits
                                  release_id for downstream jobs
  finalize --release-id ID --tag vX.Y.Z
                                  publish the draft; an already-published
                                  release is verified only, never edited

Python 3 standard library only. HTTP 404 means "absent" (and is
distinguished from 401/403 authentication and other failures); every other
non-2xx status and every network error fails the run with an actionable
message.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request

# Release matrix; keep in sync with release.yml and release_artifacts.py.
TARGETS = [
    ("x86_64-unknown-linux-musl", "tar.gz"),
    ("aarch64-unknown-linux-musl", "tar.gz"),
    ("x86_64-apple-darwin", "tar.gz"),
    ("aarch64-apple-darwin", "tar.gz"),
    ("x86_64-pc-windows-msvc", "zip"),
    ("aarch64-pc-windows-msvc", "zip"),
]
ARCHIVE_NAMES = [f"agent-coexist-{target}.{ext}" for target, ext in TARGETS]
# SHA256SUMS is uploaded last: its presence marks a complete cohort.
COHORT_NAMES = ARCHIVE_NAMES + ["SHA256SUMS"]

TAG_RE = re.compile(r"^v\d+\.\d+\.\d+$")
VERSION_RE = re.compile(r"^\d+\.\d+\.\d+$")
USER_AGENT = "agent-coexist-release"


def die(message: str) -> None:
    first = message.splitlines()[0] if message else "release helper failure"
    print(f"::error::{first[:900]}", file=sys.stderr)
    print(message, file=sys.stderr)
    sys.exit(1)


def set_output(key: str, value: str) -> None:
    path = os.environ.get("GITHUB_OUTPUT")
    if path:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{key}={value}\n")


def heading_line_regex(version: str) -> re.Pattern[str]:
    return re.compile(rf"^## \[{re.escape(version)}\] - \d{{4}}-\d{{2}}-\d{{2}}\s*$")


def require_tag(tag: str) -> str:
    if not TAG_RE.match(tag):
        die(f"tag {tag!r} is not a strict vX.Y.Z tag")
    return tag


def require_version(version: str) -> str:
    if not VERSION_RE.match(version):
        die(f"version {version!r} is not strict X.Y.Z")
    return version


def _token() -> str:
    token = os.environ.get("GH_TOKEN", "")
    if not token:
        die("GH_TOKEN is not set; the workflow must pass github.token")
    return token


def _base() -> str:
    return os.environ.get("GITHUB_API_URL", "https://api.github.com").rstrip("/")


def _repo() -> str:
    repo = os.environ.get("GITHUB_REPOSITORY", "")
    if not repo:
        die("GITHUB_REPOSITORY is not set")
    return repo


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    """Signed asset URLs must be fetched without the Authorization header."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


_OPENER = urllib.request.build_opener(_NoRedirect)


def _request(url, method="GET", data=None, accept="application/vnd.github+json", token=None):
    headers = {"User-Agent": USER_AGENT, "Accept": accept}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    if data is not None:
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with _OPENER.open(request, timeout=120) as response:
            return response.status, response.read(), response.headers
    except urllib.error.HTTPError as error:
        return error.code, error.read(), error.headers
    except urllib.error.URLError as error:
        die(f"network failure calling {url}: {error.reason}")


def release_by_tag(tag: str):
    """Published-only tag lookup (the by-tag endpoint never returns drafts).

    Returns the release JSON, or None when no published release has the tag.
    Used by the read-only guard/reuse discovery; draft-aware callers use
    find_release_any_state or release_by_id.
    """
    path = f"/repos/{_repo()}/releases/tags/{urllib.parse.quote(tag)}"
    status, body, _ = _request(_base() + path, token=_token())
    if status == 404:
        return None
    if status in (401, 403):
        die(f"release lookup for {tag} failed with HTTP {status} (authentication/permission)")
    if status >= 400:
        snippet = body[:400].decode("utf-8", "replace")
        die(f"GitHub API GET {path} failed with HTTP {status}: {snippet}")
    return json.loads(body)


def _release_list_page(page: int) -> list:
    path = f"/repos/{_repo()}/releases?per_page=100&page={page}"
    status, body, _ = _request(_base() + path, token=_token())
    if status in (401, 403):
        die(
            f"listing releases failed with HTTP {status} (authentication/"
            "permission); seeing drafts requires contents:write"
        )
    if status == 404:
        die(f"listing releases for {_repo()} failed with HTTP 404 (repository missing?)")
    if status >= 400:
        snippet = body[:400].decode("utf-8", "replace")
        die(f"GitHub API GET {path} failed with HTTP {status}: {snippet}")
    return json.loads(body)


def find_release_any_state(tag: str):
    """Draft-aware tag lookup for write-scoped callers.

    GET /releases/tags/{tag} is published-only, so page through the
    releases list (the same fallback the GitHub CLI uses) and match
    tag_name; drafts are visible there to tokens with write access.
    """
    page = 1
    while True:
        releases = _release_list_page(page)
        for release in releases:
            if release.get("tag_name") == tag:
                return release
        if len(releases) < 100:
            return None
        page += 1


def release_by_id(release_id: str):
    """Release lookup by id — unlike the by-tag endpoint, this sees drafts."""
    path = f"/repos/{_repo()}/releases/{urllib.parse.quote(str(release_id))}"
    status, body, _ = _request(_base() + path, token=_token())
    if status == 404:
        return None
    if status in (401, 403):
        die(f"release lookup by id {release_id} failed with HTTP {status} (authentication/permission)")
    if status >= 400:
        snippet = body[:400].decode("utf-8", "replace")
        die(f"GitHub API GET {path} failed with HTTP {status}: {snippet}")
    return json.loads(body)


def api_json(method: str, path: str, payload=None):
    """Call the GitHub REST API; any non-2xx status is a hard failure."""
    data = json.dumps(payload).encode() if payload is not None else None
    status, body, _ = _request(_base() + path, method=method, data=data, token=_token())
    if status >= 400:
        snippet = body[:400].decode("utf-8", "replace")
        die(f"GitHub API {method} {path} failed with HTTP {status}: {snippet}")
    return json.loads(body) if body else None


def fetch_asset(asset: dict) -> bytes:
    status, body, headers = _request(
        asset["url"], accept="application/octet-stream", token=_token()
    )
    if status in (301, 302, 303, 307, 308):
        location = headers.get("Location")
        if not location:
            die(f"asset {asset['name']} redirected without a Location header")
        status, body, _ = _request(location, accept="application/octet-stream")
    if status >= 400:
        die(f"downloading release asset {asset['name']} failed with HTTP {status}")
    return body


def file_sha256(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_cohort(dist: str) -> dict[str, str]:
    if not os.path.isdir(dist):
        die(f"cohort directory {dist} does not exist")
    names = set(os.listdir(dist))
    missing = [name for name in COHORT_NAMES if name not in names]
    extra = sorted(names - set(COHORT_NAMES))
    if missing:
        die(f"assembled cohort in {dist} is missing: {', '.join(missing)}")
    if extra:
        die(f"assembled cohort in {dist} has unexpected files: {', '.join(extra)}")
    return {name: os.path.join(dist, name) for name in COHORT_NAMES}


def forbid_extra_assets(tag: str, assets: dict) -> None:
    extra = sorted(set(assets) - set(COHORT_NAMES))
    if extra:
        die(
            f"release {tag} carries unexpected assets: {', '.join(extra)}.\n"
            f"Expected exactly: {', '.join(COHORT_NAMES)}.\n"
            "Staging never deletes or overwrites release assets; manual inspection required."
        )


def changelog_section(version: str, path: str = "CHANGELOG.md") -> str:
    heading = heading_line_regex(version)
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except OSError as error:
        die(f"cannot read {path}: {error}")
    body: list[str] = []
    capturing = False
    for line in text.splitlines():
        if capturing:
            if line.startswith("## "):
                break
            body.append(line)
        elif heading.match(line):
            capturing = True
    if not capturing:
        die(f"{path} has no dated heading '## [{version}] - YYYY-MM-DD'")
    while body and not body[-1].strip():
        body.pop()
    if not body:
        die(f"{path} section for {version} is empty")
    return "\n".join(body) + "\n"


def cmd_download(args) -> None:
    if not args.tag and not args.release_id:
        die("download requires --tag (published releases) or --release-id (drafts too)")
    if args.release_id:
        release = release_by_id(args.release_id)
        if release is None:
            die(f"release id {args.release_id} not found (HTTP 404)")
        if args.tag:
            tag = require_tag(args.tag)
            if release.get("tag_name") != tag:
                die(
                    f"release id {args.release_id} has tag "
                    f"{release.get('tag_name')!r}, expected {tag}"
                )
        else:
            tag = release.get("tag_name", "?")
    else:
        tag = require_tag(args.tag)
        release = release_by_tag(tag)
        if release is None:
            die(
                f"published release {tag} not found (HTTP 404). The by-tag "
                "endpoint does not see drafts; pass --release-id for those."
            )
    assets = {asset["name"]: asset for asset in release.get("assets", [])}
    missing = [name for name in COHORT_NAMES if name not in assets]
    if missing:
        die(f"release {tag} is missing expected assets: {', '.join(missing)}")
    forbid_extra_assets(tag, assets)
    os.makedirs(args.dir, exist_ok=True)
    for name in COHORT_NAMES:
        blob = fetch_asset(assets[name])
        with open(os.path.join(args.dir, name), "wb") as handle:
            handle.write(blob)
        print(f"downloaded {name} ({len(blob)} bytes)")


def cmd_download_public(args) -> None:
    """Download the published release exactly as an anonymous user would.

    Reads no token at all: the release lookup and every asset download are
    unauthenticated, redirect-following GETs of public URLs.
    """
    tag = require_tag(args.tag)
    path = f"/repos/{_repo()}/releases/tags/{urllib.parse.quote(tag)}"
    request = urllib.request.Request(
        _base() + path, headers={"User-Agent": USER_AGENT, "Accept": "application/vnd.github+json"}
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            release = json.loads(response.read())
    except urllib.error.HTTPError as error:
        if error.code == 404:
            die(
                f"release {tag} is not publicly readable (HTTP 404): it may "
                "be missing, still a draft, or the repository is private"
            )
        die(f"public release lookup for {tag} failed with HTTP {error.code}")
    except urllib.error.URLError as error:
        die(f"network failure calling {path}: {error.reason}")
    assets = {asset["name"]: asset for asset in release.get("assets", [])}
    missing = [name for name in COHORT_NAMES if name not in assets]
    if missing:
        die(f"public release {tag} is missing expected assets: {', '.join(missing)}")
    forbid_extra_assets(tag, assets)
    os.makedirs(args.dir, exist_ok=True)
    for name in COHORT_NAMES:
        url = assets[name]["browser_download_url"]
        request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
        try:
            with urllib.request.urlopen(request, timeout=300) as response:
                blob = response.read()
        except urllib.error.HTTPError as error:
            die(f"public download of {name} failed with HTTP {error.code}")
        except urllib.error.URLError as error:
            die(f"network failure downloading {name}: {error.reason}")
        with open(os.path.join(args.dir, name), "wb") as handle:
            handle.write(blob)
        print(f"publicly downloaded {name} ({len(blob)} bytes)")


def upload_asset(release: dict, name: str, path: str) -> None:
    base = release["upload_url"].split("{", 1)[0]
    url = f"{base}?name={urllib.parse.quote(name)}"
    with open(path, "rb") as handle:
        blob = handle.read()
    request = urllib.request.Request(
        url,
        data=blob,
        method="POST",
        headers={
            "User-Agent": USER_AGENT,
            "Authorization": f"Bearer {_token()}",
            "Content-Type": "application/octet-stream",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=300) as response:
            status, body = response.status, response.read()
    except urllib.error.HTTPError as error:
        status, body = error.code, error.read()
    except urllib.error.URLError as error:
        die(f"upload of {name} failed: {error.reason}")
    if status >= 400:
        snippet = body[:400].decode("utf-8", "replace")
        die(f"upload of {name} failed with HTTP {status}: {snippet}")
    print(f"uploaded {name} ({len(blob)} bytes)")


def cmd_stage(args) -> None:
    tag = require_tag(args.tag)
    version = require_version(args.version)
    local = load_cohort(args.dir)

    # The tag must already exist: creating a release for a missing tag would
    # let GitHub auto-create the tag, and this pipeline never moves or
    # invents tags.
    probe = subprocess.run(
        ["git", "rev-parse", "--verify", "-q", f"refs/tags/{tag}"],
        capture_output=True,
    )
    if probe.returncode != 0:
        die(
            f"refs/tags/{tag} does not exist in this checkout; refusing to "
            "create a release that would auto-create the tag"
        )

    # Draft-aware discovery: the by-tag endpoint never returns drafts, so an
    # interrupted-staging retry must find the existing draft here.
    release = find_release_any_state(tag)
    if release is None:
        release = api_json(
            "POST",
            f"/repos/{_repo()}/releases",
            {
                "tag_name": tag,
                "target_commitish": args.sha,
                "name": tag,
                "body": changelog_section(version),
                "draft": True,
                "prerelease": False,
            },
        )
        print(f"created draft release {tag}: {release.get('html_url')}")
    assets = {asset["name"]: asset for asset in release.get("assets", [])}
    forbid_extra_assets(tag, assets)

    uploaded, reused = [], []
    for name in COHORT_NAMES:
        if name in assets:
            remote_digest = hashlib.sha256(fetch_asset(assets[name])).hexdigest()
            local_digest = file_sha256(local[name])
            if remote_digest != local_digest:
                die(
                    f"refusing to touch release asset {name} on {tag}: the "
                    "already-uploaded bytes differ from this run's cohort\n"
                    f"  remote sha256: {remote_digest}\n"
                    f"  local  sha256: {local_digest}\n"
                    "Release assets are never overwritten and replacement "
                    "provenance is never invented.\n"
                    "To resume an interrupted staging, re-run the ORIGINAL "
                    f"failed staging job (Actions -> the run that first staged {tag} "
                    "-> Re-run failed jobs); it re-downloads its own "
                    "byte-identical cohort and uploads only the missing files.\n"
                    "Otherwise an owner must delete the incomplete draft "
                    "release and start a fresh dispatch."
                )
            reused.append(name)
            continue
        if not release.get("draft"):
            die(
                f"release {tag} is published and lacks asset {name}; published "
                "releases are verified, never edited or re-uploaded — manual "
                "inspection required"
            )
        upload_asset(release, name, local[name])
        uploaded.append(name)

    set_output("release_id", str(release["id"]))
    if uploaded or reused:
        parts = [f"staged {tag}: uploaded {len(uploaded)} asset(s)"]
        if reused:
            parts.append(f"verified and kept {len(reused)} already-present asset(s)")
        print("; ".join(parts))
    else:
        print(f"staged {tag}: complete cohort already present, nothing to do")


def cmd_finalize(args) -> None:
    tag = require_tag(args.tag)
    release = release_by_id(args.release_id)
    if release is None:
        die(f"release id {args.release_id} not found (HTTP 404); staging should have created it")
    if release.get("tag_name") != tag:
        die(
            f"release id {args.release_id} has tag {release.get('tag_name')!r}, "
            f"expected {tag}"
        )
    if release.get("draft"):
        api_json("PATCH", f"/repos/{_repo()}/releases/{release['id']}", {"draft": False})
        print(f"published release {tag}")
    else:
        print(f"release {tag} is already published; verified, not edited")
    confirm = release_by_id(args.release_id)
    if confirm is None or confirm.get("draft"):
        die(f"release {tag} is still a draft after publishing")
    print(confirm["html_url"])


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    download = commands.add_parser("download")
    download.add_argument("--tag")
    download.add_argument("--release-id", dest="release_id")
    download.add_argument("--dir", required=True)
    download.set_defaults(func=cmd_download)

    download_public = commands.add_parser("download-public")
    download_public.add_argument("--tag", required=True)
    download_public.add_argument("--dir", required=True)
    download_public.set_defaults(func=cmd_download_public)

    stage = commands.add_parser("stage")
    stage.add_argument("--tag", required=True)
    stage.add_argument("--version", required=True)
    stage.add_argument("--sha", required=True)
    stage.add_argument("--dir", required=True)
    stage.set_defaults(func=cmd_stage)

    finalize = commands.add_parser("finalize")
    finalize.add_argument("--release-id", dest="release_id", required=True)
    finalize.add_argument("--tag", required=True)
    finalize.set_defaults(func=cmd_finalize)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
