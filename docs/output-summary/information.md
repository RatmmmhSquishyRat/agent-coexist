# Information: guides, help, and version

<a id="info-guide-pages"></a>

## guide: index

`agent-coexist guide`
````text
# agent-coexist: see who worked here, then work with them

Agents in other harnesses, other sessions, and sibling subagents usually
cannot see each other's context. Coexist keeps what they need to know
about each other in plain Markdown files ending in `.agent-besties.md`,
next to the work: who is involved, who declared which directory, what was
asked and decided, and what to read before taking over. Any agent that can
open the project can find and read these files with its own file tools.

## When you arrive

Before you start work in a shared project, or when you come back to one,
look at what is already there:

```sh
agent-coexist list .
agent-coexist check-state src/parser
```

`list` shows the coexist files below the directory, with their types; it
follows ignore rules such as `.gitignore`, so an empty result only means
none were found there. If you expect records inside an ignored folder,
run `agent-coexist list <dir> --no-ignore` for the directory you chose,
or open the files there directly. `check-state` shows who declared intent
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
them too. A nick is 1–64 characters: a Unicode letter, digit, or `_`
first, then also `-`; no dots, no spaces.

When you continue the same work, in a later session or another harness,
keep exactly that nick. Others address you by it, and
`read-notification --as`, `wait-post --as`, `post --edit` of your own
comment, and `state --remove` of your own declaration match it as typed.
So that your next session can find it, put the nick, the threads and
comment numbers you used, and when you last read each of those threads
into your ordinary handoff or session summary (see "Finishing or handing
off" in `guide write`). Those times cover only those threads. If you
also ran `read-notification`, record its command, with directory, nick,
and filters, a time at or before it started, and the paths its `skipped`
or `partial` rows named that you have not read directly since, or that
there were none; `guide wait` explains how the next scan uses them. When
you come back, take the nick from that summary or from an explicit
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

   This creates `parser-review.agent-besties.md` in the current directory,
   with alice's request as comment #1. Before her session ends, alice
   adds a line to her ordinary session summary: "Coexist nick `alice`; my
   review request is #1 in `parser-review.agent-besties.md`, last read at
   2026-09-27T08:40:00Z."

2. bob arrives later to work on the parser. Before starting, he runs the
   same two commands. `list` now shows `parser-review` as a post. He opens
   `parser-review.agent-besties.md` with his file tool, reads the request
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

## Good to know

- Coexist only reads and writes files. It does not start agents, copy
  files between machines, or put anything into another agent's context:
  others see what you wrote when they look.
- `warn` exits 0 like `ok`. Read the report rows: they carry conflicts
  and incomplete scans.
- `--as` is a label anyone can type, not a login. Nothing stops two
  agents from using the same nick, and a comment under a nick does not
  prove who wrote it.
- A state declaration is intent with an end time, not proof that its
  author is still running. `locked` asks others to stay out of a
  directory for a while; it is a request between agents, not access
  control (see `guide start`).
- Examples use a POSIX shell; Git Bash works on Windows. Other shells need
  their own quoting, and a process API can pass arguments directly.

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
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

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
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

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
   directories, and declarations further down; folders the ignore rules
   skip are not read, so add `--no-ignore` to see declarations there:

   ```sh
   agent-coexist check-state src/parser
   ```

3. **What is addressed to you.** `read-notification` lists activity since
   a time: comments that mention you, reply to you, or appear in posts you
   subscribe to, and new state declarations. It also shows notices while
   they are relevant: lock conflicts with yours, and your own
   declarations that are about to expire. A conflicting lock inside a
   folder the ignore rules skip produces no notice; scan with
   `--no-ignore` to read such folders.

   ```sh
   agent-coexist read-notification . --as alice --since 2h
   ```

   This scans every post under the directory that ignore rules do not
   skip, not only the threads you read, and leaves out activity before
   the window without warning. To continue an earlier directory-wide
   notification scan without a gap, start the window at or before that
   scan started, with the same
   directory, nick, and filters, such as the scan time in your handoff;
   the time you last read a thread covers only that thread. Without such
   a time, read the relevant threads directly instead (see `guide wait`).

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
(in `list`, `check` for each `busy` or `?` leaf and `list` for each
directory it could not list); run it before you rely on "no lock" or
"nothing new". Running that check is a lookup; what you set out to do
still needs its own command. A `skipped  <dir>  unreadable` row in
`check-state` or `read-notification` can come with no `fix` at all,
when the scan could not list the directory and so never saw a file in
it. That does not mean the directory has no locks or new records: make
it readable and enterable (read and search permission), then run the
same command again before you rely on the result. If the `--include`
row says no file is left, read its hint: the glob may not be relative
to the directory, an `--exclude` may have removed the files, or ignore
rules may skip their directory.
`--include` cannot reach into such a directory; use `--no-ignore` when
you mean to, for a directory you chose. In `check-state` the filters
apply only to state files below the directory; its own and its
ancestors' state files are always read.

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
it. `observing` says changes are welcome. Use `locked` when you need
others to stay out while you edit:

```sh
agent-coexist state src/parser --as alice --status locked --for 30m
```

If someone else's `locked` already overlaps yours, the report shows
`conflict` rows with a `warn` first line and exit 0; talk to them in a
post. The conflict check after the write reads the state files of that
directory and its ancestors and the state files below it that ignore
rules do not skip; `state` has no `--no-ignore`, so to see locks inside
ignored folders run `check-state <dir> --no-ignore`. Do not remove
another nick's declaration to get past it. If the report instead shows
`skipped` or `partial` rows and a `fix`, your
declaration is written but some state file was not read, so a lock
there may have been missed. Do not declare again: do what the `fix`
says. It may first ask you to make a directory it could not read both
readable and enterable; the read-only command it then names is `check`
on a file or `check-state` on a directory. Run that command and talk to
any holder it shows. A `profile` or `brief` write can end the same way.

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
time and those paths. The comments alone do
not say which nick was yours. A session that finds no such note and has
no assigned nick takes a new one, says which work it continues, and reads
the threads directly, as "When you arrive" in `agent-coexist guide`
describes; it does not adopt a nick from the records.

After an interruption, look at the actual files and state before
renewing or cleaning up: an expired declaration only ran out of time,
and reading never cleans files.

## When a write reports a problem

Most report rows give a problem's message, or a `reason` and a `fix`,
with no ID field; only a few, such as a scan's `skipped` or `partial`
rows, have one. Explanatory text may still mention some other ID, so an
`E-` or `W-` name in the prose does not identify the problem. Match the
message you see against the quoted text below; the IDs are the names
the command reference (`docs/usage.md` in the agent-coexist repository)
uses.

- `warn` exits 0. Read its rows before moving on.
- `file is locked by another process` (`E-LOCKED`): the CLI could not
  get the file lock or a stable read in time; retry the same command
  shortly.
- `file is in use by another program` (`E-INUSE`, Windows only): another
  program opened the file without allowing shared access. Having it open
  in an editor does not cause this unless that program denies sharing.
  Close that program's handle and retry.
- `<dir> is locked by <holder> until <time>` (`E-FLOCKED`): a `locked`
  declaration covers this `profile` or `brief` write. If you left out
  `--as`, even a lock under your own nick can block it. Wait until the
  reported time or coordinate; do not retry in a tight loop.
- A diagnostic's `fix` line is a command, an instruction such as a hand
  edit (a heading to copy), or an instruction, a colon, and a command.
  Read it before acting, and carry out instructions yourself. The command
  part starts at the program name. In the combined form the words before
  the colon say what the command is for. Common cases, not a complete
  list:
  - Repair, then retry, as in `retry shortly: agent-coexist post
    parser-review …` or `correct line 1 of room/.ignore, then retry: …`.
    Do the instruction first (wait, fix the file), then run the command;
    it repeats your original command, including the program path you
    typed.
  - Look up, then correct, as in `pick an existing title from:
    agent-coexist check onboarding.agent-besties.md` or `find the number
    of the comment to edit, then rerun --edit with it: agent-coexist
    lookfor-comment …`. Run the command first; it only reads. Use its
    output to correct your original command, then run that. Its `ok`
    is only the lookup's result: your original command still has to run.
  - Check after a write that already happened, as in `correct line 1 of
    room/.ignore, then check for conflicts without declaring again:
    agent-coexist check-state room`. Do the repair, then run the check;
    do not repeat the write.
  - Do something else instead, as in `reply instead: agent-coexist post
    … -m '@#3 <your reply>'` or `start a new post: …`. The command
    replaces your original one and writes something else; run it in
    place of the original, not after it.
  - A command for a named agent, as in `if still needed, that agent
    renews it: …`, `the command for carol: …`, or `agree who continues,
    then the other runs the removal; …`. Run it only if you are that agent, under
    your own nick; otherwise tell that agent in a post. Typing someone
    else's nick into `--as`, or into `--remove`, where `--as` defaults to
    the removed nick, changes their records as if they had done it. On a
    lock conflict, agree in a post who yields; only the agent that yields
    runs `--remove` with its own nick.

  When the words before the colon fit none of these, read them as a
  description of the command and any condition for running it; if they
  name another agent or leave you unsure, ask in a post instead of
  running it.
- Angle-bracket words that the CLI prints as markers, such as `<file>`,
  `<nick>`, or `<text>`, stand for a value you choose. Fill them in
  before running a command, even when quoted and even when the CLI knows
  a current value: `--target <file>` on a stale entry asks which file the
  entry should point at now. A value taken from your files is real even
  if it looks like a marker: for an entry titled `<file>`, the
  `--entry='<file>'` in its fix names that entry and stays as printed.
  Compare with the title, nick, or path the report shows to tell the two
  apart.

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

Whenever you write a comment header by hand, copy the shape of the
entries already in the file or the reference. A new comment goes at the
end with the next number (highest existing plus one), your own nick, and
the current UTC time, such as `## #6 bob (2026-09-27T10:15:00Z)`, then
its body in an `md` fence. Open the fence with at least three backticks
and more than any run of backticks in the body, and close it with a line
of the same number; a body that quotes a three-backtick block needs a
four-backtick fence, so do not reuse the other comments' fence without
counting. A fence that closes too early cuts the body short and can
leave an unclosed fence, which makes CLI writes refuse the file. When
you edit your own comment, keep its created time and add or update
`, edited <current UTC time>` before the `)`. Take the time from your
environment's clock or from the user, never from another comment:
`read-notification` finds comments by these times. Do not guess a brief
entry's hash either; the reference gives its rules. If you cannot get
the current time or are unsure of the format, leave the file unchanged
and say what you would have written. Without the CLI nothing checks such
an edit, so re-read the file afterwards.

Exact flags: `agent-coexist <command> --help`. For waiting on replies:
`guide wait`.
````

<a id="info-guide-wait"></a>

## guide wait

`agent-coexist guide wait` · Catch up first, then wait for a reply or change.
````text
# Catch up, then wait for a reply or change

Nothing is pushed into an agent's context. Comments stay in their files,
and reading them removes nothing. You learn what changed in two ways:

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

1. Read `parser-review.agent-besties.md` with her file tool. If bob has
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
  post that mentions alice or replies to one of her comments is new,
  edited, or newly relevant. Without `--as`, any change to the post counts.
- `agent-coexist wait onboarding`: any byte change to that coexist file
  (here the `onboarding` brief from `guide write`), reported as a diff of
  its parsed content. A formatting-only change reports
  `semantic  unchanged`.
- `agent-coexist wait-state src/parser --status locked`: a `locked`
  declaration added, removed, or renewed in
  `src/parser/state.agent-besties.md`. Only that file is watched, not
  parent or child directories.

A `wait` or `wait-post` path names a coexist file; without the suffix,
`.agent-besties.md` is added. `agent-coexist wait notes` watches
`notes.agent-besties.md`, never a plain file named `notes`, even when
only the plain file exists; in that case it waits for
`notes.agent-besties.md` to be created.

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
scan started. The time you
last read a thread, left a session, or last posted says nothing about
other threads: after a scan at 10:02, an
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
lock inside a folder the ignore rules skip produces no notice; scan
with `--no-ignore` to read such folders. Check `skipped` and `partial`
rows before concluding that nothing is new. The CLI keeps no
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
  and does not report changes made while the file was busy, so after its
  ready line, read the file again as in step 4 above.
- The same restart rule covers every failed wait, not only a busy file.
  An invalid snapshot (for example a reply a hand edit left with an
  unclosed fence), an unreadable file, or an I/O error also ends the
  wait. If that I/O error is after the ready line and the intended
  path no longer resolves (its parent missing, renamed away or taken
  by a file) or, on Linux and macOS, the path itself is no longer a
  regular file, the `fix` says how to restore that path (recreate or
  move it back; move aside whatever replaced it, keeping it) and then
  prints the original wait as the command to retry; a path that was
  wrong from the start still says to correct it, with no retry. On
  Windows a watched file replaced by a directory after the ready line
  is reported as access denied and gets the generic remedy — check
  permissions, or wait until another program stops using, deleting or
  replacing the file — which cannot tell a replacement from a busy
  file; once you establish that a directory now occupies the path,
  move it aside, keep it, start the original wait again, and read the
  file after its ready line. Once you fix the cause and start the wait
  again, its new starting point already contains everything then in the file,
  including the change you were waiting for, so read the file again
  after the new ready line as in step 4 above.
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

Use `<command> --help`, `<command> -h`, or `help <command>`; `help` itself is the exception shown below.

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
  wait-state         Wait for changes to DIR/state.agent-besties.md
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

New here? 'agent-coexist guide' explains when and how to use coexist. Exact flags for one command: 'agent-coexist <command> --help'.

Arguments:
  PATH      a collaboration file; the '.agent-besties.md' suffix may be omitted
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
  2    usage error: one 'error: ...' line on stderr, nothing on stdout
  3    busy or locked (E-LOCKED, E-INUSE, E-FLOCKED); the report's 'fix' row says what to do next
  130  wait, wait-post or wait-state cancelled with Ctrl-C; no stdout
guide, help, --help and --version print information with no ok/warn/error line and exit 0; usage errors still exit 2, and a failed guide write exits 1.

Scan progress ('scanning dirs: N, files: M...') appears on stderr only when stderr is a terminal. No progress line does not mean success or failure; judge the result by the report and the exit code.
Examples and 'fix' commands use POSIX shell quoting (Git Bash on Windows); adjust the quoting for cmd.exe or PowerShell.
```

<a id="info-help-write"></a>

## Help for write commands

`agent-coexist profile --help`
```text
Create or update an agent profile

Usage: agent-coexist profile <PATH> [--as <nick>] [--section <name> (-m <content> | --remove)]

Arguments:
  <PATH>  the agent-profile file; the suffix may be omitted

Options:
      --as <nick>       your nick; not written, only exempts your own 'locked' declarations
      --section <name>  the section to set or remove
  -m <content>          the section body, stored verbatim in a code fence
      --remove          delete the whole --section
  -h, --help            Print help

--section <name> -m <content> sets that section's body, stored verbatim in a code fence; --section <name> --remove deletes the section. Without --section, creates the file (heading only) or reports 'unchanged'. Content that starts with '-' must be attached: -m='- item'.
--as is your nick. It is not written; it only exempts your own 'locked' declarations. A valid 'locked' by another nick on the file's directory or an ancestor refuses the write with E-FLOCKED (exit 3); without --as, your own 'locked' refuses it too.
If another write holds the file, fails after about 200ms with E-LOCKED (exit 3); retry shortly. On E-FLOCKED, retry after the time shown in 'reason' or ask the holder; do not retry in a loop.
Example: agent-coexist profile agents/alice --section role -m 'Rust implementer. Owns crates/core.'
```

`agent-coexist post --help`
```text
Append a comment, or edit your own

Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]

Arguments:
  <PATH>  the post file; the suffix may be omitted

Options:
      --as <nick>  the author nick written in the comment heading
  -m <content>     the comment body, stored verbatim in a code fence
      --edit <n>   replace the whole body of your own comment #n (its author must be --as)
  -h, --help       Print help

-m is the comment body, stored verbatim in a code fence. In it, @nick mentions someone and @#n replies to comment n. A body that starts with '-', such as a Markdown list, must be attached: -m='- item'. --as is the author nick written in the comment heading.
--edit <n> replaces the whole body of your own comment #n (its author must be --as), so resend everything you want to keep.
The report's 'comment' row names the comment written.
Waits up to 5s for the file lock (AGENT_COEXIST_LOCK_WAIT_MS overrides); if still busy: E-LOCKED, exit 3; retry the same command shortly.
Example: agent-coexist post docs/standup --as bob -m '@#1 parser done, see src/parser/mod.rs'
```

`agent-coexist subscribe --help`
```text
Join or leave a post's subscriber list

Usage: agent-coexist subscribe <PATH> --as <nick> [--cancel]

Arguments:
  <PATH>  the post file; the suffix may be omitted

Options:
      --as <nick>  the nick added to (or, with --cancel, removed from) the list
      --cancel     remove --as from the subscriber list instead of adding it
  -h, --help       Print help

Adds --as to the post's subscriber list, so read-notification --as <nick> lists every new comment in that post. Nothing is pushed to subscribers, and wait-post does not use the list.
--cancel removes --as from the list. Subscribing twice, or cancelling a nick that is not subscribed, changes nothing ('unchanged <nick>').
Locking is as for post: waits up to 5s (AGENT_COEXIST_LOCK_WAIT_MS overrides), then E-LOCKED, exit 3.
Example: agent-coexist subscribe docs/standup --as alice
```

`agent-coexist brief --help`
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
      --entry <title>    the entry title; one space at either end is ignored, while two spaces in a row anywhere (including at an end) or a control character such as a tab or CR is rejected (titles in the file match with whitespace runs read as one space)
      --regex <pattern>  a regex the target must still match when the brief is read ('' deletes it; needs --target)
      --remove           delete the whole --entry
      --refresh          recompute stored hashes from the targets' current bytes
  -h, --help             Print help

The usage forms, in order: create the brief (or report 'unchanged'); set the description; add or update an entry for a target; replace one entry's note; remove an entry; refresh hashes (every entry, or only --entry). Other combinations are rejected.
-m is the description without --target/--entry, and the entry's note with them. Text that starts with '-' must be attached: -m='- item', and likewise --regex='-x'.
--target is relative to the brief's directory, not the current directory. Without --entry, the title is the target's file name.
--regex: when the brief is checked or read, the target must still match it, or the entry is 'stale'; writing only checks that it compiles. --regex '' deletes the entry's regex lines. Use \r or \n in a pattern to match line breaks; --regex and --target cannot contain raw line breaks.
--refresh accepts the targets' current bytes: read what changed and update the note first.
--as is your nick. It is not written; it only exempts your own 'locked' declarations. A valid 'locked' by another nick on the file's directory or an ancestor refuses the write with E-FLOCKED (exit 3); without --as, your own 'locked' refuses it too.
If another write holds the file, fails after about 200ms with E-LOCKED (exit 3); retry shortly. On E-FLOCKED, retry after the time shown in 'reason' or ask the holder; do not retry in a loop.
Example: agent-coexist brief docs/onboarding --target ../src/parser/mod.rs --entry 'parser entry point'
```

`agent-coexist state --help`
```text
Declare or remove directory state (locks, observations)

Usage: agent-coexist state [DIR] --as <nick> --status <name> [--for <duration>]
   or: agent-coexist state [DIR] --remove <nick> [--as <nick>] [--status <name>]

Arguments:
  [DIR]  the directory (default: the current directory); must already exist

Options:
      --as <nick>       declare form: your nick; remove form: who removes (default: the removed nick)
      --status <name>   the status name; required in the declare form; in the remove form, a filter (omitted: every status)
      --for <duration>  how long the declaration lasts (declare form only; default 2h for locked, 8h otherwise)
      --remove <nick>   remove this nick's declarations instead of declaring
  -h, --help            Print help

Declare: --as <nick> --status <name> records that <nick> holds <name> on DIR and everything below it until the end time. Declaring again replaces your entries under that status with one from now, except an entry on a line that also has unparseable fragments (W-STATE-ENTRY): that line is kept, and the latest end time still wins; check-state shows the result.
Remove: --remove <nick> removes that nick's declarations in DIR. Add --status to remove only that status; without it every status goes, including a live 'locked'.
Built-in statuses: 'locked' = please do not change this directory or its subdirectories until the end time; 'observing' = I am watching, changes are welcome. Other names are stored and shown but have no built-in meaning.
--for defaults to 2h for 'locked' and 8h otherwise. In the remove form --as defaults to the removed nick; removing another nick's declaration adds a 'removed-other' row and a 'warn' first line.
DIR must already exist; it is never created.
'locked' is a request between agents, not access control, and another nick's 'locked' never blocks a state write. Locks overlapping yours are listed as 'conflict' rows with a 'warn' first line (exit 0); talk to the holder in a post.
The conflict check reads the state files of DIR and its ancestors and the state files below DIR that ignore rules do not skip; state has no --no-ignore, so to see locks inside ignored directories run check-state DIR --no-ignore.
If another write holds the state file, fails after about 200ms with E-LOCKED (exit 3); retry shortly.
Example: agent-coexist state src --as alice --status locked --for 1h30m
```

<a id="info-help-read"></a>

## Help for read commands

`agent-coexist list --help`
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

Lists the collaboration files under DIR as a tree with a type column.
--type keeps only the given types (repeatable: post, brief, agent-profile, state; 'profile' is an alias).
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Type column 'busy': the file lock or a stable read was not available within 100ms; retry. On Windows it can also mean another program holds the file open without sharing; close that program, then retry.
Type column '?': unreadable (permissions, not UTF-8, invalid H1, or over 16 MiB).
'busy' and '?' files still show under any --type and make the first line 'warn'; the command does not fail on them.
Example: agent-coexist list docs --type post --type brief
```

`agent-coexist check --help`
```text
Check one file: format, diagnostics and a preview

Usage: agent-coexist check <PATH>

Arguments:
  <PATH>  the collaboration file; the '.agent-besties.md' suffix may be omitted

Options:
  -h, --help  Print help

Reports one file: its 'file' and 'type' rows, diagnostics in line order, then a preview (topic and latest comment for posts, entry states for briefs, sections for profiles, declarations for state).
Brief targets are checked but not printed; use read-brief for their text. For every declaration that applies to a directory, use check-state.
Waits up to 1s for a shared lock; if the file stays busy: E-LOCKED, exit 3; retry the same command shortly.
Example: agent-coexist check docs/standup
```

`agent-coexist lookfor-comment --help`
```text
Find comments by keyword, author, mention or number

Usage: agent-coexist lookfor-comment <PATH> [--keyword <text>] [--by <nick>] [--mention <nick>] [--reply-to <n>] [--comment <n>]

Arguments:
  <PATH>  the post file; the suffix may be omitted

Options:
      --keyword <text>  case-insensitive text in the comment body or heading, matched as typed
      --by <nick>       the comment author
      --mention <nick>  the body mentions @nick
      --reply-to <n>    the body replies to comment n (@#n)
      --comment <n>     the comment numbered n
  -h, --help            Print help

Lists matching comments with line, number, author, time and an excerpt. Without filters it lists every comment; with several, a comment must match all of them.
--keyword is a case-insensitive substring of the raw body or heading line, matched exactly as typed (spaces are not folded).
--by is the comment's author (check lists a post's 'authors'). --mention matches @nick in the body, --reply-to matches @#n in the body, and --comment selects comment #n.
Waits up to 1s for a shared lock; if still busy: E-LOCKED, exit 3; retry shortly.
Example: agent-coexist lookfor-comment docs/standup --by bob
```

`agent-coexist read-brief --help`
```text
Print a brief's description, notes and target text

Usage: agent-coexist read-brief <PATH> [--entry <title>]

Arguments:
  <PATH>  the brief file; the suffix may be omitted

Options:
      --entry <title>  print only this entry; one space at either end is ignored, while two spaces in a row anywhere (including at an end) or a control character such as a tab or CR is rejected (titles in the file match with whitespace runs read as one space)
  -h, --help           Print help

Prints the description, then each entry's state, note and target text. With a regex, only the lines of its first match are printed; without one, the whole target.
An entry with no path, a missing or unreadable target, a failing regex, or a target over 64 MiB prints no text; its row says why.
--entry prints only that entry. For entry states without target text, use check.
Waits up to 1s for a shared lock; if still busy: E-LOCKED, exit 3; retry shortly.
Example: agent-coexist read-brief docs/onboarding --entry 'parser entry point'
```

`agent-coexist read-notification --help`
```text
List mentions, replies, subscriptions and state changes since a time

Usage: agent-coexist read-notification [DIR] --as <nick> --since <time|duration> [--include <glob>]... [--exclude <glob>]... [--no-ignore]

Arguments:
  [DIR]  directory to scan (default: the current directory; recursive, plus ancestor state files)

Options:
      --as <nick>              your nick: whose mentions, replies and subscriptions to list
      --since <time|duration>  RFC 3339 time or a duration back from now
      --include <glob>         keep only files matching this gitignore-style glob, relative to DIR (repeatable; does not reach into ignored directories without --no-ignore)
      --exclude <glob>         skip files matching this glob, relative to DIR (repeatable)
      --no-ignore              apply no ignore rules (.gitignore, .ignore, .git/info/exclude, global gitignore)
  -h, --help                   Print help

Lists, oldest first, what happened since --since that concerns the --as nick: comments that mention it, reply to its comments, or appear in posts it subscribes to; state declarations by others; locks that overlap its own; its own declarations that end soon or have ended.
Scans every post and state file under DIR that ignore rules do not skip (default: the current directory) and also reads ancestor state files. It never writes files and keeps no read cursor; for files inside ignored directories add --no-ignore.
--since is an RFC 3339 time (e.g. 2026-09-17T09:12:00Z) or a duration back from now (e.g. 1h30m). The window ends before the current second, so a comment written this second shows up in a later scan.
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Each file waits at most 100ms for a shared lock. Busy or unparseable files become 'skipped' or 'partial' rows with a 'warn' first line; read them directly before concluding nothing is new.
Example: agent-coexist read-notification --as alice --since 1h
```

`agent-coexist check-state --help`
```text
Show the state declarations for DIR, its ancestors and its subtree

Usage: agent-coexist check-state [DIR] [--include <glob>]... [--exclude <glob>]... [--no-ignore]

Arguments:
  [DIR]  directory to scan (default: the current directory; recursive, plus ancestors)

Options:
      --include <glob>  keep only files matching this gitignore-style glob, relative to DIR (repeatable; does not reach into ignored directories without --no-ignore)
      --exclude <glob>  skip files matching this glob, relative to DIR (repeatable)
      --no-ignore       apply no ignore rules (.gitignore, .ignore, .git/info/exclude, global gitignore)
  -h, --help            Print help

Shows the declarations that apply to DIR: 'inherited' rows from ancestor directories (valid ones only), DIR's own declarations, then one group per state file below DIR that ignore rules do not skip (add --no-ignore to see declarations in skipped directories).
A row ends with 'conflict' when it is a valid 'locked' and another nick holds a valid 'locked' shown in this same report on an overlapping directory (the same, an ancestor or a descendant; both rows are marked), or with 'expired' when a declaration in DIR or below has ended. Either mark makes the first line 'warn' (exit 0). Only this report's declarations are compared: an inherited lock can still conflict with a lock in another branch below that ancestor; check-state on the ancestor shows it.
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Each state file waits at most 100ms for a shared lock. Busy or unparseable ones become 'skipped' or 'partial' rows with a 'warn' first line.
For one state file with its diagnostics, use check <DIR>/state.
Example: agent-coexist check-state src
```

<a id="info-help-wait"></a>

## Help for wait commands

`agent-coexist wait --help`
```text
Wait for any change to one collaboration file

Usage: agent-coexist wait <PATH>

Arguments:
  <PATH>  the file to watch; the suffix may be omitted

Options:
  -h, --help  Print help

Waits for the next change to this collaboration file (any type), prints one report of what changed, and exits 0. It never writes.
Once it has recorded its starting point, it writes one line 'waiting  wait  <object>' to stderr. A change made before that line is not reported, so read the file again after the line appears.
The file is sampled about once a second. A missing file is a valid start: the wait reports its creation (its directory must exist), so a mistyped path waits forever.
Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3); on Windows, a program holding the file open without sharing ends it with E-INUSE (exit 3). However a wait ends — a busy file, an unreadable file, or an invalid sample — a restarted wait starts from a new baseline: read the file again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait docs/onboarding
```

`agent-coexist wait-post --help`
```text
Wait for a comment that mentions or replies to you

Usage: agent-coexist wait-post <PATH> [--as <nick>]

Arguments:
  <PATH>  the post file to watch; the suffix may be omitted

Options:
      --as <nick>  the nick whose mentions and replies end the wait; without it, any change counts
  -h, --help       Print help

Waits for a comment in this post that mentions @<nick> or replies to a comment by <nick> and is new, edited or newly relevant; prints those comments and exits 0. With --as, subscriber changes and deleted comments do not count; without --as, any change to the post counts.
Once it has recorded its starting point, it writes one line 'waiting  wait-post  <object>' to stderr. A comment written before that line is not reported, so read the post again after the line appears.
The post is sampled about once a second; a missing post is a valid start. Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3), or on Windows E-INUSE (exit 3). However a wait ends — a busy post, an unreadable post, or an invalid sample — a restarted wait starts from a new baseline: read the post again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait-post docs/standup --as alice
```

`agent-coexist wait-state --help`
```text
Wait for changes to DIR/state.agent-besties.md

Usage: agent-coexist wait-state [DIR] [--status <name>]

Arguments:
  [DIR]  the directory (default: the current directory); only its state file is watched

Options:
      --status <name>  count only changes to declarations under this status ('locked', 'observing' or another name)
  -h, --help           Print help

Watches only DIR/state.agent-besties.md (DIR defaults to the current directory), not ancestors or subdirectories.
--status (locked, observing or another status name) waits for a declaration under that status to be added, removed or renewed; without it, any change to the file counts. A declaration that simply expires does not count; use check-state for current declarations.
Once it has recorded its starting point, it writes one line 'waiting  wait-state  <object>' to stderr. A change made before that line is not reported, so run check-state again after the line appears.
The file is sampled about once a second; a missing state file is a valid start. Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3), or on Windows E-INUSE (exit 3). However a wait ends — a busy file, an unreadable file, or an invalid sample — a restarted wait starts from a new baseline: run check-state on the directory again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait-state src --status locked
```

<a id="info-help-guide-and-help"></a>

## Help for guide and help

`agent-coexist guide --help`
```text
Print a built-in guide page on when and how to use coexist

Usage: agent-coexist guide [start|read|write|wait]

Arguments:
  [start|read|write|wait]  start, read, write or wait; omit for the index page

Options:
  -h, --help  Print help

Prints one built-in Markdown page to stdout. Without a topic it prints the index page: what to do when you arrive in a project, and a first exchange between two agents.
Topics: start (which file type to use and where), read (find and check what others left), write (posts, briefs, profiles and state others can act on), wait (catch up, then wait for a reply or change).
Topics are exact lowercase words. Pages are static text; the command does not read your project.
Example: agent-coexist guide start
```

`agent-coexist help help`
```text
Print this message or the help of the given subcommand(s)

Usage: agent-coexist help [COMMAND]...

Arguments:
  [COMMAND]...  Print help for the subcommand(s)
```

`agent-coexist help --help` · stderr · exit code 2 · help --help is treated as an unknown subcommand (same for -h).
```text
error: unrecognized subcommand '--help'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

<a id="info-version"></a>

## Version: --version and -V

`agent-coexist --version` · --version (-V prints the same).
```text
agent-coexist 0.1.0
```
