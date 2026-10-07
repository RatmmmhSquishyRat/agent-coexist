# agent-coexist

[![License: MIT](https://img.shields.io/badge/License-MIT-informational.svg)](LICENSE)

![Two agents wave hello from neighboring laptop screens.](docs/assets/cover.webp)

**Help agents find each other, catch up, and work together.**

agent-coexist is a small command-line tool for projects that more than
one AI agent works on. It keeps their threads, directory claims, reading
lists, and profiles as plain Markdown files in the project, where any
agent with file access can find them.

## Why

Agents on the same project usually cannot see each other's context. Most
harnesses do not talk to one another, main sessions rarely see each other,
subagents rarely see their siblings, and a session that resumes or moves to
another harness often has no record of what came before.

agent-coexist does not monitor harness chat transcripts, sync across
networks, or automatically capture session histories. Instead, agents
explicitly write messages, directory claims, and reading routes to
Markdown files (`*.agent-besties.md`) in the workspace. Any agent with
file access can inspect them with standard file tools or the CLI.

## Install

Requires Rust 1.89 or newer.

```sh
git clone https://github.com/RatmmmhSquishyRat/agent-coexist.git
cd agent-coexist
cargo install --path crates/cli --locked
agent-coexist --version
agent-coexist guide
```

`agent-coexist guide` prints the built-in tutorial written for agents. It
creates no files.

## Getting started

Run `agent-coexist guide` for the complete onboarding tutorial, or inspect
existing records in your project:

```sh
# Check for existing records
agent-coexist list .

# Post a message or review request
agent-coexist post parser-review --as alice -m 'Error mapping updated. Please review.'
```

For the complete walkthrough covering multiple agents, mentions, replies,
and notifications, see the [built-in guide](crates/cli/guide/index.md).

## Use it with your agents

To run commands, an agent needs `agent-coexist` on PATH (or its full path).

To teach agents when and how to look for project records, copy the bundled
skill in [`.agents/skills/agent-coexist/`](.agents/skills/agent-coexist/SKILL.md)
into your project's `.agents/skills/` (or `.claude/skills/` for Claude Code).
It instructs an agent to check for existing records upon arrival and run
`agent-coexist guide`. Reading records or editing them by hand needs only
standard file tools; the CLI keeps writes consistent and checks records
across a directory tree.

## What gets recorded

Each record type serves a specific coordination need:

| Record | Answers | Holds |
|---|---|---|
| post | What was asked and decided? Is anyone asking me? | Numbered comments with `@nick` mentions and `@#n` replies |
| state | Who declared intent for this directory, and until when? | Expiring declarations such as `locked` or `observing` |
| brief | What should I read before taking over, and has it changed? | Target file paths with notes, hashes, and optional regex patterns |
| agent-profile | Who is this agent, and what does it handle? | Optional role, scope, and contact notes |

Every record is stored as a Markdown file ending in `.agent-besties.md`.
Commands accept paths with or without that suffix. See the
[file format reference](docs/file-format.md) for disk layout.

## Operating boundaries

- **Explicit workspace files, not chat logs:** agent-coexist does not hook
  into agent conversations, sync across a network, or maintain an automatic
  activity feed. Agents explicitly write to `.agent-besties.md` files;
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
  [crates/cli/guide/](crates/cli/guide/index.md).
- [Command reference](docs/usage.md): complete reference for commands,
  flags, exit codes, and output formats.
- [Output summary](docs/output-summary.md): representative catalog of
  actual commands and captured outputs, including the full stdout of the
  five built-in `guide` pages.
- [File format reference](docs/file-format.md): disk structure for each
  record type.
- [Project skill](.agents/skills/agent-coexist/SKILL.md): bundled agent
  skill instructions.
- [Contributing](CONTRIBUTING.md) · [Security policy](SECURITY.md) ·
  [Changelog](CHANGELOG.md) ·
  [CI workflow](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)

## License

The project's own code and documentation are licensed under the
[MIT License](LICENSE). The vendored copy of `ignore` in
[`vendor/ignore/`](vendor/ignore/COPYING) keeps its upstream copyright
notices and license (Unlicense OR MIT). The cover artwork under
`docs/assets/` is not part of the MIT grant and remains with its creator.
