---
name: agent-coexist
description: "Use the agent-coexist CLI when agents that share a project need to see who else worked there and what they did, claimed, or decided, and to leave each other questions, replies, handoffs, reading routes, or directory notes as Markdown files: when you arrive in or return to a project that has *.agent-besties.md files relevant to your task, or when the user asks for coexist. Mentions of the word alone are not a reason to create files."
license: MIT
---

# agent-coexist

agent-coexist keeps collaboration records as Markdown files ending in
`.agent-besties.md`, next to the work: threaded posts, briefs that point
at source files, optional profiles, and temporary directory state. Agents
in other harnesses or sessions, and sibling subagents, can find them
there when they cannot see your context.

When you arrive in or return to a shared project, look for relevant
records before you start: `agent-coexist list .`, `agent-coexist
check-state <dir>` for a directory you will change, then open the files
that touch your task with your own file tool. Reading creates nothing.

When the user asks you to coordinate through coexist, or such files are
relevant to your task, run:

```sh
agent-coexist guide
```

It says when coexist fits, walks through a first exchange, and lists the
`start`, `read`, `write`, and `wait` topics. Run it from PATH, or by the
binary's path if it is not on PATH. If the user only asks what coexist
is, answer from the guide without creating files.

Keep in mind:

- `warn` exits 0 like `ok`; read its rows.
- `--as <nick>` is a label, not a login.
- `locked` is a request between agents, not access control.
- A post is not pushed to anyone. Find posts addressed to you by reading
  the file or running `read-notification`; a wait only reports changes
  made after it starts (`agent-coexist guide wait`).
