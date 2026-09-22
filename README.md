# agent-coexist

A small local CLI that lets software agents (and people) collaborate through
plain Markdown files: introduce themselves, leave threaded comments, hand
over pointers to source files, and declare directory state such as locks.

Everything lives in ordinary files ending in `.agent-besties.md` — no server,
no database, no background service. Four file types, recognized by their
first line:

| Type | First line | Holds |
|---|---|---|
| `post` | `# <name>.post` | Threaded comments (`## #<n> <nick> (<time>)` headings, fenced bodies) and a subscriber list |
| `brief` | `# <name>.brief` | A description plus entries pointing at target files with SHA-256 hashes (and optional regexes) |
| `agent-profile` | `# <name>.agent-profile` | Sections describing an agent (`role`, `scope`, `prefer`, …) |
| `state` | `# .state` | Per-status declarations (`<nick> <start>..<until>`) describing the file's directory subtree |

Built files keep their unknown hand-written content, original line endings
and byte layout: writes are splices, never full re-serializations.

## Build and install

Requires Rust 1.89 or newer (the repo pins 1.89.0 in `rust-toolchain.toml`).

```sh
# from the repository root
cargo build --release

# install the `agent-coexist` binary from source
cargo install --path crates/cli
agent-coexist --version
```

Run `cargo test --workspace` for the mechanism tests.

## Commands

`PATH` arguments may omit the `.agent-besties.md` suffix (it is appended
automatically). `DIR` arguments default to the current directory; directory
scans are recursive. Text bodies are always passed with `-m`; what `-m`
means depends on the command (each command's `--help` says which).

Eleven base commands:

```text
agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]
agent-coexist check <PATH>
agent-coexist profile <PATH> [--as <nick>] [--section <name> (-m <content> | --remove)]
agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]
agent-coexist subscribe <PATH> --as <nick> [--cancel]
agent-coexist brief <PATH> [--as <nick>]                                    # create skeleton / unchanged
agent-coexist brief <PATH> [--as <nick>] -m <description>
agent-coexist brief <PATH> [--as <nick>] --target <path> [--entry <title>] [--regex <pattern>] [-m <note>]
agent-coexist brief <PATH> [--as <nick>] --entry <title> -m <note>
agent-coexist brief <PATH> [--as <nick>] --entry <title> --remove
agent-coexist brief <PATH> [--as <nick>] [--entry <title>] --refresh
agent-coexist state [DIR] --as <nick> --status <name> [--for <duration>]
agent-coexist state [DIR] --remove <nick> [--as <nick>] [--status <name>]
agent-coexist read-notification [DIR] --as <nick> --since <time|duration> [--include <glob>]... [--exclude <glob>]... [--no-ignore]
agent-coexist lookfor-comment <PATH> [--keyword <text>] [--by <nick>] [--mention <nick>] [--reply-to <n>] [--comment <n>]
agent-coexist read-brief <PATH> [--entry <title>]
agent-coexist check-state [DIR] [--include <glob>]... [--exclude <glob>]... [--no-ignore]
```

Three optional wait commands (extension; the base CLI works without them):

```text
agent-coexist wait <PATH>                       # any of the four types; exits on any content change
agent-coexist wait-post <PATH> [--as <nick>]    # a post; filter: mentions/replies involving the nick
agent-coexist wait-state [DIR] [--status <name>]# exactly DIR/state.agent-besties.md; filter: status
```

Wait commands print a `waiting  <command>  <object>` line to stderr once
the baseline snapshot is taken, then poll the single file once per second
until a qualifying change, Ctrl-C (exit 130), or a real failure.

## Example session

```sh
mkdir -p /tmp/demo/agents /tmp/demo/docs /tmp/demo/src/parser
cd /tmp/demo
printf 'pub fn parse(_input: &str) -> Result<(), ()> { Ok(()) }\n' > src/parser/mod.rs

# alice introduces herself
agent-coexist profile agents/alice --section role -m 'Rust implementer.'

# alice starts a standup thread and subscribes bob
agent-coexist post docs/standup --as alice -m 'Daily standup thread. @bob please post your status here.'
agent-coexist subscribe docs/standup --as bob

# bob replies (comment #2, mentioning alice)
agent-coexist post docs/standup --as bob -m '@#1 parser done, see src/parser/mod.rs. @alice review?'

# hand over a pointer to a source file (path is relative to the brief file)
agent-coexist brief docs/onboarding --target ../src/parser/mod.rs --entry 'parser entry point' -m 'Start here.'

# declare a two-hour lock on src/parser
agent-coexist state src/parser --as alice --status locked

# read things back
agent-coexist check docs/standup
agent-coexist read-brief docs/onboarding
agent-coexist check-state src
agent-coexist read-notification . --as alice --since 1h
agent-coexist lookfor-comment docs/standup --by bob
agent-coexist list .
```

## Output conventions

* Every result's first line is `<status>  <command>  <object>` with status
  `ok`, `warn` or `error`. All other lines are two-space-indented columns;
  splitting any line on two or more spaces yields its columns.
* List-style outputs end with a `total` row. Diagnostics carry a `why` and
  an actionable `fix` line; `fix` commands target POSIX shells (Git Bash on
  Windows).
* Progress (`scanning N dirs, M files...`) appears on stderr only when
  stderr is a TTY; stdout is always clean for redirection.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | First line `ok` or `warn` — note `warn` is also 0; detect conflicts by parsing the first line |
| 1 | First line `error`: diagnostics, type mismatches, missing files/entries, I/O failures |
| 2 | Usage error (one `error: ...` line on stderr, nothing on stdout) |
| 3 | Retryable: file locked by another `agent-coexist` process, file held by another program, or a functional lock (`E-FLOCKED` — retry at the expiry time shown in `reason`, not at millisecond cadence) |
| 130 | Wait command cancelled with Ctrl-C |

Environment overrides used by the CLI: `AGENT_COEXIST_NOW` (fixed `now` for
testing, strict `YYYY-MM-DDTHH:MM:SSZ`) and `AGENT_COEXIST_LOCK_WAIT_MS`
(post/subscribe exclusive-lock wait budget).

## Repository layout

```text
crates/core   coexist-core: raw-byte parsing, type views, splices, file
              locking/lifecycle, walking, hashing, command logic (pure data)
crates/cli    agent-coexist: argument grammar + validation, the single
              renderer, stderr progress, exit codes, wait wiring
```

See `CHANGELOG.md` for release notes.
