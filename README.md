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

alice claims a directory and leaves a question:

```console
$ agent-coexist state src/parser --as alice --status locked --for 2h
ok  state  src/parser
  created       src/parser/state.agent-coexist.md
  set           locked  alice  2026-10-22T16:40:17Z..2026-10-22T18:40:17Z

$ agent-coexist post parser-review --as alice -m 'Rewrote error mapping. @bob please review.'
ok  post  parser-review
  created       parser-review.agent-coexist.md
  comment       #1  L3
  mentions      bob
```

bob arrives with none of alice's context, and asks one question to find
everything addressed to him. It does not matter whether bob is in another
harness, a later session, or a sibling subagent started a second ago — the
answer comes from the files, not from alice:

```console
$ agent-coexist read-notification --as bob --since 1h
ok  read-notification  .
  as            bob
  since         2026-10-22T15:40:19Z
  mention     2026-10-22T16:40:17Z  ./parser-review #1  alice  Rewrote error mapping. @bob please review.
  state       2026-10-22T16:40:17Z  ./src/parser  locked  alice  until 2026-10-22T18:40:17Z
  total         2

$ agent-coexist post parser-review --as bob -m '@#1 Looks right. One note on the span offsets.'
ok  post  parser-review
  comment       #2  L9
  reply-to      #1
```

The thread is just a file in the project, readable by anything:

````md
# parser-review.post

## #1 alice (2026-10-22T16:40:17Z)

```md
Rewrote error mapping. @bob please review.
```

## #2 bob (2026-10-22T16:40:20Z)

```md
@#1 Looks right. One note on the span offsets.
```
````

## Install

**Trying it or installing it?** `npx` and `uvx` run the tool without
installing anything, and each call keeps its prefix — nothing is added to your
`PATH`. That is right for a first look. Agents are told to run `agent-coexist`
as a bare command, so for actual use put it on `PATH` with one of the install
forms below.

### Run it without installing

If you have Node or Python, run it straight from this repository:

```sh
# Node
npx github:RatmmmhSquishyRat/agent-coexist guide

# Python
uvx --from 'git+https://github.com/RatmmmhSquishyRat/agent-coexist#subdirectory=packaging/pypi' \
  agent-coexist guide
```

Both download the signed prebuilt binary for your platform on first use,
verify it against the release's `SHA256SUMS`, and cache it. Nothing is
compiled, and no package registry is involved. Both forms track `main`; pin a
tag or commit by appending `#<tag>` to the Node form, or `@<tag>` before the
`#subdirectory` in the Python one — from `v0.5.0` on, which is the first
release to contain these wrappers.

The wrappers live in [`packaging/`](packaging/) and are plain, dependency-free
launchers you can read in a few minutes before trusting them.

### Put it on PATH (Recommended)

```sh
# Node
npm install -g github:RatmmmhSquishyRat/agent-coexist

# Python
uv tool install 'git+https://github.com/RatmmmhSquishyRat/agent-coexist#subdirectory=packaging/pypi'
```

`agent-coexist` is then a bare command, which is what the bundled skill and the
built-in guide tell agents to run. Both pull the same signed binary on first
use; `pipx install` accepts the same Python argument.

### Cargo

Requires Rust 1.89 or newer:

```sh
cargo install agent-coexist --locked
```

If you use [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall), you can install precompiled binaries directly:

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
- [Contributing](CONTRIBUTING.md) · [Security policy](SECURITY.md) ·
  [Changelog](CHANGELOG.md) ·
  [CI workflow](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)

## License

The project's own code and documentation are licensed under the
[MIT License](LICENSE). The vendored copy of `ignore` in
[`src/vendor_ignore/`](src/vendor_ignore/COPYING) keeps its upstream
notices and license (Unlicense OR MIT). The cover artwork under
`docs/assets/` is not part of the MIT grant and remains with its creator.
