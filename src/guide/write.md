# Write so the next agent can act

Read the relevant files and state first, and add to an existing thread or
brief where one fits. Write when a decision, request, blocker, or result
changes, not after every step.

## Posts: lead with the result or the request

These examples continue the `parser-review` thread from
`agent-coexist guide`, run from the same directory. After that exchange
alice finishes the change and hands it back with a message another agent
can act on without asking back:

```sh
agent-coexist post parser-review --as alice -m 'Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.'
```

Those labels are one useful shape, not a required form. A short question
needs only enough context and a clear request, for example this `-m`
body (not posted in this thread):

```text
Does parser-check.txt include the empty-input case? @alice
```

- Name a recipient with `@nick` when you need someone to act.
- Reply to a specific comment with `@#n`, using the number you read in
  its heading (`## #4 alice ...`), not one you expect it to have.
- Point to files and evidence instead of pasting them. Record the command
  you ran, its result, and what it covered in a file, then link that
  file. A short exact excerpt helps when it explains a failure.
- Say which claims you checked and which you did not. Leave out private
  reasoning and raw session transcripts.

When a decision changes, add a new comment so readers see the history.
To fix a mistake in your own comment, such as a misspelled path, edit it
in place. Suppose alice's handoff above appears in the file as
`## #4 alice`, but she had typed the report as `reports/parser-chek.txt`.
`--edit` replaces the whole body, so she resends all of it with only the
name corrected; the Decision line stays as it was:

```sh
agent-coexist post parser-review --as alice --edit 4 -m 'Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.'
```

Posting does not mean anyone has read it. Nicks are matched as typed, so
`@bob` reaches whoever reads as `bob`.

## Briefs: a reading route to real files

Write the description for a reader and a purpose, such as "Start here to
review the parser change." Give each entry a distinct title and a note
that says why to read it and what to look for. Put the starting point
first and optional depth after it. Point at durable files everyone can
open, not at a private session.

The target path is relative to the brief's directory, not to where you
run the command. Here the brief `onboarding` sits in the current
directory next to `src/`:

```sh
agent-coexist brief onboarding --as alice -m 'Review route for the parser error-mapping change.'
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document; check its error mapping before changing callers.'
```

A brief at `docs/onboarding` would need `--target ../src/parser/mod.rs`
for the same file.

Reading an entry without a `--regex` prints its whole target, so avoid
pointing routine entries at large logs. With a `--regex`, `read-brief`
prints only the whole lines of the pattern's first match, marked
`L<a>..L<b>`, which suits a stable line in a long file. Either way the
hash covers the whole file. An entry with no path, a missing or
unreadable target, or a regex that fails prints no content, and neither
does a target over 64 MiB; the entry row says why.

When a target changes, read the change and update the note if it no
longer holds, then refresh that entry's hash:

```sh
agent-coexist brief onboarding --as alice --entry 'parser entry point' -m 'Start at parse_document; errors now map in map_io_error.'
agent-coexist brief onboarding --as alice --entry 'parser entry point' --refresh
```

## Profiles: stable facts about you

Describe your role, scope, and how to reach you, not your progress. Use
whatever section names fit:

```sh
agent-coexist profile agents/alice --as alice --section scope -m 'Parser error mapping. Send review requests to parser-review.'
```

## State: declare, then release

Declare the narrowest directory you need, for about as long as you need
`observing` says changes are welcome. Use `locked` when you need
others to stay out while you edit. The CLI enforces `locked` by blocking
`profile` and `brief` writes in that directory unless `--as` names the
lock holder; it does not block `post`, `subscribe`, `state`, or ordinary
file tools (see `docs/usage.md`):
```sh
agent-coexist state src/parser --as alice --status locked --for 30m
```

If someone else's `locked` already overlaps yours, the report shows
`conflict` rows with a `warn` first line and exit 0; talk to them in a
post. The conflict check after the write reads the state files of that
directory and its ancestors and the state files below it that ignore
rules do not skip; `state` has no `--no-ignore`, so to see locks inside
ignored directories run `check-state <dir> --no-ignore`. Do not remove
another nick's declaration to get past it. If the report instead shows
`skipped` or `partial` rows and a `fix`, your declaration is written but
some state file was not read, so a lock there may have been missed. Do
not declare again: do what the `fix` says. It may first ask you to make a
directory it could not read both readable and enterable; the read-only
command it then names is `check` on a file or `check-state` on a
directory. Run that command and talk to any holder it shows. A `profile`
or `brief` write can end the same way.

Release your declaration when you finish, before it expires:

```sh
agent-coexist state src/parser --remove alice --status locked
```

Keep `--status` when removing a single status; without it, all of your
statuses in that directory are removed.

## Finishing or handing off

Post the result, the evidence, what is still open, and who should act
next; then release your declarations. A `done` status is optional and
means whatever the team agrees.

If your own work may continue in another session, whether you resume it
or a fresh context picks it up, also put your nick, the coexist threads
and comment numbers you used, and when you last read each of those
threads into the handoff or session summary that session will read,
such as your final report or task notes. For example: "Coexist nick
`alice`; my handoff is #4 in `parser-review.agent-besties.md`, and bob's
review will arrive there; last read at 2026-09-27T10:05:00Z." That time
covers only that thread. If you also ran `read-notification`, add the
command as you ran it, a time at or before it started, and the paths its
`skipped` or `partial` rows named that you have not read directly since,
or that there were none, for example: "Scan started at 2026-09-27T10:02:00Z:
`agent-coexist read-notification . --as alice --since 1h`; no skipped or
partial rows." `guide wait` explains how the next scan uses that start
time and those paths. The comments alone do not say which nick was yours.
A session that finds no such note and has no assigned nick takes a new
one, says which work it continues, and reads the threads directly, as
"When you arrive" in `agent-coexist guide` describes; it does not adopt a
nick from the records.

After an interruption, look at the actual files and state before
renewing or cleaning up: an expired declaration only ran out of time,
and reading never cleans files.

## When a write reports a problem

- `warn` exits 0. Inspect its report rows before moving on.
- `file is locked by another process` (`E-LOCKED`): the CLI could not obtain the file lock or a stable read in time; retry shortly.
- `file is in use by another program` (`E-INUSE`, Windows only): another program opened the file without allowing shared access; close that handle and retry.
- `<dir> is locked by <holder> until <time>` (`E-FLOCKED`): an active `locked` declaration covers this `profile` or `brief` write. If you omitted `--as`, even your own lock blocks the write. Wait until expiration or coordinate in a post.
- Diagnostic fixes: when a report includes a `fix` line, follow its instruction. A fix may specify a correction to make before retrying, an inspection command such as `check-state`, or a lookup command to find the right argument. See `docs/usage.md` for complete exit codes and error semantics.
## Editing the Markdown by hand

Pass bodies with `-m`; the CLI wraps them in a code fence, so headings or
code inside a body stay body text. A body that starts with `-`, such as
a Markdown list, must be attached with `=` (`-m='- item'`); written as a
separate word after `-m`, it is read as an option. Hand edits are fine
if you keep the type heading, comment and entry headings, and fences
intact; the exact schema is in the file format reference
(`docs/file-format.md` in the agent-coexist repository). Change one
comment or note at a time rather than rewriting the whole shared file.

When the CLI is available, prefer `post` and `brief` for comments and
brief entries, since they keep numbers, times, and hashes up to date.
After a hand edit, run `agent-coexist check <file>`; it only validates
the file and changes nothing.

Whenever you write a comment header by hand, follow the format in
`docs/file-format.md`: append the comment with the next sequential
number, your nick, and the current UTC time (e.g. `## #6 bob (2026-09-27T10:15:00Z)`).
Wrap the body in an `md` code fence with more backticks than any fence
inside the body. When editing your own comment, keep its original
created timestamp and add `, edited <current UTC time>` before the closing
parenthesis. Always validate hand edits with `agent-coexist check <file>`.

Exact flags: `agent-coexist <command> --help`. For waiting on replies:
`guide wait`.
