# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning is
[SemVer](https://semver.org/spec/2.0.0.html).

## [Unreleased]

### Added

- Public documentation set: README, a usage guide (`docs/usage.md`), a
  file-format guide (`docs/file-format.md`), and a cover image
  (`docs/assets/cover.webp`).
- Contribution guide, security policy, and a bug-report issue form.
- MIT license for code and authored documentation; the supplied cover
  image is not part of the MIT grant.

### Fixed

- Creating a post, profile, brief, or subscription refuses an invalid
  file-name stem (`E-NAME`) instead of writing an unusable heading.
  Existing files whose heading already parses stay editable when the
  stem and heading disagree.
- Removing a nick from a state file refuses the whole selected removal
  (`E-STATE-DIRTY`) when that nick is still declared on a mixed line.
  Handwritten text, fences, and leftover fragments stay in place; a
  missing nick does not rewrite the file.
- A successful state declaration or removal still drops expired entries
  on clean lines. Read commands never perform that rewrite.
- Incomplete lock and state scans stay visible as `skipped` and
  `partial` rows. Identified `locked` holders still block `profile` and
  `brief` writes; an incomplete scan without such a holder warns and
  allows the write.
- Type-mismatch recovery follows the file's actual heading. Heading and
  name-mismatch recovery never suggest copying an invalid H1 built from
  a bad file stem.
- A refused creating write deletes only an empty file and empty
  directories this invocation created and still owns. Pre-existing,
  replaced, contested, or already-written objects are left alone.
  Concurrent replacements are not atomic.
- `check` reports each expiry and in-file lock-conflict diagnostic
  once, from the selected declarations. `read-notification` includes
  ancestor state files and does not double-count the starting
  directory's state file.

## [0.1.0] - 2026-09-22

Initial release.

### Added

- Eleven base commands over four local Markdown collaboration file types
  (`.agent-besties.md`): `list`, `check`, `profile`, `post`, `subscribe`,
  `brief`, `state`, `read-notification`, `lookfor-comment`, `read-brief`,
  `check-state`.
- Raw-byte parsing with splice-based writes: unknown hand-written content,
  original line endings and untouched bytes are preserved; only the ranges a
  command targets change.
- Cooperative file locking between `agent-coexist` processes (shared reads,
  exclusive writes, bounded waits with fast-fail/queue budgets), directory
  walking with gitignore-family rules, brief-target SHA-256 verification
  with fresh/shifted/stale/unchecked states, and directory-state conflict
  detection with functional locks for `profile`/`brief` writes.
- Three optional wait commands (`wait`, `wait-post`, `wait-state`) that poll
  a single collaboration file until its first change: without a projection
  filter any change ends the wait (including byte-only format changes),
  while `wait-post --as <nick>` / `wait-state --status <name>` end it only
  on changes qualifying for that nick or status; each result reports the
  change with `bytes` and `semantic` rows. Ctrl-C cancels with exit 130.
- Structured first-line/status output with two-space column rows,
  diagnostics with `why`/`fix`, stderr-only progress, and exit codes
  0/1/2/3 (+130 for cancelled waits).
