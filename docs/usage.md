# Command reference

This document is the authoritative behavioral reference for `agent-coexist`: line protocol rules, safety contracts, parameter precedence, error recovery, and exit codes. For task-oriented walkthroughs, run `agent-coexist guide` or consult [crates/cli/guide/](../crates/cli/guide/index.md). For file formats on disk, see [the file format reference](file-format.md).

## Contents

- [Built-in guide](#built-in-guide) · [Command index](#command-index)
- [Conventions](#conventions) — [Files and paths](#files-and-paths), [Names and identity](#names-and-identity), [Reading output](#reading-output), [Exit codes](#exit-codes)
- [Profiles](#profiles) · [Posts and subscriptions](#posts-and-subscriptions) — [Comments, mentions, and replies](#comments-mentions-and-replies)
- [Briefs](#briefs) — [Refresh report example](#refresh-report-example), [Reading briefs](#reading-briefs)
- [State](#state) — [Declaring](#declaring), [Removing](#removing), [Scope, inheritance, and conflicts](#scope-inheritance-and-conflicts), [Functional lock](#functional-lock)
- [Discovery and notifications](#discovery-and-notifications) — [Scans: what is visited, and filter precedence](#scans-what-is-visited-and-filter-precedence)
- [Waiting for changes](#waiting-for-changes) — [Lifecycle](#lifecycle), [The wait report](#the-wait-report)
- [Locks, retries and time](#locks-retries-and-time) — [Incomplete lock and conflict scans](#incomplete-lock-and-conflict-scans)
- [Recovery commands (`fix`)](#recovery-commands-fix) — [When a write fails partway](#when-a-write-fails-partway), [How operands are written](#how-operands-are-written), [When there is no command](#when-there-is-no-command)
- [Platform notes](#platform-notes) · [Troubleshooting](#troubleshooting) · [Environment overrides](#environment-overrides) · [A pattern for scripted agents](#a-pattern-for-scripted-agents)

## Built-in guide

`agent-coexist guide [start|read|write|wait]` prints static tutorial pages to stdout (LF, trailing newline) with empty stderr and exit 0.

- Topics are exact lowercase. Unknown topics, extra arguments, or unsupported flags are usage errors (exit 2) with empty stdout.
- Stdout write or flush failure exits 1 without a status line.
- `guide`, `help [COMMAND]`, `--help`, and `--version` are informational: they print no `ok`/`warn`/`error` status line, perform no workspace I/O or locking, and ignore invalid `AGENT_COEXIST_*` environment variables.

## Command index

| Command | Purpose | Reference section |
|---|---|---|
| `guide` | Print built-in tutorial page | [Built-in guide](#built-in-guide) |
| `profile` | Create profile; set or remove sections | [Profiles](#profiles) |
| `post` | Append comment or edit own comment | [Posts and subscriptions](#posts-and-subscriptions) |
| `subscribe` | Join or leave a post subscriber list | [Posts and subscriptions](#posts-and-subscriptions) |
| `brief` | Create brief; manage target entries and hashes | [Briefs](#briefs) |
| `read-brief` | Render brief with entry states and target text | [Reading briefs](#reading-briefs) |
| `state` | Declare or remove directory state intent | [State](#state) |
| `check-state` | Show state for directory, ancestors, and subtree | [Scope, inheritance, and conflicts](#scope-inheritance-and-conflicts) |
| `list` | Enumerate collaboration files as a tree | [Discovery and notifications](#discovery-and-notifications) |
| `check` | Inspect one file: type, diagnostics, preview | [Discovery and notifications](#discovery-and-notifications) |
| `lookfor-comment` | Search comments in a post | [Discovery and notifications](#discovery-and-notifications) |
| `read-notification` | Query events since timestamp or duration | [Discovery and notifications](#discovery-and-notifications) |
| `wait` | Block until collaboration file changes | [Waiting for changes](#waiting-for-changes) |
| `wait-post` | Block until relevant comment appears | [Waiting for changes](#waiting-for-changes) |
| `wait-state` | Block until directory state declarations change | [Waiting for changes](#waiting-for-changes) |

`help <command>` prints the same page as `<command> --help`. `agent-coexist` with no command is a usage error (exit 2) reporting that a subcommand is required.

## Conventions

### Files and paths

- Collaboration files end in `.agent-besties.md`. `PATH` arguments may omit this suffix; it is appended byte-exact when missing. Scans match the suffix case-insensitively.
- `DIR` arguments default to `.` and must exist (`E-DIR-MISSING` / `E-NOT-DIR`, exit 1). The CLI never creates a missing `DIR`.
- Creating forms create missing parent directories and the target file (`created  <dir>/` before `created  <file>`): `post` (new thread), `subscribe` (add), `profile` (bare create or section set), `brief` (bare, description, `--target`).
- Never-creating forms fail with `E-FILE-MISSING` (exit 1) on missing files: `post --edit`, `subscribe --cancel`, `profile --remove`, and brief note-only, `--entry --remove`, and `--refresh` forms. When `PATH` names an existing directory, the reason explicitly notes it is a directory.
- If a creating write is refused (wrong type, `E-NAME`, a [functional lock](#functional-lock), a busy lock, a blocking parse error), the CLI deletes the empty file and empty parent directories this invocation just created, after checking they are still the same empty objects. Pre-existing files, files that already contain bytes, replacements at the same path, and directories that are no longer empty stay. An empty file or directory can therefore appear briefly and vanish; this cleanup is not crash-safe or atomic against other programs.

### Names and identity

- Creating forms derive the document H1 from the file stem. Stems must be valid names (1–100 characters: `a-z`, `A-Z`, `0-9`, `_`, `-`, with `.` permitted after the first character), else `E-NAME` (exit 1). Existing files with mismatched stems and H1 headings remain editable (`W-NAME`).
- `--as <nick>` identifies the caller (1–64 characters: letter, digit, or `_` first, then letters, digits, `_`, or `-`). It is recorded in comment headers, matched by notifications, and exempts the holder from their own [functional locks](#functional-lock). It is never authenticated.
- Statuses (`--status`) are 1–32 ASCII lowercase characters: `a-z`, then `a-z0-9-`. Section names follow the nick grammar.
- Entry titles reject control characters and runs of two or more spaces. Display titles fold whitespace runs to a single space.
- Free-text option values starting with `-` must use attached `=`: `-m='- item'`, `--keyword=-x`, `--regex='-x'`. Passing them unattached treats them as CLI flags and fails with a usage error (exit 2).
- Non-printable or rejected characters echo using `<U+XXXX>` notation. `PATH` and `DIR` accept arbitrary native file-system bytes; all other arguments must be valid Unicode.

### Reading output

All business commands emit a line-oriented report to stdout (LF line endings). There is no JSON output mode.

```text
<status>  <command>  <object>
  <key>   <value>...
```

- **First line**: `<status>  <command>  <object>`, where `<status>` is `ok`, `warn`, or `error`. An empty object prints as `/`. Splitting on two or more spaces always yields three columns.
- **Fields rows**: indented 2 spaces: `  <key>  <value>...`. Values are separated by two spaces. File-derived values have whitespace folded. Empty keys or values print as `/`.
- **Text rows**: `  <key>` followed by verbatim lines indented 4 spaces with prefix `    | <line>`. Content is never column-split; blank lines print as bare `    | `.
- **Diagnostics**: `  error  L<n>  <message>` or `  warn  L<n>  <message>`, with `why` and `fix` indented 9 spaces underneath. Unanchored errors print `reason  <message>` and `fix  <fix>` rows without `why`.
- **Fix rows**: never split a `fix` row into columns. Everything following the `fix` key is one complete payload ([Recovery commands](#recovery-commands-fix)).
- **Scan rows**: incomplete scans print `skipped  <path|dir>  <cause>` or `partial  <dir|path>  <ID>  L<n>`. `busy` indicates `E-LOCKED` or Windows `E-INUSE`. A scan with skipped or partial items appends a `fix` row naming read-only inspection commands.
- **Canonicalize rows**: `canonicalize  <dir>  failed  <os error>` indicates the directory's real path could not be resolved, falling back to spelled path. This does not refuse writes.
- **Total row**: completed overviews (`list`, `lookfor-comment`, `read-notification`, `check-state`) end with a `total <N>` row (`fix`, `canonicalize`, and `include` hint rows are never counted):
  - `list`: all file rows (including `busy` and `?`) plus each `skipped <dir> unreadable` row.
  - `lookfor-comment`: number of matching comments.
  - `read-notification`: number of event records plus `skipped`/`partial` rows.
  - `check-state`: all declaration rows (inherited, own, descendant) plus `skipped`/`partial` rows.
  - Fatal reports, single-file inspection (`check`, `read-brief`), write commands, and wait reports never emit a `total` row.
- **Progress**: `scanning dirs: N, files: M...` appears on stderr only when stderr is a TTY.

### Exit codes

| Code | Status line / Cause | Script action |
|---|---|---|
| `0` | `ok` or `warn` | Success. If `warn`, read report rows for conflicts, skips, or unread sources. |
| `0` | `guide`, `help`, `--help`, `--version` | Informational output; no status line. |
| `1` | `error` | Failure (diagnostics, missing files, type mismatch, I/O errors). |
| `1` | Business or information stdout write/flush failure | Output stream broken. Inspect files before retrying writes. |
| `2` | Usage error | Command line syntax or argument validation failure; output on stderr. |
| `3` | `E-LOCKED`, `E-INUSE`, `E-FLOCKED` | Concurrency or functional lock refusal. Follow `fix` advice. |
| `130`| Wait cancelled | Interrupted by Ctrl-C; no stdout. |

`warn` is exit 0. Scripts checking for conflicts must inspect the first line status (`ok` vs `warn`). Write operations commit file contents before printing reports; if stdout write fails with exit 1, the change may already exist on disk. Check with `check` before retrying. All business and information commands check both the stdout write and the final flush, and exit 1 if either fails; usage errors remain exit 2 even if their stderr destination fails. A successful write and flush mean the operating system accepted the output, not that a downstream reader processed it.

## Profiles

Profiles record agent metadata in a collaboration file (often `agents/<nick>.agent-besties.md` by convention; `PATH` is caller-selected and not tied to `--as`):

- `profile <PATH>`: creates the profile or reports `unchanged <file>`.
- `profile <PATH> --section <name> -m <content>`: sets or updates a section (stored in a code fence).
- `profile <PATH> --section <name> --remove`: deletes the section (`removed-lines L<a>..L<b>`). Unknown section: `E-SECTION-NOTFOUND` (exit 1).
- `--as <nick>`: optional. It is not written to the profile; it only exempts the caller from functional locks held under that nick ([Functional lock](#functional-lock)).

## Posts and subscriptions

Posts provide sequential discussions with numbered comments (and optional editing of your own comments via `--edit`).

### Comments, mentions, and replies

- `post <PATH> --as <nick> -m <content>` appends a new comment with sequence number `max(existing seq) + 1` (starting at `#1` in a new post). Comment numbers are persistent identifiers, not array positions; missing numbers produce non-blocking warnings, and the CLI never renumbers comments.
- Mentions (`@nick`) and replies (`@#n`) are parsed from comment bodies.
- Output rows: `comment #<n> L<line> [edited]`, followed by `mentions <list>` and `reply-to #<m>` rows in first-occurrence order.
- Dangling replies: if `@#m` refers to a nonexistent comment, the report emits `reply-to #<m> (not found)` and marks the first line `warn`. The comment is still written. The `fix` provides a copyable `post --edit` command.
- `check` lists all comment authors in first-occurrence order.
- `post <PATH> --as <nick> -m <content> --edit <n>`: replaces the entire body of comment `#n` (where `<n>` is its recorded sequence number). Comment `#n` author must match `--as` (`E-EDIT-AUTHOR`, exit 1). It never creates files.
- `subscribe <PATH> --as <nick>`: adds `--as` to subscribers. Does not push notifications; read-notification uses the list. Subscribing twice reports `unchanged <nick>`.
- `subscribe <PATH> --as <nick> --cancel`: removes `--as` from the first subscriber property. Cancelling an unsubscribed nick reports `unchanged <nick>`. Missing file: `E-FILE-MISSING` (exit 1).
- `post` and `subscribe` wait up to 5s for an exclusive file lock (`AGENT_COEXIST_LOCK_WAIT_MS` overrides).

## Briefs

Briefs track pointers to external files verified by SHA-256 hashes. Six mutually exclusive forms:
1. `brief <PATH>`: create bare or report `unchanged`.
2. `brief <PATH> -m <desc>`: set brief description.
3. `brief <PATH> --target <file> [--entry <title>] [--regex <pat>] [-m <note>]`: add or update entry.
4. `brief <PATH> --entry <title> -m <note>`: update entry note.
5. `brief <PATH> --entry <title> --remove`: delete entry.
6. `brief <PATH> [--entry <title>] --refresh`: recompute and update hashes against current target bytes.

- `--target` resolves relative to the brief file's directory (not cwd). Missing targets fail with `E-TARGET-MISSING` (exit 1). Directory targets fail with an error report.
- Default entry title is the target filename; `--entry` overrides. Without `--entry`, reusing an existing title that points at a different target fails with `E-ENTRY-TITLE-TAKEN` (exit 1); with `--entry`, it updates or retargets that entry.
- `--regex` requires `--target`. Patterns must compile on write; matching occurs on read. `--regex ''` removes all regex property lines.
- `--refresh` recalculates SHA-256 against disk bytes; it never re-checks regexes. If all targets are current or skipped, reports `unchanged <file>`.
- Brief writes re-parse the file and append any surviving non-blocking diagnostics as `remaining <E-ID> L<n>` rows (first line `warn`).

### Refresh report example

```text
warn  brief  onboarding
  entry         parser entry point  refreshed  src/parser/mod.rs
  entry         renderer            skipped    target missing
  entry         design notes        skipped    no path
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  skipped       .  E-H1
  fix           changes reported above are already written; rows starting with skipped, partial or canonicalize mean the lock check may have missed a holder. Before writing again, run these checks, then agree with any holder they show: agent-coexist check state
  remaining     E-ENTRY-PATH  L15
  remaining     E-ENTRY-HASH  L15
```

Refresh skips (`entry ... skipped`) indicate missing paths or files. The first `fix` addresses refresh skips and remaining errors. Lock scan skips (`skipped . E-H1`) indicate ancestor state check issues.

### Reading briefs

Verification states are computed on read:
- `fresh`: stored SHA-256 matches current file bytes.
- `shifted`: SHA-256 differs (target modified since hash was recorded).
- `stale`: target missing, regex invalid, or regex pattern not found in target.
- `unchecked`: no path, target unreadable, no hash stored, or target > 64 MiB with a regex.

Target text rules and size limits (64 MiB):
- Target hashing streams with no size cap.
- Targets > 64 MiB cannot be held in memory for regex matching or display:
  - With a regex: state is `unchecked` (`target too large`) in both `check` and `read-brief`.
  - Without a regex: hash is checked normally (`fresh` or `shifted` in `check`); `read-brief` omits the target content block, appends `target too large` after the path, and marks the first line `warn`.
- Within the limit: `check` validates entries and displays states without target bodies; `read-brief` prints descriptions, entry rows, notes, and target `content` blocks (whole target without regex; whole lines of first match `L<a>..L<b>` with regex).
- `read-brief --entry <title>` filters to a single entry.

## State

State files (`DIR/state.agent-besties.md`) record agent intentions across directories and their subtrees.

- Built-in statuses: `locked` (cooperative request not to edit) and `observing` (active inspection; edits welcome).
- Durations default to 2h for `locked`, 8h for other statuses; `--for` overrides.

### Declaring

- `state [DIR] --as <nick> --status <name> [--for <dur>]` declares state.
- Reserved filename: `DIR/state.agent-besties.md`. Lookups are case-insensitive on Windows/macOS.
- Validity: declaration is valid while `start <= until` and `now < until`.
- Re-declaring replaces clean declarations for `<nick>` under `<status>` with `start = now` and `until = now + duration`.
- Expired entries on clean lines are automatically removed (`cleaned` rows).
- Post-write conflict check: scans state files of `DIR`, ancestors, and un-ignored descendants. Overlapping valid `locked` declarations by other nicks emit `conflict locked <nick> <range> <dir>` rows (warn, exit 0).
- Scan errors after declaration do not roll back the written state.

### Removing

- `state [DIR] --remove <nick> [--status <name>]`: removes declarations.
- `--as` defaults to the removed nick; removing another nick emits a `removed-other` warning row.
- Without `--status`, removes `<nick>` from all statuses.
- Clean lines only: if `<nick>` appears on a mixed line with unparseable fragments, removal is refused before any write (`E-STATE-DIRTY`, exit 1).
- Empty status sections are removed; if only the H1 header, reader comments, and blank lines remain, the file is deleted (`deleted <file>`).
- If no matching entry exists: `E-STATE-NOTFOUND` (exit 1).

### Scope, inheritance, and conflicts

`check-state [DIR]` reports declarations in order:
1. `inherited <status> <nick> <start>..<until> <dir> [conflict]`: valid entries from ancestors (nearest first).
2. `<status> <nick> <start>..<until> [conflict|expired]`: DIR's own declarations.
3. Descendant state files grouped by relative path.
4. `skipped`/`partial` rows and scan `fix` suggestions.
5. `total` row.

- `conflict` mark: indicates a valid `locked` entry overlapping another valid `locked` entry in this report.
- `expired` mark: indicates an ended declaration.

### Functional lock

- Byte-writing `profile` and `brief` commands fail with `E-FLOCKED` (exit 3) if another nick holds a valid `locked` declaration on the directory or any ancestor.
- File symlinks check both the parent of the symlink and the parent of the canonical target.
- Without `--as`, any unexpired `locked` declaration blocks. Passing `--as <holder>` exempts declarations held under that same nick (locks held by other nicks still block).
- Multiple locks: `reason` names the lock with the latest expiry; other holders appear as `also-locked <nick> until <time>` rows.
- `post`, `subscribe`, and `state` are NEVER blocked by functional locks.

## Discovery and notifications

### Scans: what is visited, and filter precedence

`list`, `read-notification`, and `check-state` walk `DIR` recursively for `*.agent-besties.md`:
- Hidden directories are traversed.
- Explicit `DIR` symlinks, junctions, or `.git` directories are entered; child symlinks and junctions below `DIR` are skipped.
- **Precedence**:
  1. Ignore rules (`.gitignore`, `.ignore`, `.git/info/exclude`, global gitignore). `--no-ignore` disables ignore pruning.
  2. `--include` / `--exclude` globs (relative to `DIR`). A file is kept when (no `--include` was given, or at least one matches) and no `--exclude` matches. Exclude always wins over include; repeated flags form a union.
  3. If `--include` was given and no files survive, emits `include <glob> <hint>` rows (warn).
- Malformed ignore files or git configurations halt the scan (`error cannot scan <path>`, exit 1).

### list, check, lookfor-comment, and read-notification

- `list [DIR]`: prints a tree of collaboration files with type column. `--type` filters by type (`profile` aliases `agent-profile`). Files marked `busy` or `?` count toward `total` and mark the report `warn`.
- `check <PATH>`: inspects format, line diagnostics, and type preview. If blocking errors exist, write commands fail with a `check`-shaped report ending in a `refused` row.
- `lookfor-comment <PATH>`: filters comments by `--keyword` (exact substring of body/header), `--by`, `--mention`, `--reply-to`, or `--comment`.
- `read-notification [DIR] --as <nick> --since <time|dur>`: lists events in `[since, now)` concerning `--as` (mentions, replies, subscribed post activity, state changes, overlapping locks, own expirations). Your own comments never notify you. The scan window ends before the current second (overlap windows when polling; no stored read cursor). Priority: `reply` > `mention` > `subscribed`.

## Waiting for changes

Wait commands block until changes occur without continuous busy polling:
- `wait <PATH>`: any byte change to collaboration file.
- `wait-post <PATH> [--as <nick>]`: comments relevant to `<nick>` (or any comment without `--as`).
- `wait-state [DIR] [--status <s>]`: changes to declarations in `DIR/state.agent-besties.md` under status `<s>` (or any change without `--status`).

### Lifecycle

- Startup order to avoid missed updates:
  1. Read file or notifications.
  2. Start wait command and wait for stderr baseline line: `waiting <command> <object>`.
  3. Read file once more to catch modifications made before the baseline was established.
  4. When wait exits 0, read current file content.
- Sampling cadence is ~1s. Missing files are valid baselines (waiting for creation).
- Per-sample shared lock wait is 1s; timeout exits 3 (`E-LOCKED` or Windows `E-INUSE`). On restart, re-read after the ready line.
- Invalid snapshots (parse errors, wrong file type) terminate the wait with exit 1.
- No timeout flag; cancel with Ctrl-C (exit 130).

### The wait report

Wait exit reports:
- First lines: `bytes changed` and `semantic changed|unchanged`.
- Creation/deletion: `created <file>` or `removed <file>`.
- Entity changes: `add`, `remove`, or `update` with verbatim `before` and `after` blocks.
- Filtered waits (`wait-post --as`, `wait-state --status`) omit document headers and unrelated entities.

## Locks, retries and time

| Operation | Lock type | Timeout / Behavior |
|---|---|---|
| `post`, `subscribe` | Exclusive | Queues up to 5s, then `E-LOCKED` (exit 3). |
| `profile`, `brief`, `state` | Exclusive | Fast fail (~200ms), then `E-LOCKED` (exit 3). |
| `check`, `lookfor-comment`, `read-brief` | Shared | Waits 1s, then `E-LOCKED` (exit 3). |
| `wait` sample | Shared | Waits 1s, then ends wait with `E-LOCKED` (exit 3). |
| Scans (`list`, `read-notification`, `check-state`) | Shared | Waits ~100ms; skipped/partial files mark report `warn`. |
| Brief reference targets | Shared | Waits up to 1s per target; a busy target is unreadable for this attempt. |

- `E-LOCKED`: file locked or concurrent mutation. Retry shortly.
- `E-INUSE` (Windows): file open in another process without shared access. Close process handle.
- `E-FLOCKED`: functional directory lock. Coordinate with holder or wait until expiry.
- 16 MiB size cap: files exceeding 16 MiB refuse parsing and writes (`E-TOO-LARGE`, exit 1).
- Brief target locks apply to ordinary files as well as collaboration files, including symlink and hardlink aliases. A busy target yields `unchecked` / `E-ENTRY-TARGET` during validation, with no partial content or hash; `brief --target` or `--refresh` fails without changing the brief. Wait for the writer to finish, then retry. A self-reference hashes the pre-write snapshot already held under the brief's exclusive lock, so changing the brief can make its own entry shifted.
- Without a clock override, a post takes its creation or edit time, and a state declaration its start/end pair, after acquiring the exclusive lock and before assembling the write. Time spent queued is not recorded as the event time; a declaration keeps its full duration from that start. Reads, state removals, and wait admission use the startup clock.

### Incomplete lock and conflict scans

When scans cannot read state files due to locks or permissions:
- `skipped` and `partial` rows record the unread target.
- For `profile` and `brief`, writes proceed if no lock holder is proven, appending scan warnings.
- Reports with incomplete scans append a `fix` row with read-only commands to inspect the unread files.

### Time formats

- Output and stored timestamps: canonical UTC `YYYY-MM-DDTHH:MM:SSZ` (leap seconds keep `:60`). Upper limit is `9999-12-31T23:59:59Z`.
- `--since` input: accepts an RFC 3339 timestamp (timezone offsets are converted to UTC, fractional seconds are floored, `T` separator required) or a duration back from now.
- Durations: strict descending units `Nd`, `Nh`, `Nm`, `Ns` (e.g. `1h30m`, `45s`); each unit at most once; total > 0.
- `AGENT_COEXIST_NOW`: strict 20-character UTC format only (`YYYY-MM-DDTHH:MM:SSZ`); offsets, fractions, and leap seconds are rejected.
## Recovery commands (`fix`)

`fix` lines provide guidance or copyable shell commands. Never execute `fix` commands blindly without inspecting `why` and `reason`.

### When a write fails partway

Writes modify files in place through locked handles (tail written first, then mutated span, then truncate, then sync).
- If failure occurs before sync, the CLI attempts to restore previous bytes:
  - Previous content restored: `cannot write <file>: <os error>` (exit 1). File is intact; resolve cause and retry.
  - Rollback failed: `cannot write <file>: ...; putting its previous content back also failed` (exit 1). File may contain mixed bytes; inspect with `check` and repair by hand.
  - Sync failed: `wrote <file>, but could not confirm that the change is stored on disk` (exit 1). Bytes were written; verify with `check` before retrying to prevent duplicate comments.
- Cleanup and rollback are not crash-safe or atomic against sudden power loss or process kill.

### How operands are written

Command-form `fix` lines format operands for POSIX shells (bash, zsh, Git Bash):
- Unquoted words contain only `a-z`, `A-Z`, `0-9`, and `_@%+,./-`.
- Words with spaces or special characters are single-quoted (`'...'`). Inner single quotes escape as `'\''`.
- Unix non-UTF-8 bytes appear as octal printf sequences: `"$(printf '\377')"`.
- Positional `PATH` operands that begin with `-` are prefixed with `./` (e.g. `./-notes`). Option values use `--opt='-value'`.
- Placeholders vs real operands: angle-bracket markers emitted by templates (`<file>`, `<nick>`, `<title>`, `<the whole body, with the reference corrected>`) are placeholders to fill in before executing. Real runtime values that happen to contain angle brackets (such as an entry literally titled `<file>`) appear quoted as real operands (`--entry='<file>'`).
### When there is no command

The CLI provides human instructions rather than copyable commands when:
- Arguments contain newlines, NUL, or characters not representable in POSIX shell syntax.
- Headings contain control characters or syntax invalid for CLI manipulation.
- A state file is not the directory's canonical `state.agent-besties.md`.
- Comment numbers exceed 64-bit bounds.

## Platform notes

- **Windows paths**: drive-relative paths display as `C:notes`. Ordinary and verbatim paths (`\\?\C:\...`) display relative to cwd when possible. Ignore rules match drive letters and volume prefixes case-insensitively.
- **Git layouts**: worktrees and `.git` files are followed; relative `commondir` references resolve correctly.
- **Global excludes**: `core.excludesFile` values support quoted paths with spaces, comments, escaped characters, and continued lines. The last value in the selected config source's `[core]` section wins; an empty value disables the global excludes file. A malformed selected value is reported as a configuration-source error.
- **Case sensitivity**: `.agent-besties.md` suffix and glob matches are case-insensitive on Windows/macOS. Stem vs H1 checks (`W-NAME`) compare against actual disk casing on Windows/macOS.

## Troubleshooting

- `target <path> does not exist; looked for <resolved>` (`E-TARGET-MISSING`): `--target` resolves relative to the brief directory, not cwd.
- `<file> does not exist; <dir> is a directory` (`E-FILE-MISSING`): a file argument received a directory path on a never-creating command.
- `directory <dir> does not exist` (`E-DIR-MISSING`): `DIR` arguments must already exist; commands that take `DIR` never create missing directories (unlike creating `PATH` forms, which create missing parent directories).
- `<dir> is locked by <holder> until <time>` (`E-FLOCKED`): functional lock active. Pass `--as <holder>` if you are the holder, or wait for expiry.
- `<nick> is still declared on line <n> ... mixed with unparseable fragments` (`E-STATE-DIRTY`): edit the line by hand or clean unparseable fragments before removing.
- `unexpected argument '-...' found`: attach option values starting with `-` using `=` (e.g. `-m='- list'`).
- Wait never triggers: check path (plain files without `.agent-besties.md` are not watched) and verify filter conditions.

## Environment overrides

For automated testing only:
- `AGENT_COEXIST_NOW`: pins the current time (`YYYY-MM-DDTHH:MM:SSZ`).
- `AGENT_COEXIST_LOCK_WAIT_MS`: overrides the 5000ms lock timeout for `post` and `subscribe`.

## A pattern for scripted agents

```sh
# Scripted pattern checking status and handling concurrency
if report="$(agent-coexist check docs/standup)"; then
  code=0
else
  code=$?
fi
printf '%s\n' "$report"
first="$(printf '%s\n' "$report" | sed -n '1p')"

case "$code:$first" in
  0:ok*)   echo 'clean' ;;
  0:warn*) echo 'warning present: inspect report rows' ;;
  3:*)     echo 'busy or locked: follow reason/fix advice' ;;
  *)       printf 'failed with exit code %s\n' "$code" ;;
esac
```
