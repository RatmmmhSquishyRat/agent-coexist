# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning is
[SemVer](https://semver.org/spec/2.0.0.html).

## [Unreleased]

### Added

- Built-in offline guide: `agent-coexist guide [start|read|write|wait]`
  prints one Markdown tutorial page from the binary (omit the topic for
  the index). It is information output like `--help`: stdout, empty
  stderr, exit 0, no status line; an unknown topic is a usage error,
  exit 2.
- Canonical project skill at `.agents/skills/agent-coexist/SKILL.md`.
  Copy that directory into a consuming project's `.agents/skills/` (or
  Claude Code's `.claude/skills/`); there is no installer.
- Public documentation set: README, command reference
  (`docs/usage.md`), file format reference (`docs/file-format.md`),
  contribution guide, security policy, bug-report form, and a cover
  image (`docs/assets/cover.webp`).
- MIT license for code and authored documentation; the supplied cover
  image is not part of the MIT grant.

### Changed

- Documentation reorganized around what a first-time reader needs. The
  guide index now walks through one complete exchange between two
  agents, and the topic pages split into choosing a file type (`start`),
  reading (`read`), writing and closing out (`write`), and waiting
  (`wait`). `docs/usage.md` is a lookup reference with new
  "Recovery commands (`fix`)" and "Platform notes" sections; trust
  boundaries are collected in `SECURITY.md`. Command behavior is
  unaffected.
- README, guide index, and project skill now start from the problem the
  tool addresses: agents in other harnesses, sibling subagents, and
  resumed sessions often cannot see who else worked on a project or what
  they did. The README example follows a newcomer who lists the records,
  checks directory state, and reads the existing thread before joining
  in; the guide index and skill tell agents to look for relevant records
  when they arrive in or return to a project.
- **Script-visible:** an empty column in a first line, a wait ready line,
  or a Fields key/value now prints as `/`, so splitting on two or more
  spaces always yields every column. A stored value that is itself `/`
  prints the same way. Verbatim `| ` text blocks are not substituted.
- **Script-visible:** usage errors, including argument-parser failures,
  are one `error: ...` line on stderr (CR/LF shown as spaces), stdout
  empty, exit 2.
- Short help for `check` and `read-brief` explains the split: `check`
  reports entry states without target bodies; `read-brief` prints
  bodies, and `--entry` still prints that entry's content.
- `profile` and `brief` show TTY progress on stderr during their
  ancestor lock scans.

### Fixed

Creating and naming files

- Creating a post, profile, brief, or subscription with an invalid
  file-name stem fails with `E-NAME` instead of writing an unusable
  heading. Existing files whose heading already parses stay editable.
- A refused creating write removes only the empty file and empty
  directories this invocation created, and leaves pre-existing or
  already-written objects alone.
- Type-mismatch (`E-TYPE`) and heading recovery follow the file's actual
  heading and never suggest an H1 built from an invalid stem.

State files and locks

- `state --remove` refuses the whole removal with `E-STATE-DIRTY` when
  the nick is still declared on a line with unparseable fragments,
  instead of risking those bytes. The printed retry keeps the original
  `--status`.
- A removal with `--status` that finds no matching entry
  (`E-STATE-NOTFOUND`) names that status and suggests a read-only
  `check-state`, never a broader removal.
- A same-file lock-conflict `fix` includes `--status=locked`, so
  copying it removes only that lock.
- Recovery for a misnamed or unselected state-content file is a hand edit
  of that file; it never prints a directory-level `state` command that
  would change a different, reserved `state.agent-besties.md`, and
  rename advice never overwrites an existing state file. Wait-state
  reports apply the same check at the final snapshot.
- `profile`/`brief` writes through a file symlink also honor `locked`
  on the symlink target's parent directory chain.
- Incomplete lock scans show `skipped`/`partial` rows (including stray
  state text, `W-STRAY`); identified `locked` holders still block, and
  otherwise the write proceeds with a warning.
- `check` reports each expiry and same-file lock conflict once.
  `read-notification` includes ancestor state files without counting
  the starting directory's file twice.

Scans

- `check-state` fails on a broken ignore source the same way as `list`
  and `read-notification` (`cannot scan`, exit 1). If a `state` write
  already succeeded, the same error is a `warn` with a `scan` row,
  exit 0, and the write stays.
- Ignore rules follow Git worktree, gitfile, separate-gitdir, and
  `commondir` layouts and an explicit global Git config; a leading `~`
  in `core.excludesFile` expands only as a home component. Scans use a
  pinned, locally patched `ignore` 0.4.33.
- Scans keep native path identity: distinct directories whose display
  names fold together stay distinct, roots with spaces relativize
  correctly, and Windows ordinary/verbatim volume spellings agree.
- An explicit scan `DIR` that is a directory symlink, junction, or
  `.git` directory is entered; ones found below it stay pruned.
- The unmatched-`--include` hint row is display-folded; matching still
  uses the glob as typed.

Recovery commands and display

- Recovery `fix` commands name the original file: operands keep the
  managed suffix, internal whitespace, and Windows verbatim spelling,
  and are quoted for a POSIX shell. On Unix, non-UTF-8 bytes appear as
  `"$(printf '\377')"`-style substitutions instead of pointing at a
  U+FFFD look-alike. When an operand cannot be written safely (NUL, CR,
  LF, or unpaired UTF-16 on Windows), the whole recipe becomes a manual
  instruction.
- Literal and hand-edit `fix` lines are no longer shell-quoted. Headings
  with control characters, invalid profile section names, and invalid
  status names get hand-edit advice instead of a command the CLI would
  reject; suggested section names and comment numbers are unique within
  one report.
- Titles and diagnostic text fold whitespace and show control characters
  as U+FFFD without changing stored bytes.
- `--target` and `--regex` reject raw CR/LF; `--remove` with
  `--target` is refused; `--regex ''` deletes every regex line of the
  entry.

Posts, waits, and file bytes

- `@alice-@bob` is two mentions (`alice-` and `bob`). `check` lists
  post `authors` in first-occurrence order; write reports list mentions
  and replies in body order.
- Appending after a file that ends in an isolated `CR` no longer turns
  it into `CRLF`, and inserted properties or bodies reuse an existing
  whitespace-only gap line.
- `wait` and `wait-post` report a non-directory parent as a path
  error. A wait refuses a file over 16 MiB with the same report as
  `check`, also when the file grows past the cap later. A wait report
  shows a brief's description once when the file is created, deleted, or
  changes type.
- Leap seconds read from files print as `:60`.

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
