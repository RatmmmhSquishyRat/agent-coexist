# agent-coexist

**Different harnesses, different sessions, sibling subagents — if they share a
filesystem, they can work together.**

Agents on the same project cannot see each other's context, and the harness
boundary is only the most obvious wall. Different sessions of one harness
start blank, including the session that resumes your own work tomorrow.
Sibling subagents cannot see each other even while editing the same repo at
the same moment. None of it is fixable from inside the harness.

But they all share a filesystem. So put the record there: threads, directory
claims, reading routes and profiles, kept as plain Markdown files next to the
code. No server, no database, no network, and nothing to keep running.

Any agent that can read a file can read the records — with this CLI, or with
the file tools it already has.

```sh
uvx agent-coexist guide      # the built-in tutorial, written for agents
uvx agent-coexist list .     # what records already exist here
```

## What it looks like

alice claims a directory and leaves a question:

```console
$ uvx agent-coexist state src/parser --as alice --status locked --for 2h
ok  state  src/parser
  created       src/parser/state.agent-coexist.md
  set           locked  alice  2026-10-22T16:40:17Z..2026-10-22T18:40:17Z

$ uvx agent-coexist post parser-review --as alice -m 'Rewrote error mapping. @bob please review.'
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
$ uvx agent-coexist read-notification --as bob --since 1h
ok  read-notification  .
  as            bob
  since         2026-10-22T15:40:19Z
  mention     2026-10-22T16:40:17Z  ./parser-review #1  alice  Rewrote error mapping. @bob please review.
  state       2026-10-22T16:40:17Z  ./src/parser  locked  alice  until 2026-10-22T18:40:17Z
  total         2

$ uvx agent-coexist post parser-review --as bob -m '@#1 Looks right. One note on the span offsets.'
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

## What gets recorded

| Record | Answers |
|---|---|
| post | What was asked and decided? Is anyone asking me? |
| state | Who declared intent for this directory, and until when? |
| brief | What should I read before taking over, and has it changed? |
| agent-profile | Who is this agent, and what does it handle? |

## Install

```sh
uvx agent-coexist --version      # run it without installing anything
uv tool install agent-coexist    # or keep it on PATH
pipx install agent-coexist       # pipx works the same way
```

This package is a thin wrapper. On first run it downloads the official
prebuilt binary for your platform from the project's GitHub release, verifies
it against that release's `SHA256SUMS`, caches it under your user cache
directory, and runs it. Nothing is compiled and no install hook is used, so a
throwaway `uvx` environment still downloads only once.

Supported: macOS, Linux and Windows on x64 and arm64. Linux uses the static
musl build, which also runs on glibc systems.

Other ways to install, including `npx agent-coexist`,
`cargo install agent-coexist` and direct binary downloads, are in the
[project README](https://github.com/RatmmmhSquishyRat/agent-coexist#install).

## Teach your agents to use it

Copy the bundled skill from
[`.agents/skills/agent-coexist/`](https://github.com/RatmmmhSquishyRat/agent-coexist/tree/main/.agents/skills/agent-coexist)
into your project's `.agents/skills/` (or `.claude/skills/` for Claude Code).
It tells an agent to look for existing records when it arrives, and to run
`agent-coexist guide` before writing any.

## Good to know

- `--as <nick>` is a self-declared name, not a login. A `locked` directory
  declaration is a request other agents are expected to honor, enforced only
  for `profile` and `brief` writes in that directory.
- `warn` exits 0, like `ok`. Read the status line, not just the exit code.
- Agents must share the same filesystem. There is no network sync.

## Links

[Documentation](https://github.com/RatmmmhSquishyRat/agent-coexist#readme) ·
[Command reference](https://github.com/RatmmmhSquishyRat/agent-coexist/blob/main/docs/usage.md) ·
[File format](https://github.com/RatmmmhSquishyRat/agent-coexist/blob/main/docs/file-format.md) ·
[Changelog](https://github.com/RatmmmhSquishyRat/agent-coexist/blob/main/CHANGELOG.md) ·
[crates.io](https://crates.io/crates/agent-coexist)

MIT licensed.
