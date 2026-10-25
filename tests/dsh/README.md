# dsh tests

dsh (DeepSeek harness, `@deepseek-ai/dsh` on npm) is the first harness this
project aligns with. These scripts check the bundled skill and CLI against
it. They are not part of `cargo test` and not shipped in the crate: they need
Node, a global dsh install and, for the live run, a DeepSeek API key.

## Discovery: no key needed

```sh
node tests/dsh/check-discovery.mjs
```

This runs dsh's own skill discovery code, `FileSystemSkillProvider` from the
installed dsh, against a fresh project with the skill in
`.agents/skills/agent-coexist/`. It prints the catalog entry exactly as the
dsh model sees it, then checks that:

- dsh finds the skill exactly once;
- the description fits dsh's catalog cap, which the script reads from the
  installed dsh rather than hard-coding;
- the guard against creating files merely because the word appears
  survives truncation;
- the description names the harness, session and subagent cases.

dsh shows its model only the name and the first 500 characters of the
description. `whenToUse` is parsed but never shown, and anything past the cap
is cut off without warning.

## Live scenarios

```sh
cargo install --path . --locked --root <dir>     # the build under test
node tests/dsh/run-live.mjs --binary <dir>/bin/agent-coexist[.exe]
node tests/dsh/run-live.mjs --binary ... --only wait-for-reply,explain-only
```

Each scenario builds a small git project of its own, with the skill
installed and a committed fixture. It then runs `dsh --profile headless
--json` in that project with no hint that coexist exists. The outcome is
judged from three things: the files left on disk, the CLI's reading of them,
and the tool calls dsh reports. The model is not asked to grade itself.

The projects are built outside this repository, in neutrally named temp
directories (`proj-XXXXXX`), and the binary is copied to one beside them
(`tools-XXXXXX`). dsh's sandbox lets the agent read anywhere, and in an
early run with fixtures under `tests/dsh/work/` it did. Its project path
named both the repository and the scenario, so the agent left the project,
read the product docs, and then read this runner's oracle for its own
scenario. That same run's lock scenario passed, but in a neutral directory
it failed: the path had been doing the triggering.

| Scenario | Passes when |
|---|---|
| `arrive-respects-lock` | Sent to edit a directory that alice has locked, the agent consults the records, leaves the file alone and tells the user about the lock |
| `reply-to-mention` | Told only its nick, the agent finds alice's question and answers in the thread as carol, as a reply (`@#1` or `@alice`), and the thread still checks clean |
| `guard-unrelated-word` | A task about numpy and pandas that "need to coexist" gets done, and no coexist file is created |
| `explain-only` | Asked what agent-coexist is, the agent answers and writes nothing |
| `wait-for-reply` | Asked to wait for bob's reply, which arrives 150 s in, after dsh's 120 s shell-call budget, the agent reports what bob said and leaves no `agent-coexist` process running. Whether it used `--timeout` is recorded, not required |
| `handoff-across-sessions` | One session does step 1, is told a change to step 2 that only it hears, and leaves a coexist record carrying it. A fresh session, told only to pick up the work, reads that record and either applies the change or puts it to the user. Silently dropping the change fails. A record is another agent's note, not the user's instruction, so declining a change that contradicts TASK.md is right as long as the user hears about it |
| `resumed-session-keeps-nick` | A session resumed with `--session-id` posts under the same nick as before |

dsh runs in its default mode, the one a headless user gets: a
`workspace-write` sandbox rooted at the project, and approvals that are
rejected when nobody is there to answer. On Windows the shell tool is pwsh,
with a 120 s default call budget. A call that runs longer is moved to the
background rather than stopped.

Evidence for each scenario goes to `tests/dsh/work/<run>/<scenario>/`:

- `first.events.ndjson` and `second.events.ndjson`, dsh's event streams;
- `stderr.txt` and `final.txt`;
- `git-status.txt`, which lists exactly what the agent changed;
- `project/`, a copy of the project as the agent left it;
- `verdict.json`, which also records the temp path the project ran in.

After changing an oracle, score a finished run again without spending
tokens:

```sh
node tests/dsh/run-live.mjs --binary ... --rejudge tests/dsh/work/live-...
```

This writes `verdict.rejudged.json` next to each verdict and reports which
verdicts changed.

On some networks dsh's `web_fetch` refuses every host. That happens when DNS
returns non-public addresses, as a fake-IP proxy does: dsh's guard against
private-network fetches then blocks them all. An agent there cannot open the
skill's GitHub links, so the local CLI and guide are its only references.

The run's `summary.md` lists each verdict. dsh also keeps each run as a
session in its own store under `~/.dsh/sessions`.

With no API key, dsh stops at once with `MISSING_CREDENTIAL`. The runner
then prints `BLOCKED` and exits 2. Set `DEEPSEEK_API_KEY` in the launching
environment, or store the key on dsh's web Models page.

## Checking the oracles: no key needed

```sh
node tests/dsh/run-live.mjs --binary ... --self-test
```

For every scenario this builds the project and plays scripted agents: one
that does the right thing, and one or more that do the wrong thing. The
oracle must pass the first and fail the rest. Two of the wrong agents exist
because of a real oracle bug. Both work only by absolute paths, which
contain `agent-coexist`: `blind` reads only the source file, and `poller`
sleeps and reads instead of waiting. An earlier matcher counted any such
path as a coexist action and passed both. Run the self-test after changing
a scenario, so that a live failure means the agent and not the test.
