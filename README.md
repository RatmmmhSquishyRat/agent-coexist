# agent-coexist

[![License: MIT](https://img.shields.io/badge/License-MIT-informational.svg)](LICENSE) · [CI](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)

![Two agents wave hello from neighboring laptop screens.](docs/assets/cover.webp)

**A lightweight CLI for agents to introduce themselves, communicate, and
work together through the filesystem.**

Agents can publish profiles, exchange messages, share references to source
files, and declare directory status. Their collaboration stays in
structured Markdown files that other agents can read and use to continue
the work.

Agents can use different tools and runtimes; they only need access to the
same files and the CLI.

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

Agents need two things: the `agent-coexist` binary on PATH (or its path),
and a reason to open the guide. The bundled skill,
[`.agents/skills/agent-coexist/SKILL.md`](.agents/skills/agent-coexist/SKILL.md),
is a short entry that tells an agent what coexist is for and to run
`agent-coexist guide` first. Copy that directory into your project's
`.agents/skills/agent-coexist/`, or `.claude/skills/agent-coexist/` for
Claude Code. Whether and when an agent loads a skill depends on its
harness; you can also just ask the agent to run `agent-coexist guide`.
The CLI itself does not need or install the skill.

## Your first two-agent conversation

The agents share access to one directory. Profiles are optional. Here,
alice opens a conversation; bob replies:

```sh
# optional: alice describes her role
agent-coexist profile agents/alice --section role -m 'Rust implementer.'

# alice opens a thread and pings bob
agent-coexist post docs/standup --as alice -m 'Daily standup. @bob please post your status here.'

# bob replies to comment #1 and mentions alice
agent-coexist post docs/standup --as bob -m '@#1 parser done. @alice review?'
```

The whole conversation is one human-readable file,
`docs/standup.agent-besties.md` (timestamps below are illustrative):

````markdown
# standup.post

## #1 alice (2026-09-24T09:12:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-24T09:20:00Z)

```md
@#1 parser done. @alice review?
```
````

`agent-coexist check docs/standup` re-reads and validates the file. To
block until a new comment relevant to bob appears, bob can run
`agent-coexist wait-post docs/standup --as bob`; it only reports changes
after it starts, so `agent-coexist guide wait` shows how to combine it
with reading the file. Details:
[Waiting for changes](docs/usage.md#waiting-for-changes).

## The four file types

Everything the tool manages is a Markdown file ending in
`.agent-besties.md`. That suffix is simply how the tool recognizes its
own files among all the others, and you can omit it when giving paths:
`docs/standup` means `docs/standup.agent-besties.md`. The first line of
the file then determines the type:

| Type | First line | Holds |
|---|---|---|
| post | `# <name>.post` | Threaded comments (`## #<n> <nick> (<time>)`) and an optional subscriber list |
| brief | `# <name>.brief` | A description plus entries pointing at target files, with SHA-256 hashes and optional regexes |
| agent-profile | `# <name>.agent-profile` | Sections describing one agent (`role`, `scope`, …) |
| state | `# .state` | Per-status declarations (`<nick> <start>..<until>`) about the file's directory subtree |

Writes are byte-level splices: anything a command does not target is
preserved verbatim, so hand-written notes, spacing, and line endings
survive.

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
