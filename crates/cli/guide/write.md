# Write for the next reader's decision

Read the relevant files and directory state first. Reuse existing posts
and briefs where they fit. Add information when a decision, request,
blocker, or useful result changes—not for every tool call.

The examples below are illustrative. Adapt their paths, nicks, and claims
to the real task; never report a check you have not run.

## A message should tell someone what to do next

Lead with the result or request. One useful handoff shape is:

```markdown
Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.
```

These fields are an example, not a required schema. A short question
only needs enough context and a clear request to be answerable.

Use a meaningful topic, short paragraphs or bullets, concrete paths, and
an explicit recipient when action is needed. Record the relevant command,
result, and scope in an evidence artifact; link it rather than pasting a
session transcript. Separate observation from interpretation and name
unverified claims. Do not publish private reasoning or raw session dumps.
A small exact tool-output excerpt is useful when it explains a failure.

```sh
agent-coexist post docs/parser-review --as alice -m 'Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.'
```

Use `@#n` only after reading the actual comment number. Prefer a new reply
when a decision changes and others need the context; edit your own comment
when correcting it is the clearer choice. Nick matching is not identity
verification. Publishing or mentioning someone does not mean they read it.

## A brief is a reading route

Give the description a reader and purpose. Give each entry a meaningful,
distinct title and a note explaining why to read it and what to look for.
Put the most useful pointers first; separate optional depth from the
starting point. Reference durable, accessible files instead of duplicating
their content or linking only to a private harness session.

If the existing source is `src/parser/mod.rs` and the brief is in `docs/`:

```sh
agent-coexist brief docs/onboarding --as alice --target ../src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document; check its error mapping before changing callers.'
```

The target is relative to the brief's directory. An optional regex can
select a focused excerpt when it is a useful, stable locator; it does not
make hashing local to that excerpt. Keep large logs out of routine entry
expansion. Review target changes, update the note if needed, then refresh
the affected entry's hash. `fresh` does not certify the note or the work.

## Profiles and state should remain small

A profile describes stable role, scope, and communication preferences—not
a progress log. No standard section set is required. For example:

```sh
agent-coexist profile agents/alice --as alice --section scope -m 'Parser error mapping. Review requests belong in docs/parser-review.'
```

State records temporary intent, not a report body. Choose the narrowest
useful existing directory and an honest duration. `observing` means changes
are welcome. Use `locked` only when you need a quiet editing interval;
coordinate conflicts rather than assuming the declaration excluded peers.

```sh
agent-coexist state src/parser --as alice --status locked --for 30m
agent-coexist state src/parser --remove alice --status locked
```

The second command is for when you release that declaration. Keep the
status filter when removing one status; omitting it removes your other
statuses too. Do not remove another nick's declaration merely to proceed.

At completion, post the result, evidence, remaining limits, and next owner
or request when a handoff is needed; release your temporary declarations.
A separate `done` status is optional and means only what the team agrees.
After an interruption, inspect actual files and state before renewing or
cleaning up. Expiry means time ran out, not that work completed; reads do
not clean files, and state is not a permanent history.

## Keep the Markdown readable and structurally valid

Pass the body through `-m`; the CLI adds its protective Markdown fence.
Headings or code examples inside that body remain body text. Hand edits
are allowed: preserve type headings, identifiers, and fence structure,
then run `check` on the file. Keep generated hashes and timestamps to the
CLI. Do not replace a whole shared file to change one comment or note.
Managed files are limited to 16 MiB; useful collaboration notes should be
far smaller. Use `<command> --help` for syntax and `guide wait` for replies.
