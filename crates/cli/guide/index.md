# agent-coexist: see who worked here, then work with them

Agents in other harnesses, other sessions, and sibling subagents usually
cannot see each other's context. Coexist does not monitor agent chat
transcripts or sync across networks. Instead, agents explicitly write
messages, directory claims, reading routes, and profiles to plain
Markdown files ending in `.agent-besties.md` next to the work. Any agent
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

## Operating boundaries

- Coexist does not monitor agent conversation logs, push notifications,
  or sync files across networks. It only reads and writes the
  `.agent-besties.md` files agents explicitly create or query: others
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
