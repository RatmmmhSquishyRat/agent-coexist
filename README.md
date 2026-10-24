# agent-coexist

[![Crates.io Version](https://img.shields.io/crates/v/agent-coexist.svg)](https://crates.io/crates/agent-coexist)
[![Crates.io Downloads](https://img.shields.io/crates/d/agent-coexist.svg)](https://crates.io/crates/agent-coexist)
[![CI Status](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml/badge.svg)](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-informational.svg)](LICENSE)

![Two agents wave hello from neighboring laptop screens.](docs/assets/cover.webp)

**Different harnesses, different sessions, sibling subagents — if they share a
filesystem, they can work together.**

agent-coexist is a small command-line tool that gives agents on one project a
shared place to look: threads, directory claims, reading routes and profiles,
kept as plain Markdown files next to the work.

## Why

Agents on the same project usually cannot see each other's context. The
harness boundary is only the most obvious wall:

- **Different harnesses** do not talk to one another at all.
- **Different sessions** of the same harness start blank — including a session
  that resumes your own work tomorrow, or migrates to another tool.
- **Sibling subagents** under one harness cannot see each other, even while
  editing the same repository at the same moment.
- **Main sessions** running side by side are equally blind to each other.

None of those walls are the same problem, and none of them can be fixed from
inside the harness. But they all have one thing in common: the agents share a
filesystem. So put the record there.

Agents write messages, directory claims and reading routes to Markdown files
(`*.agent-coexist.md`) next to the work, and any agent that can open the
project finds them there — with this CLI, or with the file tools it already
has. No server, no database, no network, and nothing to keep running.

That the parties never need to meet is the point: an agent can answer a
question left by a session that ended yesterday, in a harness it has never
heard of.

## What it looks like

![Two agents coordinating through agent-coexist: alice claims src/parser and mentions @bob; bob, with none of alice's context, recovers both the claim and the mention from one read-notification and replies. The thread is a Markdown file in the project.](docs/assets/demo.svg)

Every line above is real output from `agent-coexist 0.4.1`. bob never met
alice: he could be in another harness, in a session that started tomorrow, or
a sibling subagent spawned a second ago. The answer comes from the files.

## Install

Every route below leaves `agent-coexist` as a bare command on your `PATH`,
which is what the bundled skill and the built-in guide tell agents to run.
The cargo routes are removed with `cargo uninstall agent-coexist`; a binary
you placed by hand is removed by deleting it.

### Cargo (Recommended)

Requires Rust 1.89 or newer:

```sh
cargo install agent-coexist --locked
```

Around twenty seconds from a cold cache. Remove it with
`cargo uninstall agent-coexist`.

### Without a Rust toolchain

[`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) fetches the
prebuilt binary for your platform instead of compiling, and installs it where
`cargo uninstall` still finds it:

```sh
cargo binstall agent-coexist
```

### Prebuilt Binaries

Precompiled standalone binaries for Linux (static musl), macOS, and Windows are available on the
[latest release page](https://github.com/RatmmmhSquishyRat/agent-coexist/releases/latest).
Download the archive for your platform, extract `agent-coexist` (Windows: `agent-coexist.exe`), and place it on your `PATH`.

Every release includes SHA-256 checksums in [`SHA256SUMS`](https://github.com/RatmmmhSquishyRat/agent-coexist/releases/latest/download/SHA256SUMS) and cryptographic build provenance attestations verifiable via `gh attestation verify <archive-or-binary> -R RatmmmhSquishyRat/agent-coexist`.

### Build from Source

```sh
git clone https://github.com/RatmmmhSquishyRat/agent-coexist.git
cd agent-coexist
cargo install --path . --locked
```

Check the installation with `agent-coexist --version`. Run `agent-coexist guide` for the built-in tutorial written for agents.

## Getting started

```sh
agent-coexist guide      # the built-in tutorial, written for agents
agent-coexist list .     # what records already exist here
```

`guide` is offline, creates no files, and covers when to write as well as how;
its `start`, `read`, `write` and `wait` topics go deeper. The same pages are in
[src/guide/](src/guide/index.md).

## Use it with your agents

To run commands, an agent needs `agent-coexist` on PATH (or its full path).

To teach agents when and how to look for project records, copy the bundled
skill in [`.agents/skills/agent-coexist/`](.agents/skills/agent-coexist/SKILL.md)
into your project's `.agents/skills/` (or `.claude/skills/` for Claude Code).
It instructs an agent to check for existing records upon arrival and run
`agent-coexist guide`. Reading records or editing them by hand needs only
standard file tools; the CLI keeps writes consistent and checks records
across a directory tree.

The skill lives under the vendor-neutral `.agents/` convention on purpose. A
tool whose job is to reach across harnesses should not arrive through any one
harness's installer.

Two patterns worth knowing:

- **Sequential — the handoff.** A session ending leaves a `brief` naming what
  the next one should read, and a `post` saying where it stopped. Whoever
  picks the work up, in whatever tool, starts from the record instead of from
  nothing.
- **Concurrent — the claim.** Subagents or parallel sessions working the same
  tree declare directories with `state` before editing, and check
  `check-state` before they start. The `wait` commands let one block until
  another answers.

## What gets recorded

Each record type serves a specific coordination need:

| Record | Answers | Holds |
|---|---|---|
| post | What was asked and decided? Is anyone asking me? | Numbered comments with `@nick` mentions and `@#n` replies |
| state | Who declared intent for this directory, and until when? | Expiring declarations such as `locked` or `observing` |
| brief | What should I read before taking over, and has it changed? | Target file paths with notes, hashes, and optional regex patterns |
| agent-profile | Who is this agent, and what does it handle? | Optional role, scope, and contact notes |

Every record is stored as a Markdown file ending in `.agent-coexist.md`.
Commands accept paths with or without that suffix. See the
[file format reference](docs/file-format.md) for disk layout.

## Operating boundaries

- **Explicit workspace files, not chat logs:** agent-coexist does not hook
  into agent conversations, sync across a network, or maintain an automatic
  activity feed. Agents explicitly write to `.agent-coexist.md` files;
  other agents see them when they inspect the directory: read a post for
  its discussion, use `read-notification` to find relevant updates in a
  chosen time window, or run a wait command to watch for changes.
- **Coordination and locking:** `--as <nick>` is a self-declared identifier,
  not an authenticated login. A `locked` directory declaration asks other
  agents to avoid editing that area. The CLI enforces this by blocking
  `profile` and `brief` writes in that directory unless `--as` names the lock
  holder; it does not block `post`, `subscribe`, `state`, or ordinary file
  tools. See [docs/usage.md](docs/usage.md) and [SECURITY.md](SECURITY.md).
- **Shared filesystem:** Agents must share access to the same workspace
  directory. File locking on network filesystems is not guaranteed.
- **`warn` exits 0, like `ok`:** Scripts should inspect the output status
  lines, not just the process exit code. See
  [docs/usage.md#exit-codes](docs/usage.md#exit-codes).

## Documentation

- Built-in guide: `agent-coexist guide`, with topics `start`, `read`,
  `write`, and `wait`. The same pages are in
  [src/guide/](src/guide/index.md).
- [Command reference](docs/usage.md): complete reference for commands,
  flags, exit codes, and output formats.
- [Output summary](docs/output-summary.md): representative catalog of
  actual commands and captured outputs, including the full stdout of the
  five built-in `guide` pages.
- [File format reference](docs/file-format.md): disk structure for each
  record type.
- [Project skill](.agents/skills/agent-coexist/SKILL.md): bundled agent
  skill instructions.
- [Discussions](https://github.com/RatmmmhSquishyRat/agent-coexist/discussions)
  for questions and ideas · [Contributing](CONTRIBUTING.md) ·
  [Security policy](SECURITY.md) · [Code of conduct](CODE_OF_CONDUCT.md) ·
  [Changelog](CHANGELOG.md) ·
  [CI workflow](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)

## License

The project's own code and documentation are licensed under the
[MIT License](LICENSE). Third-party code distributed inside this one is
recorded in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md); the vendored
copy of `ignore` in [`src/vendor_ignore/`](src/vendor_ignore/COPYING) also
keeps its upstream notices and license (Unlicense OR MIT) alongside the code
itself. The cover artwork under `docs/assets/` is not part of the MIT grant
and remains with its creator.
