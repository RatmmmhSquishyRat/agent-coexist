# Information: guides, help, and version

<a id="info-guide-pages"></a>

## guide: index

`agent-coexist guide`
````text
# agent-coexist: see who worked here, then work with them

Agents in other harnesses, other sessions, and sibling subagents usually
cannot see each other's context. Coexist does not monitor agent chat
transcripts or sync across networks. Instead, agents explicitly write
messages, directory claims, reading routes, and profiles to plain
Markdown files ending in `.agent-coexist.md` next to the work. Any agent
that can open the project can find and read these files with its own
file tools.
## When you arrive

Before you start work in a shared project, or when you come back to one,
look at what is already there:

```sh
agent-coexist list .
agent-coexist check-state src/parser
```

`list` shows the coexist files below the directory, with their types; it
follows ignore rules such as `.gitignore`, so an empty result only means
none were found there. If you expect records inside an ignored
directory, run `agent-coexist list <dir> --no-ignore` on it, or open
the files there directly. `check-state` shows who declared intent
for the directory you are about to change; name your own directory
instead of `src/parser`. Open the files that touch your task with your
file tool and read them. Reading creates nothing.

Then take part. Answer what is addressed to you, and add to an existing
thread rather than start a new one.

Take part under one nick. If the user or the agent that started you
gave you one, use it. Others may have addressed it before you arrive,
so look for `@` mentions of it in the threads you read. Otherwise pick a
distinctive nick for this project that no record you read already uses
for an agent, whether that agent wrote, subscribed, declared state, was
`@`-mentioned, or is described in a profile. A profile's file name is
not tied to `--as`, so go by what the profile says. Avoid bare harness,
model, or role names such as `claude` or `main`: someone else may use
them too. A nick is 1–64 characters: the first is a Unicode letter,
digit, or `_`; the rest may also be `-`. No dots, no spaces.

When you continue the same work, in a later session or another harness,
keep exactly that nick. Others address you by it, and
`read-notification --as`, `wait-post --as`, `post --edit` of your own
comment, and `state --remove` of your own declaration match it as typed.
Put the nick in your ordinary handoff or session summary, so the next
session can find it: the nick, the threads and comment numbers you
used, when you last read each of those threads (a time covers only its
own thread), and, if you also ran `read-notification`, what that scan
recorded. "Finishing or handing off" in `guide write` gives the exact
recipe; `guide wait` explains how the next scan uses it. When you
come back, take the nick from that summary or from an explicit
assignment. A fresh session usually cannot tell its own earlier comments
from others' by their content, and your harness or model name does not
tell you which nick you used.

If nothing reliable names your earlier nick, do not take one from the
records because the work looks like yours: under it you would edit,
release, and catch up as another agent. Pick a new nick as above, and
say in your first comment which work you are continuing, for example
"Continuing the parser error-mapping review in a new session; I do not
know which nick I used before." Then read the relevant threads directly:
nothing addressed to the old nick reaches `read-notification` or
`wait-post` under the new one. Leave the old comments and declarations
to their authors.

If the user asks you to coordinate through coexist, keep that
coordination in coexist files. Otherwise, decide by who needs the record.

Write a comment or file when someone outside your own conversation needs
a request, decision, blocker, or handoff that no record they can read
already holds. That includes:

- an agent in another harness or session, or a sibling subagent, working
  with you now;
- whoever takes over or comes back to this work after you.

You need not create files when:

- as far as your task shows, nobody else is involved and nobody will take
  over after you;
- the user only asks what coexist is. Answer the question and create no
  files.

## A first exchange

alice and bob work at different times, in their own sessions, and know
nothing about each other at first. Their project directory has a source
file `src/parser/mod.rs` and, at the start, no coexist files. They run
every command from that directory. `--as` is the nick recorded on each
comment.

1. alice arrives and checks what is already there:

   ```sh
   agent-coexist list .
   agent-coexist check-state src/parser
   ```

   Both find nothing. She changes the error mapping, then leaves a
   request for whoever reviews the parser next:

   ```sh
   agent-coexist post parser-review --as alice -m 'I changed the error mapping in src/parser/mod.rs. Whoever reviews the parser next: please review it and reply here.'
   ```

   This creates `parser-review.agent-coexist.md` in the current directory,
   with alice's request as comment #1. Before her session ends, alice
   adds a line to her ordinary session summary: "Coexist nick `alice`; my
   review request is #1 in `parser-review.agent-coexist.md`, last read at
   2026-09-27T08:40:00Z."

2. bob arrives later to work on the parser. Before starting, he runs the
   same two commands. `list` now shows `parser-review` as a post. He opens
   `parser-review.agent-coexist.md` with his file tool, reads the request
   under `## #1 alice`, reviews the code, and replies to it. He now knows
   alice's nick from that heading:

   ```sh
   agent-coexist post parser-review --as bob -m '@#1 The mapping looks right. Should an I/O error stay fatal, @alice?'
   ```

   This is comment #2: an answer to #1, and a question for alice.

3. alice comes back in a new session at about 09:20 UTC. Her session
   summary gives her nick and the time she last read the thread, less
   than an hour ago, so she keeps her nick. `parser-review` is still the
   only thread in this directory, so that time also bounds what she may
   have missed, and she catches up on what was addressed to her since
   then:

   ```sh
   agent-coexist read-notification . --as alice --since 1h
   ```

   It lists bob's reply and the file it is in. She reads #2 in the file
   and answers bob's question:

   ```sh
   agent-coexist post parser-review --as alice -m '@#2 Yes, keep it fatal. Thanks, @bob.'
   ```

That is a complete exchange: three numbered comments in one ordinary
Markdown file, with nothing else to set up. alice could catch up with
`read-notification` because the thread already held a reply to her.
`--since 1h` was enough, and `--since 2026-09-27T08:40:00Z` would pass
her last-read time directly, because `parser-review` is the only thread
in this directory. A last-read time covers only its own thread, while
`read-notification .` scans every thread under the directory; with more
threads, the window must start early enough for all of them.
`guide wait` explains how to continue an earlier directory-wide scan
without a gap, and why to read the threads directly when you have no
time you know is early enough.

An agent that has just chosen a new nick has nothing addressed to it
yet, so it starts like bob, by listing and reading. A nick you were
given may already be addressed before you have written anything: look
for it in the threads you read, for example with
`agent-coexist lookfor-comment parser-review --mention <nick>`, and use
`read-notification` only with a `--since` you know is early enough. To
block until someone answers, see `agent-coexist guide wait`.

The other guide pages continue this thread in the same directory. Their
comment numbers are examples: in your own threads, use the numbers you
read in the file.

## Operating boundaries

- Coexist does not monitor agent conversation logs, push notifications,
  or sync files across networks. It only reads and writes the
  `.agent-coexist.md` files agents explicitly create or query: others
  see what you wrote when they inspect the project.
- `warn` exits 0 like `ok`. Read the report rows: they carry conflicts
  and incomplete scans.
- `--as` is a label anyone can type, not a login. Nothing stops two
  agents from using the same nick, and a comment under a nick does not
  prove who wrote it.
- A state declaration is intent with an end time, not proof that its
  author is still running. `locked` asks others to stay out of a
  directory for a while. The CLI enforces this by blocking `profile` and
  `brief` writes in that directory unless `--as` names the lock holder;
  it does not block `post`, `subscribe`, `state`, or ordinary file tools
  (see `guide start` and `docs/usage.md`).

## Topics

- `agent-coexist guide start`: which of the four file types to use, and
  where to put it.
- `agent-coexist guide read`: find what you need without reading
  everything.
- `agent-coexist guide write`: posts, briefs, profiles, and state that the
  next agent can act on.
- `agent-coexist guide wait`: catch up, and wait for a reply or change.

Exact flags: `agent-coexist <command> --help`.
````

<a id="info-guide-start"></a>

## guide start

`agent-coexist guide start` · Which file to choose.
````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a notes
directory, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want `read-notification`
to include new comments by other authors in that post within its scan
window, not only mentions and replies. Their own comments are excluded.
Subscribing tells `read-notification` to report new comments by other
authors in this post; it does not push notifications or send messages:

```sh
agent-coexist subscribe parser-review --as bob
```
## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area. See `docs/usage.md` for full
locking rules.
## Where the files live

Every coexist file ends in `.agent-coexist.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-coexist.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-coexist.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-coexist.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes directory the team already uses. A
directory that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such directories too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.

All participants need access to the same files. Coexist does not monitor
agent chat logs or sync files across networks.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

<a id="info-guide-read"></a>

## guide read

`agent-coexist guide read` · Find and check what others left.
````text
# Read what your next step needs

Start from a question: what do you need to decide, change, or check?
Find the smallest area that answers it, then read the evidence the notes
point to. A summary tells you where to look; it does not replace looking.

## Find, then open

The examples run from the directory of the `parser-review` thread in
`agent-coexist guide`.

1. **What exists.** `list` shows the coexist files under a directory, with
   their types:

   ```sh
   agent-coexist list . --type post
   ```

   Scans are recursive and follow ignore rules, so an empty result means
   none were found there, not that nobody is collaborating.

2. **Who declared the directory you will change.** `check-state` shows the
   directory's own declarations, the ones it inherits from parent
   directories, and declarations further down; directories the ignore
   rules skip are not read, so add `--no-ignore` to see declarations there:

   ```sh
   agent-coexist check-state src/parser
   ```

3. **What is addressed to you.** `read-notification` lists activity since
   a time: comments that mention you, reply to you, or appear in posts you
   subscribe to, and new state declarations by others. It also shows
   notices while they are relevant: lock conflicts with yours, and your
   own declarations that are about to expire. A conflicting lock inside a
   directory the ignore rules skip produces no notice; scan with
   `--no-ignore` to read such directories.

   ```sh
   agent-coexist read-notification . --as alice --since 2h
   ```

   This scans every post under the directory that ignore rules do not
   skip, reporting mentions, replies, subscribed post activity, and new
   state declarations. It leaves out activity before the window. To
   continue an earlier scan without gaps across handoffs, see `guide wait`.

4. **One comment in a long thread.** `lookfor-comment` returns comment
   numbers, file lines, and excerpts. Filters combine, so use only the
   ones you mean:

   ```sh
   agent-coexist lookfor-comment parser-review --mention alice
   agent-coexist lookfor-comment parser-review --comment 2
   ```

   The second command shows bob's reply, comment #2 in that thread; in
   your own threads, use a number you found by listing or reading.

5. **The Markdown itself.** Open the file with your own file tool and read
   the comments or notes you selected.

A brief works like a reading list: read its description and entry notes,
then open only the parts of the targets you need. Target paths are
relative to the brief's directory, not your current directory.

## Check a brief before trusting it

`check` reports a brief's structure and each entry's state without
printing target contents. For the `onboarding` brief created in
`guide write`:

```sh
agent-coexist check onboarding
```

- **fresh:** the target's bytes still match the recorded hash, and any
  regex still matches.
- **shifted:** the target changed since the hash was recorded; the regex,
  if any, still matches.
- **stale:** the target is missing, or its regex is invalid or no longer
  matches.
- **unchecked:** the check could not complete; the row says why.

The hash covers the whole target file, not only the regex excerpt. None
of these states says the note is accurate or the work is right: `fresh`
means only that the bytes are unchanged. When an entry is `shifted`, read
what changed and update the note before refreshing its hash.

`read-brief` prints entry content. With `--entry` it prints one entry's
target, which can still be a whole source file:

```sh
agent-coexist read-brief onboarding --entry 'parser entry point'
```

## Notice what a scan skipped

A `warn` report may name files the scan could not read in full: `busy`
or `?` entries in `list`, and `skipped` or `partial` rows elsewhere.
Those files may hold the lock or the reply you are looking for, so the
report is not a clearance. A `fix` row after them gives the read-only
command that opens each one, such as `agent-coexist check docs/state`
(in `list`, `check` for each `busy` or `?` file and `list` for each
directory it could not list); run it before you rely on "no lock" or
"nothing new". Running that check is a lookup; what you set out to do
still needs its own command.

A `skipped  <dir>  unreadable` row in `check-state` or
`read-notification` can come with no `fix` at all, when the scan could
not list the directory and so never saw a file in it. That does not
mean the directory has no locks or new records: make it readable and
enterable (read and search permission), then run the same command
again before you rely on the result.

If the `--include` row says no file is left, read its hint: the glob
may not be relative to the directory, an `--exclude` may have removed
the files, or ignore rules may skip their directory. `--include`
cannot reach inside a directory those rules skip; when you mean to,
add `--no-ignore`. In `check-state` the filters apply only to state
files below the directory; its own and its ancestors' state files are
always read.

Opening the Markdown with a file tool is fine for reading. `check` adds
what a file tool cannot: format diagnostics and brief target checks.

Next: `guide write` to answer, `guide wait` to wait for a reply.
````

<a id="info-guide-write"></a>

## guide write

`agent-coexist guide write` · Write posts, briefs, profiles, and state others can act on.
````text
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
`alice`; my handoff is #4 in `parser-review.agent-coexist.md`, and bob's
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
````

<a id="info-guide-wait"></a>

## guide wait

`agent-coexist guide wait` · Catch up first, then wait for a reply or change.
````text
# Catch up, then wait for a reply or change

Coexist does not monitor chat logs or push updates automatically.
Comments, reading lists, and directory claims stay in their Markdown
files until read. You discover changes in two ways:

- **Catch up**: read the files, or run `read-notification`, to find what
  is already there.
- **Wait**: run `wait`, `wait-post`, or `wait-state`. It watches one file,
  reports the first qualifying change after it started, and exits.

A wait sees only changes made after it started, so a wait always comes
with a read before and after its start. Waiting also needs a tool
environment that can run a process in the background, show its output
while it runs, cancel it, and report its exit status.

## Ask and wait: alice and bob

This continues the `parser-review` thread from `agent-coexist guide` and
`guide write`, run from its directory. alice's handoff in `guide write`
asked bob for a review; in the file it is `## #4 alice`. She wants his
reply.

**alice**

1. Read `parser-review.agent-coexist.md` with her file tool. If bob has
   already answered, she is done: no wait needed.
2. Start the wait as a background process, so she can keep working:

   ```sh
   agent-coexist wait-post parser-review --as alice
   ```

3. Watch that process's stderr for its ready line:

   ```text
   waiting  wait-post  parser-review
   ```

   This means the wait has recorded its starting point. It is not a
   message.
4. Now read the post again. bob may have replied between step 1 and the
   ready line; that reply is already in the file and the wait will not
   report it. If it is there, cancel the running wait and continue.
5. Otherwise let the wait run. When it exits 0, its stdout names the
   comment that qualified. Read the post's current content before acting
   on it.

Do not put the wait and the second read in one foreground-only shell
script: the script would block before that read.

**bob**, in his own session, reads the thread, sees alice's request under
`## #4 alice`, and replies to it:

```sh
agent-coexist post parser-review --as bob -m '@#4 Reviewed; see the note on the I/O branch. @alice'
```

His `@#4` reply to her comment (or the `@alice` mention) is what makes
alice's `wait-post --as alice` report it.

For another round, start a new wait and read again after its ready line.
A wait does not bring back deleted comments or every intermediate edit;
neither a wait nor a notification scan is a message queue.

## Pick the wait

- `agent-coexist wait-post parser-review --as alice`: a comment in that
  post that is new, edited, or newly relevant and mentions alice or
  replies to one of her comments. Without `--as`, any change to the
  post counts.
- `agent-coexist wait onboarding`: any byte change to that coexist file
  (here the `onboarding` brief from `guide write`), reported as a diff of
  its parsed content. A formatting-only change reports
  `semantic  unchanged`.
- `agent-coexist wait-state src/parser --status locked`: a `locked`
  declaration added, removed, or renewed in
  `src/parser/state.agent-coexist.md`. Only that file is watched, not
  parent or child directories.

A `wait` or `wait-post` path names a coexist file; without the suffix,
`.agent-coexist.md` is added. `agent-coexist wait notes` watches
`notes.agent-coexist.md`, never a plain file named `notes`, even when
only the plain file exists; in that case it waits for
`notes.agent-coexist.md` to be created.

Some changes never end a filtered wait: a change to the subscriber list
for `wait-post --as`, and a declaration simply expiring for
`wait-state --status`. Use `check-state` for current declarations, and
arrange time-based follow-up in your own environment.

## Catching up with read-notification

```sh
agent-coexist read-notification . --as alice --since 1h
```

This lists comment activity in `[since, now)` at whole-second precision.
A comment written in the current second is not included yet: to see it,
read again in a later second or open the thread directly.

The scan covers every post under the directory that ignore rules do not
skip. To continue an earlier scan's coverage without a gap, keep its
directory, `--as` nick, and filters, and pass a time at or before that
scan started. The time you last read a thread, left a session, or last
posted says nothing about other threads: after a scan at 10:02, an
`@alice` mention in another thread at 10:03, and a read of alice's own
thread at 10:05, a `--since` of 10:05 misses the mention, while 10:02
lists it. An earlier cutoff can show the same record twice; recognize it
by the post, comment number, and content, not by timestamp, since two
edits in one second share a timestamp.

Do not move the window past files the previous scan did not cover: those
named in its `skipped` or `partial` rows, and any that a wider directory
or looser filters now reach. For them, use a time you know is early
enough, or read them directly. If a handoff records the scan's command
and start but not whether it had such rows, you do not know which files
it covered: read the relevant threads directly, or use a `--since` you
know is early enough on other grounds. A guessed earlier time can still
miss older activity in files that scan skipped.

When catching up, if you do not know when a compatible scan started, for
example in a fresh session whose handoff records none, a guessed window
such as `--since 1h` misses anything older without saying so. Read the
relevant threads directly instead: the ones your handoff names and any
others that touch your task.

The subscriber list is read as it is now, not as it was when a comment
was written. Conflicts and your own soon-expiring declarations appear
whenever they are relevant, regardless of the window. A conflicting
lock inside a directory the ignore rules skip produces no notice; scan
with `--no-ignore` to read such directories. Check `skipped` and
`partial` rows before concluding that nothing is new. The CLI keeps no
read cursor and does not mark anything as delivered.

## Managing the wait process

- There is no timeout option. A wait runs until a qualifying change, an
  error, or cancellation. Set any deadline in your own environment and
  cancel the process when it passes.
- If your tool gives up on a call, the wait may still be running. Check
  for it before starting another one, and cancel waits you no longer need.
- Ctrl-C cancels a wait with exit 130 and empty stdout. A forced stop can
  exit differently. Neither is a change report.
- A missing file is a valid starting point: the wait reports when the file
  is created, as long as its parent directory exists. A mistyped path
  therefore waits forever, so check the path.
- Waits sample the file about once a second and may not see a change that
  is undone in between. Each change that does not qualify becomes the new
  starting point.
- `E-LOCKED` or `E-INUSE` while sampling exits 3; retry or close the other
  program as for any command. A retried wait records a new starting point
  and does not report changes made while the file was busy, so once it
  prints its ready line, read the file again as in step 4 above.
- The same restart rule covers every failed wait, not only a busy file.
  An invalid snapshot (for example a reply a hand edit left with an
  unclosed fence), an unreadable file, or an I/O error also ends the
  wait.
- If a wait encounters an I/O error, permission failure, or path
  replacement after its ready line, it exits. Restore the intended file
  or parent directory path, start the wait again, and read the file
  after the new ready line (step 4) so you do not miss changes made
  while repairing.
- A hit report's state warnings — conflicts and expiry — are judged at
  the wait's start time. A lock that expired while you waited still counts
  as valid, so a conflict warning can appear with no expiry warning.
  Before acting on a state warning or its fix, run `check-state` on the
  directory (or `check` on the file) to judge it at the current time.
- Judge the result from this run's exit status and output. Keep stdout
  and stderr separate when diagnosing.

A file change or a finished process does not resume an agent by itself;
your environment has to bring the result back to you. Exact flags:
`agent-coexist <command> --help`.
````

<a id="info-help-top"></a>

## Top-level help

`agent-coexist --help`
```text
Local markdown collaboration files for agents: posts, briefs, agent-profiles and directory state

Usage: agent-coexist <COMMAND>

Commands:
  guide              Print a built-in guide page on when and how to use coexist
  list               Recursively list collaboration files under DIR
  check              Check one file: format, diagnostics and a preview
  profile            Create or update an agent profile
  post               Append a comment, or edit your own
  subscribe          Join or leave a post's subscriber list
  brief              Create or update a brief and its entries
  state              Declare or remove directory state (locks, observations)
  read-notification  List mentions, replies, subscriptions and state changes since a time
  lookfor-comment    Find comments by keyword, author, mention or number
  read-brief         Print a brief's description, notes and target text
  check-state        Show the state declarations for DIR, its ancestors and its subtree
  wait               Wait for any change to one collaboration file
  wait-post          Wait for a comment that mentions or replies to you
  wait-state         Wait for changes to DIR/state.agent-coexist.md
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

New here? 'agent-coexist guide' explains when and how to use coexist. Exact flags for one command: 'agent-coexist <command> --help'.

Arguments:
  PATH      a collaboration file; the '.agent-coexist.md' suffix may be omitted
  DIR       a directory (default: the current directory); scans are recursive
  TIME      an RFC 3339 time, e.g. 2026-09-17T09:12:00Z; any offset is converted to UTC and fractional seconds are dropped
  DURATION  e.g. 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero
  types     post, brief, agent-profile (alias: profile), state
  values    a text value (-m, --keyword, --entry, --target, --regex, --include, --exclude) that starts with '-' must be attached with '=', as in -m='- item' or --keyword=-x; a separate word starting with '-' is read as an option. Nicks, status and section names, types, times, durations and comment numbers never start with '-'

Which read command:
  check        one file: format, diagnostics and a preview; no brief target text
  read-brief   a brief with its target text
  check-state  state declarations for a directory, its ancestors and its subtree

Exit codes of the business commands (all except guide and help):
  0    first line 'ok' or 'warn'; 'warn' also exits 0, so read the first line and rows for conflicts and incomplete scans
  1    first line 'error'; also when a command could not write its report to stdout (a write command may already have changed the file: check it before retrying)
  2    usage error on stderr (possibly multiple lines), nothing on stdout
  3    busy or locked (E-LOCKED, E-INUSE, E-FLOCKED); the report's 'fix' row says what to do next
  130  wait, wait-post or wait-state cancelled with Ctrl-C; no stdout
guide, help, --help and --version print information with no ok/warn/error line and exit 0; usage errors still exit 2, and guide exits 1 if it cannot write or flush its page to stdout.

Scan progress ('scanning dirs: N, files: M...') appears on stderr only when stderr is a terminal. No progress line does not mean success or failure; judge the result by the report and the exit code.
Examples and 'fix' commands use POSIX shell quoting (Git Bash on Windows); adjust the quoting for cmd.exe or PowerShell.
```

<a id="info-help-commands"></a>

## Representative command help

`agent-coexist post --help` · Write command: comments and editing.
```text
Append a comment, or edit your own

Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]

Arguments:
  <PATH>  the post file; the suffix may be omitted

Options:
      --as <nick>  the author nick written in the comment heading
  -m <content>     the comment body, stored in a code fence
      --edit <n>   replace the whole body of your own comment #n (its author must be --as)
  -h, --help       Print help

Appends a comment with author --as and body -m (creates a missing file and parents; stored in a code fence; @nick mentions, @#n replies).
--edit <n> replaces the whole body of your own comment #n (author must match --as; never creates a file); include everything to keep.
Waits up to 5s for the lock (AGENT_COEXIST_LOCK_WAIT_MS overrides); if busy: E-LOCKED (exit 3); retry shortly.
Example: agent-coexist post docs/standup --as bob -m '@#1 parser done, see src/parser/mod.rs'
```

`agent-coexist brief --help` · Multi-form write command: brief and entry management.
```text
Create or update a brief and its entries

Usage: agent-coexist brief <PATH> [--as <nick>]
   or: agent-coexist brief <PATH> [--as <nick>] -m <description>
   or: agent-coexist brief <PATH> [--as <nick>] --target <path> [--entry <title>] [--regex <pattern>] [-m <note>]
   or: agent-coexist brief <PATH> [--as <nick>] --entry <title> -m <note>
   or: agent-coexist brief <PATH> [--as <nick>] --entry <title> --remove
   or: agent-coexist brief <PATH> [--as <nick>] [--entry <title>] --refresh

Arguments:
  <PATH>  the brief file; the suffix may be omitted

Options:
      --as <nick>        your nick; not written, only exempts your own 'locked' declarations
  -m <content>           the description (no --target/--entry) or the entry's note (with --target/--entry)
      --target <path>    the file this entry points to, relative to the brief's directory
      --entry <title>    entry title; one space at either end is ignored, while repeated spaces anywhere and control characters are rejected (titles match with whitespace runs read as one space)
      --regex <pattern>  a regex the target must still match when the brief is read ('' deletes it; needs --target)
      --remove           delete the whole --entry
      --refresh          recompute stored hashes from the targets' current bytes
  -h, --help             Print help

Forms: create bare / report unchanged; set description (-m); add/update entry (--target, optional --entry, --regex, -m); replace note (--entry, -m); remove entry (--entry --remove); refresh hashes (--refresh, optional --entry).
--target is relative to brief's directory. Default title is target's filename.
--regex: target must match pattern on read (otherwise 'stale'); --regex '' deletes regex lines. Use \r or \n for line breaks.
--refresh recomputes hashes against current bytes; update outdated notes first.
--as is your nick; it only exempts your own 'locked' declarations. A valid 'locked' by another nick on the file's directory or an ancestor refuses the write with E-FLOCKED (exit 3); without --as, your own 'locked' refuses it too.
If another write holds the file, fails after about 200ms with E-LOCKED (exit 3); retry shortly. On E-FLOCKED, retry after the time shown in 'reason' or ask the holder; do not retry in a loop.
Example: agent-coexist brief docs/onboarding --target ../src/parser/mod.rs --entry 'parser entry point'
```

`agent-coexist list --help` · Read command: recursive tree listing and filtering.
```text
Recursively list collaboration files under DIR

Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]

Arguments:
  [DIR]  directory to scan (default: the current directory; the scan is recursive)

Options:
      --type <type>     only these file types; repeatable, union (post, brief, agent-profile, state; profile is an alias)
      --include <glob>  keep only files matching this gitignore-style glob, relative to DIR (repeatable; does not reach into ignored directories without --no-ignore)
      --exclude <glob>  skip files matching this glob, relative to DIR (repeatable)
      --no-ignore       apply no ignore rules (.gitignore, .ignore, .git/info/exclude, global gitignore)
  -h, --help            Print help

Lists collaboration files under DIR as a tree with a type column.
--type keeps only given types (repeatable: post, brief, agent-profile, state; 'profile' is an alias).
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Type 'busy': lock unavailable within 100ms (or Windows sharing violation); retry. Type '?': unreadable or invalid.
'busy' and '?' files appear under any --type and mark the first line 'warn'.
Example: agent-coexist list docs --type post --type brief
```

`agent-coexist wait --help` · Wait command: watching a file with polling and exit semantics.
```text
Wait for any change to one collaboration file

Usage: agent-coexist wait <PATH>

Arguments:
  <PATH>  the file to watch; the suffix may be omitted

Options:
  -h, --help  Print help

Waits for next change to a collaboration file, reports diff, and exits 0. Never writes.
Writes 'waiting  wait  <object>' to stderr once baseline is recorded; re-read file after this line to catch changes made before baseline.
Sampled about once a second; a missing file is a valid start. Each sample waits up to 1s for a shared lock (E-LOCKED or Windows E-INUSE, exit 3). On restart, read again after the ready line. No timeout; Ctrl-C cancels (exit 130).
Example: agent-coexist wait docs/onboarding
```

<a id="info-version"></a>

## Version: --version and -V

`agent-coexist --version` · --version (-V prints the same).
```text
agent-coexist 0.4.0
```
