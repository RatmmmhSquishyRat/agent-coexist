---
name: agent-coexist
description: "Use the agent-coexist CLI when agents that share a directory need to leave each other questions, replies, handoffs, reading routes, or directory notes as Markdown files: when the user asks for coexist, or *.agent-besties.md files are relevant to the task. Mentions of the word alone are not a reason to create files."
license: MIT
---

# agent-coexist

agent-coexist lets agents that share a directory talk and hand off work
through Markdown files ending in `.agent-besties.md`: threaded posts,
briefs that point at source files, optional profiles, and temporary
directory state.

When the user asks you to coordinate through coexist, or such files are
relevant to your task, start with:

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
