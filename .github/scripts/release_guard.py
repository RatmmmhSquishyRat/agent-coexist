#!/usr/bin/env python3
"""Release guard: validates the triggering source and picks the run path.

Reads the workflow environment (no arguments) and emits step outputs through
GITHUB_OUTPUT:

  version     Cargo.toml version of the triggering source
  tag         vX.Y.Z for tag runs, empty otherwise
  source_sha  GITHUB_SHA verbatim — the triggering commit and the single
              source identity used everywhere: checkouts, ancestry, epoch,
              stage target, and attestation verification. It is also the
              value the SLSA provenance records (the OIDC job token's `sha`
              claim), so --source-digest and --signer-digest must receive
              exactly this value, unchanged.
  source_ref  the full triggering ref
  epoch       commit timestamp (unix seconds) for SOURCE_DATE_EPOCH
  dry_run     "true"/"false"
  reusable    "true" only when a release for the tag already carries exactly
              the complete seven-asset cohort

Validation: tag ⇔ Cargo version, dated changelog heading, and tagged-commit
ancestry of origin/main. Those tag assertions are skipped only for a non-tag
dry run; a non-dry run always requires a version tag. HTTP 404 from the
release lookup means "no release yet"; any other API or network failure is a
hard error.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tomllib

# The import below must not dirty the checkout (cargo package/publish refuse
# dirty trees elsewhere in the pipeline).
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from release_state import (  # noqa: E402
    COHORT_NAMES,
    VERSION_RE,
    TAG_RE,
    die,
    heading_line_regex,
    release_by_tag,
    set_output,
)


def git(*arguments: str) -> str:
    result = subprocess.run(["git", *arguments], capture_output=True, text=True)
    if result.returncode != 0:
        die(f"git {' '.join(arguments)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def main() -> None:
    ref = os.environ.get("GITHUB_REF", "")
    ref_name = os.environ.get("GITHUB_REF_NAME", "")
    sha = os.environ.get("GITHUB_SHA", "")
    dry_run = os.environ.get("RELEASE_DRY_RUN", "false") == "true"
    if not ref or not sha:
        die("GITHUB_REF and GITHUB_SHA must be set by the workflow")

    # Named public gate: no release work (dry or not) on a private repository.
    if os.environ.get("RELEASE_REPOSITORY_PRIVATE", "") != "false":
        die(
            "the release pipeline requires a public repository "
            "(RELEASE_REPOSITORY_PRIVATE="
            f"{os.environ.get('RELEASE_REPOSITORY_PRIVATE', '')!r}); "
            "repository visibility is user-owned"
        )

    is_tag = ref.startswith("refs/tags/")
    if not is_tag and not dry_run:
        die(
            "a non-dry release requires a version tag: dispatch with "
            "--ref vX.Y.Z and dry_run=false, or push a v* tag"
        )

    try:
        with open("Cargo.toml", "rb") as handle:
            version = tomllib.load(handle)["package"]["version"]
    except (OSError, KeyError) as error:
        die(f"cannot read package version from Cargo.toml: {error}")
    if not VERSION_RE.match(version):
        die(f"Cargo.toml version {version!r} is not strict X.Y.Z")

    tag = ""
    if is_tag:
        tag = ref_name
        if not TAG_RE.match(tag):
            die(f"tag {tag!r} is not a strict vX.Y.Z tag")
        if tag[1:] != version:
            die(f"tag {tag} does not match Cargo.toml version {version}")
        heading = heading_line_regex(version)
        try:
            with open("CHANGELOG.md", encoding="utf-8") as handle:
                changelog = handle.read()
        except OSError as error:
            die(f"cannot read CHANGELOG.md: {error}")
        if not any(heading.match(line) for line in changelog.splitlines()):
            die(f"CHANGELOG.md has no dated heading '## [{version}] - YYYY-MM-DD'")
        ancestry = subprocess.run(
            ["git", "merge-base", "--is-ancestor", sha, "origin/main"],
            capture_output=True,
        )
        if ancestry.returncode != 0:
            die(f"tagged commit {sha} is not an ancestor of origin/main")

    epoch = git("show", "-s", "--format=%ct", sha)
    if not epoch.isdigit():
        die(f"could not determine a numeric commit timestamp for {sha}")

    reusable = "false"
    if is_tag and not dry_run:
        release = release_by_tag(tag)
        if release is not None:
            names = {asset["name"] for asset in release.get("assets", [])}
            reusable = str(names == set(COHORT_NAMES)).lower()
            state = "draft" if release.get("draft") else "published"
            summary = ", ".join(sorted(names)) or "(no assets)"
            print(f"existing {state} release {tag}, assets: {summary}")

    set_output("version", version)
    set_output("tag", tag)
    set_output("source_sha", sha)
    set_output("source_ref", ref)
    set_output("epoch", epoch)
    set_output("dry_run", "true" if dry_run else "false")
    set_output("reusable", reusable)
    print(
        f"guard OK: version={version} tag={tag or '-'} dry_run={dry_run} "
        f"reusable={reusable} commit={sha} epoch={epoch}"
    )


if __name__ == "__main__":
    main()
