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
`agent-coexist` with no command is also a usage error: one `error: …` line
saying that a subcommand is required, with the list of commands and a
pointer to `--help`, exit 2. It does not print the full help page.

## Conventions

### Files and paths

- Collaboration files end in `.agent-besties.md`. `PATH` arguments may omit the suffix; it is appended when missing. That check is byte-exact, so type the suffix in lowercase or leave it out: `docs/standup.AGENT-BESTIES.md` gets a second suffix appended. Scans match the suffix case-insensitively.
- `DIR` arguments default to `.` and must already exist (`E-DIR-MISSING` / `E-NOT-DIR`, exit 1). No command creates `DIR` itself.
- These forms create the file and any missing parent directories, printing `created  <dir>/` rows before `created  <file>`: `post` (new thread), `subscribe` (add), `profile` (bare or section set), `brief` (bare, description, `--target`).
- These forms never create: `post --edit`, `subscribe --cancel`, `profile --remove`, and the brief note-only, `--entry --remove` and `--refresh` forms. On a missing file they fail with `E-FILE-MISSING`, exit 1. When the `PATH` you gave is a directory, the reason says so ([Troubleshooting](#troubleshooting)).
- If a creating write is refused (wrong type, `E-NAME`, a [functional lock](#functional-lock), a busy lock, a blocking parse error), the CLI deletes the empty file and empty parent directories this invocation just created, after checking they are still the same empty objects. Pre-existing files, files that already contain bytes, replacements at the same path, and directories that are no longer empty stay. An empty file or directory can therefore appear briefly and vanish; this cleanup is not crash-safe or atomic against other programs.
- Windows path display rules are under [Platform notes](#platform-notes).

### Names and identity

- Creating `post`, `profile`, `brief`, and `subscribe` forms build the H1 from the file-name stem as you typed it. The stem must be a valid name: 1–100 letters, digits, `_`, `-`, with `.` allowed after the first character. Otherwise the command fails with `E-NAME` (exit 1); the CLI never renames, truncates, or substitutes another name.
- An existing file that already parses stays editable when its stem and H1 disagree (`W-NAME` on `check`). Rename the file to the H1 name if no file has that name yet; never overwrite an existing file. Copying the stem into the H1 is offered only when the stem is itself a valid name.
- `--as <nick>` is your name. It is recorded in comment headers and matched by notifications. For `profile`/`brief` writes, it exempts you from functional locks recorded under that same nick (a lock held by any other nick still blocks). It is never authenticated (see [SECURITY.md](../SECURITY.md)). Nicks are 1–64 characters: a letter, digit, or `_` first, then letters, digits, `_`, or `-`.
- Statuses (`--status`) are 1–32 ASCII lowercase characters: `a-z`, then `a-z0-9-`. Section names follow the nick grammar.
- Entry titles have no grammar, but `--entry` rejects control characters and two spaces in a row anywhere, including at either end (`--entry 'parser  entry'` and `--entry 'parser  '` are usage errors); one space at either end is ignored. Titles in the file are matched and displayed after whitespace folding: runs of spaces collapse to one, ends are trimmed.
- Title columns and human diagnostic text (message and `why`) use that folded display, with any remaining control characters shown as U+FFFD. Stored heading text and body bytes are not rewritten. A stored heading that already contains control characters is fixed by editing it by hand ([Headings the CLI cannot address](file-format.md#headings-the-cli-cannot-address)).
- `-m` bodies: CRLF becomes LF, trailing whitespace and newlines are stripped, leading whitespace is kept, and the result must be non-empty; otherwise a usage error, exit 2.
- A free-text option value that starts with `-`, such as a body that begins with a Markdown list item, must be attached with `=`: `-m='- item'`, `--keyword=-x`, `--regex='-x'`. This applies to `-m`, `--keyword`, `--entry`, `--target`, `--regex`, `--include`, and `--exclude`. As a separate word it is read as an option and the command fails with a usage error that shows the attached form (`to pass it as the value of -m, attach it with '=': -m='- item'`) instead of clap's `-- -x`, which would turn the word into the `PATH`. If that word is not valid Unicode, the error names the encoding first (`the value of -m is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode`) and says a value that starts with `-` must also be attached with `=` on the Unicode text (`start that same argument with '-m=' followed directly by the whole Unicode text`); it does not offer a copyable command that pastes the original bytes. A body that spans several lines has no one-line form, so the error says to start that same argument with `-m=` followed directly by the whole text. A value with a character the error shows as `<U+XXXX>` (see the next item) gets no copyable form either: the error shows the text with those escapes and says to start the same argument with `-m=` followed by it, typing each `<U+XXXX>` as the one character it names. Every other value option (nicks, statuses, section names, `--type`, `--since`, `--for`, and the comment numbers of `--edit`, `--reply-to`, `--comment`) never takes a value starting with `-`, attached or not; its error restates what the option takes, such as `--edit takes a comment number made of digits only (such as --edit 3), which never starts with '-'`.
- A rejected value is echoed as typed, never folded or trimmed: `(got 'alice ')`. Controls, whitespace other than a plain space, and other characters that do not print (Unicode format characters such as a BOM `U+FEFF`, a zero-width space `U+200B` or the bidirectional controls `U+202A`–`U+202E`/`U+2066`–`U+2069`, plus unassigned and private-use code points) print as `<U+XXXX>`, so a trailing CR from a CRLF file shows as `<U+000D>` and an invisible BOM in front of a nick shows as `<U+FEFF>`. Letters, combining marks, quotes and backslashes print unchanged. This is only the echo: no character is removed for you, and the same values are accepted as before. It applies to nicks, statuses, section names, `--entry`, `--type`, `guide` topics, `--for`, `--since`, `--target`, `--regex`, `--include`/`--exclude` globs, the cause of an invalid glob, `AGENT_COEXIST_NOW`, `AGENT_COEXIST_LOCK_WAIT_MS`, the values quoted in the argument parser's own errors (`invalid value '…'`, `unexpected argument '…'`), and the option named in an unknown-option error (`'post' has no option --bad<U+202E>`); ordinary values print unchanged. For all of these except regexes and globs, when only whitespace at either end makes the value invalid, the error adds `remove the space before or after 'alice'` (`whitespace` instead of `space` when it is not a plain space).
- `PATH` and `DIR` accept native file names that are not valid Unicode. Every other argument must be valid Unicode text. Otherwise the usage error names that argument where it can (for example `--as`) and says to retype it or re-encode its source as Unicode.

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
- Text rows are a `  <key>` line followed by `    | <line>` lines that reproduce content verbatim (read-brief bodies, wait before/after values; a binary or non-UTF-8 target is decoded lossily first). They may contain any spacing; read them by the `| ` prefix, never as columns. There is one `| ` line per content line: a final line break ends the last line and adds no extra line, so a one-line target shown as `L1..L1` prints one line. A blank line inside the content prints as a bare `    | `, and empty content prints the key line alone. No `/` is substituted.
- Diagnostics print as `  error  L<n>  <message>` (or `warn`), with `why` and `fix` lines indented 9 spaces underneath.
- Failures that are not anchored to a file line (a missing file, directory, or entry; the wrong type; a busy file; a functional lock; a refused state removal) print `reason  <message>` and `fix  <fix>` rows instead, with no `why`.
- IDs such as `E-LOCKED` or `W-STATE-CONFLICT` on these pages are catalog names for the kinds of diagnostic. Diagnostic rows and `reason` rows do not carry their own ID as a field; identify them by the printed message ([Troubleshooting](#troubleshooting) lists common ones). A `why` or `fix` text may mention another ID in passing, such as `(W-STATE-ENTRY)`; that names a related diagnostic, not the one reported. Only two row kinds carry an ID as a field: `remaining  <ID>  L<n>` after a brief write, and the `skipped`/`partial` rows of scans described next.
- Scan rows name what a scan could not fully read (a state file by its directory, any other file by its path), then the cause. A `skipped` row carries the ID that stopped the read, or the word `busy` or `unreadable` (the file could not be read, or the directory could not be listed or, on Unix, entered for lack of search permission). `busy` stands for either of two IDs: `E-LOCKED` (the file lock or a locked read stayed busy) or, on Windows, `E-INUSE` (another program holds the file open without allowing shared access); the recovery for each is under [Locks, retries and time](#locks-retries-and-time). A `partial` row carries an ID and `L<n>` as two columns: `partial  <dir|path>  <ID>  L<n>`, for example `partial  docs/standup  E-HEAD  L9`. These rows cover any file a scan reads, posts included (for example `skipped  ./broken  E-FENCE` from `read-notification`); each command's section says which files its scan reads.
- When a scan leaves something unread, a `fix` row follows the scan rows (it is not counted in `total`). It says what may be missing and ends with read-only commands that open exactly what was not read, joined by `; `: `agent-coexist check <dir>/state` for a skipped or partial state file (`check state` for `.`), `agent-coexist check <path>` for another skipped or partial file or for a `busy` or `?` leaf in `list`, `agent-coexist list <dir>` for a directory `list` could not read, and, after a write, `agent-coexist check-state <dir>` for an unreadable directory. When the CLI could see that an unreadable directory holds a state file (for example, the directory's names could be listed but the file could not be opened), the directory and its state file show as one `skipped  <dir>  unreadable` row whose command is `agent-coexist check <dir>/state`. A directory that cannot be listed at all gives the CLI no way to tell whether it holds a state file, so its row names no state file. Whenever an unreadable directory is involved, the advice adds ` (first make each directory marked unreadable both readable and enterable: it needs read and search permission)`. Run them to see what the scan missed; they are lookups, not a retry of your command. When a path cannot be written as a copyable command, the row says to run them by hand and why. Which commands print this row, and what each one says, is under [Incomplete lock and conflict scans](#incomplete-lock-and-conflict-scans).
- `canonicalize  <dir>  failed  <os error>` means the real path of that directory (the scanned `DIR`, or a directory holding a state file the scan found) could not be resolved, so its identity fell back to the path as spelled. `<os error>` is the operating-system message from that resolution (for example `Permission denied (os error 13)` on Linux, `Access is denied. (os error 5)` on Windows). Failure is any resolution error: a missing path, a missing ancestor, lack of search/traverse permission, lack of read/open permission on a directory, or a symlink or junction that cannot be followed, not only link aliasing. The row does not refuse a write. Locks and conflicts are then related by spelling alone, and an overlap reached through a symlink, a junction, or another spelling of the same directory can be missed. `check-state`, `read-notification`, the `state` conflict scan, and the `profile`/`brief` lock scan can print it. It is not a record, so a `total` row does not count it. It turns an `ok` first line into `warn`; a first line that is already `error`, such as `E-FLOCKED`, stays `error` with its exit code. After those rows, `check-state` and `read-notification` print `this command could not resolve the real path of each directory marked canonicalize; first restore that each of those paths exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission), then run this command again` (not counted in `total`). A `skipped  <dir>  unreadable` row for a directory whose state file the CLI did not see gets no read-path `fix` (see [Incomplete lock and conflict scans](#incomplete-lock-and-conflict-scans)). The canonicalize `fix` does not cover unreadable-directory rows.
- A `brief` report, `--refresh` included, can hold both kinds of skip. `entry  <title>  skipped  no path` is a refresh skip and carries no ID; a row whose key is `skipped` or `partial` there always comes from the lock scan of the brief's directory and its ancestors ([example](#refresh-report-example)).
- Completed `list`, `lookfor-comment`, `read-notification`, and `check-state` overviews (`ok`/`warn`, including zero records) end with a `total` row. Fatal reports of those commands (missing dir or file, not a directory, `cannot scan`, other OS errors, blocking diagnostics) have no `total` row. `check`, `read-brief`, write commands, and wait reports never have one.
- Lines always end in LF, even on Windows.
- Progress (`scanning dirs: N, files: M...`) goes to stderr, and only when stderr is a TTY. Besides the read scans, it covers the ancestor lock scans of `profile` and `brief` and the post-write conflict scan of a `state` declaration, which reads the state files of `DIR` and its ancestors and the state files below `DIR` that ignore rules do not skip ([Declaring](#declaring)), so it can take a while on a large tree. The `state --remove` form runs no such scan. Stdout stays clean for redirection.

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

- Comments are numbered `#1`, `#2`, … in file order. A gap is a non-blocking warning (`comment numbers jump from #<a> to #<b>`); when the first legal number is greater than 1, the warning reads `the first comment in this post is #<b>, not #1` instead. The CLI never renumbers. Its `why` says which case it is. When some number in between appears nowhere in the post, it names the first such number and the `why` reads `no comment in this post has number #<m>, so @#<m> cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order` (with a malformed header in between, that header may be the missing comment; before the first comment, the `why` instead reads `that comment may have been deleted or renumbered by hand, or the first comment was written with a higher number`). When every number in between (or, for a first number above 1, every number below it) is used elsewhere in the post, the comments are only out of order: `no comment is missing and @#n still resolves; only the order is unusual`.
- Mentions (`@nick`) and replies (`@#n`) are plain text in the body. A mention starts at the beginning of the text or after any character other than a letter, digit, `_`, or `@`: `@alice-@bob` mentions `alice-` and `bob`; `a@bob` is not a mention.
- A successful write prints `comment  #<n>  L<line>`: the comment's number and the line of its header. With `--edit` the row has a third column, `comment  #<n>  L<line>  edited`. A new thread prints its `created` rows first.
- Next comes a single `mentions` row when the new body mentions anyone: every distinct `@nick`, joined by `, ` in first-occurrence order (`mentions  bob, alice`). Then one `reply-to  #<m>` row per distinct `@#m`, in first-occurrence order. A body with no mentions has no `mentions` row, and one with no replies has no `reply-to` rows; when every reply resolves, nothing follows these rows.
- A reply whose number has no comment in the file after the write prints `reply-to  #<m>  (not found)` and makes the first line `warn`; the comment is still written. One `fix` row follows the `reply-to` rows: `if you meant another comment, edit this one so the reply names a number this post carries: agent-coexist post … --as=<nick> --edit=<n> -m '<the whole body, with the reference corrected>'`, with your own nick and the just-written comment's number already filled in. `--edit` rewrites the whole body, so that marker is the comment's full body with the reference corrected — resending only the fixed line would drop the rest of the body. Only its author may edit the comment, and it was just written by you.

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
    fix           if you meant another comment, edit this one so the reply names a number this post carries: agent-coexist post docs/standup.agent-besties.md --as=carol --edit=3 -m '<the whole body, with the reference corrected>'
  ```

- `check` of a post lists `authors` in first-occurrence order (`none` when empty; a single author whose nick is literally `none` prints the same, so read the file when the difference matters).
- When `check` prints copyable header repairs for several malformed or duplicate headers in one report, each repair gets its own unused sequence number, so copying every printed header is enough. The printed number is used only if no reply in this post already names it (`@#n`); otherwise the `fix` says to choose another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and to change the replies that meant this comment to that number. A header that cannot get a remaining number stays a manual heading repair. Bodies and already-legal comments are never rewritten.

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

- `--target` resolves against the brief file's directory, not the shell's cwd: for `docs/onboarding`, `--target ../src/parser/mod.rs` means `src/parser/mod.rs` from the workspace root. Paths starting with `/` (or a Windows drive letter) are used as-is. A missing target is `E-TARGET-MISSING`, exit 1, detected before the brief is opened; its reason names the path as typed and the path the CLI looked for, relative to your cwd: `target src/parser/mod.rs does not exist; looked for docs/src/parser/mod.rs`. A target that is a directory fails on every platform with `reason  cannot read <target>: is a directory, not a file` and the `fix` `fix the path, then run the corrected command`, exit 1; nothing is written.
- The default entry title is the target's file name (`mod.rs`); `--entry` overrides it. Reusing an existing title that points at a *different* target, without `--entry`, is `E-ENTRY-TITLE-TAKEN`, exit 1. Its `fix` is your original command with `--entry '<title>'` added (before any `--` separator), so `--as`, `-m`, `--regex` and every other option stay: fill in `<title>` and run it. When your command line cannot be printed, the `fix` says to rerun it by hand with that option added.
- `--regex` requires `--target`. Writing checks that the pattern compiles; matching is checked on read, and a target that does not match makes the entry `stale` (`regex not found`). `--regex ''` removes every `- regex:` line in that entry's property area, including handwritten duplicates.
- Raw CR or LF in `--target` or `--regex` is a usage error (exit 2) and writes nothing; other whitespace is kept as part of the stored value. Both errors echo the value: `--target must not contain raw CR or LF (got 'src/main.rs<U+000D>'); remove the whitespace before or after 'src/main.rs'`, while `--regex` says to write `\r` or `\n` in the pattern to match them. To match CR or LF in a target, use the regex escapes `\r` and `\n`.
- The target form writes `entry  <title>  added|updated  <path>` plus `hash  <sha256>`, and `note  set` when `-m` is given. The description form writes `description  set`; the note-only form `note  set`; removal writes `entry  <title>  removed` and `removed-lines  L<a>..L<b>`.
- `--refresh` recomputes and rewrites stored hashes only; it never re-checks regexes. Per selected entry it prints `entry  <title>  refreshed  <path>`, or a skip row `entry  <title>  skipped  no path` / `entry  <title>  skipped  target missing` (skips make the first line `warn`, exit stays 0). `refreshed` means the hash was checked against the target's current bytes, not that it changed. Without `--entry`, every entry is refreshed. Unknown title: `E-ENTRY-NOTFOUND`, exit 1.
- When a description, note, `--target`, or `--refresh` form would leave the file byte-for-byte as it is (the same text, the same path and hash, hashes already current, every entry skipped, or a brief with no entries), nothing is written: the form's own rows are followed by `unchanged  <file>`, as for the bare form, and no `remaining` rows follow. An `ok` or `warn` report of these forms without that row wrote the file.
- Review changed targets and update outdated notes before refreshing: `fresh` means the hash matches, not that anyone has checked the note. Every `fix` that re-points an entry or rewrites its hash — a missing `- path:` line, a missing or invalid `- hash:` line (with or without a path), an unreadable or directory target, an absolute path, a shifted hash, a stale entry whose regex no longer matches or whose target is missing, and a regex that does not compile (with or without a path) — says which file to choose when the entry's own target is missing or unreadable, says to review the target and the note and to update the note if it no longer describes the target, and says that the command also records the target's current hash.
- Combination rules: `--regex` requires `--target` and cannot be combined with `--remove`; `--refresh` excludes `--target`, `--regex`, `-m`, and `--remove`; `--remove` requires `--entry` and excludes `--target` and `-m`; without `--target`, `--entry` requires `-m`, `--remove`, or `--refresh`. Violations are usage errors, exit 2, and write nothing. A `--regex` given with `--refresh` or `--remove` is reported as that combination (`--refresh cannot be combined with --target, --regex, -m or --remove`, `--regex cannot be combined with --remove`), not as a missing `--target`.
- After a brief form that writes bytes, the new bytes are re-parsed and every surviving non-blocking error-level (`E-…`) diagnostic is appended as a `remaining  <E-ID>  L<n>` row, in line order, making the first line `warn`. These are the last rows, after any lock-scan rows and their `fix`. Warnings (`W-…`) and entry states are not listed. When that write leaves skipped entries or remaining errors, the report also prints one `skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check <path>` row (or `skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints; run agent-coexist check on this file by hand; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly` when the path cannot be copied): after the form's own rows and before any lock-scan `skipped`/`partial`/`canonicalize` rows and their `fix`; remaining stays last. The same sentence is used for `--refresh` skips and for remaining on every writing form, and it is printed once. Follow that `check` report's `fix` rows; the lock-scan `fix` is not their recovery.

### Refresh report example

A refresh of the brief `onboarding` with three entries: `parser entry
point`, whose target changed; `renderer`, whose target is missing; and
`design notes`, which has no path. The same directory's
`state.agent-besties.md` has the H1 `# repo locks`, which is not a valid
state heading. The report is:

```text
warn  brief  onboarding
  entry         parser entry point  refreshed  src/parser/mod.rs
  entry         renderer  skipped  target missing
  entry         design notes  skipped  no path
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  skipped       .  E-H1
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check state
  remaining     E-ENTRY-PATH  L15
  remaining     E-ENTRY-HASH  L15
```

The two `entry … skipped` rows are refresh skips. The first `fix` row
is their recovery and the recovery for the `remaining` rows: it names
the printed operand `agent-coexist check onboarding.agent-besties.md`.
That row comes first, after the `entry` rows and before `skipped  .  E-H1`.
`skipped  .  E-H1` comes from the lock scan: the directory's own state
file has no recognizable H1, so its declarations were not read. The
lock-scan `fix` row after it means only the rows that start with
`skipped`, `partial` or `canonicalize`, not the two refresh skips; it
says the refresh is already written and names the read-only
`agent-coexist check state`, which opens that file. Run it and agree with
any holder it names, but do not refresh again. The `remaining` rows after
that lock-scan `fix` are the errors still left in `design notes`, the
entry without a path, and they stay last. Follow the first `fix`'s
`check`. The lock-scan `fix` is not their recovery.

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
- After writing, the declare form checks the declaring nick's own valid `locked` entries in the directory's scope, whatever status this write declared: a nick that already holds a valid lock and now declares `observing` is still checked against it. The conflict check reads the state files of `DIR` and its ancestors and the state files below `DIR` that ignore rules do not skip; `state` has no `--no-ignore`, so to see locks inside ignored directories run `check-state DIR --no-ignore`. Each valid `locked` entry held by another nick on an overlapping directory (equal, or ancestor/descendant) produces a `conflict  locked  <nick>  <start>..<until>  <dir>` row that names that other holder and its directory, and the first line becomes `warn`; exit stays 0. A conflict between two other nicks is not reported by this declaration; `check-state` marks conflicting entries across the queried scope (see [Scope, inheritance, and conflicts](#scope-inheritance-and-conflicts)). Another agent's `locked` never blocks a state write.
- If that post-write scan hits a fatal ignore-source error, the write is already committed: first line `warn`, the usual `created` / `set` / `cleaned` rows, then `scan  <path>  <cause>` and a `fix` row, exit 0. There is no rollback; `check` of the state file shows the written declaration. The `fix` names the repair for the source (see [Scans](#scans-what-is-visited-and-filter-precedence)), then reruns only the conflict check with `check-state`, not the declaration:

  ```text
  warn  state  room
    created       room/state.agent-besties.md
    set           observing  bob  2026-09-25T12:00:00Z..2026-09-25T20:00:00Z
    scan          room/.ignore  line 1: error parsing glob '{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
    fix           correct line 1 of room/.ignore, then check for conflicts without declaring again: agent-coexist check-state room
  ```

  Other scan failures after the write (an OS error that does not come from an ignore source, such as a current directory that global excludes need but that is no longer available) print the `scan  <path>  <cause>` row (`scan  <dir>  failed` when there is no cause to show) and a `fix` that starts `the declaration above is written, but the conflict check stopped at the error in the scan row above, so a conflicting lock may have been missed; without declaring again, resolve that error, then check for conflicts: agent-coexist check-state <dir>`. First line `warn`, exit 0. When the directory cannot be written as a copyable command, the `fix` says to run `agent-coexist check-state` on this directory by hand, and why.

- When that scan completes but cannot read every state file, the report adds the `skipped`/`partial` rows after any `conflict` rows, then a `fix` that starts `the declaration above is written, but the conflict check could not read everything it needed` and names each file to open, without declaring again ([Incomplete lock and conflict scans](#incomplete-lock-and-conflict-scans)). First line `warn`, exit 0.

### Removing

- `--remove <nick>` prints `removed  <status>  <nick>` per section. `--as` defaults to the removed nick; a different `--as` still removes but adds a `removed-other` warn row.
- Without `--status`, the nick is removed from every status that holds a clean entry for it. That span is intentional; add `--status` to remove only one status.
- Only clean lines are rewritten. A line is clean when every comma fragment that is nonempty after trimming parses as a declaration; empty fragments are ignored. A comma-only line holds no declarations and can disappear with its section.
- Mixed lines: if the nick still has a parseable declaration on a line with an unparseable nonempty fragment, under a selected status, the whole removal is refused before any write (`E-STATE-DIRTY`, exit 1) and the line stays byte-for-byte. Either delete the nick's declaration from that line by hand, which completes the removal (re-run only if the nick has more entries the command should remove), or have whoever wrote the unparseable fragments make them parseable, then re-run. The file does not record who wrote a fragment, and a fragment on another nick's line is not that nick's; ask rather than delete it. The printed retry keeps the complete original command, including `--status`. Legal fragments on mixed lines still count in `check` and lock scans.
- A status section whose remaining bytes are only its heading and blank lines is deleted (`removed-lines`). Leftover handwritten text, mixed lines, and fences keep the heading and those bytes.
- A file reduced to its heading, reader comments, and blanks is deleted whole (`deleted  <file>`).
- Missing file or nick: `E-STATE-NOTFOUND`, exit 1, nothing written. The `fix` is always a read-only `check-state` of the directory. Without `--status` it says to check which nicks hold entries there, then rerun with the right nick or directory. With `--status`, the reason also names that status, and the `fix` says to check which statuses hold the nick, then rerun with the right `--status`. Do not drop `--status` and remove a different declaration instead; a live `locked` entry would go with it.
- A same-file lock-conflict `fix` includes `--status=locked`, so copying it removes only that lock and leaves the nick's other statuses.
- `--for` is rejected in the remove form (usage error, exit 2).

### Scope, inheritance, and conflicts

- A state file describes its directory's whole subtree.
- `check-state DIR` prints, in order: `inherited` rows from ancestor directories (valid entries only, nearest first), DIR's own entries, then one group per state file below DIR that ignore rules do not skip (`--no-ignore` also sees declarations in skipped directories), headed by its path relative to DIR plus `/` and ordered by directory. After the entries come `skipped`/`partial` rows sorted by directory, then a `fix` row naming the skipped or partial state files to open ([Incomplete lock and conflict scans](#incomplete-lock-and-conflict-scans)), then `canonicalize  <dir>  failed  <os error>` rows ([Reading output](#reading-output)), then, when those exist, `this command could not resolve the real path of each directory marked canonicalize; first restore that each of those paths exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission), then run this command again`, then `include` hints ([Scans](#scans-what-is-visited-and-filter-precedence)), then `total`. Within one file, entry rows keep native file order.
- Row shapes, split on two or more spaces:
  - `inherited  <status>  <nick>  <start>..<until>  <dir>  [conflict]`: `<dir>` is the ancestor that holds the entry, spelled from the current directory.
  - `<status>  <nick>  <start>..<until>  [conflict|expired]`: DIR's own entries, at indent 2. Entries in a descendant group use the same shape at indent 4, under their group heading.
- The last column is an optional mark, at most one per row. `conflict` means that the entry is a valid `locked` and that another nick holds a valid `locked` on an overlapping directory (equal, or ancestor/descendant) somewhere in the same report. Both entries of such a pair are marked, whether they are inherited, own, or descendant, and whether or not they are in the same file. Sibling directories do not overlap. Only this report's declarations are compared: an inherited lock can still conflict with a lock in another branch below that ancestor, which `check-state` on the ancestor shows. `expired` marks an own or descendant entry that is no longer valid. Inherited rows are always valid, so they never carry `expired`. Either mark makes the first line `warn` (exit 0). `total` counts every entry row shown — inherited, own, and descendant — plus the `skipped`/`partial` rows; `fix`, `canonicalize`, and `include` rows are not counted.
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
- `check` on a state file reports each expiry and each same-file lock conflict (`W-STATE-CONFLICT`) once, from the selected declaration per nick. It also marks the file's own preview rows `conflict` or `expired`. These `W-STATE-CONFLICT` pair diagnostics (`locked is held by both <a> and <b>`) cover pairs inside one file. They appear only in `check` and in a wait report that hits on a file that still exists. A complete `check` lists every pair, so many overlapping `locked` entries make a large report.
- The other commands that report overlapping locks print no pair diagnostics. `check-state` marks the entries, `read-notification` prints `conflict` records for locks that overlap your own, and a `state` declaration prints `conflict` rows. `list` does not report conflicts.
- A `read-notification` conflict names the deepest overlapping own lock; when depths tie, the last file in scope order wins. Per-file conflict rows are never dropped.

### Functional lock

- The byte-writing `profile` and `brief` forms fail with `E-FLOCKED` (exit 3) when a *different* nick holds a valid `locked` on the supplied file's directory or any ancestor. Without `--as`, any holder blocks.
- A write through a file symlink also checks the canonical referent's parent directory and its ancestors. If the file's real path cannot be resolved, only the supplied path's chain is checked, and that alone prints no row. If the supplied directory's own real path cannot be resolved, its chain is checked as spelled and the report adds `canonicalize  <dir>  failed  <os error>` ([Reading output](#reading-output)). There is no volume-wide hardlink search.
- Retry at the expiry time shown in `reason`, or ask the holder in a post to release the lock (`post` is not blocked). Pass `--as <holder>` only when the lock really is yours. The `fix` ends with `then retry: <original command>`; run it once the lock has expired or been released, not straight away.
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
2. `--include`/`--exclude` then filter the collected file names. They cannot reopen an ignore-pruned directory. When you passed at least one `--include` and no file survived step 3, with or without `--no-ignore`, the command prints one `include  <glob>  <hint>` row for every `--include` you passed, and the first line becomes `warn`. The hint states the empty result and names only the causes this command line makes possible: `no file is left after the filters; check that this glob is relative to DIR` (in `check-state`: `no file under DIR is left after the filters (the state files of DIR and its ancestors are read regardless); check that this glob is relative to DIR`, because the filters apply only to state files below `DIR`), plus `, and that --exclude does not remove the files it matches` when you passed any `--exclude`, plus `; if the files are in a directory that ignore rules skip, --include cannot reach them: add --no-ignore` when an ignore source was loaded (no `--no-ignore`, and the `ignore` row of `list` is not `none`). These rows report the empty final result, not a per-glob match count:
   - `--include 'a/**' --include 'b/**'`, where a file under `a/` is kept: no `include` rows, even though `b/**` matched nothing.
   - `--include 'a/**' --exclude 'a/old/**'`, where every `a/` file is under `a/old/`: an `include  a/**  no file is left after the filters; …, and that --exclude does not remove the files it matches…` row, even though `a/**` matched files. The exclude removed them, not ignore rules.
3. A file is kept when (no `--include` was given, or one matches) and no `--exclude` matches. Exclude always wins; repeated flags form a union.

- An ignore or Git configuration source that cannot be used fails the scan: first line `error`, `reason  cannot scan <path>: <cause>`, a `fix`, exit 1, with no overview rows and no descendant groups. `--no-ignore` skips those sources. Ordinary unreadable directories are only `skipped  <dir>  unreadable` warnings. That includes a Unix directory with read but no search (execute) permission: its names can be listed, but nothing inside it can be opened, so its ignore files are not reported as failing sources. The `fix` names a repair for the cause, then repeats the original command as `then retry: <command>`:

  | Cause | `fix` starts with |
  |---|---|
  | a pattern that does not parse, or a line that is not UTF-8 (`<cause>` begins `line <n>:`) | `correct line <n> of <path>` |
  | a malformed `.git` file or `commondir` reference, or a gitdir that points to a missing path or to something that is not a directory, such as a regular file (`<path>` is then the `.git` file, and `<cause>` reads `gitdir points to <target>: …`) | `correct the contents of <path>` |
  | the source is a directory or another non-regular file | `replace <path> with a regular file or remove it` |
  | any other OS error opening or reading the source, such as a permission denial | `check that <path> can be read` |
  | every line parses, but the file's rules together exceed the pattern matcher's size or nesting limits (`<cause>` is the walker's own text, such as `error building NFA`) | `shorten, simplify or remove rules in <path> until they fit the pattern matcher's size and nesting limits` |

  For example:

  ```text
  error  check-state  room
    reason        cannot scan room/.ignore: line 1: error parsing glob '{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
    fix           correct line 1 of room/.ignore, then retry: agent-coexist check-state room
  ```

  An OS error during the scan that comes from no source file, such as a current directory that global excludes need but that is no longer available, is an ordinary I/O failure instead: `reason  cannot scan <DIR>: <os error>`, and the usual I/O `fix` ([Other I/O failures](#other-io-failures)). If the command line cannot be written as a copyable command ([When there is no command](#when-there-is-no-command)), a source `fix` reads `<repair>, then retry by hand; …` and says why no command is shown. The `fix` never suggests `--no-ignore`, which would scan without your ignore rules; pass it yourself only when that is what you mean.
- Globs are gitignore-style and relative to `DIR`: a leading `/` anchors at `DIR`; a trailing `/` matches files under that directory, not a same-named file; `*` and `?` never cross `/`, but `**` does. Matching is case-insensitive on Windows and macOS.
- Globs are matched exactly as typed, including internal double spaces. Only the display of the `include` hint row is folded, with control characters shown as U+FFFD. An invalid glob is a usage error (exit 2) that echoes the glob as typed, and echoes the same way any user characters in the cause ([Names and identity](#names-and-identity)).

### list

Prints a tree with a type column. The first row names the active ignore
sources (a subset of `.gitignore, .ignore, exclude, global`, or `none`, or
`--no-ignore`). Leaves shown as `busy` (`E-LOCKED`, or on Windows `E-INUSE`,
as for scan rows in [Reading output](#reading-output)) or `?` (unreadable:
permissions, not UTF-8, bad heading, over the size cap) still count toward
`total` and turn the first line `warn`, never a failure. `total` counts every
leaf in the tree, `busy` and `?` leaves included, plus each
`skipped  <dir>  unreadable` row; the first row naming the ignore sources
and any `fix` or `include` rows are not counted. `--type` filters
by type and repeats as a union; `profile` is an accepted alias for
`agent-profile`. `busy` and `?` leaves stay shown and counted under any
`--type`, because their type is unknown.

When the tree has a `busy` or `?` leaf or a `skipped  <dir>  unreadable`
row, a `fix` row follows the `skipped` rows (before any `include` rows;
not counted in `total`). It says types or files may be missing and names
read-only commands, joined by `; `: `agent-coexist check <path>` for each
`busy` or `?` leaf in tree order (the leaf's path without the suffix),
then `agent-coexist list <dir>` for each unreadable directory; the row
says such a directory `could not be read or entered`, and with one it
adds ` (first make each directory marked unreadable both readable and
enterable: it needs read and search permission)`. `check` on a `busy`
leaf waits for the lock as any
single-file read does. When a path cannot be printed as a command, the
`fix` says how to run the same commands by hand.

### check

Reports one file: `file` and `type` rows, diagnostics in line order, then
previews: comments, `authors` (first-occurrence order), topic, latest, and
subscribers for posts; entry counts and states for briefs (not target
bodies; `read-brief` prints those); sections for profiles; per-status rows
with `conflict` and `expired` marks for state files. Any error-level
diagnostic makes the first line `error` (exit 1). Blocking diagnostics also
refuse every write command with a report shaped like `check`, followed by
one last `refused` row that says nothing was written and names the rows
that block it:

```text
error  post  docs/standup
  file          docs/standup.agent-besties.md
  type          post
  error  L9  …message…
         why  …why…
         fix  …fix…
  warn   L14  …message…
         why  …why…
         fix  …fix…
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-FENCE at L9 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

The last clause appears only when the report also shows rows that do not
block writes. Fix the named rows, then run your original command again; the
other rows can wait. A write refused only because its result would pass the
16 MiB cap has no `refused` row: its own `fix` already says nothing was
written.

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
  fix           some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one: agent-coexist check ./docs/notes
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

- After all records: `skipped`/`partial` rows (`<kind>  <path>  <cause>`, no time column, sorted by path; causes under [Reading output](#reading-output)), then, when any of them names a file, a `fix` row naming each file to open before concluding nothing is new ([Incomplete lock and conflict scans](#incomplete-lock-and-conflict-scans)), then `canonicalize  <dir>  failed  <os error>` rows, then, when those exist, `this command could not resolve the real path of each directory marked canonicalize; first restore that each of those paths exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission), then run this command again`, then, when `--include` was given and no file survived the filters, one `include  <glob>  no file is left after the filters; check that this glob is relative to DIR` row per `--include` ([Scans](#scans-what-is-visited-and-filter-precedence)).
- `total` counts records plus `skipped`/`partial` rows; `fix`, `canonicalize`, and `include` rows are not counted. The example has five records and one `skipped` row.

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
- Every sample, the baseline included, waits up to 1 s for a shared lock. A lock still busy after that second ends the wait with `E-LOCKED`, exit 3; the next sample does not retry it. On Windows, a file another program opened without allowing shared access ends the wait at once with `E-INUSE`, exit 3. This 1 s budget covers only that one lock acquisition; it is not a timeout for the wait as a whole or for reading the file. The `fix` for both says that a retried wait starts from a new baseline, so after its ready line you read the file again (`wait-state`: run `check-state` on the directory again), then gives the retry. Any other failure after the ready line whose `fix` offers a retry — an unreadable file, for example — leads with the same new-baseline sentence, saying the retried wait does not report changes made while the file could not be read, so you read the file again after its ready line there too.
- A path-class fault before the ready line (not found, not a directory, a directory where a file is needed, or an invalid name) still prints `fix the path, then run the corrected command` and offers no retry. Invalid-name faults keep this advice even after the ready line. After the ready line, for a missing path, a parent replaced by a file, or — on Unix — a path that no longer names a regular file, the `fix` leads with that new-baseline sentence, then says how to restore the path without deleting anything blindly (`restore the intended path of <file>: if the directory on it is missing, recreate it, or move it back if it was renamed away; if a file now occupies the directory's place, move that file aside, keeping it; if <file> is a dangling link, restore the file it points at`; or `a directory on the path of <file> is now a file; move that file aside, keeping it, and recreate the directory it replaced`; or, on Unix, `the path of <file> itself no longer names a regular file; move whatever now occupies it aside, keeping it; the watch also accepts the file absent, so nothing needs to be created in its place` — on Windows a watched file replaced by a directory stays the permission-denied repair with the same new-baseline lead and retry), then prints the original wait as the command to retry.
- A missing file is a legitimate baseline: `wait-post docs/standup` on a thread that does not exist yet waits for its creation. A mistyped path therefore waits forever; check the path.
- `PATH` gets the managed suffix as in every command: `agent-coexist wait notes` watches `notes.agent-besties.md`, never a plain file named `notes`, even when only the plain file exists; in that case it waits for `notes.agent-besties.md` to be created.
- An invalid snapshot fails the wait with an error report, exit 1, whether it is the baseline or any later sample; it never becomes the baseline. Invalid means blocking diagnostics, an unknown type, the wrong type for `wait-post`/`wait-state` (`E-TYPE`), or duplicate `(status, nick)` pairs in a state file. A hand edit after the ready line that, for example, leaves an unclosed fence therefore ends the wait with exit 1 rather than waiting on. Any wait that fails and is then restarted takes its baseline from the file as it is at the restart: the repaired content, for example the reply whose fence you closed, is already in the new baseline and will not be reported as a change, so after the new ready line, read the file again as in step 3 (`wait-state`: run `check-state` on the directory again). For a duplicate pair in a state file watched by `wait` or `wait-state`, the `fix` says to start the wait again only after the duplicate is gone; unlike `check`, it does not call the merge optional. Its command is the merge, for the listed nick itself to run (`the command for <nick>: agent-coexist state <dir> --as=<nick> --status=<status>`); restarting the wait is the step after it and has no command of its own. That merge replaces both entries with one entry from now only when both entries are on clean lines. When either entry sits on a line that also has unparseable fragments (`W-STATE-ENTRY`), that line is kept as is and the duplicate stays: running the printed command then leaves the wait refusing identically — the declaration still writes and prints `set`, but ends `warn` with that line's `W-STATE-ENTRY` partial row and its `check` pointer — and this refusal does not list that row. There `<nick>` inspects the file, deletes its own entries from every such line by hand, keeping each line's other content, then looks at the remaining entries and declares again to merge any duplicate that is left or to start one entry from now. (`wait-post` refuses a state file by type before that.)
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
- `wait-post --as <nick>` can also report `update  comment  #<n>  relevance` when no field of that comment changed but it became relevant, for example because the comment it replies to is now authored by `<nick>`. `before` is `none` and `after` is the reference that makes it relevant (`@<nick>` or `@#<m>`). This is not a stored field.
- Comments, entries, sections, and declarations are paired by raw identity (first match wins) and reported in file order.
- Filtered hits (`wait-post --as`, `wait-state --status`) list only the qualifying entities. They still print the `bytes`, `semantic`, `created`/`removed`, and any warning rows, but never the document name or type, a brief description, or order rows.
- A brief's description appears once when the file is created, deleted, or changes type.
- Warning rows, including same-file lock-conflict warnings, come from the snapshot after the change, but their validity — conflicts and expiry — is judged at the wait's start time, not at the hit: a lock that expired while you waited still counts as valid, so the report can show a conflict warning with no expiry warning. Before acting on a state warning or its fix, run `check-state` on the directory (or `check` on the file) to judge it at the current time. A `removed` report has no after snapshot and therefore no warning rows: the deleted file's problems, such as a lock conflict, went away with it, so its first line is `ok` and it lists only the `bytes`, `semantic`, `removed`, and `remove` rows.
- State `fix` recipes in the report are checked against the directory's currently selected `state.agent-besties.md` at that final snapshot.

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
- Collaboration files are capped at 16 MiB (`E-TOO-LARGE`). A file already over the cap reads `file is larger than 16 MiB`. A write whose result would pass the cap is refused with `this write would make the file larger than 16 MiB: it is <size> bytes now and would become <result> bytes`; nothing is written and the file stays readable, so shorten the new text or put it in a new file (for a comment, a new post).
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
"no locks" nor as a reason to refuse every write. An incomplete scan is
not a clearance. Each report that has one ends its scan rows with a `fix`
row that names, as read-only commands, exactly what was not read
([Reading output](#reading-output)). Run them before you rely on "no
lock" or "nothing new":

| Report | `fix` begins | What to do |
|---|---|---|
| `profile`/`brief` write | `any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there` | do not repeat the write; open each named file and agree with any holder before writing again. When a canonicalize row is present, that same `fix` also includes ` (first restore that each path marked canonicalize exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission))` before the named check-state |
| `state` declaration | `the declaration above is written, but the conflict check could not read everything it needed (the rows above that start with skipped or partial), so a conflicting lock may have been missed; without declaring again, look at what it could not read and agree with any holder named there` | do not declare again; open each named file and agree with any holder |
| `check-state` | `some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds` | open each named state file; an overview without its locks is not proof that none apply. A `skipped  <dir>  unreadable` row for a directory whose state file the CLI did not see prints no `fix`, whether or not a state file is there: make that directory readable and enterable and run `check-state` again before relying on the overview |
| `read-notification` | `some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one` | open each named file before concluding nothing is new. A post under the reserved state file name can look clean in `check` (`type post`); its `skipped … E-TYPE` row already tells you that directory has no readable state declarations. As for `check-state`, a directory row with no file the CLI saw inside prints no `fix`, so make that directory readable and enterable and run the scan again before concluding nothing is new there |

For example, after `profile` wrote a section while `team/state.agent-besties.md` had an unparseable line (column padding is illustrative):

```text
warn  profile  team/alice
  section       role  set
  partial       team  W-STATE-ENTRY  L7
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check team/state
```

`agent-coexist check team/state` shows line 7 and any lock it holds. A
`canonicalize  <dir>  failed  <os error>` row marks another kind of gap:
the real path could not be resolved (a missing path, a missing ancestor,
lack of search/traverse or read/open permission, or a symlink or junction that cannot be
followed). Directories were then compared by spelled path instead of
real path, so an overlap through a link can be missed ([Reading
output](#reading-output)). After a `profile`/`brief` write that gap gets
a `fix` naming `agent-coexist check-state <dir>` for the written file's
directory, and that `fix` includes ` (first restore that each path marked canonicalize exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission))` when identity fell back to the spelled path. Restore existence and that every directory on the path, ancestors included, can be opened (read and traverse/search permission) before that check-state, and before retrying the same `check-state` or `read-notification`.

### Time formats

- Times print as `YYYY-MM-DDTHH:MM:SSZ` (UTC, seconds). A leap second read from a file keeps `:60`.
- `--since` accepts an RFC 3339 timestamp (any offset is converted, fractional seconds are floored, a space instead of `T` is rejected) or a duration counted back from now. A value that is neither gets `--since must be an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '<value>')`; a date and time separated by a space also get `; separate the date and time with 'T', not a space`. When only whitespace at either end is wrong, the error says to remove it instead (see [Names and identity](#names-and-identity)).
- Durations (`Nd`, `Nh`, `Nm`, `Ns`) use units in strictly descending order, each at most once, with a total greater than zero: `1h30m`, `90m`, and `2d` are valid; `1m30h` and `0s` are not. A bad `--for` states the same rule: `--for must be a duration such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '<value>')`.
- Times stop at `9999-12-31T23:59:59Z`. A `state` declaration whose end would pass it is refused with `the declaration would end after 9999-12-31T23:59:59Z, the latest representable time; use a shorter --for (without --for, 'locked' lasts 2h and other statuses 8h)`. A `--for` too long to count at all reads `--for '<value>' is too long: the declaration would end after 9999-12-31T23:59:59Z, …`. A `--since` duration reaching back before year 0001 reads `--since '<value>' reaches back before year 0001; use a shorter duration or an absolute time`.

## Recovery commands (`fix`)

Diagnostics and most failure reasons carry a `fix` line. Read it before
acting: it is an instruction, a command, or an instruction followed by a
command, and a command may still need a decision or a filled-in
placeholder. Never pipe `fix` output into a shell unconditionally.

### Command or instruction

| Form | Shape | What to do |
|---|---|---|
| Hand edit or file content | `set line 1 to "# <name>.post" …`, `add a closing line of 3 backticks where the block should end`, `… whoever wrote it moves it where it belongs … or deletes it, …; anyone else asks who wrote it before moving or deleting it` | edit the file yourself, then run `check` again; when the `fix` says only the author or whoever wrote the text acts, and that is not you, ask them instead |
| Command | `agent-coexist profile <path> --section role -m '<what you do>'` | fill in its `<what you do>` marker, then run it after reading `why` |
| Repair, then retry | `retry shortly: <original command>`, `correct line 1 of room/.ignore, then retry: agent-coexist check-state room`, `review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief …` | do the instruction first, then run the command; a retry repeats your original command |
| Look up, then correct | `pick an existing title from: agent-coexist check <path>`, `find the number of the comment to edit, then rerun --edit with it: agent-coexist lookfor-comment …`, `check which nicks hold entries here, then rerun with the right nick or directory: agent-coexist check-state <dir>`, `…; collaboration files under that directory: agent-coexist list <dir>` | run the read-only command first, use its output to correct your original command, then run that corrected command; the lookup's `ok` does not complete the original task |
| Check after a completed write | `correct line 1 of room/.ignore, then check for conflicts without declaring again: agent-coexist check-state room` | the change is already written: do the repair, run the check, and do not repeat the write |
| Do this instead | `reply instead: agent-coexist post … -m '@#3 <your reply>'`, `start a new post: agent-coexist post <new path> …` | the command replaces your original one and writes something else; run it in place of the original, not after it |
| The repair itself | `choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; if it no longer exists, remove or rewrite the entry; …: agent-coexist brief … --target <file> --entry=…`, `the path names a directory, not a file; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; …: agent-coexist brief …` | the words say what the command changes; fill in its markers and run it as the repair |
| Command for a named agent | `if still needed, that agent renews it: agent-coexist state …`, `the command for carol: agent-coexist post …`, `agree who continues, then the other runs the removal; …: agent-coexist state … --remove <nick> --status=locked` | run it only if you are that agent, under your own nick; otherwise tell that agent in a post. Typing another nick into `--as` (or into `--remove`, where `--as` defaults to the removed nick) changes their records as if they had done it |

In every combined form the command is everything after the instruction's
colon and starts with the program name (a retry repeats the program path
you typed). The words before the colon say what the command is for and
who runs it. The rows above are common cases, not a complete list; when
the words fit none of them, read them as a description of the command
and any condition for running it, and ask in a post if they name another
agent or leave you unsure.

- Take the whole remainder after the `fix` key as one payload; never split it on spaces.
- Angle-bracket words written by the recipe itself are static placeholders.
  Fill them in before running: `<file>`, `<relative path>`, `<new path>`,
  `<pattern>`, `<title>`, `<text>`, `<the whole body, with the reference
  corrected>`, `<your reply>`, `<what you do>`, `<name>`, `<nick>`, and
  `<same flags>`. These are the common ones, not
  a complete list: header repairs also print `<n>`, `<nick>`, and
  `<time>` ([Before you run a command](#before-you-run-a-command)), and
  by-hand recipes print markers such as `<path of this file>` or
  `<path of the parent directory>` ([When there is no
  command](#when-there-is-no-command)). They stay
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

- `W-STATE-CONFLICT` (`locked is held by both <a> and <b>`): agree with the other holder first; only the agent that yields runs the `--remove … --status=locked` command. If the yielding declaration sits on a line with unparseable fragments, that removal is refused with `E-STATE-DIRTY`, whose fix names the by-hand deletion of the nick's declaration from that line; the printed conflict command says so too.
- `E-FLOCKED`: the `fix` reads `if the lock is yours, add --as=<holder> before any -- separator; otherwise wait for it to expire or ask its holder to release it, then retry: <original command>`. When your command already has `--as`, the lock can be yours only if that `--as` named the wrong nick, so the `fix` reads `if the lock is yours and the --as in this command names someone else, rerun it with --as=<holder> in place of that --as; otherwise …` instead; a second `--as` would be a usage error. Run the retry only after the expiry or the release. Using `--as` with the holder's nick is only for the holder itself.
- A comment without a fenced body (`W-BODY`): only the comment's author can add it with `post --edit`, so the `fix` names the author, tells everyone else to ask the author or reply with `@#<n>`, and then gives the command for the author. Do not run it under the author's nick yourself.
- Text outside any fence (`W-STRAY`) and an unparseable state fragment (`W-STATE-ENTRY`): the file does not record who wrote that text, even when it sits under another agent's heading or on another agent's state line. Whoever wrote it moves it where it belongs (a fenced body, or an entry line under a status) or deletes it, and only a comment's author changes that comment. If you did not write it, ask in a post who did before moving or deleting it; do not move it into the nearest fence, which may be someone else's comment.
- A repeated `- subscribers:` line (`W-SUB-DUP`): keep only the first line, with each nick once, and delete the later line instead of merging it in. A nick named only on the later line is not subscribed now; merging would subscribe it without its choice. Each agent named only there decides for itself whether to subscribe, with `subscribe --as <its own nick>`.
- A malformed comment header (`E-HEAD`): the `fix` is the corrected header to write, keeping the comment's own number, nick and times where the broken header still shows them (`rewrite the header as "## #2 bob (2026-09-17T09:20:00Z)"`). A number set off by a space (`## # 2 bob (…)`) is kept as the number, never taken as the nick. When the nick or time can no longer be read, or could be either (an all-digit word that is not clearly the nick, such as the `2` in `## # 2 (…)`), that part prints as `<nick>`/`<time>`. A time written only after `edited` is kept as the edit time, and the creation time before it prints as `<time>`. Replace only those placeholders, with the comment's original author and the time it was first written, and keep the rest as printed; do not use the current time. When the repair gives the comment a different number than its header shows (the number is already taken), the `fix` adds `its number changes from #<old> to #<new> if no reply in this post already names @#<new>, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names; change any @#<old> that meant this comment to that number` (or, with no usable number, `the header shows no usable number, so use #<new> if no reply in this post already names @#<new>, otherwise another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and change any reply that meant this comment to that number; if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change`); otherwise such a reply would point at another comment or at none, or two comments would end up sharing one number. When no number above the highest one in the post is left, see [When there is no command](#when-there-is-no-command).
- `W-ENTRY-SHIFTED` (`entry "<title>" is shifted: <target> changed since its hash was taken`): review the target and update the note before `--refresh`; refreshing only accepts the new hash.
- `E-STATE-DIRTY`: the `fix` gives two ways. Deleting the nick's declaration from the named line by hand is itself the removal; re-run only if the nick has more entries this command should remove. Or have whoever wrote that line's unparseable fragments make them parseable, then paste the printed retry. The retry keeps the original `--status`; do not rebuild a bare `--remove` that would also drop other statuses.
- `E-STATE-NOTFOUND`: the command is a read-only `check-state`; inspect its output, then correct your own `--remove` (the nick, the directory, or the `--status`) and rerun it ([Removing](#removing)).
- `cannot scan <path>: …`: an ignore or Git configuration source stopped the scan. Do the repair the `fix` names first: correct the named line (the message may suggest an escape, such as `[{]` for a literal `{`), correct the `.git` reference, replace a directory with a file, make the file readable, or, when the file's rules together are too large or too deeply nested, shorten, simplify or remove rules in it. Then run the command after `then retry:` ([Scans](#scans-what-is-visited-and-filter-precedence)). After a `state` write, run the `check-state` the `fix` gives instead of declaring again; the declaration is already written.
- State recipes (`W-STATE-DUP`, `W-STATE-RANGE`, `W-STATE-EXPIRED`, `W-STATE-CONFLICT`) print a directory-level `state` command only when the inspected file is that directory's selected `state.agent-besties.md`. Otherwise no state command is printed: first settle which file should be this directory's `state.agent-besties.md` (when a `W-NAME-STATE` row is shown, its `fix` describes how; never overwrite a separate existing state file), then run `check` on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry. `W-STATE-DUP`, `W-STATE-RANGE`, and `W-STATE-EXPIRED` are optional: the command is for the agent named in the entry, only if its declaration is still needed, and redeclaring replaces that agent's entries under that status with one from now, except an entry on a line that also has unparseable fragments (`W-STATE-ENTRY`): that line is kept, and the latest end time still wins; `check-state` shows the result. `W-STATE-DUP` merges only on the next state write that rewrites the section holding the duplicate — a declaration or removal under that status, or expiry cleanup of an entry in it — and only when both entries are on clean lines; a duplicate under another status survives writes that do not touch its section. When either duplicate entry sits on a line that also has unparseable fragments (`W-STATE-ENTRY`), that line is kept as is and the duplicate stays: delete that nick's entries from every such line by hand, keeping each line's other content, then look at the remaining entries and declare again to merge any duplicate that is left or to start one entry from now.

### Other I/O failures

An operating-system error that has no catalog ID prints `reason  cannot <action> <path>: <os error>` and exit 1. `<action>` names the step that failed: `open`, `read`, `write`, `create`, `delete`, `lock`, `stat`, or `scan`, for example `cannot open docs/standup.agent-besties.md: Read-only file system (os error 30)`. The `fix` depends on the kind of error, not on the action, and never claims a single cause:

| Error kind | `fix` |
|---|---|
| the path itself is wrong (not found, a directory where a file is needed, not a directory, an invalid or too-long name) | `fix the path, then run the corrected command` |
| the path itself is wrong and contains a line break or other control character | `the path contains a line break or other control character (original: '<echoed path>'); fix the path, then run the corrected command`. The `reason` path stays folded; `<echoed path>` is the original bytes with controls as `<U+XXXX>` (for example `standup<U+000A>notes.agent-besties.md`) |
| permission denied | `check the permissions of <path> and of the directories on its path, then retry: <command>`; on Windows it adds `or wait until another program stops using, deleting or replacing it`, because a file another program is deleting gives the same error |
| disk full or quota exceeded | `free disk space or quota where <path> is stored, then retry: <command>` |
| the file would pass a file-size limit (`EFBIG`), which free space does not fix | on Unix, `raise this shell's file-size limit (ulimit -f), or move <path> to a file system that allows larger files, then retry: <command>`; elsewhere, `move <path> to a file system that allows larger files, then retry: <command>` |
| read-only file system | `make the file system that holds <path> writable, then retry: <command>` |
| anything else | `resolve the error shown in 'reason', then retry: <command>` |

When the command line cannot be written as a copyable command, the retry reads `then retry the original command by hand;` followed by the reason ([When there is no command](#when-there-is-no-command)).

After a wait's ready line, a path-class fault (the watched path's directory gone, a directory on it replaced by a file, or — on Unix — the path itself no longer a regular file) is not the start-time row above: the `fix` says how to restore the intended path and prints the original wait to retry, as under [Lifecycle](#lifecycle). On Windows a watched file replaced by a directory stays the permission-denied row with the new-baseline lead and retry.

### When a write fails partway

A write command changes the file in place, through its locked handle. It writes any new tail first, then only the bytes after the part that stays the same, then shortens the file if needed, then syncs. Running out of space or into a file-size limit while the file grows therefore leaves the previous bytes untouched. If a step fails before the sync, the CLI tries to put the previous content back, and the report tells you which case you are in:

- The previous content is back, or was never changed: the usual I/O report above (`cannot write <file>: <os error>`), exit 1. The file is as it was, so do the repair and run the retry.
- Putting the previous content back failed too: `reason  cannot write <file>: <cause>; putting its previous content back also failed: <rollback error>` and a `fix` that says how many leading bytes are unchanged, that the rest may mix this change with the previous content, and not to run the same command again. Do the repair it names, then `agent-coexist check <file>` and correct the file by hand. Exit 1.
- Every byte was written but the final sync failed: `reason  wrote <file>, but could not confirm that the change is stored on disk: <cause>`. The change is in the file, and running the command again could apply it twice (for example a second comment). Do the repair, then run `agent-coexist check <file>` to confirm the change is still there. Exit 1.

In both uncertain cases the file and any directories the command created are kept. The restore itself is not synced and can fail, for example after a device error, or when a copy-on-write or compressing file system has no space even to overwrite existing bytes. None of this protects against a crash or power loss in the middle of a write: the file can then hold its previous bytes followed by part of the new tail, or old and new bytes mixed after the unchanged prefix. `check` points at the damage; the CLI does not repair it.

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

- An operand or original argument contains NUL, CR, or LF, or holds native data this POSIX UTF-8 text cannot represent (such as unpaired UTF-16 surrogates on Windows). The recipe keeps its own advice and says the command must be run by hand, followed by "no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly". A retry of your original command reads like `retry shortly by hand; no copyable command …`: you already have that command. A recipe that runs a different command keeps that whole command, its subcommand, options, and every operand that can be printed, and puts a marker in place of each value that cannot: `…: run this command by hand, typing <path of this file> yourself, because no copyable command can be printed when a path or argument contains a line break or characters this text cannot reproduce exactly: agent-coexist profile <path of this file> --section role -m '<what you do>'`. Type the real path where the marker stands, keep the rest, and run it; the operation it names is still the one to carry out. Only when no part of the recipe can be kept does it become "repair the original file or retry the original command by hand; " plus the same explanation. An ignore-source repair keeps its own first step and ends `then retry by hand;` with the same explanation; after a `state` write it ends `then check for conflicts without declaring again: run agent-coexist check-state on this directory by hand;` instead. Other recipes in the same report are unaffected, and the CLI never emits a lossy executable look-alike.
- A stored heading cannot be addressed by `--entry`, `--section`, or `--status` (control characters, a profile section title outside the nick grammar, an invalid status name): see [Headings the CLI cannot address](file-format.md#headings-the-cli-cannot-address).
- A state-content file is not the directory's selected `state.agent-besties.md` (see [Troubleshooting](#troubleshooting)).
- No comment number above the highest one in the post is left (`#18446744073709551614`) for a malformed header (`E-HEAD`) or a later duplicate (`E-SEQ-DUP`): lower numbers that no header in the post uses are still valid, so the `fix` says to keep the comment where it is, with its author, time and text, and give it such a number by hand. For `E-HEAD` it prints the header to write with `<n>` for that number (`rewrite the header by hand as "## #<n> bob (2026-09-17T09:20:00Z)", replacing <n> with an unused lower number: one that no header in this post carries, that you give no other header, and that no reply in this post already names (@#n)`), plus any `<nick>`/`<time>` to fill in; for `E-SEQ-DUP` it says to change only the number, with the same unused-lower-number criterion. Then update any reply that meant the comment. That lower number is a new identity, not a restore of a known original number. Every exhausted `E-HEAD` `fix` also says that if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change. A known original restore may capture an existing `@#n`; a fresh number must not. The `fix` ends by noting that a lower number can leave a `W-SEQ-ORDER` or `W-SEQ-GAP` warning, which does not block writes, and that it frees no number for new comments: once `post` stops with `E-SEQ-EXHAUSTED`, new comments go in a new post.

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
- On Windows and macOS, the H1 name check (`W-NAME`, `W-NAME-STATE`) compares the H1 with the file name as it is on disk, whatever case you typed in `PATH`: `check ROOM/STANDUP` on `room/standup.agent-besties.md` with `# standup.post` gives no warning, while the `file` row still shows the path as typed. Linux compares the name as given, which is also the name on disk.

## Troubleshooting

Each entry starts with the text a report prints, followed by the catalog ID used elsewhere on this page.

- `target <path> does not exist; looked for <resolved>` (`E-TARGET-MISSING`) on a brief whose target obviously exists: the path is relative to the brief's directory, not your cwd, and `<resolved>` shows where that put it. The `fix` says so; correct `--target` and rerun, or, if the target is meant to exist at that path, create it and run the printed retry of your original command.
- `<file> does not exist; <dir> is a directory, and this command needs one collaboration file` (`E-FILE-MISSING`): you passed a directory, such as `.`, `docs`, or `docs/`, where the command takes one file `PATH` that it will not create: `check`, `lookfor-comment`, `read-brief`, and the never-creating write forms (`post --edit`, `subscribe --cancel`, `profile --section … --remove`, and the brief note-only, `--remove`, and `--refresh` forms). The suffix was appended, so for `docs` the CLI looked for `docs.agent-besties.md`. Rerun with the path of one collaboration file in that directory (the suffix may be omitted); the `fix` ends with `agent-coexist list <dir>` to show them. Creating forms are not affected: `post docs` still creates `docs.agent-besties.md`. For any other missing file, the `fix` says to correct the path and rerun, and lists the collaboration files in that file's directory.
- `directory <dir> does not exist` (`E-DIR-MISSING`): for `list`, `read-notification`, and `check-state`, correct the directory and rerun; the `fix` lists collaboration files under the parent (`correct the directory and rerun; collaboration files under its parent: agent-coexist list <dir>`). When that parent does not exist either, the `fix` is `correct the directory and rerun; its parent does not exist either; collaboration files under the nearest existing directory: agent-coexist list <dir>`, so the listing command itself succeeds. For `state` and the waits, the `fix` says to rerun with the correct name, or, if the directory is new, create it first and then run the printed retry; no command creates directories.
- `<path> is not a directory` (`E-NOT-DIR`): the command needs a directory. When your path appears exactly once in the original command, the `fix` is that command with the path replaced by its parent directory, and every other option kept: `rerun on its directory: agent-coexist …`. Otherwise it prints `agent-coexist <command> <parent> <same flags>`, and you add your other options back yourself.
- `--include` finds nothing, or `include  <glob>  no file is left after the filters; …` rows appear: the rows mean no file survived all filters, not that this particular glob matched nothing ([Scans](#scans-what-is-visited-and-filter-precedence)). Check that each glob is relative to `DIR`; if the row names `--exclude`, check those globs too, since exclude always wins and can remove every match. If the row names ignore rules and neither explains it, the files may sit in a directory those rules skip; add `--no-ignore` only when you mean to scan without them.
- `error  check-state` / `cannot scan`: an ignore or Git configuration source failed to load or parse (the same as for `list` and `read-notification`), not an unreadable directory. Follow the repair in the `fix`, then retry ([Before you run a command](#before-you-run-a-command)). If a `state` write succeeded first, you get `warn` with a `scan  <path>  <cause>` row and a `fix` that ends in `check-state`, exit 0, and the written bytes stay.
- Exit 0 but the first line is `warn`: the command finished; read the rows. Common causes include state conflicts, refresh skips, busy or partial state files, a dangling `@#n`, and incomplete lock scans. This is not a complete list; see the command's own section and [Exit codes](#exit-codes).
- `<dir> is locked by <holder> until <time>` (`E-FLOCKED`) on `profile`/`brief`: `<holder>` has a valid `locked` on that directory or above, possibly on a symlink target's directory. Only the `--as` nick's own declaration is exempt; without `--as`, any holder blocks, including you ([Functional lock](#functional-lock)). If `<holder>` is you, re-run with `--as <holder>`, in place of any other `--as` you gave; otherwise retry at the shown expiry, or ask the holder in a post (`post` and `subscribe` are not blocked).
- `file name "<name>" is not a valid name` (`E-NAME`) on a creating command: nothing was created. The `fix` repeats the rule (1–100 letters, digits, `_`, `-` or `.`, starting with a letter, digit, `_` or `-`); run the same command with a name that follows it.
- `<file> does not exist` (`E-FILE-MISSING`) where the file's directory is missing too: the `fix` says so and lists the collaboration files under the nearest directory that does exist, so the listing command itself succeeds.
- `file type is <type>, not <type>` (`E-TYPE`): the heading names a different type from the one this command works on. Query, `subscribe`, `--edit`, `--cancel`, `--remove`, `--refresh`, and the waits print `this command needs a file of type <type>; rerun it with such a path; files of that type under this directory: agent-coexist list <dir> --type=<type>`. Creating write forms (`post` without `--edit`, `profile` without `--remove`, `brief` bare, description, or `--target`) print `this command needs a file of type <type>; rerun it with such a path, or with a new path that no file uses yet to create one; files of that type under this directory: agent-coexist list <dir> --type=<type>`. When `state` or `wait-state` finds a file of another type under the reserved name `state.agent-besties.md`, the `fix` is `state.agent-besties.md in this directory has type <type>, so it cannot hold state; rename that file to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name, then retry: <original command>`.
- `check` warns `state file is named "<name>.agent-besties.md", not "state.agent-besties.md"`: `state` and `check-state` ignore it. Inspect it and any existing `state.agent-besties.md` in the same directory, resolve their declarations, then decide which file keeps the reserved name; never overwrite a separate existing state file. Then run `check` on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry.
- `<nick> is still declared on line <n> of the state file in <dir>, mixed with unparseable fragments` (`E-STATE-DIRTY`): see [Removing](#removing).
- `<nick> has no entry in <dir>`, or `… under status "<status>" in <dir>` (`E-STATE-NOTFOUND`): no matching nick under the selected status (or no file), and nothing was written; see [Removing](#removing).
- Your own `--remove` removed more than one status: without `--status` it spans every status that holds the nick.
- `file name "<stem>" does not match the H1 name "<name>"` (`W-NAME`), or a broken H1, after a hand rename: the file stays usable if it parses. Rename the file to `<name>.agent-besties.md` only if no file has that name yet; never overwrite an existing file. If the stem is a valid name, setting line 1 to `# <stem>.<type>` also works.
- A heading with control characters, an invalid section name, or an invalid status: rename it by hand; see [Headings the CLI cannot address](file-format.md#headings-the-cli-cannot-address).
- File not created: you used a never-creating form (`--edit`, `--cancel`, `--remove`, `--refresh`, note-only `-m`) on a missing file.
- A doubled suffix in a filename: the suffix append is case-sensitive; drop or lowercase the suffix you typed.
- Duration rejected: units must be strictly descending, each at most once, and the total above zero (`1h30m`, not `1m30h` or `0s`). The error states the rule.
- `unexpected argument '-…' found` right after `-m` or another free-text option: a value starting with `-` must be attached with `=` (`-m='- item'`); see [Names and identity](#names-and-identity). Anywhere else on the line the word is an option the command does not have, and the error says so and points to `agent-coexist <command> --help`. Its `--` form applies only when that word really is the file or directory name. A single-dash word such as `-as` is told to use two dashes (`--as`). `-V` after a command is told that `--version` goes before any command (`agent-coexist --version`).
- Empty `-m` rejected: trailing whitespace is stripped and something must remain.
- A wait never returns: check the path (a missing file is a valid baseline, and a plain file without the suffix is never watched) and the filter; see [What ends a wait](#what-ends-a-wait).

## Environment overrides

Both exist for testing and advanced automation; normal use needs neither.
Business commands validate them; `guide`, `help`, `--help`, and `--version`
ignore them.

- `AGENT_COEXIST_NOW` pins `now` for reproducible runs. Strict 20-character `YYYY-MM-DDTHH:MM:SSZ`; offsets, fractions, lowercase, and leap seconds (`:60`) are rejected (usage error, exit 2, before any file is touched). The error echoes the value as typed and says to unset it; when only whitespace at either end is wrong, such as a trailing CR or space from a script, it first says to remove that whitespace, and a date and time separated by a space is told to separate them with `'T'`, both of which keep the fixed clock.
- `AGENT_COEXIST_LOCK_WAIT_MS` (digits only, 0 to 18446744073709551615 milliseconds) overrides the 5 s queue budget of `post`/`subscribe` writes; no other budget. A larger number is told the limit; any other value is told the accepted form, echoed as typed, with the same whitespace hint as `AGENT_COEXIST_NOW`; a value that is not valid Unicode is told so, as for `AGENT_COEXIST_NOW` (`AGENT_COEXIST_LOCK_WAIT_MS is not valid Unicode; set it to digits such as 5000, or unset it to use the default 5000`). Unset it to use the default 5000.

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
  3:*)     echo 'busy or locked; read the reason and fix rows' ;;
  *)       printf 'failed (exit %s)\n' "$code" ;;
esac
```

- For exit 3, tell the cases apart by the text of the `reason` row, which carries no ID field. `file is locked by another process` (`E-LOCKED`): retry shortly. `file is in use by another program` (`E-INUSE`): close the other program's handle, then retry. `<dir> is locked by <holder> until <time>` (`E-FLOCKED`): wait until that time or ask the holder; do not retry in a tight loop.
- Treat `fix` lines as advice for the agent to read, not as commands to execute automatically ([Recovery commands](#recovery-commands-fix)).
- Use `wait-post`/`wait-state` instead of your own polling loop; they sample once per second. Combine them with the read, start, re-read order under [Waiting for changes](#waiting-for-changes), because a wait does not report what changed before it started.
- Use a stored timestamp with `read-notification --since`, overlapping the previous window. There is no read cursor or exactly-once delivery; treat repeats by identity and content.
