---
name: agent-coexist
description: "Use agent-coexist records, with the agent-coexist CLI or with plain file tools, when agents that share a project need to see who else worked there and what they did, claimed, or decided, and to leave each other questions, replies, handoffs, reading routes, or directory notes as Markdown files: when you arrive in or return to a project that other agents may share, to check for *.agent-besties.md records before you start, or when the user asks for coexist. Mentions of the word alone are not a reason to create files."
license: MIT
---

# agent-coexist

agent-coexist keeps collaboration records as Markdown files ending in
`.agent-besties.md`, next to the work. Agents in other harnesses or
sessions, and sibling subagents, find them there when they cannot see
your context.

When you arrive in or return to a shared project, look before you start.
List the records, and check who declared the directory you will change
(`<dir>`):

```sh
agent-coexist list .
agent-coexist check-state <dir>
```

`list` follows ignore rules such as `.gitignore`, so empty output only
means none were found there, not that no records or earlier work exist.
Records in ignored directories may need `--no-ignore` or a direct look
at the file.

Open the files that touch your task with your own file tool. Reading
creates nothing.

If the `agent-coexist` command is not available, catching up still works:
find `*.agent-besties.md` files with your file tools, including
`state.agent-besties.md` in the directories you will change and their
parents, and read the relevant ones as plain Markdown. Loading this skill
is not a reason to install anything; install the CLI only if your task
or the user calls for it. It is published on crates.io: install it with
`cargo install agent-coexist --locked`, or follow Install in its
[README](https://github.com/RatmmmhSquishyRat/agent-coexist#install) for
prebuilt binaries.

Before you write, or when the user asks you to coordinate through
coexist, read the guide. With the CLI, run it from PATH, or by the
binary's path if it is not on PATH:

```sh
agent-coexist guide
```

It shows when to write, walks through a first exchange between two
agents, and lists the `start`, `read`, `write`, and `wait` topics.
Without the CLI, the same pages are in the repository's
[src/guide/](https://github.com/RatmmmhSquishyRat/agent-coexist/blob/main/src/guide/index.md)
if you can open them. If the user only asks what coexist is, answer from
the guide or this skill and create no files.

Write under one nick. Use the one the user or the agent that started you
gave, and look for `@` mentions of it in the threads you read, since
others may have addressed it already. Otherwise choose a distinctive
project nick that no record you read already uses for an agent, whether
that agent wrote, subscribed, declared state, was `@`-mentioned, or is
described in a profile (a profile's file name is not tied to `--as`).
Avoid bare harness, model, or role names, which may also be someone
else's. A nick is 1–64 characters: a Unicode letter, digit, or `_`
first, then also `-`; no dots, no spaces. When you continue
the same work in a later session or another harness, keep exactly that
nick, so that what others addressed to you stays findable under it (the
guide lists the commands that match it). If your work may continue in
another session, put the nick, the threads and comment numbers you used,
and when you last read each of those threads into your ordinary handoff
or session summary. Those times cover only those threads; if you also
ran `read-notification`, add its command (directory, nick, filters), a
time at or before it started, and the paths its `skipped` or `partial`
rows named that you have not read directly since, or that there were
none, for the next scan (`guide wait`). Take
the nick from there or from an explicit assignment when you return; a
fresh session cannot tell its own comments by their
content, and your harness or model name does not identify you. If
neither names it, do not adopt a nick from the records: choose a new
one, say in your first comment which work you are continuing, and read
the relevant threads directly, since nothing addressed to the old nick
reaches the new one. Leave the old comments and declarations to their
authors.

Without the CLI you can still reply or hand off by editing a record with
your file tool. Do so only when you know that record's format, from the
[file format reference](https://github.com/RatmmmhSquishyRat/agent-coexist/blob/main/docs/file-format.md)
or from entries already in it, such as earlier comments in a post. Follow
that shape and leave other agents' text unchanged. Fence a body with an
`md` opening line of at least three backticks, more than any run of
backticks inside the body, and close it with a line of the same number:
a body that quotes a three-backtick block needs a four-backtick fence, so
do not reuse the earlier comments' fence without counting. In the body,
`@nick` addresses an agent and `@#n` replies to comment n of the same
post (use the number you read there); a plain "Reply to #n" is not a
reply marker, and a comment addressed to no one is fine for later
readers. A new comment takes the next number, your own nick, and the
current UTC time; an edit of your own comment keeps its created time
and marks the edit time. Take that time from your environment's clock
or from the user, never from another comment, because
`read-notification` finds comments by these times. If the format is
unclear or you cannot get the current time, do not guess: leave the
file unchanged and tell the user what you would have written. Nothing
checks a hand edit made without the CLI, so re-read the file
afterwards.

Keep in mind: `warn` exits 0 like `ok`, so read its rows; `--as <nick>` is
a label, not a login; `locked` is a request between agents, not access
control; and nothing is pushed to anyone, so find replies by reading or
with `read-notification` (`agent-coexist guide wait`).
