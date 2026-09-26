# agent-coexist: use shared files to work together

Use coexist when agents need a shared place for decisions, requests,
handoffs, or directory intent—especially across tools or sessions.
Everyone must be able to reach the same files.

If the user asks you to use coexist, use its shared records for the task;
neither a description nor another chat channel replaces that request.
Otherwise, skip extra coordination files for a one-off solo task or when
existing communication is enough and no durable handoff is needed.

A relevant mention favors inspecting and reusing existing coexist files.
If the user only asks about the tool, this guide may be enough: no
workspace inspection or file creation is required.

Coexist does not start agents, sync machines, enforce permissions, or
deliver messages into an agent's context. Your environment must start
and stop processes and bring their results back to you.

## First move

If you choose to use coexist, identify the shared directory and inspect
it before writing:

```sh
agent-coexist list .
```

Run that from the intended directory, or pass its path instead. Read the
relevant existing thread or brief; use `check-state` before changing a
shared area. Create only the artifact that helps someone take the next
step—no initialization or complete set of files is required.

## Read next

- `agent-coexist guide start` — choose the right artifact and begin.
- `agent-coexist guide read` — find evidence without loading everything.
- `agent-coexist guide write` — write useful Markdown and finish cleanly.
- `agent-coexist guide wait` — catch up and manage a blocking wait.

Use `agent-coexist <command> --help` for exact syntax.

Remember: `warn` still exits 0; `fresh` means referenced bytes match,
not that a conclusion is correct; nicks are not authenticated, and
cooperative locks are not access control.
