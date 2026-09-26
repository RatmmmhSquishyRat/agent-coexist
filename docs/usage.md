# Usage guide

Everything you need to drive `agent-coexist` in practice: all fourteen commands, every action form, and the behaviors that catch first-time users. The [README](../README.md) covers what the tool is and how to install it; the [file format reference](file-format.md) covers what the four file types look like on disk. All examples below run as-is in a POSIX shell (Git Bash on Windows).

## Contents

- [A guided tour](#a-guided-tour) — a workspace from scratch
- [Command index](#command-index) — which command lives where
- [Conventions](#conventions) — paths, identity, output lines, exit codes
- [Profiles](#profiles) · [Posts and subscriptions](#posts-and-subscriptions)
- [Briefs](#briefs) · [State](#state)
- [Discovery and notifications](#discovery-and-notifications)
- [Waiting for changes](#waiting-for-changes)
- [Locks, retries and time](#locks-retries-and-time)
- [Troubleshooting](#troubleshooting) · [Environment overrides](#environment-overrides)
- [A pattern for scripted agents](#a-pattern-for-scripted-agents)

## A guided tour

The tour builds a small workspace, then reads it back. It touches only a fresh temporary directory and deletes nothing.

```sh
demo="$(mktemp -d)"
mkdir -p "$demo/agents" "$demo/docs" "$demo/src/parser"
cd "$demo"
printf 'pub fn parse(_input: &str) -> Result<(), ()> { Ok(()) }\n' \
  > src/parser/mod.rs

# alice introduces herself (creates agents/alice.agent-besties.md)
agent-coexist profile agents/alice --section role -m 'Rust implementer.'

# alice opens a thread and brings bob in
agent-coexist post docs/standup --as alice -m 'Standup. @bob status here.'
agent-coexist subscribe docs/standup --as bob

# bob replies to comment #1 and mentions alice
agent-coexist post docs/standup --as bob -m '@#1 parser done. @alice review?'

# point at a source file — the target is relative to the BRIEF's directory
agent-coexist brief docs/onboarding \
  --target ../src/parser/mod.rs --entry 'parser entry point' -m 'Start here.'

# ask for quiet while editing (advisory; 2h default because it is locked;
# src/parser already exists — state never creates its DIR)
agent-coexist state src/parser --as alice --status locked
```

Read everything back:

```sh
agent-coexist check docs/standup     # one file: diagnostics + previews
agent-coexist read-brief docs/onboarding
agent-coexist check-state src        # src, its ancestors and its subtree
agent-coexist read-notification --as alice --since 1h
agent-coexist lookfor-comment docs/standup --by bob
agent-coexist list .                 # the whole tree, with type column
```

For `read-notification --as alice --since 1h`, bob's comment is the one record when its timestamp is within the previous hour and strictly before the query's whole-second now — a `reply`, because `@#1` answers alice's comment. This takes precedence over its `@alice` mention. A fast same-second query may return none; query again after the clock advances. Alice's own comments and her newly declared two-hour lock produce no notification here.

One deliberate failure, because this is the classic trap:

```sh
# targets resolve against docs/, so this looks for docs/src/parser/mod.rs:
agent-coexist brief docs/onboarding --target src/parser/mod.rs
# expected result: first line error (E-TARGET-MISSING), exit 1; the brief
# is not even opened, so nothing is created or changed.
```

And a wait across two terminals. Run `pwd` in the first terminal, then open a second terminal in that same directory before running:

```sh
agent-coexist wait-post docs/standup --as alice
# prints one 'waiting  wait-post  docs/standup' line to stderr, then blocks
```

Back in the first terminal:

```sh
agent-coexist post docs/standup --as bob -m '@alice second pass done.'
```

The waiting terminal prints one diff report (comment #3 added) and exits 0. Bob's earlier `@alice` reply is *not* replayed: a wait reports only changes after its baseline.

## Command index

| Command | One line | Covered in |
|---|---|---|
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

Every command also answers `agent-coexist <command> --help`.

## Conventions

### Files and paths

- Collaboration files end in `.agent-besties.md`. `PATH` arguments may omit the suffix; it is appended when missing. The check is byte-exact, so an oddly cased suffix is *not* recognized and gets a second one appended — write the suffix lowercase or not at all. Scans match the suffix case-insensitively.
- `DIR` arguments default to `.` and must already exist (`E-DIR-MISSING` / `E-NOT-DIR`, exit 1). No command ever creates `DIR` itself.
- Forms that create files also create missing parent directories, printing `created  <dir>/` rows before `created  <file>`: `post` (new thread), `subscribe` (add), `profile` (bare or section set), `brief` (bare, description, `--target`). These never create: `post --edit`, `subscribe --cancel`, `profile --remove`, and the brief note-only, `--entry --remove` and `--refresh` forms — for those a missing file is `E-FILE-MISSING`, exit 1.
- Creating `post`, `profile`, `brief`, and `subscribe` forms build the H1 from the file-name stem as you typed it. The stem must be a valid name (1–100 letters, digits, `_`, `-`; `.` allowed after the first character). An invalid stem is `E-NAME` (exit 1): the CLI never renames, truncates, or substitutes another name. An existing file that already parses stays editable when the stem and H1 disagree (`W-NAME` on `check`); rename the file to the H1 name. Only a stem that is itself a valid name can be copied into the H1 instead.
- On Windows, drive-relative paths stay `C:notes` in display (never `C:/notes`). Equivalent ordinary and verbatim absolute volume paths are shown cwd-relative using native Prefix identity. Directory names are not case-folded, and that display does no extra I/O.

### Identity

- `--as <nick>` is your name. It is advisory — recorded in comment headers, matched by notifications, used to exempt you from functional locks — but never authenticated. Nicks are 1–64 characters: a letter, digit or `_` first, then letters, digits, `_` or `-`.
- Statuses (`--status`) are 1–32 ASCII lowercase characters (`a-z`, then `a-z0-9-`); section names follow the nick grammar. Entry titles have no grammar but reject control characters and double spaces, and are matched and displayed after whitespace folding (runs of spaces collapse to one, ends trimmed).
- Title columns and human-readable diagnostic text (message and `why`) use that folded display, with leftover control characters shown as U+FFFD. Command-form `fix` operands keep native whitespace and are not folded that way. Matching uses the folded form; stored heading text and body bytes are not rewritten. CLI flags still reject control-bearing titles; a stored heading that already contains them is corrected by editing that heading, not by relaxing that grammar.
- `-m` bodies: CRLF becomes LF, trailing whitespace and newlines are stripped, leading whitespace is kept, and the result must be non-empty — otherwise a usage error, exit 2.

### Reading output

- Every result's first line is `<status>  <command>  <object>` with status `ok`, `warn` or `error`. An empty object is shown as `/` so that column stays present when splitting *that display line* on two or more spaces. The wait ready line on stderr (`waiting  <command>  <object>`) uses the same stand-in. This is display only: stored identity is not rewritten. A nonempty object that is itself `/` prints the same way; there is no extra encoding. There is no JSON mode; this line protocol is the scripting contract.
- Fields rows (`  <key>  <value>…`) hold a key plus values joined by exactly two spaces. Ordinary display values taken from file content are whitespace-folded first, so splitting *those* display rows on two or more spaces is safe. A command-form `fix` may itself be a Fields row: native operands keep internal whitespace, and that whitespace can include the two-space delimiter, even inside the existing POSIX quotes. Do not split that remainder as columns; consume the complete remainder after the key as one POSIX command payload. An unmatched `--include` glob on a scan hint row is folded and control-sanitized the same way (leftover controls become U+FFFD); matching still uses the glob as typed. An empty key or value is shown as `/` so that column stays present; a Fields row with no values is indent plus key only, with no synthetic value column. Ordinary nonempty keys and values are unchanged.
- Text rows are different: a `  <key>` line followed by `    | <line>` lines that reproduce content verbatim (read-brief bodies, wait before/after values). They may legitimately contain any spacing — never split them as columns; consume them by the `| ` prefix. Those verbatim bodies stay distinct from ordinary display Fields and from a command-form `fix` remainder. Those `| ` lines are not slash-substituted: an empty content line stays empty.
- Diagnostics print as `  error  L<n>  <message>` (or `warn`) with 9-space-indented `why` and `fix` lines underneath. A `fix` line is either literal file content / prose, or a command with POSIX-quoted operands — not both. Literal/prose fixes tell you what to write into the file (a comment header to copy, a heading to rename by hand); they are not shell-quoted command operands. Command-form fixes keep native operands (the managed suffix when that file already has it, internal path whitespace, and Windows verbatim spelling) separately from the folded display in other columns. The remainder after the `fix` key is that command payload: consume it whole with the existing POSIX quoting; do not split it on two or more spaces as if it were display columns. Those command words are encoded for a POSIX shell consumer: a nonempty word of ASCII letters, digits, or `_@%+,./-` is left as-is; every other word, including empty, braces, `=`, quotes, `$`, and non-ASCII, is wrapped in single quotes (an inner `'` becomes `'\''`). This preserves operands for POSIX consumers; it is not a claim of identity with every shell's quoting, and cmd.exe/PowerShell still need adjustment. A native positional operand that starts with `-` is written as `./…`; option values use `--key=<quoted-value>`.
- In a generated `fix` command, a title or nick that happens to look like `<file>` or `<nick>` is the actual runtime value. The static reader markers `<file>` and `<nick>` appear only when that path or author is genuinely missing. Human title text in message/why stays sanitized. A file-name stem interpolated into diagnostic prose uses the same folded display (controls become U+FFFD); that is display text only and does not rewrite or escape the native path used to open the file. A heading that already contains control characters is not given a pasteable command: the `fix` line tells you to edit that heading by hand to a valid, distinct title, then run `check` again — it does not emit raw control characters or a command aimed at a U+FFFD sibling. A profile section whose name is outside the section grammar gets the same kind of manual heading-and-fenced-body advice, not a `--section` command the CLI would reject. CLI `--section` still refuses control characters. An invalid stored status name is the same class of repair: rename that heading by hand to a valid, distinct status (`a-z`, then `a-z0-9-`; only if that name is unused), then run `check` again — it does not emit a `--status` command the CLI would reject or a normalized sibling of the stored name. Once the heading is a valid unused name, later declaration diagnostics may again suggest a command against that name.
- Completed `list`, `lookfor-comment`, `read-notification` and `check-state` overviews (`ok`/`warn`, including zero records) end with a `total` row. Fatal reports of those same commands — I/O `reason`/`fix` (missing dir/file, not a directory, `cannot scan`, other OS errors) and check-shaped blocking diagnostics — have no `total` and are not a completed overview. `check`, `read-brief`, write commands and wait reports have none.
- Lines always end in LF, even on Windows. Progress (`scanning N dirs, M files...`) goes to stderr and only when stderr is a TTY, including ancestor scans from `profile` and `brief`; stdout is always clean for redirection.

### Exit codes

| Code | Means | Action |
|---|---|---|
| 0 | first line `ok` | done |
| 0 | first line `warn` — conflicts, skipped or partial files, refresh skips, dangling reply targets | success; read the rows |
| 1 | first line `error` — diagnostics, missing file/entry/dir, type mismatch, I/O | fix the input |
| 2 | usage error — including argument-parser failures: one `error: ...` line on stderr, stdout empty; the useful error and hint are kept, and CR/LF in that message become spaces at emission (stored file text is not rewritten); also invalid `AGENT_COEXIST_*` values | fix the command line |
| 3 | `E-LOCKED`: the required file lock, stable identity, or stable contents could not be obtained or maintained | retry the same command shortly |
| 3 | `E-INUSE`: another program (e.g. an editor) holds the file | close it, then retry |
| 3 | `E-FLOCKED`: functional lock, see [State](#state) | retry at the expiry time shown in `reason` |
| 130 | a wait was cancelled with Ctrl-C | — |

`warn` is exit 0: scripts that care about conflicts must parse the first line, not the code alone. Explicit `--help` and `--version` print to stdout and exit 0, with empty stderr.

## Profiles

```sh
agent-coexist profile agents/alice                        # create; else unchanged
agent-coexist profile agents/alice --section role -m 'Rust implementer.'
agent-coexist profile agents/alice --section role --remove
```

- Without `--section`: creates the file (heading only) or reports the existing file `unchanged`.
- `--section <name> -m <content>` sets or replaces the section's body, stored verbatim in a code fence. If a stored section title is outside the nick grammar, `check` may offer a copyable `## <name>` example only when that name is unused among the file's existing headings and the other heading repairs in the same report. If that name is already taken — including by a later heading, or by another repair that would use the same name — the `fix` tells you to rename the heading by hand to a valid name distinct from every other section and any other planned heading repair, then run `check` again. It does not suggest `--section` with the invalid name. `--section` still rejects control characters.
- `--section <name> --remove` deletes the whole section. Unknown section: `E-SECTION-NOTFOUND`, exit 1. `--remove` never creates the file.
- `--as` is optional here: it only exempts you from functional locks and is never written to the file.

## Posts and subscriptions

```sh
agent-coexist post docs/standup --as alice -m '…'           # append; creates
agent-coexist post docs/standup --as alice -m '…' --edit 2  # rewrite own #2
agent-coexist subscribe docs/standup --as bob               # add subscriber
agent-coexist subscribe docs/standup --as bob --cancel      # remove
```

- Comments are numbered `#1`, `#2`, … in file order; a first legal number greater than 1 is a non-blocking gap warning and is not renumbered. When `check` prints copyable header repairs for several malformed or duplicate headers in one report, each unused sequence is reserved for one header. Later exhaustion of remaining numbers stays a manual heading repair. Bodies and already-legal comments are not rewritten. Mentions (`@nick`) and replies (`@#n`) are plain text in the body. A mention starts at the beginning of text or after a character other than a letter, digit, `_`, or `@`, so `@alice-@bob` is both `alice-` and `bob`; `a@bob` is not a mention. The write report lists each parsed mention and reply in body order; an `@#n` pointing nowhere adds a `(not found)` warn row — the comment is still written. `check` of a post lists `authors` in first-occurrence order (`none` when empty).
- `--edit n` rewrites comment #n in place; its author must be `--as` (`E-EDIT-AUTHOR`, exit 1) and it must exist (`E-COMMENT-NOTFOUND`, exit 1). The header gains `, edited <time>`. `--edit` never creates the file. In the tour workspace this fails as labeled:

  ```sh
  agent-coexist post docs/standup --as alice -m '…' --edit 2
  # expected: error (E-EDIT-AUTHOR), exit 1 — comment #2 is bob's
  ```

- Subscribing twice, or cancelling when not subscribed, leaves the file unchanged (`unchanged  <nick>`, `ok`). Cancel edits only the first authoritative subscriber property. Deleting its last nick deletes that property, leaves later duplicate properties as stored, then reports the subscribers of the resulting document. A surviving duplicate can become the list; `none` only when that list is empty. `--cancel` on a missing file is `E-FILE-MISSING`, exit 1.
- `post` and `subscribe` queue up to 5 s on a busy lock; see [Locks, retries and time](#locks-retries-and-time).

## Briefs

A brief hands over pointers to real source files, verified by SHA-256. Six mutually exclusive forms:

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

- `--target` resolves against the brief file's directory, not the shell's cwd: inside `docs/onboarding`, `--target ../src/parser/mod.rs` means `src/parser/mod.rs` from the workspace root. Paths starting `/` (or a Windows drive letter) are used as-is. A missing target is `E-TARGET-MISSING`, exit 1, detected before the brief is opened. Raw CR or LF in `--target` is a usage error (exit 2) and writes nothing; other whitespace is kept as part of the stored path.
- The default entry title is the target's file name (`mod.rs`); `--entry` overrides it. Reusing an existing title that points at a *different* target, without `--entry`, is `E-ENTRY-TITLE-TAKEN`, exit 1.
- `--regex` requires `--target`. Writing checks that the pattern compiles; matching is checked on read. A target that does not match makes the entry `stale` (`regex not found`). Passing `--regex ''` removes every `- regex:` line in that entry's property area, including handwritten duplicates, so a leftover line cannot become the constraint. Raw CR or LF in `--regex` is a usage error (exit 2) and writes nothing; other whitespace is kept. To match CR or LF in the target, use regex escapes such as `\r` and `\n`.
- The target form writes `entry  <title>  added|updated  <path>` plus `hash  <sha256>`, and `note  set` when `-m` is given. The description form writes `description  set`; the note-only form `note  set`; removal writes `entry  <title>  removed` and `removed-lines  L<a>..L<b>`.
- `--refresh` recomputes and rewrites stored hashes only — it never re-checks regexes. Per selected entry it prints `entry  <title>  refreshed  <path>`, or `skipped  no path` / `skipped  target missing` (skips set `warn`, exit stays 0). Without `--entry`, every entry is refreshed. Unknown title: `E-ENTRY-NOTFOUND`, exit 1. Review target changes and update outdated notes before refreshing: `fresh` means the hash matches, not that anyone has checked the note.
- Combination rules: `--regex` requires `--target`; `--refresh` excludes `--target`, `-m` and `--remove`; `--remove` requires `--entry` and excludes `--target` and `-m`; without `--target`, `--entry` requires `-m`, `--remove` or `--refresh`. Violations are usage errors, exit 2, and write nothing.
- After any byte-writing brief form, surviving non-blocking diagnostics are re-parsed and appended as `remaining  <E-ID>  L<n>` warn rows.

### Entry states (what readers see)

| State | Meaning | `reason` shown |
|---|---|---|
| `fresh` | stored hash matches the target's current bytes | — |
| `shifted` | hash differs — target changed after the hash was written | — |
| `stale` | target missing, regex invalid, or regex not found | that reason |
| `unchecked` | no path, target unreadable, target too large, or no hash | that reason |

`target too large` applies to targets over 64 MiB that need regex verification; hashing itself is streaming and uncapped.

### Reading briefs

`read-brief` renders the description, then per entry its state, note and the target's content, as verbatim `| `-prefixed text blocks, plus rows for regex capture groups when a regex is set. `--entry <title>` renders one entry (unknown title: `E-ENTRY-NOTFOUND`, exit 1). Any shifted, stale or unchecked entry turns the first line `warn`.

## State

```sh
agent-coexist state src/parser --as alice --status locked              # 2h
agent-coexist state src/parser --as alice --status locked --for 1h30m
agent-coexist state src --as bob --status observing
agent-coexist state src/parser --remove alice       # --as defaults to alice
agent-coexist state src --remove bob --status observing  # --status filters
```

- Built-in statuses: `locked` — please do not change this directory or its subdirectories until the end time; `observing` — I am watching, changes are welcome. Custom names are stored and shown but carry no lock meaning. Durations default to 2 h for `locked`, 8 h for anything else; `--for` overrides.
- Declarations live in `DIR/state.agent-besties.md`. `state` and `check-state` select that reserved file by the same native lookup they already use (case-insensitive on Windows/macOS), not by another file's basename or H1. A misnamed `# .state` file can still be read with `check`, but it is not that selected target. A declaration is valid while `start <= until` and `now < until`. A later declaration, and a successful removal, clean expired entries on clean lines (`cleaned` rows); read commands never write. Re-declaring the same nick under the selected status replaces that nick's clean declarations there with `start = now` and `until = now +` the `--for` or default duration, so an explicit renewal can shorten an existing lifetime. Keeping the latest `until` is the separate selection and merge rule for duplicate declarations that remain stored; mixed lines are not rewritten and keep their existing preservation semantics.
- The declare form writes first, then scans for conflicts: two different nicks holding valid `locked` whose directories overlap (equal, or ancestor/descendant) produce `conflict  locked  <nick>  <span>  <dir>` rows and a `warn` first line — the exit code stays 0. Another agent's `locked` never blocks a state write. If that post-write scan hits a fatal ignore-source error, the write is already committed: first line `warn`, the existing `created` / `set` / `cleaned` rows, then `scan  <path>  <cause>` (display path and the I/O or parse source), exit 0. There is no `fix` row and no rollback; a later `check` of the state file still shows the written declaration. That is not a `skipped  <dir>  unreadable` row.
- Remove: `removed  <status>  <nick>` per section. Only entries on clean lines (every comma fragment on the line parses) are rewritten. If the nick still has a parseable declaration on a mixed line of a selected status, the whole removal is refused before any write (`E-STATE-DIRTY`, exit 1) — fix that line by hand, then re-run the printed command, which keeps the complete original argv including `--status` so only that selected status is removed; legal fragments on mixed lines stay visible in `check` and lock scans. A status whose remaining bytes are only its heading and blank lines is deleted (`removed-lines`); leftover handwritten text, mixed lines, and fences keep the heading and those bytes. A file reduced to its heading, reader comments, and blanks is deleted whole (`deleted  <file>`). `--as` defaults to the removed nick — a different `--as` still removes but adds a `removed-other` warn row. Omitting `--status` still removes that nick from every status that holds a clean entry for it; that span is intentional. Missing file or nick: `E-STATE-NOTFOUND`, exit 1, and nothing is written. When `--status` is given, the reason names that status; the `fix` is a read-only `check-state` of the directory and keeps the intended selection. Do not drop `--status` and remove another declaration — a live `locked` entry stays in place. A generated same-file lock-conflict `fix` includes `--status=locked`, so copying it removes only that lock and leaves other live statuses. `--for` is rejected in the remove form (usage, exit 2).
- Scope and inheritance: a state file describes its directory's subtree. `check-state DIR` reports DIR's own entries, `inherited` rows from ancestor directories (valid entries only, nearest first), and descendant groups whose headings are native paths relative to DIR before display folding; `read-notification` also reads ancestor state files and counts DIR's own state file once. `check` on a state file reports each expiry and same-file lock conflict (`W-STATE-CONFLICT`) once, from the selected declaration per nick. Those pairs are expanded for that `check` report and for a wait that hits (including the snapshot from just before a deletion); ordinary parse and scans do not build them. A complete `check` still lists every pair, so many overlapping `locked` entries can make that report large. `check-state` overlapping lock rows keep native file order. A `read-notification` conflict names the deepest overlapping own lock; when depths tie, the last file in that order wins. Required per-file conflict rows are not dropped. Functional locks (next point) consider the supplied file's directory and its ancestors, and for a file symlink also the canonical referent's parent directory and its ancestors. That is not a volume-wide hardlink search.
- Functional lock: the byte-writing `profile`/`brief` forms fail with `E-FLOCKED` (exit 3) when a *different* nick holds a valid `locked` on the supplied file's directory or any ancestor — without `--as`, any other holder blocks. A write through a file symlink also consults the canonical referent's parent directory and its ancestors; own `--as`, expiry, and the incomplete-scan fallback are unchanged. Failed canonicalization keeps the supplied chain. This is not a volume-wide hardlink search, and concurrent replacements are not atomic. Retry at the expiry time shown in `reason`. Unreadable or incomplete state files in that chain stay listed as `skipped`/`partial`; they do not by themselves refuse the write. `post`, `subscribe` and `state` itself are never functional-lock blocked.
- This is cooperation, not enforcement: state files record intent for agents that choose to respect it. Nothing here is access control.

## Discovery and notifications

### Scans: what is visited, and filter precedence

`list`, `read-notification` and `check-state` walk `DIR` recursively and consider only files named `*.agent-besties.md` (matched case-insensitively). Hidden directories are traversed. An explicit `DIR` that is already a directory — including a directory symlink, a junction, or a `.git` directory — is entered and counted as a directory, never as a managed-file leaf. Directory symlinks, junctions, and `.git` directories discovered under that root remain pruned. Precedence, in order:

1. Ignore rules prune during the walk: `.gitignore`, `.ignore`, `.git/info/exclude`, your global gitignore, plus ignore files in the root's physical parent directories. `--no-ignore` disables all of them. A source that is not a regular file, or that cannot be read or parsed, fails the scan for `list`, `read-notification`, and `check-state`: first line `error`, `reason  cannot scan <path>: <source>`, a retry `fix`, exit 1 — no overview rows, no descendant groups, and no `skipped  <dir>  unreadable` stand-in. `--no-ignore` skips those sources. Ordinary unreadable directories stay `skipped` warnings. On Windows, ignore matching compares ordinary and verbatim volume prefixes as native disk/UNC values (`C:` and `c:` are the same disk; `C:\` and `\\?\C:\` are the same volume). Prefix stripping consumes every remaining root component, so a shorter candidate is not treated as under that root. Ordinary and verbatim spellings of the same volume agree on global directory excludes. Directory names are not given extra case-folding, and the path used for I/O is not rewritten.
2. `--include`/`--exclude` then filter the collected file names. They cannot reopen an ignore-pruned directory; if every `--include` matched nothing while ignore rules were active, `list`, `read-notification` and `check-state` add a warn row suggesting `--no-ignore`. The glob on that hint row is folded and control-sanitized for display only; matching still uses the glob as typed.
3. A file is kept when (no `--include` was given, or one matches) and no `--exclude` matches — exclude always wins; repeated flags form a union.

Globs are gitignore-style, relative to `DIR`: a leading `/` anchors at `DIR`; a trailing `/` matches files under that directory, sparing a same-named file; `*` and `?` never cross `/` but `**` does; matching is case-insensitive on Windows and macOS. Matching uses the glob as typed, including internal double spaces; only an unmatched-hint display is folded and control-sanitized. An invalid glob is a usage error (exit 2).

### list

Prints a tree with a type column. The first row names the active ignore sources (a subset of `.gitignore, .ignore, exclude, global`, or `none`, or `--no-ignore`). Leaves shown as `busy` (lock was busy) or `?` (unreadable: permissions, not UTF-8, bad heading, over the size cap) still count toward `total` and turn the first line `warn` — never a failure. `--type` filters by type and repeats as a union; `profile` is an accepted alias for `agent-profile`.

### check

Reports one file: `file` and `type` rows, diagnostics in line order, then previews — comments, `authors` (first-occurrence order), topic/latest/subscribers for posts, entry counts and states for briefs, sections for profiles, per-status rows with `conflict` and `expired` marks for state files. Same-file lock-conflict warnings (`W-STATE-CONFLICT`) are expanded in this report, not during ordinary parse. Any error-level diagnostic makes the first line `error` (exit 1). Blocking diagnostics also refuse every write command with a check-shaped report.

### lookfor-comment

```sh
agent-coexist lookfor-comment docs/standup --by bob --mention alice
agent-coexist lookfor-comment docs/standup --keyword 'parser' --reply-to 1
agent-coexist lookfor-comment docs/standup --comment 2
agent-coexist lookfor-comment docs/standup    # every comment
```

Filters intersect; no filter lists every comment. Each row is `L<line>  #<seq>  <author>  <time>` plus an excerpt. `--keyword` is a case-insensitive substring over the raw body and the comment's original (unfolded) header line, used exactly as typed (double spaces allowed; whitespace is not folded); `--by` is the author, `--mention` a body `@nick`, `--reply-to` a body `@#n`, and `--comment` selects one sequence number.

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
| `conflict` | others | a valid `locked` overlapping yours — `overlaps your lock on <dir>`, naming the deepest overlapping own lock (last file in scope order when depths tie) |
| `expiring` | you | your valid entry ends within 30 minutes — `ends in Nm` |
| `expired` | you | your entry ended inside the window — `expired at <t>` |

Comment records land when their activity time (created or edited) is in `[since, now)`; your own comments never notify you. Each comment produces at most one record for you: `reply` takes precedence over `mention`, which takes precedence over `subscribed`. Expiry awareness lives here: `check-state` marks expired rows but never raises them as records. Busy or unparseable files become `skipped`/`partial` rows and warn, as do conflict rows.

## Waiting for changes

Lifecycle of any wait: the parent directory must exist (`E-DIR-MISSING`, exit 1). If that parent exists but is not a directory, `wait` and `wait-post` fail as a path-class error; `wait-state` still reports `E-NOT-DIR`. A baseline snapshot is then taken; one `waiting  <command>  <object>` line goes to stderr; the single file is then sampled every second under a shared lock until the first qualifying change prints one report (exit 0). Nothing is written, and no lock is held while idle.

- A missing file is a legitimate baseline — `wait-post docs/standup` on a thread that does not exist yet waits for its creation. The flip side: a typo'd path waits forever.
- An invalid baseline fails instead: blocking diagnostics, an unknown type, the wrong type for `wait-post`/`wait-state` (`E-TYPE`), or duplicate `(status, nick)` pairs in a state file all produce an error report, exit 1.
- A present file over the 16 MiB cap is refused with the same check-shaped report as `check` of that file: first line `error`, a `file` row, the `E-TOO-LARGE` diagnostic, exit 1, and no `type` row. That applies on the initial sample and if the file later grows past the cap after the ready line. Wait does not invent a type from an earlier snapshot, and the report is not a change hit.
- Baselines advance: a valid snapshot that changed but did not qualify still becomes the new baseline. Diffs are always between adjacent snapshots, so a wait never replays messages that predate it — start the waiter first, or catch up with `read-notification --since`.

| Command | Filter | Hits on |
|---|---|---|
| `wait` | — | any byte change, any of the four types |
| `wait-post` | — | any byte change of the post |
| `wait-post` | `--as <nick>` | a comment relevant to `<nick>` — a direct `@nick`, or an `@#n` whose author is `<nick>` — that is new, edited, or newly relevant |
| `wait-state` | — | any byte change of `DIR/state.agent-besties.md` |
| `wait-state` | `--status <s>` | an add, removal, or `start`/`until` change of declarations under `<s>` (renewals hit) |

- `wait-state` watches exactly `DIR/state.agent-besties.md` — no ancestors, no subtree.
- Wait pairs comments, entries, sections, and declarations by their raw identity (first match wins) and still emits in file order. A filtered `wait-state --status` non-hit no longer walks those identities repeatedly; filters, diffs, baselines, admission, and required conflict-pair output are unchanged. Qualifying profile updates keep the same report; filtered cancellations still exit 130 with no stdout. Sampled times are not a universal latency promise.
- Subscriber-list changes, comment deletions and reorders never hit `wait-post --as`: relevance is comment-based, so a subscription by itself is not a wait trigger.
- Clock expiry alone never hits `wait-state --status`; a file deletion hits only if the file held `<s>`-declarations before.
- The report starts with `bytes  changed` and `semantic  changed|unchanged`, adds `created  <file>` / `removed  <file>` when the file appeared or vanished, then entity `add`/`remove`/`update` rows with field values as verbatim text blocks. Updates use `before`/`after`; additions and removals name fields of the observed side; a missing field reads `none`. Order rows come last. Filtered hits list only the qualified entities. A brief description appears once when the file is created, deleted, or changes type. Same-file lock-conflict warnings are expanded on a qualifying hit, including the snapshot from just before a deletion, using the wait's start time; idle sampling does not build those pairs. State `fix` recipes on that report are gated once against the currently selected reserved file at this final snapshot, not retained from the ready line. A `removed` warning from the deleted baseline can remain; it does not print a directory-level `state` command against a later selected file.
- Ctrl-C: exit 130, no stdout. `E-LOCKED`/`E-INUSE` while sampling: 3.

## Locks, retries and time

| Commands | Lock | Behavior when busy |
|---|---|---|
| `post`, `subscribe` | exclusive | queue up to 5 s, then `E-LOCKED` (3) |
| `profile`, `brief`, `state` | exclusive | fail fast (~200 ms), `E-LOCKED` (3) |
| `check`, `lookfor-comment`, `read-brief` | shared | 1 s, then `E-LOCKED` (3) |
| scans, plus state's conflict scan | shared | ~100 ms, then `busy`/`skipped`/`partial` rows + `warn` |

`E-LOCKED` means the required file lock, stable identity, or stable contents could not be obtained or maintained — retry the same command shortly. It does not name a holder: any native lock holder, exhausted identity-reopen retries, or changing contents that prevent a stable read can cause it. `E-INUSE` means another program (an editor, say) had it open — close it and retry. `E-FLOCKED` is the functional lock described under [State](#state): retry at the expiry time shown in `reason`, not at millisecond cadence. Collaboration files are capped at 16 MiB (`E-TOO-LARGE`).

A lock or conflict scan that cannot fully read a state file still reports what it found. `skipped` rows mean the file was busy, structurally unreadable, or the wrong type; `partial` rows mean some declarations were kept while a listed line could not be incorporated, or that non-comment stray text (`W-STRAY`) never became a section. Valid declarations remain; omitted bytes never invent a lock. Recognized `locked` holders still block `profile`/`brief`. When no such holder is identified, the write proceeds and the incomplete scan is appended with a `warn` first line — unread metadata is never treated as "no locks", and it is not a blanket refusal of every write.

Times always print as `YYYY-MM-DDTHH:MM:SSZ` (UTC, seconds precision). A leap second parsed from a file keeps `:60` in that form; `AGENT_COEXIST_NOW` still rejects `:60`. `--since` accepts either an RFC 3339 timestamp — any offset is converted, fractional seconds are floored, but a space instead of `T` is rejected — or a duration counted back from now. Durations (`Nd/Nh/Nm/Ns`) concatenate units in strictly descending order, each at most once, total greater than zero: `1h30m`, `90m` and `2d` are valid; `1m30h` and `0s` are not.

## Troubleshooting

- `E-TARGET-MISSING` on a brief whose target obviously exists — the path is relative to the brief's directory, not your cwd.
- `--include` matched nothing — ignore rules pruned those directories; rerun with `--no-ignore`. The glob on that hint row is folded for display (controls become U+FFFD); matching still uses the glob as typed.
- `error  check-state` / `cannot scan` — an ignore source failed to load or parse (the same class as `list` / `read-notification`). This is not an unreadable directory. If a `state` write already succeeded and then the conflict scan failed the same way, the first line is `warn`, a `scan  <path>  <cause>` row names the source, exit is 0, and the written bytes stay.
- Exit 0 but the first line is `warn` — read the rows: state conflicts, refresh skips, busy or partial state files, dangling `@#n` reply targets, incomplete lock scans.
- `E-FLOCKED` on `profile`/`brief` — someone declared `locked` upstream; retry at the shown expiry, or use `post`/`subscribe`, which are exempt. A write through a file symlink is also refused when the canonical referent's parent chain holds a different nick's lock; own `--as` still writes.
- `E-NAME` on a creating `post`/`profile`/`brief`/`subscribe` — the file-name stem is not a valid name; choose a valid name and re-run with that path. The CLI will not rename, truncate, or replace it.
- `E-TYPE` — the heading already names a different type; use the command for that type, or pick another path. For a non-state heading, the suggested command follows that heading, not whether you passed a directory. For a state heading, a directory-level `state` command is printed only when the inspected file is that directory's selected `state.agent-besties.md`.
- A `check` of a misnamed state-content file (for example `room/draft.agent-besties.md` whose H1 is `# .state`) warns that `state` and `check-state` look only for `state.agent-besties.md`. The rename `fix` tells you to inspect this file and any reserved state file already in its directory, resolve their declarations, then choose which file should use that name — never overwrite a separate existing state file. Duplicate, inverted-range, expired, same-file lock-conflict, and type-mismatch advice on that inspected file is hand-edit only; it does not print a `state DIR` command that would mutate a sibling reserved file. The selected reserved file still prints the existing directory-level recipes.
- A wait-state `removed` report can still list warnings from the deleted baseline. Those recipes are gated once at the final report: if that inspected file is no longer the directory's selected `state.agent-besties.md`, the `fix` is hand-edit of the inspected snapshot and does not print a directory-level `state` command against a later selected file.
- `E-STATE-DIRTY` — the nick is still declared on a mixed line; make that line fully parseable (or delete the nick from it) and paste the printed retry, which keeps the original `--status` selection. Do not reconstruct a bare `--remove` that would also drop other statuses.
- `E-STATE-NOTFOUND` — no matching nick under the selected status (or no file). Nothing is written. Follow the printed read-only `check-state` inspection and keep `--status`; do not reconstruct a bare `--remove` that would delete another declaration.
- A same-file lock-conflict `fix` includes `--status=locked`. Copying it removes only that lock. Omitting `--status` on your own `--remove` still spans statuses.
- `W-NAME` or a broken H1 after a hand rename — the existing file stays usable if it already parses. If the stem is not a valid name, rename the file to match the H1; do not copy an H1 built from the bad stem.
- A heading that contains control characters — the `fix` line tells you to edit that heading by hand to a valid, distinct title, then run `check` again. Do not paste a command that targets a U+FFFD sibling, and do not pass those characters to `--entry`, `--section`, or `--status`. A profile section whose title is outside the nick grammar is the same class: a copyable `## <name>` example appears only when that name is unused among existing headings and other heading repairs in the same report; otherwise rename the heading by hand to a valid, distinct name, then run `check` again — not with `--section`. An invalid stored status name is the same: rename that `## ` heading by hand to a valid, distinct status (only if unused), then run `check` again; do not paste a `--status` command aimed at the rejected or a normalized name.
- Several malformed or duplicate comment headers in one `check` report receive distinct unused sequence numbers. Copying every printed header in that report is enough while numbers remain; a later header that cannot take a remaining number stays a manual heading repair.
- When the `fix` is a command, consume the complete remainder after the `fix` key — paste it for a POSIX shell with its existing POSIX quoting; do not rebuild it by splitting on spaces. When its path differs from the folded first-line object, that remainder is still the native operand: managed suffix when present, internal spaces, Windows verbatim spelling, and `./` before a leading `-`. cmd.exe and PowerShell still need their own quoting. Literal/prose `fix` lines are copied into the file or followed by hand, not executed as shell.
- File not created — you used a never-creating form (`--edit`, `--cancel`, `--remove`, `--refresh`) on a missing file.
- A doubled suffix appeared in a filename — the suffix append is case-sensitive; drop or lowercase the suffix you typed.
- Duration rejected — units must be strictly descending (`1h30m`, not `1m30h`).
- Empty `-m` rejected — trailing whitespace is stripped and the body must survive it; leading whitespace is kept.

## Environment overrides

Both exist for testing and advanced automation. Nothing in normal use requires them.

- `AGENT_COEXIST_NOW` pins `now` for reproducible runs. Strict 20-character `YYYY-MM-DDTHH:MM:SSZ`; offsets, fractions, lowercase, and leap seconds (`:60`) are rejected (usage error, exit 2, before any file is touched).
- `AGENT_COEXIST_LOCK_WAIT_MS` — digits only — overrides the 5 s queue budget of `post`/`subscribe` writes; no other budget.

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

- For exit 3, retry `E-LOCKED` shortly, resolve the other program's open handle for `E-INUSE`, or wait until the stated expiry for `E-FLOCKED`.
- Use `wait-post`/`wait-state` for their change reports instead of implementing another polling loop. They still sample the file once per second.
- Use a stored timestamp with `read-notification --since` to limit the time window. The CLI stores no read cursor and does not provide exactly-once notification delivery.
