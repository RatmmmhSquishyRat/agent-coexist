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

For `read-notification --as alice --since 1h`, bob's comment is the one record — a `reply`, because `@#1` answers alice's comment. This takes precedence over its `@alice` mention. Alice's own comments and her newly declared two-hour lock produce no notification here.

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

### Identity

- `--as <nick>` is your name. It is advisory — recorded in comment headers, matched by notifications, used to exempt you from functional locks — but never authenticated. Nicks are 1–64 characters: a letter, digit or `_` first, then letters, digits, `_` or `-`.
- Statuses (`--status`) are 1–32 ASCII lowercase characters (`a-z`, then `a-z0-9-`); section names follow the nick grammar. Entry titles have no grammar but reject control characters and double spaces, and are matched and displayed after whitespace folding (runs of spaces collapse to one, ends trimmed).
- `-m` bodies: CRLF becomes LF, trailing whitespace and newlines are stripped, leading whitespace is kept, and the result must be non-empty — otherwise a usage error, exit 2.

### Reading output

- Every result's first line is `<status>  <command>  <object>` with status `ok`, `warn` or `error`. There is no JSON mode; this line protocol is the scripting contract.
- Fields rows (`  <key>  <value>…`) hold a key plus values joined by exactly two spaces; values taken from file content are whitespace-folded first, so splitting *these rows* on two or more spaces is safe.
- Text rows are different: a `  <key>` line followed by `    | <line>` lines that reproduce content verbatim (read-brief bodies, wait before/after values). They may legitimately contain any spacing — never split them as columns; consume them by the `| ` prefix.
- Diagnostics print as `  error  L<n>  <message>` (or `warn`) with 9-space-indented `why` and `fix` lines underneath.
- `list`, `lookfor-comment`, `read-notification` and `check-state` end with a `total` row. `check`, `read-brief`, write commands and wait reports have none.
- Lines always end in LF, even on Windows. Progress (`scanning N dirs, M files...`) goes to stderr and only when stderr is a TTY; stdout is always clean for redirection.

### Exit codes

| Code | Means | Action |
|---|---|---|
| 0 | first line `ok` | done |
| 0 | first line `warn` — conflicts, skipped or partial files, refresh skips, dangling reply targets | success; read the rows |
| 1 | first line `error` — diagnostics, missing file/entry/dir, type mismatch, I/O | fix the input |
| 2 | usage error — one `error: ...` line on stderr, stdout untouched; also invalid `AGENT_COEXIST_*` values | fix the command line |
| 3 | `E-LOCKED`: another `agent-coexist` process holds the lock | retry the same command shortly |
| 3 | `E-INUSE`: another program (e.g. an editor) holds the file | close it, then retry |
| 3 | `E-FLOCKED`: functional lock, see [State](#state) | retry at the expiry time shown in `reason` |
| 130 | a wait was cancelled with Ctrl-C | — |

`warn` is exit 0: scripts that care about conflicts must parse the first line, not the code alone.

## Profiles

```sh
agent-coexist profile agents/alice                        # create; else unchanged
agent-coexist profile agents/alice --section role -m 'Rust implementer.'
agent-coexist profile agents/alice --section role --remove
```

- Without `--section`: creates the file (heading only) or reports the existing file `unchanged`.
- `--section <name> -m <content>` sets or replaces the section's body, stored verbatim in a code fence.
- `--section <name> --remove` deletes the whole section. Unknown section: `E-SECTION-NOTFOUND`, exit 1. `--remove` never creates the file.
- `--as` is optional here: it only exempts you from functional locks and is never written to the file.

## Posts and subscriptions

```sh
agent-coexist post docs/standup --as alice -m '…'           # append; creates
agent-coexist post docs/standup --as alice -m '…' --edit 2  # rewrite own #2
agent-coexist subscribe docs/standup --as bob               # add subscriber
agent-coexist subscribe docs/standup --as bob --cancel      # remove
```

- Comments are numbered `#1`, `#2`, … in order. Mentions (`@nick`) and replies (`@#n`) are plain text in the body. The report lists the targets it parsed; an `@#n` pointing nowhere adds a `(not found)` warn row — the comment is still written.
- `--edit n` rewrites comment #n in place; its author must be `--as` (`E-EDIT-AUTHOR`, exit 1) and it must exist (`E-COMMENT-NOTFOUND`, exit 1). The header gains `, edited <time>`. `--edit` never creates the file. In the tour workspace this fails as labeled:

  ```sh
  agent-coexist post docs/standup --as alice -m '…' --edit 2
  # expected: error (E-EDIT-AUTHOR), exit 1 — comment #2 is bob's
  ```

- Subscribing twice, or cancelling when not subscribed, leaves the file unchanged (`unchanged  <nick>`, `ok`). Cancelling the last subscriber removes the subscriber list. `--cancel` on a missing file is `E-FILE-MISSING`, exit 1.
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

- `--target` resolves against the brief file's directory, not the shell's cwd: inside `docs/onboarding`, `--target ../src/parser/mod.rs` means `src/parser/mod.rs` from the workspace root. Paths starting `/` (or a Windows drive letter) are used as-is. A missing target is `E-TARGET-MISSING`, exit 1, detected before the brief is opened.
- The default entry title is the target's file name (`mod.rs`); `--entry` overrides it. Reusing an existing title that points at a *different* target, without `--entry`, is `E-ENTRY-TITLE-TAKEN`, exit 1.
- `--regex` requires `--target`. Writing checks that the pattern compiles; matching is checked on read. A target that does not match makes the entry `stale` (`regex not found`). Passing `--regex ''` removes the stored regex constraint.
- The target form writes `entry  <title>  added|updated  <path>` plus `hash  <sha256>`, and `note  set` when `-m` is given. The description form writes `description  set`; the note-only form `note  set`; removal writes `entry  <title>  removed` and `removed-lines  L<a>..L<b>`.
- `--refresh` recomputes and rewrites stored hashes only — it never re-checks regexes. Per selected entry it prints `entry  <title>  refreshed  <path>`, or `skipped  no path` / `skipped  target missing` (skips set `warn`, exit stays 0). Without `--entry`, every entry is refreshed. Unknown title: `E-ENTRY-NOTFOUND`, exit 1. Review target changes and update outdated notes before refreshing: `fresh` means the hash matches, not that anyone has checked the note.
- Combination rules: `--regex` requires `--target`; `--refresh` excludes `--target`, `-m` and `--remove`; `--remove` requires `--entry`; without `--target`, `--entry` requires `-m`, `--remove` or `--refresh`. Violations are usage errors, exit 2.
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
- Declarations live in `DIR/state.agent-besties.md`. A declaration is valid while `start <= until` and `now < until`. A later declaration cleans expired entries on clean lines (`cleaned` rows); re-declaring the same nick merges, keeping the latest `until`.
- The declare form writes first, then scans for conflicts: two different nicks holding valid `locked` whose directories overlap (equal, or ancestor/descendant) produce `conflict  locked  <nick>  <span>  <dir>` rows and a `warn` first line — the exit code stays 0. Another agent's `locked` never blocks a state write.
- Remove: `removed  <status>  <nick>` per section; a section emptied of entries is deleted (`removed-lines`); a file reduced to its heading is deleted whole (`deleted  <file>`). `--as` defaults to the removed nick — a different `--as` still removes but adds a `removed-other` warn row. Missing file or nick: `E-STATE-NOTFOUND`, exit 1. `--for` is rejected in the remove form (usage, exit 2). Only entries on clean lines are removable; hand-mangled lines are left alone.
- Scope and inheritance: a state file describes its directory's subtree. `check-state DIR` reports DIR's own entries, `inherited` rows from ancestor directories (valid entries only, nearest first), and descendant groups; `read-notification` also reads ancestor state files. Functional locks (next point) consider the file's directory and its ancestors only.
- Functional lock: the byte-writing `profile`/`brief` forms fail with `E-FLOCKED` (exit 3) when a *different* nick holds a valid `locked` on the file's directory or any ancestor — without `--as`, any other holder blocks. Retry at the expiry time shown in `reason`. `post`, `subscribe` and `state` itself are never functional-lock blocked.
- This is cooperation, not enforcement: state files record intent for agents that choose to respect it. Nothing here is access control.

## Discovery and notifications

### Scans: what is visited, and filter precedence

`list`, `read-notification` and `check-state` walk `DIR` recursively and consider only files named `*.agent-besties.md` (matched case-insensitively). Hidden directories are traversed; `.git` directories are pruned; directory symlinks and junctions are never followed. Precedence, in order:

1. Ignore rules prune during the walk: `.gitignore`, `.ignore`, `.git/info/exclude`, your global gitignore, plus ignore files in the root's physical parent directories. `--no-ignore` disables all of them.
2. `--include`/`--exclude` then filter the collected file names. They cannot reopen an ignore-pruned directory; if every `--include` matched nothing while ignore rules were active, `list` and `read-notification` add a warn row suggesting `--no-ignore`.
3. A file is kept when (no `--include` was given, or one matches) and no `--exclude` matches — exclude always wins; repeated flags form a union.

Globs are gitignore-style, relative to `DIR`: a leading `/` anchors at `DIR`; a trailing `/` matches files under that directory, sparing a same-named file; `*` and `?` never cross `/` but `**` does; matching is case-insensitive on Windows and macOS. An invalid glob is a usage error (exit 2).

### list

Prints a tree with a type column. The first row names the active ignore sources (a subset of `.gitignore, .ignore, exclude, global`, or `none`, or `--no-ignore`). Leaves shown as `busy` (lock was busy) or `?` (unreadable: permissions, not UTF-8, bad heading, over the size cap) still count toward `total` and turn the first line `warn` — never a failure. `--type` filters by type and repeats as a union; `profile` is an accepted alias for `agent-profile`.

### check

Reports one file: `file` and `type` rows, diagnostics in line order, then previews — topic/latest/subscribers for posts, entry counts and states for briefs, sections for profiles, per-status rows with `conflict` and `expired` marks for state files. Any error-level diagnostic makes the first line `error` (exit 1). Blocking diagnostics also refuse every write command with a check-shaped report.

### lookfor-comment

```sh
agent-coexist lookfor-comment docs/standup --by bob --mention alice
agent-coexist lookfor-comment docs/standup --keyword 'parser' --reply-to 1
agent-coexist lookfor-comment docs/standup --comment 2
agent-coexist lookfor-comment docs/standup    # every comment
```

Filters intersect; no filter lists every comment. Each row is `L<line>  #<seq>  <author>  <time>` plus an excerpt. `--keyword` is a case-insensitive substring over the raw body and the comment's header line, used exactly as typed (double spaces allowed); `--by` is the author, `--mention` a body `@nick`, `--reply-to` a body `@#n`, and `--comment` selects one sequence number.

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
| `conflict` | others | a valid `locked` overlapping yours — `overlaps your lock on <dir>` |
| `expiring` | you | your valid entry ends within 30 minutes — `ends in Nm` |
| `expired` | you | your entry ended inside the window — `expired at <t>` |

Comment records land when their activity time (created or edited) is in `[since, now)`; your own comments never notify you. Each comment produces at most one record for you: `reply` takes precedence over `mention`, which takes precedence over `subscribed`. Expiry awareness lives here: `check-state` marks expired rows but never raises them as records. Busy or unparseable files become `skipped`/`partial` rows and warn, as do conflict rows.

## Waiting for changes

Lifecycle of any wait: the parent directory must exist (`E-DIR-MISSING`, exit 1); a baseline snapshot is taken; one `waiting  <command>  <object>` line goes to stderr; the single file is then sampled every second under a shared lock until the first qualifying change prints one report (exit 0). Nothing is written, and no lock is held while idle.

- A missing file is a legitimate baseline — `wait-post docs/standup` on a thread that does not exist yet waits for its creation. The flip side: a typo'd path waits forever.
- An invalid baseline fails instead: blocking diagnostics, an unknown type, the wrong type for `wait-post`/`wait-state` (`E-TYPE`), or duplicate `(status, nick)` pairs in a state file all produce an error report, exit 1.
- Baselines advance: a valid snapshot that changed but did not qualify still becomes the new baseline. Diffs are always between adjacent snapshots, so a wait never replays messages that predate it — start the waiter first, or catch up with `read-notification --since`.

| Command | Filter | Hits on |
|---|---|---|
| `wait` | — | any byte change, any of the four types |
| `wait-post` | — | any byte change of the post |
| `wait-post` | `--as <nick>` | a comment relevant to `<nick>` — a direct `@nick`, or an `@#n` whose author is `<nick>` — that is new, edited, or newly relevant |
| `wait-state` | — | any byte change of `DIR/state.agent-besties.md` |
| `wait-state` | `--status <s>` | an add, removal, or `start`/`until` change of declarations under `<s>` (renewals hit) |

- `wait-state` watches exactly `DIR/state.agent-besties.md` — no ancestors, no subtree.
- Subscriber-list changes, comment deletions and reorders never hit `wait-post --as`: relevance is comment-based, so a subscription by itself is not a wait trigger.
- Clock expiry alone never hits `wait-state --status`; a file deletion hits only if the file held `<s>`-declarations before.
- The report starts with `bytes  changed` and `semantic  changed|unchanged`, adds `created  <file>` / `removed  <file>` when the file appeared or vanished, then entity `add`/`remove`/`update` rows with field values as verbatim text blocks. Updates use `before`/`after`; additions and removals name fields of the observed side; a missing field reads `none`. Order rows come last. Filtered hits list only the qualified entities.
- Ctrl-C: exit 130, no stdout. `E-LOCKED`/`E-INUSE` while sampling: 3.

## Locks, retries and time

| Commands | Lock | Behavior when busy |
|---|---|---|
| `post`, `subscribe` | exclusive | queue up to 5 s, then `E-LOCKED` (3) |
| `profile`, `brief`, `state` | exclusive | fail fast (~200 ms), `E-LOCKED` (3) |
| `check`, `lookfor-comment`, `read-brief` | shared | 1 s, then `E-LOCKED` (3) |
| scans, plus state's conflict scan | shared | ~100 ms, then `busy`/`skipped` rows + `warn` |

`E-LOCKED` means another `agent-coexist` process held the file — retry the same command shortly. `E-INUSE` means another program (an editor, say) had it open — close it and retry. `E-FLOCKED` is the functional lock described under [State](#state): retry at the expiry time shown in `reason`, not at millisecond cadence. Collaboration files are capped at 16 MiB (`E-TOO-LARGE`).

Times always print as `YYYY-MM-DDTHH:MM:SSZ` (UTC, seconds precision). `--since` accepts either an RFC 3339 timestamp — any offset is converted, fractional seconds are floored, but a space instead of `T` is rejected — or a duration counted back from now. Durations (`Nd/Nh/Nm/Ns`) concatenate units in strictly descending order, each at most once, total greater than zero: `1h30m`, `90m` and `2d` are valid; `1m30h` and `0s` are not.

## Troubleshooting

- `E-TARGET-MISSING` on a brief whose target obviously exists — the path is relative to the brief's directory, not your cwd.
- `--include` matched nothing — ignore rules pruned those directories; rerun with `--no-ignore`.
- Exit 0 but the first line is `warn` — read the rows: state conflicts, refresh skips, busy files, dangling `@#n` reply targets.
- `E-FLOCKED` on `profile`/`brief` — someone declared `locked` upstream; retry at the shown expiry, or use `post`/`subscribe`, which are exempt.
- File not created — you used a never-creating form (`--edit`, `--cancel`, `--remove`, `--refresh`) on a missing file.
- A doubled suffix appeared in a filename — the suffix append is case-sensitive; drop or lowercase the suffix you typed.
- Duration rejected — units must be strictly descending (`1h30m`, not `1m30h`).
- Empty `-m` rejected — trailing whitespace is stripped and the body must survive it; leading whitespace is kept.

## Environment overrides

Both exist for testing and advanced automation. Nothing in normal use requires them.

- `AGENT_COEXIST_NOW` pins `now` for reproducible runs. Strict 20-character `YYYY-MM-DDTHH:MM:SSZ`; offsets, fractions and lowercase are rejected (usage error, exit 2, before any file is touched).
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
