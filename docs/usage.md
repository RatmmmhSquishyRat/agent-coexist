# Command reference

This page lists every command, flag, exit code, and output rule, so you can
look up exactly what a command prints and how to read it. For how to work
with other agents (what to write, how to catch a reply), run
`agent-coexist guide`; the same pages are in
[crates/cli/guide/](../crates/cli/guide/index.md). The
[README](../README.md) covers what the tool is and how to install it; the
[file format reference](file-format.md) shows the four file types on disk.
Examples use POSIX shell quoting (Git Bash on Windows). Each one assumes
the files, directories, and earlier comments or declarations it refers to
already exist; an example marked `# expected: error` fails on purpose.

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
| `guide` | [index.md](../crates/cli/guide/index.md) | what coexist is for, what to do when you arrive, and one complete exchange between two agents |
| `guide start` | [start.md](../crates/cli/guide/start.md) | choosing between post, brief, profile, and state |
| `guide read` | [read.md](../crates/cli/guide/read.md) | finding and reading existing files, and brief entry states |
| `guide write` | [write.md](../crates/cli/guide/write.md) | writing posts and handoffs, briefs, profiles, state, and closing out |
| `guide wait` | [wait.md](../crates/cli/guide/wait.md) | catching up, re-reading after the waiter is ready, and managing a blocking wait |

- Each call prints one static Markdown page to stdout (LF, trailing newline), with empty stderr and exit 0.
- Topics are exact lowercase; there is no `index`, `all`, or other alias. An unknown topic, extra arguments, or unsupported flags are a one-line stderr usage error, exit 2, with empty stdout and no workspace access.
- A stdout write or flush failure is exit 1, with no panic and no status line.
- `guide`, `help [COMMAND]`, `--help`, and `--version` are information output: no `ok`/`warn`/`error` first line, and no cwd, clock, storage, or lock setup. Invalid `AGENT_COEXIST_NOW` or `AGENT_COEXIST_LOCK_WAIT_MS` does not block them.

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

Each command in this table also answers `agent-coexist <command> --help`,
and `agent-coexist --help` lists them all. The fourteen rows after `guide`
are the business commands; they all use the line protocol described under
[Reading output](#reading-output).

The argument parser also adds `agent-coexist help [COMMAND]`. It is
information output, not a business command: `help` prints the same page as
`agent-coexist --help`. For any command in the table, `help <command>` prints
the same page as `<command> --help`. `help help` prints the helper's own
usage. `help --help` and `help <unknown>` are usage errors (exit 2).

## Conventions

### Files and paths

- Collaboration files end in `.agent-besties.md`. `PATH` arguments may omit the suffix; it is appended when missing. That check is byte-exact, so type the suffix in lowercase or leave it out: `docs/standup.AGENT-BESTIES.md` gets a second suffix appended. Scans match the suffix case-insensitively.
- `DIR` arguments default to `.` and must already exist (`E-DIR-MISSING` / `E-NOT-DIR`, exit 1). No command creates `DIR` itself.
- These forms create the file and any missing parent directories, printing `created  <dir>/` rows before `created  <file>`: `post` (new thread), `subscribe` (add), `profile` (bare or section set), `brief` (bare, description, `--target`).
- These forms never create: `post --edit`, `subscribe --cancel`, `profile --remove`, and the brief note-only, `--entry --remove` and `--refresh` forms. On a missing file they fail with `E-FILE-MISSING`, exit 1.
- If a creating write is refused (wrong type, `E-NAME`, a [functional lock](#functional-lock), a busy lock, a blocking parse error), the CLI deletes the empty file and empty parent directories this invocation just created, after checking they are still the same empty objects. Pre-existing files, files that already contain bytes, replacements at the same path, and directories that are no longer empty stay. An empty file or directory can therefore appear briefly and vanish; this cleanup is not crash-safe or atomic against other programs.
- Windows path display rules are under [Platform notes](#platform-notes).

### Names and identity

- Creating `post`, `profile`, `brief`, and `subscribe` forms build the H1 from the file-name stem as you typed it. The stem must be a valid name: 1–100 letters, digits, `_`, `-`, with `.` allowed after the first character. Otherwise the command fails with `E-NAME` (exit 1); the CLI never renames, truncates, or substitutes another name.
- An existing file that already parses stays editable when its stem and H1 disagree (`W-NAME` on `check`). Rename the file to the H1 name. Copying the stem into the H1 is offered only when the stem is itself a valid name.
- `--as <nick>` is your name. It is recorded in comment headers and matched by notifications. For `profile`/`brief` writes, it exempts you from functional locks recorded under that same nick (a lock held by any other nick still blocks). It is never authenticated (see [SECURITY.md](../SECURITY.md)). Nicks are 1–64 characters: a letter, digit, or `_` first, then letters, digits, `_`, or `-`.
- Statuses (`--status`) are 1–32 ASCII lowercase characters: `a-z`, then `a-z0-9-`. Section names follow the nick grammar.
- Entry titles have no grammar, but CLI flags reject control characters and double spaces. Titles are matched and displayed after whitespace folding: runs of spaces collapse to one, ends are trimmed.
- Title columns and human diagnostic text (message and `why`) use that folded display, with any remaining control characters shown as U+FFFD. Stored heading text and body bytes are not rewritten. A stored heading that already contains control characters is fixed by editing it by hand ([Headings the CLI cannot address](file-format.md#headings-the-cli-cannot-address)).
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
- Text rows are a `  <key>` line followed by `    | <line>` lines that reproduce content verbatim (read-brief bodies, wait before/after values; a binary or non-UTF-8 target is decoded lossily first). They may contain any spacing; read them by the `| ` prefix, never as columns. Empty content lines stay empty; no `/` is substituted.
- Diagnostics print as `  error  L<n>  <message>` (or `warn`), with `why` and `fix` lines indented 9 spaces underneath.
- Failures that are not anchored to a file line (a missing file, directory, or entry; the wrong type; a busy file; a functional lock; a refused state removal) print `reason  <message>` and `fix  <fix>` rows instead, with no `why`.
- IDs such as `E-LOCKED` or `W-STATE-CONFLICT` on these pages are catalog names for the kinds of diagnostic. Diagnostic rows and `reason` rows do not carry their own ID as a field; identify them by the printed message ([Troubleshooting](#troubleshooting) lists common ones). A `why` or `fix` text may mention another ID in passing, such as `(W-STATE-ENTRY)`; that names a related diagnostic, not the one reported. Only two row kinds carry an ID as a field: `remaining  <ID>  L<n>` after a brief write, and the `skipped`/`partial` rows of scans described next.
- Scan rows name what a scan could not fully read (a state file by its directory, any other file by its path), then the cause. A `skipped` row carries the ID that stopped the read, or the word `busy` or `unreadable` (the file or directory could not be read or listed). `busy` stands for either of two IDs: `E-LOCKED` (the file lock or a locked read stayed busy) or, on Windows, `E-INUSE` (another program holds the file open without allowing shared access); the recovery for each is under [Locks, retries and time](#locks-retries-and-time). A `partial` row carries an ID and `L<n>`. These rows cover any file a scan reads, posts included (for example `skipped  ./broken  E-FENCE` from `read-notification`); each command's section says which files its scan reads.
- The `partial` layout differs by producer. `read-notification`, `check-state`, and the `state` conflict scan print the ID and line as one value (`partial  <dir>  <ID> L<n>`, for example `partial  docs/standup  E-HEAD L9`); the ancestor lock scan of `profile` and `brief` prints them as two columns (`partial  <dir>  <ID>  L<n>`). Match the ID by its pattern, not by a fixed column count.
- `canonicalize  <dir>  failed` means the real path of that directory (the scanned `DIR`, or a directory holding a state file the scan found) could not be resolved, so its identity fell back to the path as spelled. Locks and conflicts are then related by spelling alone, and an overlap reached through a symlink, a junction, or another spelling of the same directory can be missed. `check-state`, `read-notification`, the `state` conflict scan, and the `profile`/`brief` lock scan can print it. It is not a record, so a `total` row does not count it. It turns an `ok` first line into `warn`; a first line that is already `error`, such as `E-FLOCKED`, stays `error` with its exit code.
- A `brief` report, `--refresh` included, can hold both kinds of skip. `entry  <title>  skipped  no path` is a refresh skip and carries no ID; a row whose key is `skipped` or `partial` there always comes from the ancestor lock scan ([example](#refresh-report-example)).
- Completed `list`, `lookfor-comment`, `read-notification`, and `check-state` overviews (`ok`/`warn`, including zero records) end with a `total` row. Fatal reports of those commands (missing dir or file, not a directory, `cannot scan`, other OS errors, blocking diagnostics) have no `total` row. `check`, `read-brief`, write commands, and wait reports never have one.
- Lines always end in LF, even on Windows.
- Progress (`scanning N dirs, M files...`) goes to stderr, and only when stderr is a TTY. Besides the read scans, it covers the ancestor lock scans of `profile` and `brief` and the post-write conflict scan of a `state` declaration, which reads the ancestor state files and walks DIR's whole subtree, so it can take a while on a large tree. The `state --remove` form runs no such scan. Stdout stays clean for redirection.

### Exit codes

| Code | Means | Action |
|---|---|---|
| 0 | first line `ok` | done |
| 0 | first line `warn`: the command completed, with rows to read, for example conflicts, skipped or partial files, refresh skips, or dangling reply targets | success; read the rows |
| 0 | `guide`, `help [COMMAND]`, `--help`, `--version` information output (no status line) | done |
| 1 | first line `error`: diagnostics, missing file/entry/dir, type mismatch, I/O | fix the input |
| 1 | a business command could not write its report to stdout (for example a closed pipe): stdout can hold any prefix of the report, even a complete `ok` or `warn` first line, and no error report is added | check the output destination; after a write command, inspect the written file before retrying (see the note below) |
| 1 | `guide` stdout write or flush failure (no status line) | retry the output sink |
| 2 | usage error, including argument-parser failures, invalid argument values or combinations, and invalid `AGENT_COEXIST_*` values on business commands: a single `error: <message>` line on stderr (CR/LF become spaces), stdout empty. When the argument parser supplies usage or help hints, they stay on that line; other usage errors, such as `error: --regex requires --target <path>` or an invalid `AGENT_COEXIST_NOW`, print only the message | fix the command line |
| 3 | `E-LOCKED`, printed as `reason  file is locked by another process`: the required file lock, stable identity, or stable contents could not be obtained or maintained | retry the same command shortly |
| 3 | `E-INUSE`, printed as `reason  file is in use by another program`: Windows only; another program opened the file without allowing shared access (a sharing violation) | close it, then retry |
| 3 | `E-FLOCKED`, printed as `reason  <dir> is locked by <holder> until <time>`: functional lock, see [Functional lock](#functional-lock) | retry at that time, or ask the holder |
| 130 | a wait was cancelled with Ctrl-C | — |

`warn` is exit 0. Scripts that care about conflicts must read the first line
of a business report, not only the exit code. Do not parse `guide`, `help`,
`--help`, or `--version` output as `ok`/`warn`/`error`. The `warn` causes in
the table are examples, not a complete list; the section for each command
describes the rows specific to it, for example [brief writes](#briefs),
[read-brief](#reading-briefs), [state declarations](#declaring), and
[state removals](#removing).

A write command commits its file change before it prints the report, so
after that exit-1 stdout failure the change may already be on disk. Check
the output destination, then read the file back (`check`, `check-state`,
or `read-brief`) before retrying; do not rerun the write blindly. A failed
final flush of a business report is ignored: the command keeps the
report's own exit code, so that code does not prove the whole report was
delivered. `guide` is different: it checks both the write and the flush,
and exits 1 if either fails.

## Profiles

```sh
agent-coexist profile agents/alice                        # create; else unchanged
agent-coexist profile agents/alice --section role -m 'Rust implementer.'
agent-coexist profile agents/alice --section role --remove
```

- Without `--section`: creates the file (heading only) or reports the existing file `unchanged`. A new file prints its `created` rows; an existing one prints `unchanged  <file>`.
- `--section <name> -m <content>` sets or replaces that section's body, stored verbatim in a code fence. `--section` rejects control characters and names outside the nick grammar.
- `--section <name> --remove` deletes the whole section. Unknown section: `E-SECTION-NOTFOUND`, exit 1. `--remove` never creates the file.
- A set prints `section  <name>  set`, after any `created` rows; replacing an existing section prints the same row. A removal prints `section  <name>  removed`, then `removed-lines  L<a>..L<b>`: the deleted lines, numbered as in the file before the write.
- After a successful byte-writing form, including the bare create, rows from the ancestor lock scan (`skipped`, `partial`, `canonicalize`) can follow and make the first line `warn` ([Reading output](#reading-output)). The `unchanged` form runs no lock scan. A refusal is described under [Functional lock](#functional-lock).
- A stored section title outside the nick grammar cannot be addressed with `--section`; `check` tells you how to rename it by hand ([Headings the CLI cannot address](file-format.md#headings-the-cli-cannot-address)).
- `--as` is optional here: it only exempts you from `locked` declarations recorded under that same nick (other holders still block; see [Functional lock](#functional-lock)) and is never written to the file.

## Posts and subscriptions

```sh
agent-coexist post docs/standup --as alice -m '…'           # append; creates
agent-coexist post docs/standup --as alice -m '…' --edit 1  # rewrite own #1
agent-coexist subscribe docs/standup --as bob               # add subscriber
agent-coexist subscribe docs/standup --as bob --cancel      # remove
```

### Comments, mentions, and replies

- Comments are numbered `#1`, `#2`, … in file order. A gap, including a first legal number greater than 1, is a non-blocking warning; the CLI never renumbers.
- Mentions (`@nick`) and replies (`@#n`) are plain text in the body. A mention starts at the beginning of the text or after any character other than a letter, digit, `_`, or `@`: `@alice-@bob` mentions `alice-` and `bob`; `a@bob` is not a mention.
- A successful write prints `comment  #<n>  L<line>`: the comment's number and the line of its header. With `--edit` the row has a third column, `comment  #<n>  L<line>  edited`. A new thread prints its `created` rows first.
- Next comes a single `mentions` row when the new body mentions anyone: every distinct `@nick`, joined by `, ` in first-occurrence order (`mentions  bob, alice`). Then one `reply-to  #<m>` row per distinct `@#m`, in first-occurrence order. A body with no mentions has no `mentions` row, and one with no replies has no `reply-to` rows; nothing follows these rows.
- A reply whose number has no comment in the file after the write prints `reply-to  #<m>  (not found)` and makes the first line `warn`; the comment is still written.

  For example, when `docs/standup` was written only by `post` and holds #1 and #2 with one-line bodies, this comment becomes #3 (column padding is illustrative):

  ```sh
  agent-coexist post docs/standup --as carol -m '@#2 @#7 Agreed, @bob and @alice. Thanks @bob.'
  ```

  ```text
  warn  post  docs/standup
    comment       #3  L15
    mentions      bob, alice
    reply-to      #2
    reply-to      #7  (not found)
  ```

- `check` of a post lists `authors` in first-occurrence order (`none` when empty; a single author whose nick is literally `none` prints the same, so read the file when the difference matters).
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

- A successful `subscribe` report is always `ok` and has one `subscribers  <list>` row: the subscribers after the command, joined by `, `. Adding to a missing file prints its `created` rows first.
- Subscribing twice, or cancelling when not subscribed, leaves the file unchanged: the `subscribers` row shows the current list and is followed by `unchanged  <nick>`.
- `--cancel` edits only the first authoritative subscribers property. Deleting its last nick deletes that property line; later duplicate properties stay as stored, and the report lists the subscribers of the resulting document. A surviving duplicate can therefore become the list. The report says `none` when that list is empty, and also when its only nick is literally `none`; read the file when the difference matters.
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
- `--refresh` recomputes and rewrites stored hashes only; it never re-checks regexes. Per selected entry it prints `entry  <title>  refreshed  <path>`, or a skip row `entry  <title>  skipped  no path` / `entry  <title>  skipped  target missing` (skips make the first line `warn`, exit stays 0). Without `--entry`, every entry is refreshed. Unknown title: `E-ENTRY-NOTFOUND`, exit 1.
- Review changed targets and update outdated notes before refreshing: `fresh` means the hash matches, not that anyone has checked the note.
- Combination rules: `--regex` requires `--target`; `--refresh` excludes `--target`, `-m`, and `--remove`; `--remove` requires `--entry` and excludes `--target` and `-m`; without `--target`, `--entry` requires `-m`, `--remove`, or `--refresh`. Violations are usage errors, exit 2, and write nothing.
- After any byte-writing brief form, the new bytes are re-parsed and every surviving non-blocking error-level (`E-…`) diagnostic is appended as a `remaining  <E-ID>  L<n>` row, in line order, making the first line `warn`. These are the last rows, after any ancestor lock-scan rows. Warnings (`W-…`) and entry states are not listed; run `check` to see them.

### Refresh report example

A refresh of a brief with one entry lacking a path, one with a missing
target, and one changed target, in a directory whose parent holds a broken
`state.agent-besties.md` (actual output):

```text
warn  brief  notes
  entry         no-path  skipped  no path
  entry         missing-target  skipped  target missing
  entry         changed-target  refreshed  target.txt
  skipped       ..  E-H1
  remaining     E-ENTRY-PATH  L9
  remaining     E-ENTRY-HASH  L9
```

The two `entry … skipped` rows are refresh skips. `skipped  ..  E-H1`
comes from the ancestor lock scan: the parent's state file has no
recognizable H1, so its declarations were not read. The `remaining` rows
are the errors left in the entry without a path. The same brief also has a
stray-text warning and a `stale` entry; the refresh report lists neither,
so run `check` to see them.

### Entry states (what readers see)

| State | Meaning | `reason` shown |
|---|---|---|
| `fresh` | stored hash matches the target's current bytes | — |
| `shifted` | hash differs: the target changed after the hash was written | — |
| `stale` | target missing, regex invalid, or regex not found | that reason |
| `unchecked` | no path, target unreadable, target too large, or no hash | that reason |

Targets are hashed by streaming, with no size cap, but at most 64 MiB of a
target is held in memory for regex matching and display. For a target over
64 MiB:

- An entry with a regex is `unchecked` with `target too large` in both `check` and `read-brief`: the regex cannot be verified, and the hash alone does not make it `fresh`.
- An entry without a regex is judged by its hash as usual, and `check` reports that state normally. `read-brief` omits its content block, adds `target too large` after the path of a `fresh` or `shifted` entry, and makes the first line `warn`.

### Reading briefs

- `check` on a brief validates every entry and reports states without printing target bodies.
- `read-brief` prints the description, then for each entry an `entry` row (`<title>  <state>  <path>`, then the reason if there is one, then `L<a>..L<b>` when content is printed for a nonempty target), `group` rows for the named capture groups of the first match, the `note` text block when the entry has a note, and a `content` text block. Without a regex, `content` is the whole target. With a regex, it is only the whole lines covered by the first match, and `L<a>..L<b>` names them. The hash still covers the whole file, so a change outside those lines makes the entry `shifted`.
- A target over 64 MiB never prints `content`, whatever the entry's state. A `fresh` or `shifted` entry then shows `target too large` as its reason; any other state keeps its own reason, such as `unchecked  no hash` (see [Entry states](#entry-states-what-readers-see)). Within that limit, `content` is also omitted for every `stale` entry (`target missing`, `regex invalid`, `regex not found`) and for `unchecked` entries with `no path` (path column `?`) or `target unreadable`, with the reason as the `entry` row's last column. An `unchecked  no hash` entry within the limit still prints its content and range.
- `read-brief --entry <title>` prints that one entry under the same rules (unknown title: `E-ENTRY-NOTFOUND`, exit 1). It is not a status-only switch; use `check` for states alone.
- Any shifted, stale, or unchecked entry, or any omitted content, makes the first line `warn`.

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
- A successful declaration prints `set  <status>  <nick>  <start>..<until>`, preceded by `created  <file>` when the state file is new, then one `cleaned  <status>  <nick>  <start>..<until>` row per expired entry it removed.
- After writing, the declare form checks the declaring nick's own valid `locked` entries in the directory's scope, whatever status this write declared: a nick that already holds a valid lock and now declares `observing` is still checked against it. Each valid `locked` entry held by another nick on an overlapping directory (equal, or ancestor/descendant) produces a `conflict  locked  <nick>  <start>..<until>  <dir>` row that names that other holder and its directory, and the first line becomes `warn`; exit stays 0. A conflict between two other nicks is not reported by this declaration; `check-state` marks conflicting entries across the queried scope (see [Scope, inheritance, and conflicts](#scope-inheritance-and-conflicts)). Another agent's `locked` never blocks a state write.
- If that post-write scan hits a fatal ignore-source error, the write is already committed: first line `warn`, the usual `created` / `set` / `cleaned` rows, then `scan  <path>  <cause>`, exit 0. There is no `fix` row and no rollback; `check` of the state file shows the written declaration.

### Removing

- `--remove <nick>` prints `removed  <status>  <nick>` per section. `--as` defaults to the removed nick; a different `--as` still removes but adds a `removed-other` warn row.
- Without `--status`, the nick is removed from every status that holds a clean entry for it. That span is intentional; add `--status` to remove only one status.
- Only clean lines are rewritten. A line is clean when every comma fragment that is nonempty after trimming parses as a declaration; empty fragments are ignored. A comma-only line holds no declarations and can disappear with its section.
- Mixed lines: if the nick still has a parseable declaration on a line with an unparseable nonempty fragment, under a selected status, the whole removal is refused before any write (`E-STATE-DIRTY`, exit 1) and the line stays byte-for-byte. Make that line parseable by hand (or delete the nick from it), then re-run. The printed retry keeps the complete original command, including `--status`. Legal fragments on mixed lines still count in `check` and lock scans.
- A status section whose remaining bytes are only its heading and blank lines is deleted (`removed-lines`). Leftover handwritten text, mixed lines, and fences keep the heading and those bytes.
- A file reduced to its heading, reader comments, and blanks is deleted whole (`deleted  <file>`).
- Missing file or nick: `E-STATE-NOTFOUND`, exit 1, nothing written. The `fix` is always a read-only `check-state` of the directory; with `--status`, the reason also names that status. Do not drop `--status` and remove a different declaration instead; a live `locked` entry would go with it.
- A same-file lock-conflict `fix` includes `--status=locked`, so copying it removes only that lock and leaves the nick's other statuses.
- `--for` is rejected in the remove form (usage error, exit 2).

### Scope, inheritance, and conflicts

- A state file describes its directory's whole subtree.
- `check-state DIR` prints, in order: `inherited` rows from ancestor directories (valid entries only, nearest first), DIR's own entries, then one group per descendant state file, headed by its path relative to DIR plus `/` and ordered by directory. After the entries come `skipped`/`partial` rows sorted by directory, then `canonicalize` rows ([Reading output](#reading-output)), then `include` hints ([Scans](#scans-what-is-visited-and-filter-precedence)), then `total`. Within one file, entry rows keep native file order.
- Row shapes, split on two or more spaces:
  - `inherited  <status>  <nick>  <start>..<until>  <dir>  [conflict]`: `<dir>` is the ancestor that holds the entry, spelled from the current directory.
  - `<status>  <nick>  <start>..<until>  [conflict|expired]`: DIR's own entries, at indent 2. Entries in a descendant group use the same shape at indent 4, under their group heading.
- The last column is an optional mark, at most one per row. `conflict` means that the entry is a valid `locked` and that another nick holds a valid `locked` on an overlapping directory (equal, or ancestor/descendant) somewhere in the same report. Both entries of such a pair are marked, whether they are inherited, own, or descendant, and whether or not they are in the same file. Sibling directories do not overlap. `expired` marks an own or descendant entry that is no longer valid. Inherited rows are always valid, so they never carry `expired`. Either mark makes the first line `warn` (exit 0). `total` counts every entry row plus the `skipped`/`partial` rows.
- Example: `./state.agent-besties.md` holds `locked alice`, `child/state.agent-besties.md` holds `locked bob` and `locked carol` plus an `observing eve` that has ended, and `child/grand/` holds `locked dave`. All locks are still valid. `check-state child` prints:

  ```text
  warn  check-state  child
    inherited     locked  alice  2026-09-17T08:00:00Z..2026-09-17T12:00:00Z  .  conflict
    locked        bob  2026-09-17T08:00:00Z..2026-09-17T12:00:00Z  conflict
    locked        carol  2026-09-17T08:30:00Z..2026-09-17T12:00:00Z  conflict
    observing     eve  2026-09-17T08:00:00Z..2026-09-17T09:00:00Z  expired
    grand/
      locked        dave  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z  conflict
    total         5
  ```

- `read-notification` also reads ancestor state files and counts DIR's own state file once.
- `check` on a state file reports each expiry and each same-file lock conflict (`W-STATE-CONFLICT`) once, from the selected declaration per nick. It also marks the file's own preview rows `conflict` or `expired`. These `W-STATE-CONFLICT` pair diagnostics (`locked is held by both <a> and <b>`) cover pairs inside one file. They appear only in `check` and in a wait report that hits (including the snapshot from just before a deletion). A complete `check` lists every pair, so many overlapping `locked` entries make a large report.
- The other commands that report overlapping locks print no pair diagnostics. `check-state` marks the entries, `read-notification` prints `conflict` records for locks that overlap your own, and a `state` declaration prints `conflict` rows. `list` does not report conflicts.
- A `read-notification` conflict names the deepest overlapping own lock; when depths tie, the last file in scope order wins. Per-file conflict rows are never dropped.

### Functional lock

- The byte-writing `profile` and `brief` forms fail with `E-FLOCKED` (exit 3) when a *different* nick holds a valid `locked` on the supplied file's directory or any ancestor. Without `--as`, any holder blocks.
- A write through a file symlink also checks the canonical referent's parent directory and its ancestors. If the file's real path cannot be resolved, only the supplied path's chain is checked, and that alone prints no row. If the supplied directory's own real path cannot be resolved, its chain is checked as spelled and the report adds `canonicalize  <dir>  failed` ([Reading output](#reading-output)). There is no volume-wide hardlink search.
- Retry at the expiry time shown in `reason`, or ask the holder in a post to release the lock (`post` is not blocked). Pass `--as <holder>` only when the lock really is yours.
- When several declarations block, `reason` names the one with the latest expiry (ties go to the nick that sorts first), and each other one follows as `also-locked  <nick>  until <time>`, latest expiry first, ties by nick. The time in `reason` is therefore when the last blocking lock found expires. `also-locked` rows name no directory; a nick holding `locked` on two directories of the chain is listed once for each. Any `skipped`/`partial` rows come next, then `canonicalize` rows. The first line stays `error`, exit 3.
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
2. `--include`/`--exclude` then filter the collected file names. They cannot reopen an ignore-pruned directory. When you passed at least one `--include`, ignore rules were active (no `--no-ignore`), and no file survived step 3, the command prints one `include  <glob>  matched 0 files; ignored directories are not reopened (pass --no-ignore)` row for every `--include` you passed, and the first line becomes `warn`. These rows report the empty final result, not a per-glob match count:
   - `--include 'a/**' --include 'b/**'`, where a file under `a/` is kept: no `include` rows, even though `b/**` matched nothing.
   - `--include 'a/**' --exclude 'a/old/**'`, where every `a/` file is under `a/old/`: an `include  a/**  matched 0 files; …` row, even though `a/**` matched files. The exclude removed them, not ignore rules.
3. A file is kept when (no `--include` was given, or one matches) and no `--exclude` matches. Exclude always wins; repeated flags form a union.

- An ignore source that is not a regular file, or cannot be read or parsed, fails the scan: first line `error`, `reason  cannot scan <path>: <source>`, a retry `fix`, exit 1, with no overview rows and no descendant groups. `--no-ignore` skips those sources. Ordinary unreadable directories are only `skipped` warnings. For a pattern that does not parse, the retry `fix` is generic and does not repair it; see [Before you run a command](#before-you-run-a-command).
- Globs are gitignore-style and relative to `DIR`: a leading `/` anchors at `DIR`; a trailing `/` matches files under that directory, not a same-named file; `*` and `?` never cross `/`, but `**` does. Matching is case-insensitive on Windows and macOS.
- Globs are matched exactly as typed, including internal double spaces. Only the display of the `include` hint row is folded, with control characters shown as U+FFFD. An invalid glob is a usage error (exit 2).

### list

Prints a tree with a type column. The first row names the active ignore
sources (a subset of `.gitignore, .ignore, exclude, global`, or `none`, or
`--no-ignore`). Leaves shown as `busy` (`E-LOCKED`, or on Windows `E-INUSE`,
as for scan rows in [Reading output](#reading-output)) or `?` (unreadable:
permissions, not UTF-8, bad heading, over the size cap) still count toward
`total` and turn the first line `warn`, never a failure. `--type` filters
by type and repeats as a union; `profile` is an accepted alias for
`agent-profile`. `busy` and `?` leaves stay shown and counted under any
`--type`, because their type is unknown.

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
`L<line>  #<seq>  <author>  <time>  <excerpt>`. `<time>` is the activity
time (`edited`, else `created`). The excerpt is the same as in
[read-notification](#read-notification) comment records: the first
non-blank body line, whitespace-folded; beyond 80 characters it is cut to
77 plus `...`. A missing or blank body has no excerpt column.

- `--keyword` is a case-insensitive substring over the raw body and the comment's original header line, used exactly as typed (whitespace is not folded).
- `--by` is the author, `--mention` a body `@nick`, `--reply-to` a body `@#n`, and `--comment` selects one sequence number.

### read-notification

```sh
agent-coexist read-notification --as alice --since 1h
agent-coexist read-notification --as alice \
  --since 2026-09-17T09:12:00Z --include 'src/**'
```

Records, oldest first. Ties are broken by path (the post, or the directory holding the state file), then by kind in the order of this table, then by comment number or file order:

| Record | About | When it appears |
|---|---|---|
| `mention` | others | a comment contains `@<you>` |
| `reply` | others | a comment's `@#n` targets a comment authored by you |
| `subscribed` | others | any comment while you are on the subscriber list |
| `state` | others | a declaration (any status) started in the window; shows `until <t>`, flagged `expired` if over |
| `conflict` | others | a valid `locked` overlapping yours: `overlaps your lock on <dir>`, naming the deepest overlapping own lock |
| `expiring` | you | your valid entry ends within 30 minutes: `ends in Nm` |
| `expired` | you | your entry ended inside the window: `expired at <t>` |

- Comment records appear when their activity time (the `edited` time if the header has one, else `created`) is in `[since, now)`. Your own comments never notify you. A hand-written header therefore needs the current time ([Editing files by hand](file-format.md#editing-files-by-hand)).
- Each comment produces at most one record for you: `reply` beats `mention`, which beats `subscribed`.
- Expiry and conflict records live here. `check-state` marks entry rows `expired` or `conflict` instead and never prints them as records ([Scope, inheritance, and conflicts](#scope-inheritance-and-conflicts)).
- Busy or unparseable files become `skipped`/`partial` rows and make the first line `warn`, as do conflict rows.
- Times are whole seconds and the current second is excluded, so a query in the same second as the activity can miss it. Make each new `--since` overlap the previous window, and treat repeated records by identity and content. There is no stored read cursor and no exactly-once delivery.

A completed report, run at `2026-09-17T10:20:00Z` with `--as alice --since 2026-09-17T08:00:00Z`:

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  mention     2026-09-17T09:20:00Z  ./docs/standup #4  bob  @alice can you review the parser change
  reply       2026-09-17T09:25:00Z  ./docs/standup #5  eve  @#2 done, tests pass
  state       2026-09-17T09:30:00Z  ./src/parser  locked  bob  until 2026-09-17T11:30:00Z
  conflict    2026-09-17T09:30:00Z  ./src/parser  locked  bob  overlaps your lock on ./src/parser
  expiring    2026-09-17T10:45:00Z  ./src/parser  locked  alice  ends in 25m
  skipped     ./docs/notes  E-H1
  total         6
```

Split rows on two or more spaces ([Reading output](#reading-output)). With the default `DIR` of `.`, paths below it start with `./`, and the root directory itself shows as `.`. Row by row:

- `as  <nick>` and `since  <time>` come first. `since` is the window start in UTC; a duration such as `1h` is already resolved against the current time.
- Comment records are `<kind>  <time>  <post> #<n>  <author>  <excerpt>`. `<time>` is the activity time (`edited`, else `created`). `<post> #<n>` is one value: the post path without `.agent-besties.md`, one space, then the comment number. `<author>` wrote the comment. The excerpt is the first non-blank body line, whitespace-folded; beyond 80 characters it is cut to 77 plus `...`. A blank body has no excerpt column.
- State records are `<kind>  <time>  <dir>  <status>  <nick>  <detail>`, where `<dir>` holds the declaration:

  | Kind | `<time>` | `<nick>` | `<detail>` |
  |---|---|---|---|
  | `state` | the declaration's start | the other agent | `until <t>`, followed by a separate `expired` column when the report runs at or after `<t>` |
  | `conflict` | the other entry's start | the other holder | `overlaps your lock on <your dir>` |
  | `expiring` | your entry's until | you | `ends in <N>m`, whole minutes rounded down |
  | `expired` | your entry's until | you | `expired at <t>`, the same time again |

- After all records: `skipped`/`partial` rows (`<kind>  <path>  <cause>`, no time column, sorted by path; causes under [Reading output](#reading-output)), then `canonicalize  <dir>  failed` rows, then, when `--include` was given, ignore rules were active (no `--no-ignore`), and no file survived the filters, one `include  <glob>  matched 0 files; …` hint per supplied `--include`, whether or not that glob matched ([Scans](#scans-what-is-visited-and-filter-precedence)), then `total`.
- `total` counts records plus `skipped`/`partial` rows; `canonicalize` and `include` rows are not counted. The example has five records and one `skipped` row.

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

- The parent directory must exist (`E-DIR-MISSING`, exit 1). If it exists but is not a directory, `wait` and `wait-post` print an `error` report with `reason  cannot read <dir>: not a directory` and the prose `fix` `fix the path, then run the corrected command`, exit 1; `wait-state` reports `E-NOT-DIR`, exit 1.
- After the baseline and ready line, the file is sampled once per second until the first qualifying change prints one report (exit 0). Nothing is written, and no lock is held between samples.
- Every sample, the baseline included, waits up to 1 s for a shared lock. A lock still busy after that second ends the wait with `E-LOCKED`, exit 3; the next sample does not retry it. On Windows, a file another program opened without allowing shared access ends the wait at once with `E-INUSE`, exit 3. This 1 s budget covers only that one lock acquisition; it is not a timeout for the wait as a whole or for reading the file.
- A missing file is a legitimate baseline: `wait-post docs/standup` on a thread that does not exist yet waits for its creation. A mistyped path therefore waits forever; check the path.
- `PATH` gets the managed suffix as in every command: `agent-coexist wait notes` watches `notes.agent-besties.md`, never a plain file named `notes`, even when only the plain file exists; in that case it waits for `notes.agent-besties.md` to be created.
- An invalid snapshot fails the wait with an error report, exit 1, whether it is the baseline or any later sample; it never becomes the baseline. Invalid means blocking diagnostics, an unknown type, the wrong type for `wait-post`/`wait-state` (`E-TYPE`), or duplicate `(status, nick)` pairs in a state file. A hand edit after the ready line that, for example, leaves an unclosed fence therefore ends the wait with exit 1 rather than waiting on.
- A file over the 16 MiB cap is refused with the same report as `check`: first line `error`, a `file` row, `E-TOO-LARGE`, exit 1, no `type` row. This applies on the first sample and if the file grows past the cap later; it is not a change hit.
- Baselines advance: a valid snapshot that changed but did not qualify becomes the new baseline. Diffs are always between adjacent snapshots, so a wait never replays earlier messages.

### What ends a wait

| Command | Filter | Hits on |
|---|---|---|
| `wait` | — | any byte change, any of the four types |
| `wait-post` | — | any byte change of the post |
| `wait-post` | `--as <nick>` | a comment relevant to `<nick>` (a direct `@nick`, or an `@#n` that targets a comment authored by `<nick>`) that is new, edited, or newly relevant |
| `wait-state` | — | any byte change of `DIR/state.agent-besties.md` |
| `wait-state` | `--status <s>` | an add, removal, or `start`/`until` change of declarations under `<s>` (renewals hit) |

- `wait-state` watches exactly `DIR/state.agent-besties.md`: no ancestors, no subtree.
- Subscriber-list changes, comment deletions, and reorders never hit `wait-post --as`. Relevance is comment-based, so subscribing does not make every comment a trigger.
- Clock expiry alone never hits `wait-state --status`. A file deletion hits only if the file held `<s>` declarations before.

### The wait report

- The report starts with `bytes  changed` and `semantic  changed|unchanged`, then `created  <file>` / `removed  <file>` if the file appeared or vanished.
- Entity rows follow: `add`, `remove`, or `update`, with field values as verbatim text blocks. Updates show `before`/`after`; additions and removals show the fields of the side that exists. A missing field reads `none`, which prints exactly like a field whose stored value is the text `none`; the report cannot tell them apart, so read the file itself when it matters. Order rows come last.
- Comments, entries, sections, and declarations are paired by raw identity (first match wins) and reported in file order.
- Filtered hits (`wait-post --as`, `wait-state --status`) list only the qualifying entities. They still print the `bytes`, `semantic`, `created`/`removed`, and warning rows, but never the document name or type, a brief description, or order rows.
- A brief's description appears once when the file is created, deleted, or changes type.
- Same-file lock-conflict warnings are included on a qualifying hit (also for the snapshot just before a deletion), evaluated at the wait's start time.
- State `fix` recipes in the report are checked against the directory's currently selected `state.agent-besties.md` at that final snapshot. A `removed` report can still show warnings from the deleted baseline, but it will not print a directory-level `state` command against a different, later selected file.

## Locks, retries and time

| Commands | Lock | Behavior when busy |
|---|---|---|
| `post`, `subscribe` | exclusive | queue up to 5 s, then `E-LOCKED` (3) |
| `profile`, `brief`, `state` | exclusive | fail fast (~200 ms), `E-LOCKED` (3) |
| `check`, `lookfor-comment`, `read-brief` | shared | 1 s, then `E-LOCKED` (3) |
| `wait`, `wait-post`, `wait-state` | shared, per sample | 1 s, then `E-LOCKED` (3), ending the wait; not retried ([Lifecycle](#lifecycle)) |
| scans, plus state's conflict scan | shared | ~100 ms, then `busy`/`skipped`/`partial` rows + `warn` |

- `E-LOCKED`: the required file lock, stable identity, or stable contents could not be obtained or maintained; retry the same command shortly. It does not name a holder: another lock holder, exhausted identity-reopen retries, or contents that keep changing can all cause it.
- `E-INUSE` (Windows only): another program opened the file without allowing shared access, a sharing violation. Merely having the file open in an editor does not cause it unless that program denies sharing. Close that program's handle and retry. A scan does not fail on it: the file is reported as `busy`, the same word as for `E-LOCKED`. If a file stays `busy` on Windows across retries, find the program holding it open without sharing and close that handle; retrying alone does not clear a sharing denial.
- `E-FLOCKED`: the functional lock described under [Functional lock](#functional-lock); retry at the expiry time in `reason`, not at millisecond cadence.
- Collaboration files are capped at 16 MiB (`E-TOO-LARGE`).
- `guide`, `help`, `--help`, and `--version` take no file locks and do not scan the workspace.
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
"no locks" nor as a reason to refuse every write. A `canonicalize  <dir>  failed`
row marks another kind of gap: directories were compared by spelled path
instead of real path, so an overlap through a link can be missed
([Reading output](#reading-output)).

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

- `W-STATE-CONFLICT` (`locked is held by both <a> and <b>`): agree with the other holder first; only the agent that yields runs the `--remove … --status=locked` command.
- `E-FLOCKED`: wait for the expiry or ask the holder. Passing `--as` with the holder's nick is only for the holder itself.
- `W-ENTRY-SHIFTED` (`entry "<title>" is shifted: <target> changed since its hash was taken`): review the target and update the note before `--refresh`; refreshing only accepts the new hash.
- `E-STATE-DIRTY`: make the named line parseable by hand, then paste the printed retry. It keeps the original `--status`; do not rebuild a bare `--remove` that would also drop other statuses.
- `E-STATE-NOTFOUND`: the fix is a read-only `check-state`; inspect, then correct your own command ([Removing](#removing)).
- `cannot scan <path>: line <n>: error parsing glob …`: an ignore source holds a malformed pattern. The printed `fix` still reads `check the file's permissions and free space, then retry: …`; that generic text is a known limitation and does not repair the pattern. Edit the pattern at the file and line named in `reason` (the message may suggest an escape, such as `[{]` for a literal `{`), then rerun. `--no-ignore` bypasses every ignore source when you mean to scan without them.
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
- A stored heading cannot be addressed by `--entry`, `--section`, or `--status` (control characters, a profile section title outside the nick grammar, an invalid status name): see [Headings the CLI cannot address](file-format.md#headings-the-cli-cannot-address).
- A state-content file is not the directory's selected `state.agent-besties.md` (see [Troubleshooting](#troubleshooting)).
- Comment numbering is exhausted: the fix describes a heading change or a new post.

Hand-edit rules for the files themselves are in
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

Each entry starts with the text a report prints, followed by the catalog ID used elsewhere on this page.

- `target <path> does not exist` (`E-TARGET-MISSING`) on a brief whose target obviously exists: the path is relative to the brief's directory, not your cwd.
- `--include` finds nothing, or `include  <glob>  matched 0 files; …` rows appear: the rows mean no file survived all filters, not that this particular glob matched nothing ([Scans](#scans-what-is-visited-and-filter-precedence)). Check that each glob is relative to `DIR`, and check your `--exclude` globs: exclude always wins and can remove every match. If neither explains it, ignore rules may have pruned those directories; try `--no-ignore`.
- `error  check-state` / `cannot scan`: an ignore source failed to load or parse (the same as for `list` and `read-notification`), not an unreadable directory. For a pattern parse error, fix the named line first ([Before you run a command](#before-you-run-a-command)). If a `state` write succeeded first, you get `warn` with a `scan  <path>  <cause>` row, exit 0, and the written bytes stay.
- Exit 0 but the first line is `warn`: the command finished; read the rows. Common causes include state conflicts, refresh skips, busy or partial state files, a dangling `@#n`, and incomplete lock scans. This is not a complete list; see the command's own section and [Exit codes](#exit-codes).
- `<dir> is locked by <holder> until <time>` (`E-FLOCKED`) on `profile`/`brief`: `<holder>` has a valid `locked` on that directory or above, possibly on a symlink target's directory. Only the `--as` nick's own declaration is exempt; without `--as`, any holder blocks, including you ([Functional lock](#functional-lock)). If `<holder>` is you, re-run with `--as <holder>`; otherwise retry at the shown expiry, or ask the holder in a post (`post` and `subscribe` are not blocked).
- `file name "<name>" is not a valid name` (`E-NAME`) on a creating command: the file-name stem is not a valid name. Choose a valid name and re-run with that path.
- `file is a <type>, not a <type>` (`E-TYPE`): the heading names a different type. Use the command for that type, or pick another path. For a state heading, a directory-level `state` command is printed only when the file is that directory's selected `state.agent-besties.md`.
- `check` warns `state file is named "<name>.agent-besties.md", not "state.agent-besties.md"`: `state` and `check-state` ignore it. Inspect it and any existing `state.agent-besties.md` in the same directory, resolve their declarations, then decide which file keeps the reserved name; never overwrite a separate existing state file. Advice about that inspected file is hand-edit only.
- `<nick> is still declared on line <n> of the state file in <dir>, mixed with unparseable fragments` (`E-STATE-DIRTY`): see [Removing](#removing).
- `<nick> has no entry in <dir>`, or `… under status "<status>" in <dir>` (`E-STATE-NOTFOUND`): no matching nick under the selected status (or no file), and nothing was written; see [Removing](#removing).
- Your own `--remove` removed more than one status: without `--status` it spans every status that holds the nick.
- `file name "<stem>" does not match the H1 name "<name>"` (`W-NAME`), or a broken H1, after a hand rename: the file stays usable if it parses. If the stem is not a valid name, rename the file to match the H1.
- A heading with control characters, an invalid section name, or an invalid status: rename it by hand; see [Headings the CLI cannot address](file-format.md#headings-the-cli-cannot-address).
- File not created: you used a never-creating form (`--edit`, `--cancel`, `--remove`, `--refresh`, note-only `-m`) on a missing file.
- A doubled suffix in a filename: the suffix append is case-sensitive; drop or lowercase the suffix you typed.
- Duration rejected: units must be strictly descending (`1h30m`, not `1m30h`).
- Empty `-m` rejected: trailing whitespace is stripped and something must remain.
- A wait never returns: check the path (a missing file is a valid baseline, and a plain file without the suffix is never watched) and the filter; see [What ends a wait](#what-ends-a-wait).

## Environment overrides

Both exist for testing and advanced automation; normal use needs neither.
Business commands validate them; `guide`, `help`, `--help`, and `--version`
ignore them.

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

- For exit 3, tell the cases apart by the text of the `reason` row, which carries no ID field. `file is locked by another process` (`E-LOCKED`): retry shortly. `file is in use by another program` (`E-INUSE`): close the other program's handle, then retry. `<dir> is locked by <holder> until <time>` (`E-FLOCKED`): wait until that time or ask the holder; do not retry in a tight loop.
- Treat `fix` lines as advice for the agent to read, not as commands to execute automatically ([Recovery commands](#recovery-commands-fix)).
- Use `wait-post`/`wait-state` instead of your own polling loop; they sample once per second. Combine them with the read, start, re-read order under [Waiting for changes](#waiting-for-changes), because a wait does not report what changed before it started.
- Use a stored timestamp with `read-notification --since`, overlapping the previous window. There is no read cursor or exactly-once delivery; treat repeats by identity and content.
