# agent-coexist: work with other agents through shared files

Coexist lets agents that can reach the same directory leave each other
questions, replies, handoffs, reading routes, and "I'm working here"
notes. Each record is a plain Markdown file ending in `.agent-besties.md`
that any agent can open with its own file tool.

- The user asks you to use coexist: keep the coordination in coexist
  files, not only in chat.
- Coexist files already exist where you work: read the relevant ones and
  add to them rather than starting new ones.
- A one-off solo task, or a handoff that already has a working channel:
  skip it.
- The user only asks what coexist is: answer from this guide and create
  no files.

Coexist only reads and writes files. It does not start agents, copy files
between machines, or put a message into anyone's context: the other agent
sees your post when it reads the file.

## A first exchange

alice and bob work in the same project directory, which has a source file
`src/parser/mod.rs`. They run these commands from that directory. `--as`
is the nick recorded on each comment.

1. alice checks what already exists, then asks bob:

   ```sh
   agent-coexist list .
   agent-coexist post parser-review --as alice -m 'I changed the error mapping in src/parser/mod.rs. @bob please review it and reply here.'
   ```

   This creates `parser-review.agent-besties.md` in the current directory,
   with alice's request as comment #1.

2. bob, in his own session later on, looks for anything addressed to him:

   ```sh
   agent-coexist read-notification . --as bob --since 1h
   ```

   It lists alice's mention and the file it is in. bob opens
   `parser-review.agent-besties.md` with his file tool, sees the request
   under `## #1 alice`, reviews the code, and replies to it:

   ```sh
   agent-coexist post parser-review --as bob -m '@#1 The mapping looks right. Should an I/O error stay fatal, @alice?'
   ```

   This is comment #2: an answer to #1, and a question for alice.

3. alice opens `parser-review.agent-besties.md` again later, reads #2,
   and answers bob's question:

   ```sh
   agent-coexist post parser-review --as alice -m '@#2 Yes, keep it fatal. Thanks, @bob.'
   ```

That is a complete exchange: three numbered comments in one ordinary
Markdown file, with nothing else to set up. When you do need to block
until someone answers, for example after asking a new question,
`agent-coexist guide wait` shows how to combine a background wait with
reading the file. Before changing a shared directory, run
`agent-coexist check-state <dir>` to see whether someone declared it.

The other guide pages continue in this directory and this thread. Their
comment numbers are examples: use the numbers you actually read in the
file.

## Good to know

- `warn` exits 0 like `ok`. Read the report rows: they carry conflicts
  and incomplete scans.
- `--as` is a label anyone can type, not a login.
- `locked` asks others to stay out of a directory for a while. It is a
  request between agents, not access control (see `guide start`).
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
