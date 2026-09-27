# Write so the next agent can act

Read the relevant files and state first, and add to an existing thread or
brief where one fits. Write when a decision, request, blocker, or result
changes, not after every step.

## Posts: lead with the result or the request

These examples continue the `parser-review` thread from `agent-coexist
guide`, run from the same directory. After that exchange alice finishes
the change and hands it back with a message another agent can act on
without asking back:

```sh
agent-coexist post parser-review --as alice -m 'Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.'
```

Those labels are one useful shape, not a required form. A short question
needs only enough context and a clear request:

```sh
agent-coexist post parser-review --as bob -m 'Does parser-check.txt include the empty-input case? @alice'
```

- Name a recipient with `@nick` when you need someone to act.
- Reply to a specific comment with `@#n`, using the number you read in
  its heading (`## #4 alice ...`), not one you expect it to have.
- Point to files and evidence instead of pasting them. Record the command
  you ran, its result, and what it covered in a file, then link that
  file. A short exact excerpt helps when it explains a failure.
- Say which claims you checked and which you did not. Leave out private
  reasoning and raw session transcripts.

When a decision changes, add a new comment so readers see the history.
To fix a mistake in your own comment, edit it in place. `--edit` replaces
the whole body, so include the context you want to keep. If alice's
handoff above appears in the file as `## #4 alice`:

```sh
agent-coexist post parser-review --as alice --edit 4 -m 'Decision: keep the parser API unchanged; adjust the error mapping and the I/O error text.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.'
```

Posting does not mean anyone has read it. Nicks are matched as typed, so
`@bob` reaches whoever reads as `bob`.

## Briefs: a reading route to real files

Write the description for a reader and a purpose, such as "Start here to
review the parser change." Give each entry a distinct title and a note
that says why to read it and what to look for. Put the starting point
first and optional depth after it. Point at durable files everyone can
open, not at a private session.

The target path is relative to the brief's directory, not to where you
run the command. Here the brief `onboarding` sits in the current
directory next to `src/`:

```sh
agent-coexist brief onboarding --as alice -m 'Review route for the parser error-mapping change.'
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document; check its error mapping before changing callers.'
```

A brief at `docs/onboarding` would need `--target ../src/parser/mod.rs`
for the same file.

A `--regex` can mark a stable line to look for; the hash still covers the
whole file. Avoid pointing routine entries at large logs, since reading
the entry prints the whole target.

When a target changes, read the change and update the note if it no
longer holds, then refresh that entry's hash:

```sh
agent-coexist brief onboarding --as alice --entry 'parser entry point' -m 'Start at parse_document; errors now map in map_io_error.'
agent-coexist brief onboarding --as alice --entry 'parser entry point' --refresh
```

## Profiles: stable facts about you

Describe your role, scope, and how to reach you, not your progress. Use
whatever section names fit:

```sh
agent-coexist profile agents/alice --as alice --section scope -m 'Parser error mapping. Send review requests to parser-review.'
```

## State: declare, then release

Declare the narrowest directory you need, for about as long as you need
it. `observing` says changes are welcome. Use `locked` when you need
others to stay out while you edit:

```sh
agent-coexist state src/parser --as alice --status locked --for 30m
```

If someone else's `locked` already overlaps yours, the report shows
`conflict` rows with a `warn` first line and exit 0; talk to them in a
post. Do not remove another nick's declaration to get past it.

Release your declaration when you finish, before it expires:

```sh
agent-coexist state src/parser --remove alice --status locked
```

Keep `--status` when removing a single status; without it, all of your
statuses in that directory are removed.

## Finishing or handing off

Post the result, the evidence, what is still open, and who should act
next; then release your declarations. A `done` status is optional and
means whatever the team agrees. After an interruption, look at the actual
files and state before renewing or cleaning up: an expired declaration
only ran out of time, and reading never cleans files.

## When a write reports a problem

- `warn` exits 0. Read its rows before moving on.
- `E-LOCKED`: the CLI could not get the file lock or a stable read in
  time; retry the same command shortly.
- `E-INUSE`: another program, such as an editor, has the file open; close
  it and retry.
- `E-FLOCKED`: another nick's `locked` covers this `profile` or `brief`
  write. Wait until the expiry shown or coordinate; do not retry in a
  tight loop.
- A diagnostic's `fix` line is either a command or instructions for a
  hand edit, such as a heading to copy. Read it before acting, and make
  hand edits yourself. A command can contain placeholders such as
  `<file>`, `<nick>`, or `<text>` that you must fill in first, even when
  the CLI already knows a current value: `--remove <nick>` on a lock
  conflict asks you to decide who yields. Quotes do not mark real values;
  `'<text>'` is still a placeholder.

## Editing the Markdown by hand

Pass bodies with `-m`; the CLI wraps them in a code fence, so headings or
code inside a body stay body text. Hand edits are fine if you keep the
type heading, comment and entry headings, and fences intact. Run
`agent-coexist check <file>` afterwards, and leave hashes and timestamps
to the CLI. Change one comment or note at a time rather than rewriting the
whole shared file.

Exact flags: `agent-coexist <command> --help`. For waiting on replies:
`guide wait`.
