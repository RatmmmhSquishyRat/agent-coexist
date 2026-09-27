# Command reference

This page lists every command, flag, exit code, and output rule, so you can
look up exactly what a command prints and how to read it. For how to work
with other agents (what to write, how to catch a reply), run
`agent-coexist guide`; the same pages are in
[crates/cli/guide/](../crates/cli/guide/index.md). The
[README](../README.md) covers what the tool is and how to install it; the
[file format reference](file-format.md) shows the four file types on disk.
All examples run as-is in a POSIX shell (Git Bash on Windows).

## Contents

- [Built-in guide](#built-in-guide) · [Command index](#command-index)
- [Conventions](#conventions) — paths, names, reading output, exit codes
- [Profiles](#profiles) · [Posts and subscriptions](#posts-and-subscriptions)
- [Briefs](#briefs) · [State](#state)
- [Discovery and notifications](#discovery-and-notifications)
- [Waiting for changes](#waiting-for-changes)
- [Locks, retries and time](#locks-retries-and-time)
- [Recovery commands (`fix`)](#recovery-commands-fix) — when a `fix` is a command, when it is an instruction, and how operands are written
- [Platform notes](#platform-notes) — Windows paths, Git layouts, case rules
- [Troubleshooting](#troubleshooting) · [Environment overrides](#environment-overrides)
- [A pattern for scripted agents](#a-pattern-for-scripted-agents)

## Built-in guide

```sh
agent-coexist guide          # the index page
agent-coexist guide wait     # one topic page
```

| Page | File | Covers |
|---|---|---|
| `guide` | [index.md](../crates/cli/guide/index.md) | when to use coexist, and one complete exchange between two agents |
| `guide start` | [start.md](../crates/cli/guide/start.md) | choosing between post, brief, profile, and state |
| `guide read` | [read.md](../crates/cli/guide/read.md) | finding and reading existing files, and brief entry states |
| `guide write` | [write.md](../crates/cli/guide/write.md) | writing posts and handoffs, briefs, profiles, state, and closing out |
| `guide wait` | [wait.md](../crates/cli/guide/wait.md) | catching up, re-reading after the waiter is ready, and managing a blocking wait |

- Each call prints one static Markdown page to stdout (LF, trailing newline), with empty stderr and exit 0.
- Topics are exact lowercase; there is no `index`, `all`, or other alias. An unknown topic, extra arguments, or unsupported flags are a one-line stderr usage error, exit 2, with empty stdout and no workspace access.
- A stdout write or flush failure is exit 1, with no panic and no status line.
- `guide`, `--help`, and `--version` are information output: no `ok`/`warn`/`error` first line, and no cwd, clock, storage, or lock setup. Invalid `AGENT_COEXIST_NOW` or `AGENT_COEXIST_LOCK_WAIT_MS` does not block them.

## Command index

| Command | One line | Covered in |
|---|---|---|
| `guide` | print one built-in tutorial page | [Built-in guide](#built-in-guide) |
| `profile` | create a profile; set/remove sections | [Profiles](#profiles) |
| `post` | append a comment; edit your own | [Posts](#posts-and-subscriptions) |
| `subscribe` | join/leave a post's subscriber list | [Posts](#posts-and-subscriptions) |
| `brief` | create a brief; manage description and entries | [Briefs](#briefs) |
| `read-brief` | render a brief with entry states | [Briefs](#briefs) |
| `state` | declare or remove directory state | [State](#state) |
| `check-state` | state of a directory, ancestors, subtree | [State](#state) |
| `list` | enumerate collaboration files as a tree | [Discovery](#discovery-and-notifications) |
| `check` | one file: type, diagnostics, previews | [Discovery](#discovery-and-notifications) |
| `lookfor-comment` | search one post's comments | [Discovery](#discovery-and-notifications) |
| `read-notification` | what happened since a point in time | [Discovery](#discovery-and-notifications) |
| `wait` | block until a file changes | [Waiting](#waiting-for-changes) |
| `wait-post` | block until a relevant comment appears | [Waiting](#waiting-for-changes) |
| `wait-state` | block until state declarations change | [Waiting](#waiting-for-changes) |

Every command also answers `agent-coexist <command> --help`. The fourteen
rows after `guide` are the business commands; they all use the line
protocol described under [Reading output](#reading-output).

## Conventions

### Files and paths

- Collaboration files end in `.agent-besties.md`. `PATH` arguments may omit the suffix; it is appended when missing. That check is byte-exact, so type the suffix in lowercase or leave it out: `docs/standup.AGENT-BESTIES.md` gets a second suffix appended. Scans match the suffix case-insensitively.
- `DIR` arguments default to `.` and must already exist (`E-DIR-MISSING` / `E-NOT-DIR`, exit 1). No command creates `DIR` itself.
- These forms create the file and any missing parent directories, printing `created  <dir>/` rows before `created  <file>`: `post` (new thread), `subscribe` (add), `profile` (bare or section set), `brief` (bare, description, `--target`).
- These forms never create: `post --edit`, `subscribe --cancel`, `profile --remove`, and the brief note-only, `--entry --remove` and `--refresh` forms. On a missing file they fail with `E-FILE-MISSING`, exit 1.
- If a creating write is refused (wrong type, `E-NAME`, another nick's `locked`, a busy lock, a blocking parse error), the CLI deletes the empty file and empty parent directories this invocation just created, after checking they are still the same empty objects. Pre-existing files, files that already contain bytes, replacements at the same path, and directories that are no longer empty stay. An empty file or directory can therefore appear briefly and vanish; this cleanup is not crash-safe or atomic against other programs.
- Windows path display rules are under [Platform notes](#platform-notes).

### Names and identity

- Creating `post`, `profile`, `brief`, and `subscribe` forms build the H1 from the file-name stem as you typed it. The stem must be a valid name: 1–100 letters, digits, `_`, `-`, with `.` allowed after the first character. Otherwise the command fails with `E-NAME` (exit 1); the CLI never renames, truncates, or substitutes another name.
- An existing file that already parses stays editable when its stem and H1 disagree (`W-NAME` on `check`). Rename the file to the H1 name. Copying the stem into the H1 is offered only when the stem is itself a valid name.
- `--as <nick>` is your name. It is recorded in comment headers, matched by notifications, and exempts you from functional locks, but it is never authenticated (see [SECURITY.md](../SECURITY.md)). Nicks are 1–64 characters: a letter, digit, or `_` first, then letters, digits, `_`, or `-`.
- Statuses (`--status`) are 1–32 ASCII lowercase characters: `a-z`, then `a-z0-9-`. Section names follow the nick grammar.
- Entry titles have no grammar, but CLI flags reject control characters and double spaces. Titles are matched and displayed after whitespace folding: runs of spaces collapse to one, ends are trimmed.
- Title columns and human diagnostic text (message and `why`) use that folded display, with any remaining control characters shown as U+FFFD. Stored heading text and body bytes are not rewritten. A stored heading that already contains control characters is fixed by editing it by hand ([Recovery commands](#when-there-is-no-command)).
- `-m` bodies: CRLF becomes LF, trailing whitespace and newlines are stripped, leading whitespace is kept, and the result must be non-empty; otherwise a usage error, exit 2.

### Reading output

A business report is shaped like this (a `check` with one warning; column padding and text are illustrative):

```text
warn  check  docs/onboarding
  file      docs/onboarding.agent-besties.md
  type      brief
  warn   L7  …message…
         why  …why…
         fix  …fix…
  …preview rows…
```

Rows vary by command; the rules below say how to read each kind.

- First line: `<status>  <command>  <object>`, where status is `ok`, `warn`, or `error`. There is no JSON mode; this line protocol is the scripting contract.
- An empty object prints as `/`, so splitting that first line on two or more spaces always yields three columns. The same stand-in applies to the wait ready line on stderr (`waiting  <command>  <object>`). This is display only; a nonempty object that is itself `/` prints the same way, with no extra encoding.
- Fields rows (`  <key>  <value>…`) hold a key plus values joined by two spaces. Values taken from file content are whitespace-folded first, so splitting those rows on two or more spaces is safe. An empty key or value prints as `/`; a Fields row with no values is the indent plus the key, with no value column.
- Exception: a `fix` row can carry a command whose native operands keep internal whitespace, including two-space runs inside quotes. Never split a `fix` row into columns; take everything after the `fix` key as one payload ([Recovery commands](#recovery-commands-fix)).
- Text rows are a `  <key>` line followed by `    | <line>` lines that reproduce content verbatim (read-brief bodies, wait before/after values). They may contain any spacing; read them by the `| ` prefix, never as columns. Empty content lines stay empty; no `/` is substituted.
- Diagnostics print as `  error  L<n>  <message>` (or `warn`), with `why` and `fix` lines indented 9 spaces underneath.
- Completed `list`, `lookfor-comment`, `read-notification`, and `check-state` overviews (`ok`/`warn`, including zero records) end with a `total` row. Fatal reports of those commands (missing dir or file, not a directory, `cannot scan`, other OS errors, blocking diagnostics) have no `total` row. `check`, `read-brief`, write commands, and wait reports never have one.
- Lines always end in LF, even on Windows.
- Progress (`scanning N dirs, M files...`) goes to stderr, and only when stderr is a TTY; this includes the ancestor lock scans of `profile` and `brief`. Stdout stays clean for redirection.

### Exit codes

| Code | Means | Action |
|---|---|---|
| 0 | first line `ok` | done |
| 0 | first line `warn`: conflicts, skipped or partial files, refresh skips, dangling reply targets | success; read the rows |
| 0 | `guide`, `--help`, `--version` information output (no status line) | done |
| 1 | first line `error`: diagnostics, missing file/entry/dir, type mismatch, I/O | fix the input |
| 1 | `guide` stdout write or flush failure (no status line) | retry the output sink |
| 2 | usage error, including argument-parser failures and invalid `AGENT_COEXIST_*` values on business commands: one `error: ...` line on stderr with the parser's error and hint (CR/LF in it become spaces), stdout empty | fix the command line |
| 3 | `E-LOCKED`: the required file lock, stable identity, or stable contents could not be obtained or maintained | retry the same command shortly |
| 3 | `E-INUSE`: another program (e.g. an editor) holds the file | close it, then retry |
| 3 | `E-FLOCKED`: functional lock, see [Functional lock](#functional-lock) | retry at the expiry time shown in `reason` |
| 130 | a wait was cancelled with Ctrl-C | — |

`warn` is exit 0. Scripts that care about conflicts must read the first line
of a business report, not only the exit code. Do not parse `guide`,
`--help`, or `--version` output as `ok`/`warn`/`error`.

## Profiles

```sh
agent-coexist profile agents/alice                        # create; else unchanged
agent-coexist profile agents/alice --section role -m 'Rust implementer.'
agent-coexist profile agents/alice --section role --remove
```

- Without `--section`: creates the file (heading only) or reports the existing file `unchanged`.
- `--section <name> -m <content>` sets or replaces that section's body, stored verbatim in a code fence. `--section` rejects control characters and names outside the nick grammar.
- `--section <name> --remove` deletes the whole section. Unknown section: `E-SECTION-NOTFOUND`, exit 1. `--remove` never creates the file.
- A stored section title outside the nick grammar cannot be addressed with `--section`; `check` tells you how to rename it by hand ([When there is no command](#when-there-is-no-command)).
- `--as` is optional here: it only exempts you from functional locks and is never written to the file.

## Posts and subscriptions

```sh
agent-coexist post docs/standup --as alice -m '…'           # append; creates
agent-coexist post docs/standup --as alice -m '…' --edit 2  # rewrite own #2
agent-coexist subscribe docs/standup --as bob               # add subscriber
agent-coexist subscribe docs/standup --as bob --cancel      # remove
```

### Comments, mentions, and replies

- Comments are numbered `#1`, `#2`, … in file order. A gap, including a first legal number greater than 1, is a non-blocking warning; the CLI never renumbers.
- Mentions (`@nick`) and replies (`@#n`) are plain text in the body. A mention starts at the beginning of the text or after any character other than a letter, digit, `_`, or `@`: `@alice-@bob` mentions `alice-` and `bob`; `a@bob` is not a mention.
- The write report lists each parsed mention and reply in body order. An `@#n` pointing nowhere adds a `(not found)` warn row; the comment is still written.
- `check` of a post lists `authors` in first-occurrence order (`none` when empty).
- When `check` prints copyable header repairs for several malformed or duplicate headers in one report, each repair gets its own unused sequence number, so copying every printed header is enough. A header that cannot get a remaining number stays a manual heading repair. Bodies and already-legal comments are never rewritten.

### Editing a comment

`--edit n` replaces the whole body of comment #n with the new `-m` text
and adds `, edited <time>` to its header; include anything from the old
body you want to keep. Its author must match `--as` (`E-EDIT-AUTHOR`,
exit 1) and it must exist (`E-COMMENT-NOTFOUND`, exit 1). `--edit` never
creates the file.

```sh
agent-coexist post docs/standup --as alice -m '…' --edit 2
# expected: error (E-EDIT-AUTHOR), exit 1 — comment #2 is bob's
```

### Subscribers

- Subscribing twice, or cancelling when not subscribed, leaves the file unchanged (`unchanged  <nick>`, `ok`).
- `--cancel` edits only the first authoritative subscribers property. Deleting its last nick deletes that property line; later duplicate properties stay as stored, and the report lists the subscribers of the resulting document. A surviving duplicate can therefore become the list; the report says `none` only when that list is empty.
- `--cancel` on a missing file is `E-FILE-MISSING`, exit 1.
- `post` and `subscribe` queue up to 5 s on a busy lock; see [Locks, retries and time](#locks-retries-and-time).

## Briefs

A brief hands over pointers to real source files, verified by SHA-256. Six
mutually exclusive forms:

```sh
agent-coexist brief docs/onboarding                    # create / unchanged
agent-coexist brief docs/onboarding -m 'Onboarding pointers'
agent-coexist brief docs/onboarding --target ../src/parser/mod.rs \
  --entry 'parser entry point' -m 'Start here.'
agent-coexist brief docs/onboarding --entry 'parser entry point' \
  -m 'After review: start at parse().'
agent-coexist brief docs/onboarding --entry 'parser entry point' --remove
agent-coexist brief docs/onboarding --refresh
```

- `--target` resolves against the brief file's directory, not the shell's cwd: for `docs/onboarding`, `--target ../src/parser/mod.rs` means `src/parser/mod.rs` from the workspace root. Paths starting with `/` (or a Windows drive letter) are used as-is. A missing target is `E-TARGET-MISSING`, exit 1, detected before the brief is opened.
- The default entry title is the target's file name (`mod.rs`); `--entry` overrides it. Reusing an existing title that points at a *different* target, without `--entry`, is `E-ENTRY-TITLE-TAKEN`, exit 1.
- `--regex` requires `--target`. Writing checks that the pattern compiles; matching is checked on read, and a target that does not match makes the entry `stale` (`regex not found`). `--regex ''` removes every `- regex:` line in that entry's property area, including handwritten duplicates.
- Raw CR or LF in `--target` or `--regex` is a usage error (exit 2) and writes nothing; other whitespace is kept as part of the stored value. To match CR or LF in a target, use the regex escapes `\r` and `\n`.
- The target form writes `entry  <title>  added|updated  <path>` plus `hash  <sha256>`, and `note  set` when `-m` is given. The description form writes `description  set`; the note-only form `note  set`; removal writes `entry  <title>  removed` and `removed-lines  L<a>..L<b>`.
- `--refresh` recomputes and rewrites stored hashes only; it never re-checks regexes. Per selected entry it prints `entry  <title>  refreshed  <path>`, or `skipped  no path` / `skipped  target missing` (skips make the first line `warn`, exit stays 0). Without `--entry`, every entry is refreshed. Unknown title: `E-ENTRY-NOTFOUND`, exit 1.
- Review changed targets and update outdated notes before refreshing: `fresh` means the hash matches, not that anyone has checked the note.
- Combination rules: `--regex` requires `--target`; `--refresh` excludes `--target`, `-m`, and `--remove`; `--remove` requires `--entry` and excludes `--target` and `-m`; without `--target`, `--entry` requires `-m`, `--remove`, or `--refresh`. Violations are usage errors, exit 2, and write nothing.
- After any byte-writing brief form, surviving non-blocking diagnostics are re-parsed and appended as `remaining  <E-ID>  L<n>` warn rows.

### Entry states (what readers see)

| State | Meaning | `reason` shown |
|---|---|---|
| `fresh` | stored hash matches the target's current bytes | — |
| `shifted` | hash differs: the target changed after the hash was written | — |
| `stale` | target missing, regex invalid, or regex not found | that reason |
| `unchecked` | no path, target unreadable, target too large, or no hash | that reason |

`target too large` applies to targets over 64 MiB that need regex
verification; hashing itself is streaming and uncapped.

### Reading briefs

- `check` on a brief validates every entry and reports states without printing target bodies.
- `read-brief` prints the description, then per entry its state, note, and the target's content as verbatim `| ` text blocks, plus rows for regex capture groups when a regex is set.
- `read-brief --entry <title>` prints that one entry, including its full target content (unknown title: `E-ENTRY-NOTFOUND`, exit 1). It is not a status-only switch; use `check` for states alone.
- Any shifted, stale, or unchecked entry makes the first line `warn`.

## State

```sh
agent-coexist state src/parser --as alice --status locked              # 2h
agent-coexist state src/parser --as alice --status locked --for 1h30m
agent-coexist state src --as bob --status observing
agent-coexist state src/parser --remove alice       # --as defaults to alice
agent-coexist state src --remove bob --status observing  # --status filters
```

- Built-in statuses: `locked` means "please do not change this directory or its subdirectories until the end time"; `observing` means "I am watching; changes are welcome". Custom status names are stored and shown but carry no lock meaning.
- Durations default to 2 h for `locked` and 8 h for anything else; `--for` overrides.
- State is cooperation, not enforcement: it records intent for agents that choose to respect it. See [SECURITY.md](../SECURITY.md).

### Declaring

- Declarations live in `DIR/state.agent-besties.md`. `state` and `check-state` select that reserved file by native lookup (case-insensitive on Windows and macOS), never by another file's basename or H1. A misnamed file whose H1 is `# .state` can still be read with `check`, but `state` and `check-state` ignore it.
- A declaration is valid while `start <= until` and `now < until`.
- Re-declaring the same nick under the same status replaces that nick's clean declarations there with `start = now` and `until = now +` the `--for` or default duration. An explicit renewal can therefore shorten an existing lifetime.
- For duplicate declarations that remain stored (for example on mixed lines), the latest `until` is the one selected.
- Every successful declaration or removal also cleans expired entries on clean lines (`cleaned` rows). Read commands never write.
- After writing, the declare form scans for conflicts: two different nicks holding valid `locked` whose directories overlap (equal, or ancestor/descendant) produce `conflict  locked  <nick>  <span>  <dir>` rows and a `warn` first line; exit stays 0. Another agent's `locked` never blocks a state write.
- If that post-write scan hits a fatal ignore-source error, the write is already committed: first line `warn`, the usual `created` / `set` / `cleaned` rows, then `scan  <path>  <cause>`, exit 0. There is no `fix` row and no rollback; `check` of the state file shows the written declaration.

### Removing

- `--remove <nick>` prints `removed  <status>  <nick>` per section. `--as` defaults to the removed nick; a different `--as` still removes but adds a `removed-other` warn row.
- Without `--status`, the nick is removed from every status that holds a clean entry for it. That span is intentional; add `--status` to remove only one status.
- Only clean lines are rewritten. A line is clean when every comma fragment that is nonempty after trimming parses as a declaration; empty fragments are ignored. A comma-only line holds no declarations and can disappear with its section.
- Mixed lines: if the nick still has a parseable declaration on a line with an unparseable nonempty fragment, under a selected status, the whole removal is refused before any write (`E-STATE-DIRTY`, exit 1) and the line stays byte-for-byte. Make that line parseable by hand (or delete the nick from it), then re-run. The printed retry keeps the complete original command, including `--status`. Legal fragments on mixed lines still count in `check` and lock scans.
- A status section whose remaining bytes are only its heading and blank lines is deleted (`removed-lines`). Leftover handwritten text, mixed lines, and fences keep the heading and those bytes.
- A file reduced to its heading, reader comments, and blanks is deleted whole (`deleted  <file>`).
- Missing file or nick: `E-STATE-NOTFOUND`, exit 1, nothing written. With `--status`, the reason names that status and the `fix` is a read-only `check-state` of the directory. Do not drop `--status` and remove a different declaration instead; a live `locked` entry would go with it.
- A same-file lock-conflict `fix` includes `--status=locked`, so copying it removes only that lock and leaves the nick's other statuses.
- `--for` is rejected in the remove form (usage error, exit 2).

### Scope, inheritance, and conflicts

- A state file describes its directory's whole subtree.
- `check-state DIR` reports DIR's own entries, `inherited` rows from ancestor directories (valid entries only, nearest first), and descendant groups headed by their paths relative to DIR. Overlapping lock rows keep native file order.
- `read-notification` also reads ancestor state files and counts DIR's own state file once.
- `check` on a state file reports each expiry and each same-file lock conflict (`W-STATE-CONFLICT`) once, from the selected declaration per nick. Conflict pairs appear in `check` and in a wait report that hits (including the snapshot from just before a deletion); `list` and other scans do not report them. A complete `check` lists every pair, so many overlapping `locked` entries make a large report.
- A `read-notification` conflict names the deepest overlapping own lock; when depths tie, the last file in scope order wins. Per-file conflict rows are never dropped.

### Functional lock

- The byte-writing `profile` and `brief` forms fail with `E-FLOCKED` (exit 3) when a *different* nick holds a valid `locked` on the supplied file's directory or any ancestor. Without `--as`, any holder blocks.
- A write through a file symlink also checks the canonical referent's parent directory and its ancestors. If canonicalization fails, only the supplied path's chain is checked. There is no volume-wide hardlink search.
- Retry at the expiry time shown in `reason`, or ask the holder to release the lock. Pass `--as <holder>` only when the lock really is yours.
- Unreadable or incomplete state files in that chain are listed as `skipped`/`partial`; on their own they do not refuse the write ([Locks, retries and time](#locks-retries-and-time)).
- `post`, `subscribe`, and `state` are never blocked by functional locks, and neither are hand edits or other tools.

## Discovery and notifications

### Scans: what is visited, and filter precedence

`list`, `read-notification`, and `check-state` walk `DIR` recursively and
consider only files named `*.agent-besties.md` (matched
case-insensitively). Hidden directories are traversed. An explicit `DIR`
that is a directory, including a directory symlink, a junction, or a
`.git` directory, is entered as a directory. Directory symlinks,
junctions, and `.git` directories found below that root are skipped.

Precedence, in order:

1. Ignore rules prune during the walk: `.gitignore`, `.ignore`, `.git/info/exclude`, your global gitignore, plus ignore files in the root's physical parent directories. `--no-ignore` disables all of them. Git worktree and gitfile layouts are covered in [Platform notes](#git-layouts-and-global-excludes).
2. `--include`/`--exclude` then filter the collected file names. They cannot reopen an ignore-pruned directory. If every `--include` matched nothing while ignore rules were active, the command adds a warn row suggesting `--no-ignore`.
3. A file is kept when (no `--include` was given, or one matches) and no `--exclude` matches. Exclude always wins; repeated flags form a union.

- An ignore source that is not a regular file, or cannot be read or parsed, fails the scan: first line `error`, `reason  cannot scan <path>: <source>`, a retry `fix`, exit 1, with no overview rows and no descendant groups. `--no-ignore` skips those sources. Ordinary unreadable directories are only `skipped` warnings.
- Globs are gitignore-style and relative to `DIR`: a leading `/` anchors at `DIR`; a trailing `/` matches files under that directory, not a same-named file; `*` and `?` never cross `/`, but `**` does. Matching is case-insensitive on Windows and macOS.
- Globs are matched exactly as typed, including internal double spaces. Only the display of the unmatched-`--include` hint row is folded, with control characters shown as U+FFFD. An invalid glob is a usage error (exit 2).

### list

Prints a tree with a type column. The first row names the active ignore
sources (a subset of `.gitignore, .ignore, exclude, global`, or `none`, or
`--no-ignore`). Leaves shown as `busy` (lock was busy) or `?` (unreadable:
permissions, not UTF-8, bad heading, over the size cap) still count toward
`total` and turn the first line `warn`, never a failure. `--type` filters
by type and repeats as a union; `profile` is an accepted alias for
`agent-profile`.

### check

Reports one file: `file` and `type` rows, diagnostics in line order, then
previews: comments, `authors` (first-occurrence order), topic, latest, and
subscribers for posts; entry counts and states for briefs (not target
bodies; `read-brief` prints those); sections for profiles; per-status rows
with `conflict` and `expired` marks for state files. Any error-level
diagnostic makes the first line `error` (exit 1). Blocking diagnostics also
refuse every write command with a report shaped like `check`.

### lookfor-comment

```sh
agent-coexist lookfor-comment docs/standup --by bob --mention alice
agent-coexist lookfor-comment docs/standup --keyword 'parser' --reply-to 1
agent-coexist lookfor-comment docs/standup --comment 2
agent-coexist lookfor-comment docs/standup    # every comment
```

Filters intersect; no filter lists every comment. Each row is
`L<line>  #<seq>  <author>  <time>` plus an excerpt.

- `--keyword` is a case-insensitive substring over the raw body and the comment's original header line, used exactly as typed (whitespace is not folded).
- `--by` is the author, `--mention` a body `@nick`, `--reply-to` a body `@#n`, and `--comment` selects one sequence number.

### read-notification

```sh
agent-coexist read-notification --as alice --since 1h
agent-coexist read-notification --as alice \
  --since 2026-09-17T09:12:00Z --include 'src/**'
```

Records, oldest first (ties broken by path, kind, then order):

| Record | About | When it appears |
|---|---|---|
| `mention` | others | a comment contains `@<you>` |
| `reply` | others | a comment's `@#n` targets a comment authored by you |
| `subscribed` | others | any comment while you are on the subscriber list |
| `state` | others | a declaration (any status) started in the window; shows `until <t>`, flagged `expired` if over |
| `conflict` | others | a valid `locked` overlapping yours: `overlaps your lock on <dir>`, naming the deepest overlapping own lock |
| `expiring` | you | your valid entry ends within 30 minutes: `ends in Nm` |
| `expired` | you | your entry ended inside the window: `expired at <t>` |

- Comment records appear when their activity time (created or edited) is in `[since, now)`. Your own comments never notify you.
- Each comment produces at most one record for you: `reply` beats `mention`, which beats `subscribed`.
- Expiry records live here; `check-state` marks expired rows but never raises them as records.
- Busy or unparseable files become `skipped`/`partial` rows and make the first line `warn`, as do conflict rows.
- Times are whole seconds and the current second is excluded, so a query in the same second as the activity can miss it. Make each new `--since` overlap the previous window, and treat repeated records by identity and content. There is no stored read cursor and no exactly-once delivery.

## Waiting for changes

```sh
agent-coexist wait docs/onboarding                  # any byte change
agent-coexist wait-post docs/standup --as alice     # a comment relevant to alice
agent-coexist wait-state src --status locked        # locked declarations in src
```

A wait never delivers messages; it tells you a file changed. The order that
avoids missing a reply:

1. Read the file (or `read-notification`) as usual.
2. Start the wait and let it print `waiting  <command>  <object>` on stderr. That line means the baseline has been taken.
3. Read the file once more. Anything that changed between step 1 and the baseline is part of the baseline and will not end the wait.
4. When the wait exits 0, read the current file (or `read-notification`) for the new content, then act.

There is no timeout flag. A caller's deadline does not stop the process by
itself; cancel with Ctrl-C (exit 130, no stdout) or terminate the process.
A wait does not wake another agent.

### Lifecycle

- The parent directory must exist (`E-DIR-MISSING`, exit 1). If it exists but is not a directory, `wait` and `wait-post` fail with a path-class error; `wait-state` reports `E-NOT-DIR`.
- After the baseline and ready line, the file is sampled once per second under a shared lock until the first qualifying change prints one report (exit 0). Nothing is written, and no lock is held between samples.
- A missing file is a legitimate baseline: `wait-post docs/standup` on a thread that does not exist yet waits for its creation. A mistyped path therefore waits forever; check the path.
- An invalid baseline fails with an error report, exit 1: blocking diagnostics, an unknown type, the wrong type for `wait-post`/`wait-state` (`E-TYPE`), or duplicate `(status, nick)` pairs in a state file.
- A file over the 16 MiB cap is refused with the same report as `check`: first line `error`, a `file` row, `E-TOO-LARGE`, exit 1, no `type` row. This applies on the first sample and if the file grows past the cap later; it is not a change hit.
- Baselines advance: a snapshot that changed but did not qualify becomes the new baseline. Diffs are always between adjacent snapshots, so a wait never replays earlier messages.
- `E-LOCKED`/`E-INUSE` while sampling: exit 3.

### What ends a wait

| Command | Filter | Hits on |
|---|---|---|
| `wait` | — | any byte change, any of the four types |
| `wait-post` | — | any byte change of the post |
| `wait-post` | `--as <nick>` | a comment relevant to `<nick>` (a direct `@nick`, or an `@#n` whose author is `<nick>`) that is new, edited, or newly relevant |
| `wait-state` | — | any byte change of `DIR/state.agent-besties.md` |
| `wait-state` | `--status <s>` | an add, removal, or `start`/`until` change of declarations under `<s>` (renewals hit) |

- `wait-state` watches exactly `DIR/state.agent-besties.md`: no ancestors, no subtree.
- Subscriber-list changes, comment deletions, and reorders never hit `wait-post --as`. Relevance is comment-based, so subscribing does not make every comment a trigger.
- Clock expiry alone never hits `wait-state --status`. A file deletion hits only if the file held `<s>` declarations before.

### The wait report

- The report starts with `bytes  changed` and `semantic  changed|unchanged`, then `created  <file>` / `removed  <file>` if the file appeared or vanished.
- Entity rows follow: `add`, `remove`, or `update`, with field values as verbatim text blocks. Updates show `before`/`after`; additions and removals show the fields of the side that exists; a missing field reads `none`. Order rows come last.
- Comments, entries, sections, and declarations are paired by raw identity (first match wins) and reported in file order. Filtered hits list only the qualifying entities.
- A brief's description appears once when the file is created, deleted, or changes type.
- Same-file lock-conflict warnings are included on a qualifying hit (also for the snapshot just before a deletion), evaluated at the wait's start time.
- State `fix` recipes in the report are checked against the directory's currently selected `state.agent-besties.md` at that final snapshot. A `removed` report can still show warnings from the deleted baseline, but it will not print a directory-level `state` command against a different, later selected file.

## Locks, retries and time

| Commands | Lock | Behavior when busy |
|---|---|---|
| `post`, `subscribe` | exclusive | queue up to 5 s, then `E-LOCKED` (3) |
| `profile`, `brief`, `state` | exclusive | fail fast (~200 ms), `E-LOCKED` (3) |
| `check`, `lookfor-comment`, `read-brief` | shared | 1 s, then `E-LOCKED` (3) |
| scans, plus state's conflict scan | shared | ~100 ms, then `busy`/`skipped`/`partial` rows + `warn` |

- `E-LOCKED`: the required file lock, stable identity, or stable contents could not be obtained or maintained; retry the same command shortly. It does not name a holder: another lock holder, exhausted identity-reopen retries, or contents that keep changing can all cause it.
- `E-INUSE`: another program (an editor, say) had the file open; close it and retry.
- `E-FLOCKED`: the functional lock described under [Functional lock](#functional-lock); retry at the expiry time in `reason`, not at millisecond cadence.
- Collaboration files are capped at 16 MiB (`E-TOO-LARGE`).
- `guide`, `--help`, and `--version` take no file locks and do not scan the workspace.
- What these OS locks do and do not exclude is in [SECURITY.md](../SECURITY.md).

### Incomplete lock and conflict scans

A scan that cannot fully read a state file still reports what it found.
`skipped` rows mean the file was busy, structurally unreadable, or the
wrong type. `partial` rows mean some declarations were kept while a listed
line could not be used, or that non-comment stray text (`W-STRAY`) never
became a section. Valid declarations still count, and unread bytes never
invent a lock.

For `profile`/`brief`, a recognized `locked` holder still blocks the write.
When no holder is found, the write proceeds and the incomplete scan is
appended with a `warn` first line: unread metadata is neither treated as
"no locks" nor as a reason to refuse every write.

### Time formats

- Times print as `YYYY-MM-DDTHH:MM:SSZ` (UTC, seconds). A leap second read from a file keeps `:60`.
- `--since` accepts an RFC 3339 timestamp (any offset is converted, fractional seconds are floored, a space instead of `T` is rejected) or a duration counted back from now.
- Durations (`Nd`, `Nh`, `Nm`, `Ns`) use units in strictly descending order, each at most once, with a total greater than zero: `1h30m`, `90m`, and `2d` are valid; `1m30h` and `0s` are not.

## Recovery commands (`fix`)

Diagnostics and most failure reasons carry a `fix` line. Read it before
acting: it is either an instruction or a command, and a command may still
need a decision or a filled-in placeholder. Never pipe `fix` output into a
shell unconditionally.

### Command or instruction

| Form | Shape | What to do |
|---|---|---|
| Hand edit or file content | `set line 1 to "# <name>.post" …`, `add a closing line of 3 backticks where the block should end`, `move it into the nearest fenced block, or delete it` | edit the file yourself, then run `check` again |
| Command | `agent-coexist brief <path> --entry=<title> --refresh` | run it after reading `why` |
| Instruction, then command | `retry shortly: <original command>`, `review the target, then: agent-coexist brief …`, `agree who continues, then the other runs: agent-coexist state …` | do the instruction first; the command is everything after the instruction's colon and starts with the program name (a retry repeats the program path you typed) |

- Take the whole remainder after the `fix` key as one payload; never split it on spaces.
- Angle-bracket words written by the recipe itself are static placeholders.
  Fill them in before running: `<file>`, `<relative path>`, `<new path>`,
  `<pattern>`, `<title>`, `<text>`, `<note>`, `<your reply>`,
  `<what you do>`, `<name>`, `<nick>`, and `<same flags>`. They stay
  placeholders even when the CLI already knows a current value:
  `--target <file>` on an unreadable or stale entry asks which target you
  want now, and `--remove <nick>` on a lock conflict asks you to decide who
  yields.
- Values taken from your files are real operands, even when they look like
  placeholders: an entry actually titled `<file>` prints as
  `--entry='<file>'`. Quotes do not tell the two apart, because recipes
  also quote placeholders (`--regex '<pattern>'`, `-m '<text>'`). Decide
  from the diagnostic and what the option means which words you must fill
  in.
- Literal and prose `fix` text (a comment header to copy, a heading to rename) is file content or instruction, not shell-quoted operands.

### Before you run a command

- `W-STATE-CONFLICT`: agree with the other holder first; only the agent that yields runs the `--remove … --status=locked` command.
- `E-FLOCKED`: wait for the expiry or ask the holder. Passing `--as` with the holder's nick is only for the holder itself.
- `W-ENTRY-SHIFTED`: review the target and update the note before `--refresh`; refreshing only accepts the new hash.
- `E-STATE-DIRTY`: make the named line parseable by hand, then paste the printed retry. It keeps the original `--status`; do not rebuild a bare `--remove` that would also drop other statuses.
- `E-STATE-NOTFOUND`: the fix is a read-only `check-state`. Inspect, then correct your own command; do not remove a different declaration to make it succeed.
- State recipes (`W-STATE-DUP`, `W-STATE-RANGE`, `W-STATE-EXPIRED`, `W-STATE-CONFLICT`, and `E-TYPE` on a state heading) print a directory-level `state` command only when the inspected file is that directory's selected `state.agent-besties.md`. Otherwise the fix tells you to edit the inspected file by hand.

### How operands are written

Command operands are native values, separate from the folded display in
other columns. A command `fix` keeps the managed suffix when the file
already has it, internal path whitespace, and Windows verbatim spelling
(`\\?\…`). Its words are encoded for a POSIX shell:

- A nonempty word of ASCII letters, digits, or `_@%+,./-` is left as-is.
- An empty word becomes `''`.
- Every other valid UTF-8 word (braces, `=`, quotes, `$`, non-ASCII, and the rest) is wrapped in single quotes; an inner `'` becomes `'\''`.
- On Unix, bytes that are not valid UTF-8 appear as double-quoted octal `printf` substitutions inside the same word: byte 0xFF becomes `"$(printf '\377')"`. The pasted command addresses the original bytes, not a U+FFFD look-alike.
- A positional operand that starts with `-` is written as `./…`; option values use `--key=<quoted-value>`.
- Retries such as `retry shortly: …` repeat the original command line, encoded the same way.

This encoding targets POSIX shells (bash, sh, zsh, Git Bash). cmd.exe and
PowerShell need their own quoting.

### When there is no command

The CLI prints an instruction instead of a command when a command would be
wrong or impossible:

- An operand or original argument contains NUL, CR, or LF, or holds native data this POSIX UTF-8 text cannot represent (such as unpaired UTF-16 surrogates on Windows). The whole recipe becomes "repair the original file or retry the original command by hand"; other recipes in the same report are unaffected. The CLI never emits a lossy executable look-alike.
- A stored heading contains control characters: edit that heading by hand to a valid, distinct title, then run `check` again. Do not pass those characters to `--entry`, `--section`, or `--status`, and do not target the U+FFFD version shown in the display.
- A profile section title is outside the nick grammar: `check` offers a copyable `## <name>` example only when that name is unused among the file's headings and other repairs in the same report. Otherwise rename the heading by hand to a distinct valid name.
- A stored status name is invalid: rename that `## ` heading by hand to a valid, unused status, then run `check` again. Only then do declaration diagnostics suggest `--status` commands for it.
- A state-content file is not the directory's selected `state.agent-besties.md` (see [Troubleshooting](#troubleshooting)).
- Comment numbering is exhausted: the fix describes a heading change or a new post.

The file-side details of these repairs are in
[Editing files by hand](file-format.md#editing-files-by-hand).

## Platform notes

### Windows paths

- Drive-relative paths stay `C:notes` in display, never `C:/notes`.
- Equivalent ordinary and verbatim absolute paths (`C:\x` and `\\?\C:\x`) are displayed relative to the cwd when they are under it. Directory names are not case-folded for this, and display does no extra I/O.
- Ignore matching compares volume prefixes as native disk/UNC values: `C:` and `c:` are the same disk; `C:\` and `\\?\C:\` are the same volume, so both spellings agree on global directory excludes. A shorter path is never treated as inside a longer root. The path used for I/O is not rewritten, and `\\.\` device paths are not treated as disks.
- Output lines end in LF; recovery commands are written for Git Bash or another POSIX shell.

### Git layouts and global excludes

- Worktrees and separate git directories are followed: a `.git` file (gitfile) points to its gitdir, and `commondir` is honored from both gitfiles and ordinary `.git` directories. Relative gitfile paths resolve from the gitfile's directory; relative `commondir` paths from the gitdir. A missing optional `commondir` keeps that gitdir's exclude rules.
- An explicitly configured global Git config replaces the default global config files.
- In `core.excludesFile`, only a leading `~` home component expands. `~user` forms and later tildes, such as Windows short names like `RUNNER~1`, stay literal.

### Case sensitivity

- The `.agent-besties.md` suffix is matched case-insensitively in scans on every platform, but appended case-sensitively to `PATH` arguments.
- Reserved `state.agent-besties.md` lookup and glob matching are case-insensitive on Windows and macOS.

## Troubleshooting

- `E-TARGET-MISSING` on a brief whose target obviously exists: the path is relative to the brief's directory, not your cwd.
- `--include` matched nothing: ignore rules pruned those directories; rerun with `--no-ignore`.
- `error  check-state` / `cannot scan`: an ignore source failed to load or parse (the same as for `list` and `read-notification`), not an unreadable directory. If a `state` write succeeded first, you get `warn` with a `scan  <path>  <cause>` row, exit 0, and the written bytes stay.
- Exit 0 but the first line is `warn`: read the rows (state conflicts, refresh skips, busy or partial state files, dangling `@#n`, incomplete lock scans).
- `E-FLOCKED` on `profile`/`brief`: another nick declared `locked` on that directory or above, possibly on a symlink target's directory. Retry at the shown expiry, or use `post`/`subscribe`, which are not blocked.
- `E-NAME` on a creating command: the file-name stem is not a valid name. Choose a valid name and re-run with that path.
- `E-TYPE`: the heading names a different type. Use the command for that type, or pick another path. For a state heading, a directory-level `state` command is printed only when the file is that directory's selected `state.agent-besties.md`.
- `check` warns that a file with H1 `# .state` is not `state.agent-besties.md`: `state` and `check-state` ignore it. Inspect it and any existing `state.agent-besties.md` in the same directory, resolve their declarations, then decide which file keeps the reserved name; never overwrite a separate existing state file. Advice about that inspected file is hand-edit only.
- `E-STATE-DIRTY`: the nick is still declared on a mixed line; see [Removing](#removing).
- `E-STATE-NOTFOUND`: no matching nick under the selected status (or no file). Nothing was written. Inspect with the printed `check-state` and keep `--status`.
- Your own `--remove` removed more than one status: without `--status` it spans every status that holds the nick.
- `W-NAME` or a broken H1 after a hand rename: the file stays usable if it parses. If the stem is not a valid name, rename the file to match the H1.
- A heading with control characters, an invalid section name, or an invalid status: rename it by hand; see [When there is no command](#when-there-is-no-command).
- File not created: you used a never-creating form (`--edit`, `--cancel`, `--remove`, `--refresh`, note-only `-m`) on a missing file.
- A doubled suffix in a filename: the suffix append is case-sensitive; drop or lowercase the suffix you typed.
- Duration rejected: units must be strictly descending (`1h30m`, not `1m30h`).
- Empty `-m` rejected: trailing whitespace is stripped and something must remain.
- A wait never returns: check the path (a missing file is a valid baseline) and the filter; see [What ends a wait](#what-ends-a-wait).

## Environment overrides

Both exist for testing and advanced automation; normal use needs neither.
Business commands validate them; `guide`, `--help`, and `--version` ignore
them.

- `AGENT_COEXIST_NOW` pins `now` for reproducible runs. Strict 20-character `YYYY-MM-DDTHH:MM:SSZ`; offsets, fractions, lowercase, and leap seconds (`:60`) are rejected (usage error, exit 2, before any file is touched).
- `AGENT_COEXIST_LOCK_WAIT_MS` (digits only) overrides the 5 s queue budget of `post`/`subscribe` writes; no other budget.

## A pattern for scripted agents

```sh
# Keep both the complete report and its exit code, even under set -e.
if report="$(agent-coexist check docs/standup)"; then
  code=0
else
  code=$?
fi
printf '%s\n' "$report"
first="$(printf '%s\n' "$report" | sed -n '1p')"
case "$code:$first" in
  0:ok*)   echo 'clean' ;;
  0:warn*) echo 'completed with warnings; inspect the report' ;;
  3:*)     echo 'retryable; follow the reported reason' ;;
  *)       printf 'failed (exit %s)\n' "$code" ;;
esac
```

- For exit 3, retry `E-LOCKED` shortly, close the other program's handle for `E-INUSE`, or wait until the stated expiry for `E-FLOCKED`.
- Treat `fix` lines as advice for the agent to read, not as commands to execute automatically ([Recovery commands](#recovery-commands-fix)).
- Use `wait-post`/`wait-state` instead of your own polling loop; they sample once per second. Combine them with the read, start, re-read order under [Waiting for changes](#waiting-for-changes), because a wait does not report what changed before it started.
- Use a stored timestamp with `read-notification --since`, overlapping the previous window. There is no read cursor or exactly-once delivery; treat repeats by identity and content.
