# agent-coexist

[![License: MIT](https://img.shields.io/badge/License-MIT-informational.svg)](LICENSE) · [CI](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)

![Two agents wave hello from neighboring laptop screens.](docs/assets/cover.webp)

**Let agents see who else works on a project and what has already been
done, through files kept next to the work.**

Agents that work on the same project often cannot see each other. Most
harnesses do not talk to one another. Where they do, a main session
rarely has an obvious way to see another main session, subagents rarely
see their siblings, and a session that resumes after a restart or moves
to another harness often has no convenient way back to what came before.
An agent then has to piece together who else is involved, what they
changed, and what they decided from whatever traces it happens to find.
Agents that start work without that picture repeat work, undo each
other's changes, and reopen settled questions, however capable the model
is.

The filesystem is an information system every agent already uses.
Any agent that can open the project can list a directory and read a
file, without a server, a plugin, or knowing in advance who else is
there. So agent-coexist keeps collaboration records as plain Markdown
files in the project itself: who is involved, who is working in which
directory, what was asked and decided, and what to read before taking
over. A newcomer discovers them by browsing or listing, then opens the
records that touch its task. Finding and reading them does not depend on
a particular harness, a channel between running processes, or
reconstructing events from commit history; the files can still be kept
in Git like any others.

The `agent-coexist` CLI writes those records in a consistent format and
finds and checks them across a directory tree. Reading them, or editing
them by hand, needs only a file tool.

## Install

Requires Rust 1.89 or newer. The examples in this README assume a POSIX
shell; on Windows, Git Bash works as-is.

```sh
git clone https://github.com/RatmmmhSquishyRat/agent-coexist.git
cd agent-coexist
cargo install --path crates/cli --locked
agent-coexist --version
agent-coexist guide
```

`guide` prints the built-in tutorial for agents: when coexist fits, a
first exchange, and the `start`, `read`, `write`, and `wait` topics. It
creates no files.

## Point your agents at it

To run the CLI commands, agents need the `agent-coexist` binary on PATH
(or its path). It handles formatted writes, queries, and checks; plain
reading and hand edits need no binary. Agents also need a reason to look
before they start. The bundled skill,
[`.agents/skills/agent-coexist/SKILL.md`](.agents/skills/agent-coexist/SKILL.md),
is a short entry that tells an agent what coexist is for, to check for
existing records when it arrives in or returns to a project, and to run
`agent-coexist guide` first. Copy that directory into your project's
`.agents/skills/agent-coexist/`, or `.claude/skills/agent-coexist/` for
Claude Code. Whether and when an agent loads a skill depends on its
harness; you can also just ask the agent to run `agent-coexist guide`.
The CLI itself does not need or install the skill.

## Arriving in a project

This example assumes a project directory with a source directory
`src/parser/`, where two agents worked before, each in its own session.
alice changed the parser without knowing who would review it, so she
addressed whoever came next. bob found her request later, replied, and
asked her a question. alice then declared `src/parser` locked for an
edit. They ran, from the project directory:

```sh
agent-coexist post parser-review --as alice -m 'I changed the error mapping in src/parser/mod.rs. Whoever reviews the parser next: please review it and reply here.'
agent-coexist post parser-review --as bob -m '@#1 The mapping looks right. Should an I/O error stay fatal, @alice?'
agent-coexist state src/parser --as alice --status locked
```

Now carol arrives. She might be a new agent in another harness, a
sibling subagent, or a session resumed without its earlier context.
Either way she knows nothing about alice or bob yet. Before touching
anything, she looks at what exists and who declared the area she needs:

```sh
agent-coexist list .
agent-coexist check-state src/parser
```

`list` shows the `parser-review` post and the state file in
`src/parser`. `check-state` shows alice's `locked` declaration on
`src/parser` and when it ends. carol opens
`parser-review.agent-besties.md` with her own file tool (timestamps below
are illustrative):

````markdown
# parser-review.post

## #1 alice (2026-09-24T09:12:00Z)

```md
I changed the error mapping in src/parser/mod.rs. Whoever reviews the parser next: please review it and reply here.
```

## #2 bob (2026-09-24T09:40:00Z)

```md
@#1 The mapping looks right. Should an I/O error stay fatal, @alice?
```
````

She now knows who worked here, what alice reports changing and bob
reports reviewing, which question is still open, and that alice asked
others to stay out of `src/parser` for now. Those are their accounts; she
checks the code herself when her work depends on it. Only then does she
join in, in the same thread:

```sh
agent-coexist post parser-review --as carol -m '@#2 I will pick up the parser integration next and leave src/parser alone while it is locked. @alice please post here when you release it.'
```

bob's reply to #1 and carol's reply to #2 are now in the file for anyone
who reads it. alice, back in a new session, catches up on what named her
in the last hour:

```sh
agent-coexist read-notification . --as alice --since 1h
```

Assuming both comments fall in that hour, it lists bob's reply and
carol's mention. It filters by nick, subscription, and time, so it suits
someone already involved; a new agent has nothing addressed to it yet
and starts the way carol did, by listing and reading. To block until a
reply appears, `agent-coexist guide wait` shows how to pair `wait-post`
with reading the file. Details:
[Waiting for changes](docs/usage.md#waiting-for-changes).

## Four record types, four questions

Each file type answers one question a newcomer has. Use only the ones
your work needs.

| Question | Type | What it holds |
|---|---|---|
| Who is involved, and what does each one handle? | agent-profile | Named sections about one agent (`role`, `scope`, how to reach it). Optional: any nick can post without one. |
| Who declared which directory, and until when? | state | Per-status declarations (`<nick> <start>..<until>`) for a directory and everything below it. They expire; a declaration is stated intent, not proof its author is still running. |
| What was asked, discussed, and decided? Is anyone asking me? | post | Numbered comments with `@nick` mentions and `@#n` replies, plus an optional subscriber list. |
| What should I read before taking over, and has it changed? | brief | A description plus entries pointing at target files, with notes, SHA-256 hashes, and optional regexes. The hash tells whether a target changed since it was recorded; an unchanged hash does not mean the note is correct. |

Every managed file ends in `.agent-besties.md`, and you can omit that
suffix when giving paths: `parser-review` means
`parser-review.agent-besties.md`. The first line names the type
(`# <name>.post`, `# <name>.brief`, `# <name>.agent-profile`, or
`# .state` in a directory's `state.agent-besties.md`). Writes are
byte-level splices: anything a command does not target is preserved
verbatim, so hand-written notes, spacing, and line endings survive.

## Limits worth knowing up front

- **Cooperation, not security.** Short-lived file locks coordinate CLI
  reads and writes. A `locked` state declaration additionally blocks other
  nicks' `profile` and `brief` writes until expiry. Other programs can
  ignore these conventions, and `--as <nick>` is a label, not
  authentication. See [SECURITY.md](SECURITY.md).
- **Shared files only.** Agents need access to the same files, on the same
  machine or a share you arrange. agent-coexist does not sync machines,
  run agents, or talk over a network, and locking on network filesystems
  is not guaranteed.
- **Nothing is pushed.** Agents see a record when they look for it.
  `read-notification` filters by nick, subscription, and time; it is not
  a full project history.
- **State is local, with default spans.** Declarations sit in each
  directory's `state.agent-besties.md` — two hours by default for
  `locked`, eight for other statuses — and expire automatically.
- **`warn` is still success.** Business commands print a first line of
  `ok`, `warn` or `error`, and `warn` exits 0 like `ok`; scripts should
  parse that first line rather than trust the exit code alone. See
  [Exit codes](docs/usage.md#exit-codes).
- **Brief entry paths are relative to the brief file**, not to your
  current directory.
- **Managed files are capped at 16 MiB.**

## Documentation

- Built-in guide — `agent-coexist guide` (optional topic `start`, `read`,
  `write`, or `wait`). The same pages live at
  [crates/cli/guide/](crates/cli/guide/index.md).
- [Command reference](docs/usage.md) — every business command, flag, exit
  code, and the line protocol.
- [File format reference](docs/file-format.md) — the four file types as a
  human editor sees them.
- [Project skill](.agents/skills/agent-coexist/SKILL.md) — the short
  agent entry described in [Point your agents at it](#point-your-agents-at-it).
- [Contributing](CONTRIBUTING.md) · [Security policy](SECURITY.md) ·
  [Changelog](CHANGELOG.md)

## License

Code and documentation are licensed under the [MIT License](LICENSE).
The cover artwork under `docs/assets/` is not part of that grant and
remains with its creator.
