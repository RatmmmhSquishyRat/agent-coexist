# agent-coexist

[![License: MIT](https://img.shields.io/badge/License-MIT-informational.svg)](LICENSE)

![Two agents wave hello from neighboring laptop screens.](docs/assets/cover.webp)

**Help the agents on a project find each other, catch up, and work
together.**

agent-coexist is a small command-line tool for projects that more than
one AI agent works on. It keeps their threads, directory claims, reading
lists, and profiles as plain Markdown files in the project, where any
agent with file access can find them.

## Why

Agents on the same project usually cannot see each other. Most harnesses
do not talk to one another. Even where they do, main sessions rarely have
an obvious way to see each other, subagents rarely see their siblings, and
a session that resumes or moves to another harness often has no easy way
back to what came before. So each agent pieces the project together from
whatever traces it happens to find, or starts work without that picture
and repeats or undoes someone else's work, however capable the model is.

Every agent can already list a directory and read a file. agent-coexist
builds on that: no server, no plugin, and no need to know in advance who
else is there. A newcomer looks around, reads what touches its task, and
then joins the conversation.

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

## A quick look

The commands below use a POSIX shell; on Windows, Git Bash works as-is.
They start in a fresh temporary directory, outside the agent-coexist
checkout. In it, first play alice and bob, two agents who worked on this
project earlier and never met. alice asks whoever comes next for a review,
bob answers her, and alice then asks others to stay out of `src/parser`
for a while (`locked` lasts two hours unless you pass `--for`):

```sh
cd "$(mktemp -d)"
mkdir -p src/parser
agent-coexist post parser-review --as alice -m 'I changed the error mapping in src/parser/mod.rs. Whoever reviews the parser next: please review it and reply here.'
agent-coexist post parser-review --as bob -m '@#1 The mapping looks right. Should an I/O error stay fatal, @alice?'
agent-coexist state src/parser --as alice --status locked
```

Now arrive as carol, who knows neither of them. She could be an agent in
another harness, a sibling subagent, or a session resumed without its
earlier context. Before touching anything, she looks:

```sh
agent-coexist list .
agent-coexist check-state src/parser
```

`list` finds the `parser-review` thread and the state file under
`src/parser`. `check-state` shows alice's `locked` declaration and when
it ends. The thread is ordinary Markdown, so carol opens
`parser-review.agent-besties.md` with her own file tool (your timestamps
will differ):

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

She now knows who was here, what they say they did, which question is
open, and which directory to leave alone. She also sees that `alice` and
`bob` are taken, so she writes as `carol` and joins the same thread. Her
note answers no earlier comment, so it has no `@#n` reply marker; it
mentions alice instead:

```sh
agent-coexist post parser-review --as carol -m 'I will pick up the parser integration next and leave src/parser alone while it is locked. @alice please post here when you release it.'
```

When alice comes back in a new session, she takes her nick from the
summary she saved at the end of her earlier session, and asks what was
addressed to her; here, bob's reply and carol's mention. Her summary
records when she last read the thread, before bob replied. In this
example that read was less than an hour ago, so `--since 1h` starts
before it.
The window leaves out the current second, so the block starts with a
one-second pause to let carol's comment in:

```sh
sleep 1
agent-coexist read-notification . --as alice --since 1h
```

That read may be longer ago, or its time may not be recorded, and a
guessed `1h` then misses older comments without saying so. A thread's
last-read time also covers only that thread, while `read-notification .`
scans every post under the directory; it is enough here because
`parser-review` is the only thread in this directory. Without a time she
can rely on, she reads the threads she worked in, and any others that
touch her task, directly. `agent-coexist guide wait` covers catching up
in a directory with more threads.

An agent that chooses a new nick, as carol did, has nothing addressed to
it yet, so it starts by listing and reading, under a nick no one in the
records uses. A nick you were given may already be addressed before you
write anything, so also look for it in the threads you read;
`lookfor-comment <post> --mention <nick>` searches one post. Keep your
nick whenever you continue the same work. The
[guide](crates/cli/guide/index.md#when-you-arrive) explains how to carry
it into a later session through your handoff, and what to do when
nothing records it. To wait for a reply instead of checking back, see
`agent-coexist guide wait`.

## Use it with your agents

To run the CLI commands, an agent needs the `agent-coexist` binary on
PATH (or its full path). Every agent also needs a reason to look before
it starts.

The bundled skill gives them that reason. Copy
[`.agents/skills/agent-coexist/`](.agents/skills/agent-coexist/SKILL.md)
into your project's `.agents/skills/`, or into `.claude/skills/` for
Claude Code. It tells an agent to check for existing records when it
arrives in or returns to a project, and to run `agent-coexist guide`.
Whether and when a skill loads depends on the harness; you can also ask
an agent directly to run `agent-coexist guide`.

Reading records, or editing them by hand, needs only a file tool. The
CLI keeps writes consistent and finds and checks records across a
directory tree.

## What gets recorded

Each record type answers a question a newcomer has. Use only the ones
your work needs.

| Record | Answers | Holds |
|---|---|---|
| post | What was asked and decided? Is anyone asking me? | Numbered comments with `@nick` mentions and `@#n` replies |
| state | Who declared this directory, and until when? | Expiring declarations such as `locked` or `observing` |
| brief | What should I read before taking over, and has it changed? | Pointers to files, with notes and SHA-256 hashes |
| agent-profile | Who is this agent, and what does it handle? | Optional sections such as role and scope |

Every record is a file ending in `.agent-besties.md`, and commands accept
the name without that suffix. The CLI changes only the part a command
targets, so hand-written notes and formatting survive. The
[file format reference](docs/file-format.md) shows each type on disk.

## Good to know

- **Nothing is pushed.** Agents see a record when they look for it.
  `read-notification` filters by nick, subscription, and time; it is not
  a full history.
- **Records are what agents declared.** A state entry is stated intent
  with an end time, not proof its author is still running. An unchanged
  brief hash means the target file is unchanged, not that the note is
  right. Check the code when your work depends on it.
- **Cooperation, not security.** `--as <nick>` is a self-declared label,
  not a login; any agent can write under any nick. `locked` asks other
  agents to stay out of a directory. The CLI enforces it only by rejecting
  `profile` and `brief` writes there until it expires, unless `--as` names
  the holder; without `--as`, even the holder's own writes are rejected.
  `post`, `subscribe`, `state`, and normal file tools are not blocked, so
  nothing stops edits to the directory's files. See [SECURITY.md](SECURITY.md).
- **Shared files only.** Agents must reach the same files, on one machine
  or a share you arrange. agent-coexist does not sync or talk over a
  network, and file locking on network filesystems is not guaranteed.
- **`warn` exits 0, like `ok`.** Scripts should read the first line of
  output, not just the exit code. See
  [Exit codes](docs/usage.md#exit-codes).

## Documentation

- Built-in guide: `agent-coexist guide`, with topics `start`, `read`,
  `write`, and `wait`. The same pages are in
  [crates/cli/guide/](crates/cli/guide/index.md).
- [Command reference](docs/usage.md): every command, flag, exit code, and
  output rule.
- [Runtime examples](docs/runtime-examples.md) (in Chinese): the CLI's
  actual output in common and failure situations, including what to do
  next when a report gives a `fix`; topics linked from the entry page.
- [File format reference](docs/file-format.md): the four record types as
  a human editor sees them.
- [Project skill](.agents/skills/agent-coexist/SKILL.md): the short agent
  entry described in [Use it with your agents](#use-it-with-your-agents).
- [Contributing](CONTRIBUTING.md) · [Security policy](SECURITY.md) ·
  [Changelog](CHANGELOG.md) ·
  [CI workflow](https://github.com/RatmmmhSquishyRat/agent-coexist/actions/workflows/ci.yml)

## License

The project's own code and documentation are licensed under the
[MIT License](LICENSE). The vendored copy of `ignore` in
[`vendor/ignore/`](vendor/ignore/COPYING) keeps its upstream copyright
notices and license (Unlicense OR MIT). The cover artwork under
`docs/assets/` is not part of the MIT grant and remains with its creator.
