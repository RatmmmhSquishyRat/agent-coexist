# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning is
[SemVer](https://semver.org/spec/2.0.0.html).

## [Unreleased]

### Fixed

- A filter given to the wrong wait command now names the command that
  takes it. `wait <post> --as <nick>` used to answer only that `wait` has
  no `--as`; it now says that `wait` takes no filter and gives
  `agent-coexist wait-post <PATH> --as <nick>`. `--status` on `wait` or
  `wait-post` points to `wait-state` the same way. `wait --help` says so up
  front. Two agents in a dsh plugin trial had each hit exit 2 this way.

## [0.5.0] - 2026-10-25

### Added

- `--timeout <DURATION>` on `wait`, `wait-post` and `wait-state`, using the
  same DURATION grammar as `--for`. When the time passes with no qualifying
  change, the wait exits **124** with empty stdout and one stderr line,
  `timed out  <command>  <object>  after <value>`, after its usual ready
  line. Like Ctrl-C cancellation (130) it is neither a change report nor a
  failure. The budget runs from command start, fixed before any file is
  opened, so it lines up with a harness's tool-call limit. Sampling never
  sleeps past the deadline, and the sample taken there is still evaluated:
  a change at the deadline is reported rather than mistaken for silence.
  Without `--timeout` the behaviour is unchanged.
- Waiting no longer needs a harness that can run background processes: one
  that only makes foreground calls can wait in bounded slices with
  `--timeout`, reading the file again after each 124.

### Changed

- The skill's description fits in 500 characters. dsh (DeepSeek harness)
  shows its model only that much of a skill description, and the old
  672-character one was cut off there, mid-sentence and before its last
  sentence: the guard against creating files merely because the word
  appears.
- The skill is reorganized to say what coexist is, when to use it, and how,
  in that order, with short sections instead of long paragraphs. Its
  description now names three occasions:
  - when the agent needs to know whether other agents are working here;
  - when it needs to ask, answer or hand off to an agent it cannot reach;
  - when it comes across `*.agent-coexist.md` files.

  The old description applied only once the agent knew the project was
  shared, which in a live dsh test meant it never applied. The skill does
  not make coexist a step of every task.
- The skill tells the agent how to check that the CLI is available: run
  `agent-coexist --version` on its own. A dsh agent combined that check with
  other PowerShell output, which hid the result. It concluded the CLI was
  missing and fell back to a plain-file handoff.
- The skill names the guide pages as commands (`agent-coexist guide write`
  and the like). A dsh agent had tried `agent-coexist write --help`.
- The skill says a record is another agent's note, not the user's
  instruction. When a record conflicts with the agent's instructions or the
  project's files, the agent tells the user rather than silently following
  either. `guide write` asks an agent handing off to also write a user's
  change to the task where the task itself is written. In a dsh test, a
  fresh session read a handoff whose change from the user existed only in
  the record and contradicted TASK.md. It declined the change as unverifiable
  and told the user; that is now the documented behaviour, and writing the
  change into TASK.md as well avoids the conflict.
- The `wait` guide page, the per-command help, the top-level exit-code
  table and the command reference describe `--timeout` and exit 124, and no
  longer say waits have no time limit.
- Questions and ideas go to GitHub Discussions; the issue chooser routes
  them there and security reports to the private reporting form, which is
  now actually enabled — `SECURITY.md` had pointed to it while it was off.

## [0.4.2] - 2026-10-24

### Added

- A rendered terminal panel in the README showing a real two-agent exchange:
  one agent claims a directory and mentions another, the second recovers both
  the claim and the mention from a single `read-notification` with no shared
  context, and the resulting thread is shown as the Markdown file it is. Every
  line is captured output, not a mock-up.

### Fixed

- GitHub classified the repository as "Other" / `NOASSERTION` rather than MIT,
  because the root `LICENSE` carried two copyright blocks: this project's MIT
  and an appended copy of the `ignore` crate's. Licence detection reads one
  licence from that file, so the third-party notice moves to
  `THIRD-PARTY-NOTICES.md` at the repository root, which is also included in
  the published crate. Nothing is dropped: the notice keeps the root-level
  visibility it was given in 0.4.0, and the upstream originals remain beside
  the code in `src/vendor_ignore/` (`COPYING`, `LICENSE-MIT`, `UNLICENSE`).

## [0.4.1] - 2026-10-24

### Changed

- Documentation states the full scope instead of leading with harnesses. The
  harness boundary is only the most visible wall between agents: sibling
  subagents under one harness cannot see each other even while editing one
  tree at the same moment, different sessions start blank including the
  session that resumes the same work later, and parallel main sessions are
  equally blind. What those cases share is a filesystem, and the parties never
  need to meet. The README tagline, `Why` section and demo captions say that,
  and gain the sequential handoff and concurrent claim patterns.
- The README carries a `What it looks like` section built from real captured
  output of a two-agent exchange, and its `Install` section names the
  uninstall for each route.
- Crate description and keywords follow the same scope; `filesystem` gives way
  to `multi-agent`.
- The bundled skill's `description` names the three cases so an agent can
  recognize its own situation, with the trigger conditions leading.

### Fixed

- `Cargo.toml` had been rewritten with CRLF line endings. It is LF again, and
  `.gitattributes` now pins every text type in the repository to LF so an
  editor or a text-mode write cannot flip one silently.

## [0.4.0] - 2026-10-17

### Added

- Automated Dependabot tracking for GitHub Actions dependencies (`.github/dependabot.yml`).
- Exhaustive test coverage for 7 specification diagnostics (`E-TARGET-MISSING`, `E-ENTRY-NOTFOUND`, `E-SECTION-NOTFOUND`, `E-STATE-NOTFOUND`, `E-EDIT-AUTHOR`, `E-COMMENT-NOTFOUND`, `E-SEQ-EXHAUSTED`) and `post --edit` in-place replacement unit tests.

### Changed

- **Breaking:** collaboration files now end in `.agent-coexist.md`, matching
  the tool name; the directory state file is `state.agent-coexist.md`. Files
  ending in `.agent-besties.md` are no longer recognized: `list` and scans
  skip them, and a path given with the old suffix gets the new suffix
  appended. To keep existing records, rename each `<name>.agent-besties.md`
  to `<name>.agent-coexist.md`; file contents and H1 lines are unchanged.
- Modernized public README to elevate `cargo install agent-coexist --locked` as the primary installation path, added live crates.io and CI badges, and streamlined prebuilt binary instructions.
- Cleaned Cargo packaging: pruned unused `thiserror` dependency, excluded local toolchain files from `package.include`, and added Linux GNU `cargo-binstall` overrides pointing to static musl archives.
- Supply chain hardening: re-enabled `[sources]` and `[bans]` in `deny.toml` with multi-version compatibility exception for `syn`.

### Fixed

- Appended Andrew Gallant's MIT copyright notice to the root `LICENSE` file for inlined `ignore` crate code.
- Added explicit job timeouts (`timeout-minutes`) across `ci.yml`, `release.yml`, and `weekly.yml` GitHub workflows.
- Eliminated Linux-only gating in `tests/output_delivery.rs` using a portable broken-pipe mechanism so output delivery error handling is natively verified on Windows and macOS.
- Unblocked Windows concurrency execution in `tests/write_clock.rs` by replacing raw Unix `flock` with portable shared locking and preserving Windows system environment variables.
- Dynamic wall-clock refresh in `wait::run`: sample `time::now()?` on byte changes so lease expirations and conflict evaluations reflect real wall-clock time instead of frozen startup timestamps.
- `read-brief` no longer hides an unexpected state behind `.unwrap_or_default()` in `src/ops/query/read_brief.rs`; it is matched explicitly.
- Prevented monotonic clock arithmetic overflow in `src/wait.rs` by using checked elapsed duration subtraction.
- Authenticated GitHub REST API metadata lookups in `release_state.py` with `GH_TOKEN`/`GITHUB_TOKEN` to prevent rate-limiting failures on shared CI runners.
- Unified Unix broken-pipe testing in `tests/output_delivery.rs` under `#[cfg(unix)]` using closed `libc::pipe` descriptors.
- Added secret token patterns (`crate_token.txt`, `*token*.txt`, `*.token`) to `.gitignore`.
- Documented internal ACID rollback verification invariant for `CommitFile` in `src/fsx/commit.rs`.

## [0.3.0] - 2026-10-11

### Added

- Maintainer release automation: a tag-driven workflow builds, smoke-tests,
  and attests six native archives (Linux musl x64/ARM64, macOS Intel and
  Apple Silicon, Windows x64/ARM64) with a `SHA256SUMS` manifest, stages
  a draft GitHub Release, publishes the crate to crates.io, verifies the
  registry package and installed binary, then publishes the release; a
  nonpublishing dry run rehearses the build path on `main`.

### Changed

- The CLI and its internal implementation now form one installable Cargo
  package; install from a checkout with `cargo install --path . --locked`.
  The core library and the vendored `ignore` fork are internal modules
  of that package.
- Reorganized file format reference rules for comment headers, numbering, and repairs into readable sections, clarifying manual reply retargeting and sequence-exhaustion recovery without changing on-disk formats.
- Streamlined diagnostic recovery help and guide headings with concise operating boundaries and clearer creation/edit contracts.
- CI hardened: locked checks on the pinned toolchain across Linux,
  Windows, and macOS, plus a packaging dry run (`cargo publish
  --dry-run`), a supply-chain scan (`cargo deny`), and a weekly re-run
  on the current stable toolchain.

### Fixed

- Canceling the last subscription now aborts the write when the
  post-splice re-parse fails, instead of writing the file while
  reporting a fabricated empty subscriber list.

Concurrency and output delivery

- A queued post or state declaration now records its activity time while
  holding the exclusive lock, before assembling the write. A notification
  scan completed while the write was waiting can resume from its start
  without losing that later event. A state declaration keeps its full
  duration, and a fixed `AGENT_COEXIST_NOW` still stays fixed.
- Brief target reads take bounded shared locks, including ordinary names
  and file aliases. Verification and refresh no longer consume an
  in-progress cooperating write. Self-references reuse the brief's locked
  pre-write snapshot. Target buffering also avoids reallocating on every
  read chunk while retaining the same memory cap and full-file hash.
  When a target is busy, `check` recommends retrying the read rather than
  changing permissions or rewriting the brief entry.
- Help, version, and business reports check stdout write and flush errors
  and return exit 1 when delivery fails. A usage error remains exit 2
  even if stderr fails; an already committed file change is retained.
- Quoted `core.excludesFile` paths with spaces are loaded correctly. The
  selected config value respects comments, escapes, continuation, and
  last-value precedence within `[core]`; malformed values report their
  source instead of silently dropping its ignore rules.
- File-size-limit regression tests capture child output through pipes,
  so writing the test log to a regular file cannot trigger the child
  process's file-size limit. Commit recovery and final-sync failure now
  have deterministic regression coverage.

## [0.2.0] - 2026-10-09

### Added

- Built-in offline guide: `agent-coexist guide [start|read|write|wait]`
  prints one Markdown tutorial page from the binary (omit the topic for
  the index). The index covers what an agent does on arrival: read the
  existing records first, choose a nick no one in them uses or check
  whether a nick it was given is already addressed, and carry its nick,
  threads, and last-read times into the handoff; a thread's last-read
  time covers only that thread. The topics cover choosing a file type
  (`start`), reading (`read`), writing and closing out, including how to
  act on a `fix` that is a command, an instruction, or both, and how to
  tell a repair followed by a retry from a lookup whose output corrects
  your original command (`write`), and waiting, including a re-read
  after a retried wait's ready line, and catching up across a directory
  without a gap (`wait`).
  `guide wait` also states that the state warnings in a hit report —
  conflicts and expiry — are judged at the wait's start time: a lock
  that expired while you waited still counts as valid, so a conflict
  warning can appear with no expiry warning; it says to run
  `check-state` on the directory (or `check` on the file) before
  acting on a state warning or its fix, to judge it at the current
  time, and it states one restart rule for every failed wait, not only
  a busy file: a restarted wait takes its baseline from the file as it
  is at the restart. `guide write` says that after a write reports
  `skipped` or `partial` rows you do what the `fix` says: it may first
  ask you to make an unreadable directory both readable and
  enterable, and it then names a read-only command — `check` on a file
  or `check-state` on a directory. The pages say "coexist file"
  throughout.
  It is information output like `--help`: stdout, empty stderr, exit 0,
  no status line; an unknown topic is a usage error, exit 2.
- Canonical project skill at `.agents/skills/agent-coexist/SKILL.md`.
  It tells an agent to look for existing records on arrival and to run
  `agent-coexist guide`. Without the CLI, it explains how to find and
  read records with ordinary file tools and how to add a comment by hand
  with a fence longer than any backtick run in the body. Copy that
  directory into a consuming project's `.agents/skills/` (or Claude
  Code's `.claude/skills/`); there is no installer.
- Public documentation set: command reference (`docs/usage.md`), file
  format reference (`docs/file-format.md`), contribution guide, security
  policy, bug-report form, and a cover image (`docs/assets/cover.webp`).
  The command reference includes recovery commands (`fix`) and platform
  notes, and states an output limit: a business command that cannot
  write its report to stdout exits 1, possibly after part of the report
  was written, and a failed final flush is ignored, so the exit code
  does not prove complete delivery. The security policy separates what
  the CLI guarantees from what it does not.
  The command reference also states that a wait `removed` report has no
  after snapshot and therefore no warning rows, and its `Lifecycle`
  section prints the three wait restore sentences in full.
- Output summary pages (`docs/output-summary.md`): a catalog of actual
  commands and their captured output (stdout/stderr/exit code) by
  situation, including the full stdout of the five built-in `guide`
  pages and every `--help`; topics are linked from the entry page. The
  pages are in English; captured output is shown as captured, so
  operating-system messages appear in the capture's locale.
- MIT license for the project's own code and authored documentation.
  The vendored `ignore` crate keeps its upstream notices and license
  (Unlicense OR MIT); the supplied cover image is not part of the MIT
  grant.

### Changed

- Consolidate authored documentation by keeping README as a minimal entry
  point, establishing the guide as the single canonical walkthrough,
  streamlining task guides without repeating syntax catalogs, and focusing
  output summaries on representative examples.

- Corrected three text promises: the absolute-path warning describes a
  portability risk, not guaranteed staleness; profile/post help and the
  command reference state that bodies are fenced after CRLF-to-LF
  conversion and end-of-body whitespace removal; subscription help and
  `guide start` limit comment notifications to other authors. Path
  resolution, body normalization, and notification behavior are unchanged.
- **Script-visible:** top-level `--help` names exit 1 not only for a
  first line `error`, but also when a command could not write its
  report to stdout (a write command may already have changed the file:
  check it before retrying). Help and version pages that clap writes
  still ignore a failed stdout write and exit 0.
- **Script-visible:** a path-class I/O failure whose path contains a
  line break or other control character prints
  `the path contains a line break or other control character (original: '<echoed path>'); fix the path, then run the corrected command`.
  The folded `reason` path is unchanged. Paths without control
  characters keep `fix the path, then run the corrected command`.
- **Script-visible:** `E-DIR-MISSING` for `list`, `read-notification`,
  and `check-state` still lists collaboration files under the parent
  when that parent exists. When it does not, the `fix` is
  `correct the directory and rerun; its parent does not exist either; collaboration files under the nearest existing directory: agent-coexist list <dir>`.
  `state` and the waits are unchanged.
- **Script-visible:** every brief write that leaves skipped entries or
  remaining errors ends with a `fix` row pointing at this file:
  `skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check <path>`
  (when the path cannot be printed as a copyable operand, the row says
  to run `check` on this file by hand). Remaining rows stay last.
- **Script-visible:** a `--include` or `--exclude` glob that is empty
  after its leading or trailing `/` is removed is reported as
  `nothing is left after removing the leading or trailing '/'`; the same
  message earlier in this unreleased series said `leading and trailing`.
  Message shape, echo, and exit 2 are unchanged.
- **Script-visible:** a `state` declaration whose post-write conflict
  check could not resolve a directory's real path now ends with the
  same scan-incomplete `fix` as a `profile`/`brief` write's lock scan —
  restore-path prefix, then `agent-coexist check-state <dir>` on the
  declared directory — instead of leaving the `canonicalize` row with
  no `fix`; the row list in that `fix` now reads `skipped, partial or
  canonicalize`.
- **Script-visible:** a `read-notification` `expired` record now ends at
  `<nick>` — `expired  <end time>  <dir>  <status>  <nick>` — with no
  trailing detail; the dropped value `expired at <t>` repeated the
  first column verbatim. Every other record kind keeps its detail
  column; ordering, window filtering, and exit codes are unchanged.

- README restructured for first-time readers. It starts with what the
  tool is for and what an agent does on arrival: read the existing
  records before joining in, under a nick no one in them uses, and keep
  that nick for continued work. It then links to the guide, skill, and
  references above. Command behavior is unaffected.
- Top-level `--help` now names exit 130 for a wait cancelled with
  Ctrl-C, and its DURATION note states that units go in descending
  order (`d`, `h`, `m`, `s`), as in `1h30m`. Cancellation and duration
  parsing behave as in 0.1.0.
- `wait`, `wait-post` and `wait-state` help now states the per-sample lock
  contract: each once-per-second sample takes a shared lock with a 1s
  wait, a still-busy lock ends the wait with `E-LOCKED` (exit 3), and a
  Windows sharing violation ends it with `E-INUSE` (exit 3). Only the
  help text changed; lock handling is unchanged.
- `state --help` now spells out what declaring again does: it
  `replaces your entries under that status with one from now, except an
  entry on a line that also has unparseable fragments (W-STATE-ENTRY):
  that line is kept, and the latest end time still wins; check-state
  shows the result`. 0.1.0's help had no such sentence. Only the help
  text changed; declaring behavior is unchanged.
- `state`, `read-notification`, and `check-state` help now state that
  these subtree scans skip
  directories that ignore rules skip, and that `state` has no
  `--no-ignore`: run `check-state DIR --no-ignore` to see locks inside
  ignored directories. Scan behavior is unchanged.
- **Script-visible:** `W-STATE-DUP` now names which write merges
  duplicates — the next state write that rewrites the `{status}` section
  (a declaration or removal under it, or expiry cleanup of an entry in
  it) — and says the merge happens only when both entries are on clean
  lines; when either entry sits on a line that also has unparseable
  fragments (`W-STATE-ENTRY`), the `fix` says that line is kept as is
  and the duplicated nick deletes its entries from every such line by
  hand, keeping each line's other content, before declaring again. In
  0.1.0 the `why` promised the next state write would merge them without
  naming which write, and the `fix` was a bare state command. Merging
  behavior is unchanged.
- **Script-visible:** when the inspected state-shaped file is not the
  directory's selected `state.agent-besties.md`, `W-STATE-DUP`,
  `W-STATE-RANGE`, and `W-STATE-EXPIRED` no longer promise a state write
  will merge or remove anything in it: the `why` reads `no state write
  merges them` / `no state write removes it` / `no state write removes
  them, because state commands do not write this file until it is
  confirmed as this directory's state.agent-besties.md`, and the `fix`
  is `this file is not confirmed as this directory's state.agent-besties.md; settle which file should be, without overwriting another state file, then run check on this file again before editing declarations by hand. Only the declaring agent may change, renew or remove its entry`
  (`W-STATE-CONFLICT` likewise). In 0.1.0 every such row named a next
  state write and gave a bare state command that would not touch this
  file. Merge and removal behavior is unchanged; the printed advice now
  matches it.
- **Script-visible:** creating write forms (`post` without `--edit`,
  `profile` without `--remove`, `brief` bare, description, or `--target`)
  print
  `this command needs a file of type <type>; rerun it with such a path, or with a new path that no file uses yet to create one; files of that type under this directory: agent-coexist list <dir> --type=<type>`.
  Query, `subscribe`, `--edit`, `--cancel`, `--remove`, `--refresh`, and
  the waits keep the catalog `E-TYPE` `fix` without that create clause.
  When `state` or `wait-state` finds another type under the reserved
  name, the `fix` is
  `state.agent-besties.md in this directory has type <type>, so it cannot hold state; rename that file to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name, then retry: <original command>`.
- **Script-visible:** a `canonicalize  <dir>  failed` row now ends with
  the operating-system cause (`canonicalize  <dir>  failed  <os error>`).
  `check-state` and `read-notification` then print
  `real paths could not be resolved for directories marked canonicalize; ensure those paths exist and all directories on them, including ancestors, can be opened with read and traverse/search permission, then run this command again`,
  not counted in `total`. A `profile`/`brief` lock scan whose identity
  fell back to the spelled path puts
  `ensure each path marked canonicalize exists and all directories on it, including ancestors, can be opened with read and traverse/search permission; `
  before the checks in that write's existing incomplete-scan `fix`, and
  names `check-state` on the written file's parent directory (by hand:
  `agent-coexist check-state on the written file's parent directory`),
  never the working directory. Canonicalize still
  does not refuse writes. A `skipped  <dir>  unreadable` row for a
  directory whose state file the CLI did not see still has no read-path
  `fix`; an unreadable directory whose state file was seen still gets
  one.
- `list --help` now says that `busy` and `?` files stay shown and
  counted under any `--type`, because their type is unknown. It also
  says that a `busy` file can mean, on Windows, that another
  program holds the file open without allowing shared access, not only
  that a file lock or stable read was unavailable. For that case it
  says to close that program, then retry. Only the help text
  changed; scans and `--type` filtering behave as in 0.1.0.
- **Script-visible:** an empty column in a first line, a wait ready line,
  or a Fields key/value now prints as `/`, so splitting on two or more
  spaces always yields every column. A stored value that is itself `/`
  prints the same way. Verbatim `| ` text blocks are not substituted.
- **Script-visible:** usage errors, stdout empty and exit 2, keep their
  producer's shape on stderr. Argument-parser failures print the
  parser's native multi-line report — an `error: <message>` line, then
  only the components that error carries, separated by blank lines
  (`tip:` lines, a `Usage:` block listed once per form when a command
  has several, `For more information, try '--help'.`) — with rejected
  values echoed
  using the same `<U+XXXX>` escaping as everywhere else and nothing
  folded onto one line. Usage errors the CLI checks itself, such as
  `error: --regex requires --target <path>` or an invalid
  `AGENT_COEXIST_NOW`, remain exactly one `error:` line, with CR/LF
  shown as spaces.
- `agent-coexist` with no command prints the parser's "requires a
  subcommand" error — the `error:` line with the command list, a
  `Usage:` line, and the `--help` pointer — not the top-level help
  page. Exit 2 and explicit `--help` are unchanged.
- **Script-visible:** an `--edit`, `--reply-to` or `--comment` value
  that is not a comment number (empty, not decimal digits, or above
  18446744073709551615) reads
  `error: invalid value 'first' for '--reply-to <n>': expected a decimal comment number from 0 to 18446744073709551615`
  instead of Rust's integer-parse text (`invalid digit found in
  string`, `number too large to fit in target type`); a value starting
  with `-` is told `--edit takes a decimal comment number (such as
  --edit 3), which never starts with '-'`. Accepted values and exit 2
  are unchanged. Top-level `--help` now gives exit 2 as `usage error on
  stderr (possibly multiple lines), nothing on stdout`, matching the
  multi-line parser reports above.
- **Script-visible:** the scan-incomplete `fix` rows put each
  prerequisite before the checks it gates (`… Before writing again, make
  each directory marked unreadable readable and enterable with read and
  search permission; run these checks, then agree with any holder they
  show: …`), instead of stacking parenthesized `(first …)` clauses
  before the colon. When no copyable command can be printed, a `state`
  declaration names `agent-coexist check-state on the declared
  directory`. `W-STRAY`, `W-SUB-DUP`, the unselected-state fix, the
  repeated `W-STATUS-NAME` fix and the exhausted-numbering repairs are
  shorter with the same rules. `W-STRAY`'s `fix` is selected by file
  type, so no row recommends a destination its file type cannot have: a
  post's row names a fenced body and keeps the comment-author rule, a
  brief or profile row names a fenced body only, and a state file's row
  names a reader note on a line starting with `> `, a misplaced actual
  state entry under its status, or deleting the text. Every `W-STRAY`
  row's shared `why` reads `this text stays in the file but is not
  included in CLI reports`, accurate for all four file types. A single
  invalid status heading now says
  to choose a valid, unused name; the printed `## <name>` is only an
  example. `E-INUSE`'s `fix` says `close the other program, then retry`.
- **Script-visible:** Text blocks (`    | ` lines) print one line per
  content line: the final line break of a target or value no longer adds
  an empty `    | ` line, so a target shown as `L1..L1` prints one line,
  and empty content prints only its key line.
- `brief --refresh` on a brief with no entries now prints
  `unchanged  <file>` instead of an empty report. The functional-lock
  check still runs first.
- **Script-visible:** a `brief` description, note, `--target`, or
  `--refresh` form that would leave the file byte-for-byte as it is
  (the same text, the same path and hash, hashes already current, or
  every entry skipped) no longer rewrites it. Its own rows are followed
  by `unchanged  <file>`, as for the bare form, and no `remaining` rows
  follow, so the report tells a verification from a rewrite.
- `E-TARGET-MISSING` now names the path the CLI looked for:
  `target src/parser/mod.rs does not exist; looked for
  docs/src/parser/mod.rs`, so a target typed relative to the cwd
  instead of the brief's directory is visible in the message.
- Usage-error messages for values now name the accepted form with an
  example and echo the rejected value: `--since` (and a hint when the
  date and time are separated by a space), `--for`, `AGENT_COEXIST_NOW`,
  `AGENT_COEXIST_LOCK_WAIT_MS`, and the end-of-time limit of a `state`
  declaration. `--since` and `--for` both state the duration rule
  (`Nd, Nh, Nm, Ns in that order, each at most once, total above zero`),
  so a value such as `0s` or `30m1h` is told why it failed; top-level
  `--help` prints the same rule. `--regex` compile errors keep only the
  regex library's one-line cause instead of its folded caret diagram.
  Glob errors name the option (`--include` or `--exclude`) and show the
  pattern once, instead of repeating it inside the glob library's cause.
  `--mention @nick` suggests the bare nick, and an empty `--keyword`
  says to omit it. Every rejected value is echoed as typed, never
  folded or trimmed, with controls, whitespace other than a plain space,
  and other characters that do not print (Unicode format characters such
  as a BOM, a zero-width space or a bidirectional control, plus
  unassigned and private-use code points) shown as `<U+XXXX>`: nicks,
  statuses, section names, `--entry` titles, `--type`, `guide` topics,
  `--for`, `--since`, `--regex`, globs, `AGENT_COEXIST_NOW`,
  `AGENT_COEXIST_LOCK_WAIT_MS`, the values the argument parser quotes
  in its own errors, and the unknown option an unknown-option error
  names (`'post' has no option --bad<U+202E>`). A trailing space or CR
  used to vanish from the echo, so `AGENT_COEXIST_NOW` could print a
  value identical to its own example, and `--as $'\ufeffalice'` printed
  `(got 'alice')` while saying it was invalid. When only whitespace at
  either end is the problem, the error says to remove it; for the two
  environment variables this comes before the advice to unset them.
  This also holds when a nick, status, section name or `--entry` title
  is rejected for a control character or two spaces in a row, which
  used to print only the rule: `--as $'alice\r'` now reports `--as must
  not contain control characters (got 'alice<U+000D>'); remove the
  whitespace before or after
  'alice'`, while `--as $'ali\tce'` gets no such hint because trimming
  does not make it valid. Invisible characters are only shown, never
  removed for you. An end time past `9999-12-31T23:59:59Z` is
  reported as such for every `--for`, including one too large to count,
  and without `--for` the message names the default lengths; a
  `--since` duration reaching before year 0001 says so. An
  `AGENT_COEXIST_LOCK_WAIT_MS` above the largest accepted number states
  that limit. Validation, exit code 2, and the single stderr line are
  unchanged. `E-ENTRY-REGEX` messages likewise keep only the one-line
  regex cause. `--target` values with a raw CR or LF join the echoed
  values with the remove-whitespace repair (`--target must not contain
  raw CR or LF (got 'src/main.rs<U+000D>'); remove the whitespace before
  or after 'src/main.rs'`), while `--regex` is told to write `\r` or
  `\n` in the pattern to match them. A date and time separated by a
  space in `AGENT_COEXIST_NOW` is told to separate them with `'T'`,
  which keeps the fixed clock, instead of only being told to unset the
  variable.
- `brief` reports `--regex` together with `--refresh` or `--remove` as
  that combination (`--refresh cannot be combined with --target,
  --regex, -m or --remove`, `--regex cannot be combined with --remove`)
  instead of asking for a `--target` the form cannot take. `brief
  --entry` with none of the options that complete it now names all of
  them (`--entry requires -m (note), --target <path>, --remove or
  --refresh`); the old message left out `--target`. Exit code 2 is
  unchanged.
- `brief --help` and `read-brief --help` describe `--entry` as it
  behaves: one space at either end is ignored, two spaces in a row
  anywhere (including at an end) or a control character such as a tab or
  CR is rejected, and titles in the file match with whitespace runs read
  as one space. 0.1.0's help said only "matched after whitespace folding".
- An option value that starts with `-`, such as a `-m` body that begins
  with a Markdown list item, is still read as an option, but the error
  no longer repeats clap's `-- <value>` tip, which turned the value into
  the `PATH`. Free-text options (`-m`, `--keyword`, `--entry`,
  `--target`, `--regex`, `--include`, `--exclude`) are told to attach
  the value with `=` (`-m='- item'`); a multi-line body is told to start
  the same argument with `-m=`. A value with a character that is shown
  as `<U+XXXX>` gets no copyable command either: the error says to start
  the same argument with `-m=` followed by the whole text, typing each
  `<U+XXXX>` as the character it names. Options whose values never
  start with `-` (nicks, statuses, section names, `--type`, `--since`,
  `--for`, and comment numbers) restate what they take instead.
  Top-level help and the `post`, `profile`, and `brief` help describe
  the `=` form. Parsing and exit code 2 are unchanged.
- An unknown option elsewhere on the command line no longer gets
  clap's `-- <word>` tip when that would only add a second `PATH`. The
  error says the command has no such option and points to
  `<command> --help`, and offers `--` only for a word that may be the
  missing file or directory name. `-V` after a command says
  `--version` goes before any command, and a single-dash long name
  such as `-as` is told to use `--as`.
- An argument that is not valid Unicode is no longer reported only as
  clap's `invalid UTF-8 was detected in one or more arguments`. The
  message names the argument where it can (for example `--as`), says to
  retype it or re-encode its source as Unicode, and notes that only
  `PATH` and `DIR` accept file names that are not valid Unicode. Exit
  code 2 is unchanged.
- A free-text option value that starts with `-` and is not valid Unicode
  is told to re-encode first, then attach the Unicode text with `=`. It
  no longer offers a copyable `{flag}=` form of the original bytes, which
  clap would reject. Valid Unicode dash values still get the attached
  form. Exit 2 is unchanged.
- An invalid glob's cause now echoes user characters the same way as the
  pattern, so a control or bidi character in a reversed range cannot
  reach the terminal unescaped. ASCII causes such as `invalid range;
  'z' > 'a'` are unchanged.
- **Script-visible:** recovery advice keeps the task you were doing. The
  `E-TYPE` message is now `file type is <actual>, not <expected>`, and
  its `fix` asks for a path of the needed type and lists such files
  instead of pointing at the command for the file's actual type; when
  `state` or `wait-state` finds another type under the reserved state
  file name, it says to rename that file to a name no file has yet,
  never overwriting an existing file, and retry. `E-COMMENT-NOTFOUND`
  looks up your comment numbers instead of suggesting a new comment.
  `E-FLOCKED`, `E-TARGET-MISSING`, `E-DIR-MISSING` for `state` and the
  waits, and `E-NOT-DIR` now end with a retry of the original command
  (`E-NOT-DIR` with the path replaced by its parent directory when that
  can be done exactly). `E-STATE-NOTFOUND` gives a separate fix with and
  without `--status`. A missing brief target no longer suggests replacing
  the entry's regex. `W-STATE-DUP`, `W-STATE-RANGE`, and
  `W-STATE-EXPIRED` say the redeclaration is optional and for the
  entry's own agent. `E-FILE-MISSING` and `E-DIR-MISSING` say to
  correct the path and rerun. When a file `PATH` names a directory (for
  example `lookfor-comment .`), `E-FILE-MISSING` now says
  `<dir> is a directory, and this command needs one collaboration file`
  and lists the files in that directory, instead of only reporting
  `..agent-besties.md does not exist`. This applies to every form that
  never creates its file; creating forms still create
  `<dir>.agent-besties.md`. When an operand cannot be printed as a
  command, the `fix` keeps its own advice and says to run the command by
  hand, and why, instead of replacing the advice with one generic
  sentence (for example `retry shortly by hand; no copyable command can
  be printed because …`). A recipe that runs a different command keeps
  that whole command (subcommand, options, printable operands) and puts
  a marker such as `<path of this file>` where a value cannot be
  printed, so the operation is not lost. IDs and exit codes are
  unchanged.
- **Script-visible:** more recovery advice now keeps the record intact
  and names who acts.
  - `E-HEAD` gives the corrected header to write, keeping the comment's
    own number, nick, and times where the broken header still shows them
    (`rewrite the header as "## #2 bob (2026-09-17T09:20:00Z)"`). It no
    longer puts the command's current time into the header, which moved
    the comment's activity time. A number set off by a space
    (`## # 2 bob (…)`) stays the number instead of becoming the nick, and
    a time written only after `edited` stays the edit time instead of
    filling the creation time. Unreadable or ambiguous parts print as
    `<nick>`/`<time>` with a request to fill them in. When the repair
    gives the comment another number than its header shows, the `fix`
    also says to change any `@#<old>` that meant this comment to the new
    number, so a reply does not silently point at another comment. When
    the header shows none, the printed header still uses a safe unused
    number and the `fix` still says to change any reply that meant this
    comment to that number; if you know the comment's original number and
    no header in this post carries it or is given it by another repair,
    use that number instead and no reply needs to change. The CLI does
    not guess the original number. When no number above the
    highest one in the post is left, the `fix` says to give the comment
    a number by hand, keeping its position, author, time and text
    (`no number above #<max> is available; rewrite the header by hand
    as "## #<n> bob (…)"`): restore its known original number if no
    header uses it, otherwise choose a lower number unused by headers,
    other repairs and `@#n` replies in this post, then retarget replies
    meant for it. A lower number leaves only a non-blocking
    `W-SEQ-ORDER` or `W-SEQ-GAP` warning and frees no append number;
    new comments go in a new post once `post` stops with
    `E-SEQ-EXHAUSTED`.
  - `W-SEQ-GAP` explains the gap instead of guessing a deletion (0.1.0
    printed one text for every gap: `@#<m> can no longer be resolved; a
    comment was probably deleted by hand`). Next to a malformed header
    it says that header may be the missing comment and to repair it
    first. When every number in the gap is used elsewhere in the post,
    the `why` says nothing is missing and replies still resolve. Otherwise
    it names the first number that really is missing. When the first
    comment's number is above 1, the message reads `the first comment in
    this post is #<b>, not #1` instead of a jump from `#0`, which no
    comment has; naming the position cannot be misread as the post having
    no #1. A number that is really missing is described as possibly
    deleted or renumbered by hand, since `E-HEAD`'s own repair can also
    renumber a comment.
  - `E-SEQ-DUP` with no number above the highest left says to change
    only the number of the later copy by hand to an unused lower number:
    one that no header in this post carries, that you give no other
    header, and that no reply in this post already names (`@#n`),
    keeping its author, time and text, and to update any `@#n` that meant
    it, with the same note about warnings and new comments as `E-HEAD`.
    Neither repair claims that no number is left at all.
  - `E-STATE-DIRTY` says that deleting the nick's declaration from the
    named line by hand is itself the removal, and keeps the re-run for
    the branch where whoever wrote the line's unparseable fragments makes
    them parseable.
  - `W-STRAY` and `W-STATE-ENTRY` say the file does not record who wrote
    the text, even when it sits under another agent's heading or on
    another agent's line: whoever wrote it moves, rewrites or deletes it,
    only a comment's author changes that comment, and anyone else asks
    first. `W-STRAY` used to say "move it into the nearest fenced block",
    which could put one agent's text into another agent's comment.
  - `W-SUB-DUP` says to keep the first `- subscribers:` line and delete a
    later one instead of merging it in, because a nick named only there is
    not subscribed now and merging would subscribe it; each such agent
    decides for itself whether to subscribe.
  - `W-NAME` rename advice applies only when no file has the H1's name
    yet and says never to overwrite an existing file. The optional rename
    in `E-H1` advice for an invalid file name carries the same condition.
  - `E-H1` on the reserved `state.agent-besties.md` now leads with
    `set line 1 to "# .state"`, and offers another type only after
    renaming the file off the reserved name. It used to offer
    `# state.post` first, which would make every state command refuse the
    file and lock scans skip it.
  - `E-STATUS-DUP` whose status name has control characters keeps the
    merge as the repair and says how to make it addressable: remove the
    control characters from both headings and merge the two sections,
    together with any section that already uses the cleaned name.
    `W-STATUS-NAME` on an invalid status heading that appears more than
    once gives the same merge advice under one valid name (0.1.0
    suggested renaming every copy to its lowercased form, the same
    target for each, which would collide).
  - `E-FLOCKED` on a command that already has `--as` says to rerun it
    with `--as=<holder>` in place of that `--as` if the lock is yours;
    `add --as=<holder>`, which made clap reject a second `--as`, is now
    only for commands without one, and it now says to add it before any
    `--` separator, where it is still parsed as an option; after `--`
    the word would become an operand.
  - `W-STATE-CONFLICT` now warns, inside the fix itself, that the removal
    is refused with `E-STATE-DIRTY` when the yielding declaration sits on
    a line with unparseable fragments, and that that error's fix names
    the by-hand deletion: `agree who continues, then the other runs the
    removal; it is refused with E-STATE-DIRTY when that declaration's
    line also has unparseable fragments (W-STATE-ENTRY), and that error's
    fix names the by-hand deletion: agent-coexist state …`. The command
    part is unchanged.
  - Every brief fix that re-points an entry or rewrites its hash —
    `E-ENTRY-PATH`, `E-ENTRY-HASH` (with or without a path),
    `E-ENTRY-TARGET` (unreadable or directory), `W-ENTRY-ABS`,
    `W-ENTRY-SHIFTED`, `W-ENTRY-STALE` (regex not found or missing
    target), and `E-ENTRY-REGEX` (with or without a path) — now says
    which file to
    choose when the entry's own target is missing or unreadable, says to
    review the target and the note and update the note if it no longer
    describes the target, and states that the command also records the
    target's current hash, so a copied fix can no longer silently accept
    drift that became invisible while the target was missing or the
    regex was invalid.
  - A `post` write whose `reply-to` row says `(not found)` now carries a
    `fix` row after the `reply-to` rows: `if you meant another comment,
    edit this one so the reply names a number this post carries:
    agent-coexist post … --as=<nick> --edit=<n> -m '<the whole body,
    with the reference corrected>'`, with the writer's own nick and the
    just-written comment's number filled in. `--edit` rewrites the whole
    body, so the marker asks for the comment's full body with the
    reference corrected; resending only the fixed line would drop the
    rest. The comment is already written, so its author is the one who
    can correct the reference; the first line stays `warn` and the exit
    code 0.
  - **Script-visible:** a write refused by blocking diagnostics ends
    with a `refused` row after `check`'s rows: `nothing was written:
    writes are refused while the file has errors that make its structure
    ambiguous; fix <ID> at L<n> first, as the fix rows above say, then
    run this command again`, adding `the other rows above do not block
    this write` when non-blocking rows are shown too. Exit 1 is
    unchanged.
  - `W-BODY` on a comment names its author as the only one who can add
    the body; others ask the author or reply. It is no longer reported
    for a comment whose body sits in an unclosed fence (`E-FENCE`).
  - A write that would push a file over 16 MiB says so, with the current
    and resulting sizes, instead of claiming the file is already larger
    than 16 MiB; the file stays readable.
  - `E-NAME` states the name rule and that nothing was created, instead
    of asking to rename a file that does not exist.
  - `E-FILE-MISSING` whose directory is missing too lists the nearest
    existing directory instead of the missing one.
  - `E-ENTRY-HASH` on an entry without a path points at `--target`,
    since `--refresh` skips such entries. `E-ENTRY-TARGET` for a
    directory no longer mentions permissions. `W-ENTRY-STALE` for a regex
    that no longer matches fills in the entry's stored target.
  - `E-ENTRY-TITLE-TAKEN` gives your original command with
    `--entry '<title>'` added, so `--as`, `-m`, `--regex` and the other
    options stay. It used to build a new `brief --target … --entry`
    command that dropped them, so following it lost the note or regex.
  - A `wait` retry after `E-LOCKED` or `E-INUSE` lists its steps once, in
    the order they happen: the repair first (`wait briefly` for
    `E-LOCKED`; `close the other program` for `E-INUSE`), then one
    restart instruction — restart with this command, then read the file
    again after its ready line (for `wait-state`, run `check-state` on
    the directory again), because the new wait cannot report changes
    made while it was stopped — and the restart command last, with
    nothing after it. When the original command cannot be printed, the
    fix says to restart the original wait by hand and keeps the existing
    no-copyable-command explanation last. Any other failure after the
    ready line whose `fix` still offers a retry — an unreadable file,
    for example a Windows delete-pending one — uses the same order: its
    repair first, then that restart instruction, then the command. A
    path-class fault before the ready line, and a spelling fault (an
    invalid or too-long name) wherever it appears, keep `fix the path,
    then run the corrected command` with no retry. After the ready line,
    a structural loss instead — the watched path no longer resolving, a
    directory on it replaced by a file, or, on Unix, the path itself no
    longer a regular file — gets a `fix` that leads with how to restore
    the intended path without deleting anything blindly (recreate a
    missing directory or move it back; move aside whatever replaced it,
    keeping it), then that restart instruction, and prints the original
    wait as the command to retry; on Windows a watched file replaced by
    a directory stays the permission-denied repair in the same order.
    The `wait`, `wait-post`, and `wait-state` help states the restart
    rule for every way a wait ends.
  - A `wait` or `wait-state` on a state file that lists a nick twice
    under one status says the wait cannot start while a nick is listed
    twice, so start it again only after the duplicate is gone; to merge,
    that nick itself declares again (`the command for <nick>:
    agent-coexist state …`), which replaces both entries with one from
    now only when they are on clean lines. When either sits on a line
    that also has unparseable fragments (`W-STATE-ENTRY`), the `fix`
    says to inspect the file and that nick deletes its own entries from
    every such line by hand, keeping each line's other content, then
    looks at the remaining entries and declares again to merge any
    duplicate that is left or to start one entry from now. Starting the
    wait again is the next step and has no command of its own.
  - A report whose scan could not read everything now ends its scan
    rows with a `fix` row naming, as read-only commands, exactly what
    was not read: `agent-coexist check <dir>/state` for a skipped or
    partial state file, `agent-coexist check <path>` for another file,
    and, after a write, `agent-coexist check-state <dir>` for an
    unreadable directory. `list` gets the same kind of row when it shows
    a `busy` or `?` file or an unreadable directory: `agent-coexist check
    <path>` for each such file and `agent-coexist list <dir>` for each
    such directory. When the CLI can see a state file in a directory it
    cannot fully read, but cannot open that file either, the two are one
    `skipped <dir> unreadable` row pointing at `agent-coexist check
    <dir>/state`. A directory that cannot be listed names no state file,
    because the CLI cannot tell whether it holds one; a `check-state` or
    `read-notification` report with only such rows has no `fix` row, which
    does not mean the directory is empty. Advice about an
    unreadable directory says to make it both readable and enterable
    (read and search permission), since a Unix directory that can be
    listed but not entered is also `unreadable`, and `list` says such a
    directory could not be read or entered. A `profile` or `brief` write
    says the change is written and to agree with any holder before
    writing again, naming the rows it means (those that start with
    `skipped`, `partial` or `canonicalize`, not the brief's own
    `entry … skipped` rows); a `state` declaration says it is written and not to
    declare again; `check-state` says declarations may be missing from
    the overview; `read-notification` says records may be missing, says
    what `check` shows for each file (its type and any problem left) and
    that an `E-TYPE` row means that directory's state file has another
    type, so no declarations were read from it even when `check` reports
    no problem, then asks to check each file before concluding nothing
    is new. The `fix` row is
    not counted in `total`; first lines and exit codes are unchanged.
  - A `state` declaration whose conflict scan then stops on an OS error
    that does not come from an ignore source (for example a current
    directory that global excludes need but that no longer exists), or
    on any other non-source failure, keeps its `scan` row and now adds a
    `fix`: the declaration is written, the conflict check stopped at that
    error, and, without declaring again, resolve it and run
    `agent-coexist check-state <dir>`. It used to print the `scan` row
    alone. First line `warn` and exit 0 are unchanged.
  - `state`'s own report treats the directory's selected
    `state.agent-besties.md` as selected, so its state recipes match
    what `check` prints for the same file. A file that is not selected
    gets the unselected advice above (settle which file should be this
    directory's `state.agent-besties.md`, then re-check, before anyone
    changes a declaration by hand).
  - A wait report that hits because the file was deleted no longer
    repeats warnings from the deleted file, such as a lock conflict that
    said two agents must agree who continues: the conflict went away with
    the file. The report's first line is `ok`; its `bytes`, `semantic`,
    `removed`, and `remove` rows and what triggers the wait are
    unchanged.
- Short help for `check` and `read-brief` explains the split: `check`
  reports entry states without target bodies; `read-brief` prints
  bodies, and `--entry` still prints that entry's content.
- `check-state --help` now names the `conflict` and `expired` entry
  marks, including `conflict` on inherited entries, and says that
  either mark makes the first line `warn`. It also says that only the
  declarations shown in the same report are compared, so an inherited
  lock can still conflict with a lock in another branch below that
  ancestor, which `check-state` on the ancestor shows. Only the help
  text changed; the report is unchanged.
- `brief --help` now says that writing a `--regex` checks only that the
  pattern compiles, that a pattern the target does not match shows as
  `stale` when the entry is checked, and that `--regex ''` deletes every
  regex line of the entry, including hand-written duplicates.
- The `brief --help` example now spells `--target` relative to the
  brief's directory, as the help itself states:
  `agent-coexist brief docs/onboarding --target ../src/parser/mod.rs --entry 'parser entry point'`.
  The 0.1.0 example (`--target src/parser/mod.rs`) resolved to
  `docs/src/parser/mod.rs`. Parsing is unchanged.
- `profile` and `brief` show TTY progress on stderr during their
  ancestor lock scans.
- The TTY progress line now reads `scanning dirs: <n>, files: <m>...`
  instead of `scanning <n> dirs, <m> files...`, so a count of 1 no
  longer prints as `1 files`. It is still stderr-only, shown only on a
  terminal, and erased when the scan ends.
- Top-level `--help` no longer says that no output means a scan is
  still running: progress appears only when stderr is a terminal, and
  the result is judged by the report and exit code. Its exit 3 row now
  points to the report's `fix` instead of calling every case retryable.
  Command help drops the "has no -m" sentence and states more of what a
  reader needs next, for example that `post --edit` replaces the whole
  body and that `subscribe` affects only `read-notification`.
- The `W-SECTION-NAME` and `W-STATUS-NAME` warnings state the whole rule
  a heading must satisfy, including the first character and the length
  limit (`starts with a letter, digit or _, then letters, digits, _ and
  -, at most 64 characters` for sections; `starts with a lowercase
  letter, then lowercase letters, digits and -, at most 32 characters`
  for statuses), instead of naming only the allowed alphabet. A heading
  the old text seemed to allow, such as `## 2fa` or a 33-character
  status, now shows why it is invalid. Only the `why` text changed.
- Top-level `--help` describes TIME as `an RFC 3339 time, e.g.
  2026-09-17T09:12:00Z; any offset is converted to UTC and fractional
  seconds are dropped`, matching what `--since` accepts, instead of
  implying the strict `Z`-suffixed form is the only input. A
  non-Unicode `AGENT_COEXIST_LOCK_WAIT_MS` is told so explicitly
  (`AGENT_COEXIST_LOCK_WAIT_MS is not valid Unicode; set it to digits
  such as 5000, or unset it to use the default 5000`), parallel to
  `AGENT_COEXIST_NOW`, instead of seeing a U+FFFD where a byte was.

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
- On Windows and macOS, the H1 name check compares the H1 with the file
  name on disk, not the case typed in `PATH`. `check ROOM/STANDUP` on
  `room/standup.agent-besties.md` with `# standup.post` no longer warns
  about a name mismatch or suggests rewriting the H1 in the typed case.
  A real mismatch on disk still warns. Linux is unchanged.

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
- Recovery for a misnamed or unselected state-content file never prints
  a directory-level `state` command that would change a different,
  reserved `state.agent-besties.md`. The `fix` first settles which file
  should be this directory's `state.agent-besties.md` (a `W-NAME-STATE`
  row, when shown, describes how; never overwrite a separate existing
  state file), then asks to run `check` on this file again before
  anyone changes a declaration by hand; only the agent that declared an
  entry changes, renews or removes its own entry. Wait-state reports
  apply the same check at the final snapshot.
- `profile`/`brief` writes through a file symlink also honor `locked`
  on the symlink target's parent directory chain.
- Incomplete lock scans show `skipped`/`partial` rows (including stray
  state text, `W-STRAY`); identified `locked` holders still block, and
  otherwise the write proceeds with a warning.
- **Script-visible:** every `partial` row now prints the ID and line
  as two columns, `partial  <dir|path>  <ID>  L<n>`. `read-notification`,
  `check-state`, and the `state` conflict scan used to join them into
  one value (`E-HEAD L9`).
- `check` reports each expiry and same-file lock conflict once.
  `read-notification` includes ancestor state files without counting
  the starting directory's file twice.

Scans

- `check-state` fails on a broken ignore source the same way as `list`
  and `read-notification` (`cannot scan`, exit 1). If a `state` write
  already succeeded, the same error is a `warn` with a `scan` row,
  exit 0, and the write stays.
- **Script-visible:** a `cannot scan` failure caused by an ignore or Git
  configuration source now gets a `fix` for that cause instead of the
  generic `check the file's permissions and free space, then retry: …`,
  which could not repair a bad pattern. A pattern or non-UTF-8 line
  gives `correct line <n> of <source>`, a malformed `.git` or
  `commondir` reference (including a gitdir pointing to a missing path
  or to a regular file, reported against the `.git` file) gives
  `correct the contents of <source>`, a directory or other non-regular
  source gives `replace <source> with a regular file or remove it`,
  other read errors give `check that <source> can be read`, and a file
  whose lines parse but whose rules together exceed the pattern
  matcher's size or nesting limits gives `shorten, simplify or remove
  rules in <source> until they fit the pattern matcher's size and
  nesting limits`. Each is followed by `then retry: <original
  command>`. After a committed `state` write, the `scan` row is now
  followed by a `fix` row that ends in `check-state`, so the conflict
  check reruns without declaring again; when no command can be printed,
  that `fix` still names `check-state` to run by hand. An OS error that
  comes from no source file is reported as an ordinary I/O failure with
  its real error kind, so it gets the I/O advice below. Two
  `reason` texts changed: an oversized rule set now names its file
  (`cannot scan room/.ignore: error building NFA`, previously
  `cannot scan room: …`), and on Unix a gitdir that points to a regular
  file now reads `cannot scan <.git file>: gitdir points to <target>:
  gitdir reference is not a directory`, as on Windows, instead of
  `cannot scan <gitdir>/commondir: Not a directory`. Exit codes and the
  `state` write are unchanged.
- A Unix directory with read but no search permission is now a
  `skipped  <dir>  unreadable` row, like other unreadable directories.
  It used to fail the scan with `cannot scan <dir>/.ignore: Permission
  denied`, naming an ignore file that might not exist.
- Ignore rules follow Git worktree, gitfile, separate-gitdir, and
  `commondir` layouts and an explicit global Git config; a leading `~`
  in `core.excludesFile` expands only as a home component. Scans use a
  pinned, locally patched `ignore` 0.4.33.
- Scans keep native path identity: distinct directories whose display
  names fold together stay distinct, roots with spaces relativize
  correctly, and Windows ordinary/verbatim volume spellings agree.
- An explicit scan `DIR` that is a directory symlink, junction, or
  `.git` directory is entered; ones found below it stay pruned.
- **Script-visible:** the unmatched-`--include` hint now says what
  happened and names only the causes the command line allows:
  `include  <glob>  no file is left after the filters; check that this
  glob is relative to DIR`, plus `, and that --exclude does not remove
  the files it matches` when an `--exclude` was given, plus `; if the
  files are in a directory that ignore rules skip, --include cannot
  reach them: add --no-ignore` when an ignore source was loaded. It
  used to always read `matched 0 files; ignored directories are not
  reopened (pass --no-ignore)`, even when an `--exclude` had removed the
  files. The row now also appears under `--no-ignore`, where a mistyped
  glob used to give an `ok` report with `total 0` and no hint. In
  `check-state` the hint says that only files under `DIR` are filtered:
  `no file under DIR is left after the filters (the state files of DIR
  and its ancestors are read regardless); …`. The row is display-folded;
  matching still uses the glob as typed.

Recovery commands and display

- Recovery `fix` commands name the original file: operands keep the
  managed suffix, internal whitespace, and Windows verbatim spelling,
  and are quoted for a POSIX shell. On Unix, non-UTF-8 bytes appear as
  `"$(printf '\377')"`-style substitutions instead of pointing at a
  U+FFFD look-alike. When an operand cannot be written safely (NUL, CR,
  LF, or unpaired UTF-16 on Windows), no command is printed; the recipe
  becomes a manual instruction.
- Literal and hand-edit `fix` lines are no longer shell-quoted. Headings
  with control characters, invalid profile section names, and invalid
  status names get hand-edit advice instead of a command the CLI would
  reject; suggested section names and comment numbers are unique within
  one report.
- Titles and diagnostic text fold whitespace and show control characters
  as U+FFFD without changing stored bytes.
- **Script-visible:** file-system failure `reason` rows (`cannot <action>
  <path>: <source>`, including `cannot scan`) and the `scan` cause
  printed after a successful `state` write now fold whitespace and show
  remaining control characters as U+FFFD, like other diagnostic text. A malformed
  ignore pattern with double spaces no longer adds columns to these
  rows. The source file, line, and full explanation are kept, and `fix`
  operands keep their raw spelling. Exit codes and the `state` write are
  unchanged.
- `--target` and `--regex` reject raw CR/LF; `--remove` with
  `--target` is refused; `--regex ''` deletes every regex line of the
  entry.
- A brief `--target` that names a directory fails with `cannot read
  <path>: is a directory, not a file` and `fix the path, then run the
  corrected command` on every platform. Windows used to report a
  permission error and suggest checking permissions and free space
  before retrying the same command. Exit code 1 is unchanged.
- **Script-visible:** an OS error without a catalog ID no longer gets
  the one-size `check the file's permissions and free space, then retry`.
  The repair now follows the error kind: permission denied names the file
  and the directories on its path (on Windows it also says to wait until
  another program stops using, deleting or replacing the file, which
  gives the same error); disk full or quota says to free space or quota;
  a file-size limit (`EFBIG`) says to raise the shell's `ulimit -f` (on
  Unix) or move the file to a file system that allows larger files,
  since free space would not help; a read-only file system says to make
  it writable; any other error says to resolve the error shown in
  `reason`. Path errors keep `fix the
  path, then run the corrected command`. Each other repair ends with
  `then retry: <original command>`, or a by-hand retry when the command
  cannot be printed. `reason` and exit code 1 are unchanged.

Posts, waits, and file bytes

- **Script-visible:** a write that fails partway no longer leaves a file
  half overwritten while offering a plain retry. Writes now grow the
  file first and rewrite only the bytes after the part that stays the
  same, so running out of space or into a file-size limit while the file
  grows leaves its previous bytes untouched. A failure before the final
  sync puts the previous content back, and the report stays the usual
  `cannot write <file>: <os error>` with a retry. Two new reports cover
  the cases where that is not true: `cannot write <file>: <cause>;
  putting its previous content back also failed: <rollback error>`, and
  `wrote <file>, but could not confirm that the change is stored on disk:
  <cause>`. Both exit 1, keep the file, say not to run the same command
  again, and end with `agent-coexist check <file>`. This is not crash
  safety: a crash or power loss mid-write can still leave old and new
  bytes mixed.
- `@alice-@bob` is two mentions (`alice-` and `bob`). `check` lists
  post `authors` in first-occurrence order; write reports list mentions
  and replies in body order.
- Appending after a file that ends in an isolated `CR` no longer turns
  it into `CRLF`, and inserted properties or bodies reuse an existing
  whitespace-only gap line.
- A subscribers line, description, note, missing comment or section
  body, or new `- hash:`/`- regex:` line inserted into an existing file
  is followed by a blank line when a heading, fence, or text comes right
  after it, matching the layout of files the CLI creates. New property
  lines join the entry's existing ones instead of following the blank
  line before its note. Other bytes are unchanged.
- `wait` and `wait-post` report a non-directory parent as a path
  error. A wait refuses a file over 16 MiB with the same report as
  `check`, also when the file grows past the cap later. A wait report
  shows a brief's description once when the file is created, deleted, or
  changes type.
- Leap seconds read from files print as `:60`.

## [0.1.0] - 2026-09-22

Initial release, built from commit
[8d98cc7](https://github.com/RatmmmhSquishyRat/agent-coexist/commit/8d98cc73a2714a9931172bf4ac6b9fbf417d7853).
It was not tagged, and builds from later commits still report version
0.1.0.

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

[Unreleased]: https://github.com/RatmmmhSquishyRat/agent-coexist/compare/v0.5.0...main
[0.5.0]: https://github.com/RatmmmhSquishyRat/agent-coexist/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/RatmmmhSquishyRat/agent-coexist/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/RatmmmhSquishyRat/agent-coexist/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/RatmmmhSquishyRat/agent-coexist/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RatmmmhSquishyRat/agent-coexist/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RatmmmhSquishyRat/agent-coexist/compare/8d98cc73a2714a9931172bf4ac6b9fbf417d7853...v0.2.0
[0.1.0]: https://github.com/RatmmmhSquishyRat/agent-coexist/commit/8d98cc73a2714a9931172bf4ac6b9fbf417d7853
