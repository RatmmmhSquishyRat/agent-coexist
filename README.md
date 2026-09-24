# agent-coexist

[![License: MIT](https://img.shields.io/badge/License-MIT-informational.svg)](LICENSE) · [CI](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)

![Two agents wave hello from neighboring laptop screens.](docs/assets/cover.webp)

agent-coexist is a small local CLI that lets software agents — and the
people working next to them — hold threaded conversations, hand over
pointers to source files, and declare directory state, all through plain
Markdown files that live next to the code. There is no server, no
database, and no background service: everything is ordinary files you can
read, diff, and edit yourself.

Because it is all files, the tools you already have understand it: git
tracks the history, editors render the threads, and grep finds anything.

## Install

Requires Rust 1.89 or newer. The examples in this README assume a POSIX
shell; on Windows, Git Bash works as-is.

```sh
git clone https://github.com/RatmmmhSquishyRat/agent-coexist.git
cd agent-coexist
cargo install --path crates/cli --locked
agent-coexist --version
```

## Your first two-agent conversation

Two agents — two terminal sessions, two AI coding helpers, or one of
each — share one directory:

```sh
# alice introduces herself
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

`agent-coexist check docs/standup` re-reads and validates the file, and
`agent-coexist wait-post docs/standup --as bob` blocks until a comment
relevant to bob appears — see
[Waiting for changes](docs/usage.md#waiting-for-changes) in the usage
guide.

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
- **`warn` is still success.** The first line of every result is `ok`,
  `warn` or `error`, and `warn` exits 0 like `ok`; scripts should parse
  the first line rather than trust the exit code alone.
- **Brief entry paths are relative to the brief file**, not to your
  current directory.
- **Managed files are capped at 16 MiB.**

## Documentation

- [Usage guide](docs/usage.md) — every command, flag, exit code, and the
  output format.
- [File format reference](docs/file-format.md) — the four file types as a
  human editor sees them.
- [Contributing](CONTRIBUTING.md) · [Security policy](SECURITY.md) ·
  [Changelog](CHANGELOG.md)

## License

Code and documentation are licensed under the [MIT License](LICENSE).
The cover artwork under `docs/assets/` is not part of that grant and
remains with its creator.
