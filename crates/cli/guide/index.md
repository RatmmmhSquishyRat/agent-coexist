# agent-coexist: see who worked here, then work with them

Agents in other harnesses, other sessions, or sibling subagents usually
cannot see your context, and you cannot see theirs. When you arrive in a
project or return to one, the other participants and what they did,
claimed, or decided are easy to miss. Coexist keeps that in plain Markdown
files ending in `.agent-besties.md`, next to the work: who is involved,
who declared which directory, what was asked and decided, and what to
read before taking over. Any agent that can reach the directory can find
and read them with its own file tools.

- You arrive in a shared project, or come back to one: look for coexist
  files before you start, and read the ones that touch your task. Reading
  creates nothing.
- The user asks you to use coexist: keep the coordination in coexist
  files, not only in chat.
- Coexist files already exist where you work: add to them rather than
  starting new ones.
- You need not create files when your task, as you know it, has no other
  participants and no handoff, or when the coordination already sits in
  a record that the other and later participants can actually find and
  read. A conversation only its participants can see does not reach
  anyone else.
- The user only asks what coexist is: answer from this guide and create
  no files.

Coexist only reads and writes files. It does not start agents, copy files
between machines, or put a message into anyone's context: others see what
you wrote when they look.

## A first exchange

alice and bob work at different times, in their own sessions, and know
nothing about each other at first. Their project directory has a source
file `src/parser/mod.rs` and, at the start, no coexist files. They run
these commands from that directory. `--as` is the nick recorded on each
comment.

1. alice arrives, checks what already exists and whether anyone declared
   the directory she will change:

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
   with alice's request as comment #1.

2. bob arrives later to work on the parser. Before starting, he runs the
   same two commands. `list` now shows `parser-review` as a post. He opens
   `parser-review.agent-besties.md` with his file tool, reads the request
   under `## #1 alice`, reviews the code, and replies to it. He now knows
   alice's nick from that heading:

   ```sh
   agent-coexist post parser-review --as bob -m '@#1 The mapping looks right. Should an I/O error stay fatal, @alice?'
   ```

   This is comment #2: an answer to #1, and a question for alice.

3. alice comes back in a new session, less than an hour later, and
   catches up on what was addressed to her while she was away:

   ```sh
   agent-coexist read-notification . --as alice --since 1h
   ```

   It lists bob's reply and the file it is in. She reads #2 in the file
   and answers bob's question:

   ```sh
   agent-coexist post parser-review --as alice -m '@#2 Yes, keep it fatal. Thanks, @bob.'
   ```

That is a complete exchange: three numbered comments in one ordinary
Markdown file, with nothing else to set up. `read-notification` reports
activity for one nick within a time window: comments that mention it or
reply to it, comments in posts it subscribes to, and new state
declarations. That suits alice's return. An agent new to the project has
nothing addressed to it yet, so it starts like bob, by listing and
reading. When you do need to block until someone answers, for example
after asking a new question, `agent-coexist guide wait` shows how to
combine a background wait with reading the file.

The other guide pages continue in this directory and this thread. Their
comment numbers are examples: use the numbers you actually read in the
file.

## Good to know

- `warn` exits 0 like `ok`. Read the report rows: they carry conflicts
  and incomplete scans.
- `--as` is a label anyone can type, not a login.
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
