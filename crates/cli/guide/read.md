# Read the next relevant piece, not the whole workspace

Start with a question: what do you need to decide, change, or verify?
Discover the smallest relevant scope, then read enough evidence to
answer it. A summary is a route to evidence, not a substitute for it.

## A selective reading route

1. **Find files.** `list` shows a directory tree and file types. Choose a
   narrow root or a type filter. Scans are recursive and honor ignore
   rules; an empty result does not prove no collaboration exists.
2. **Inspect structure.** `check` reports one managed file's diagnostics
   and preview. On a brief it also verifies references without printing
   their full contents; this still reads and hashes target bytes.
3. **Locate a message.** `lookfor-comment` returns comment numbers, file
   lines, and excerpts. Read the selected comments with your file tool.
   Filters combine as an intersection, so do not add unrelated filters.
4. **Follow a brief.** Read its description and entry notes first.
   `read-brief --entry` verifies and renders only the selected entry,
   omitting the brief description. It still prints that entry's target
   content: it is not a status-only switch. A plain entry can expand an
   entire source file.
5. **Read the source.** Open only the relevant ranges when full expansion
   would be wasteful. Paths in a brief resolve from the brief's directory,
   not your current directory. Re-check changing evidence before relying
   on it for a consequential decision.

For example, after finding these existing files:

```sh
agent-coexist check docs/onboarding
agent-coexist lookfor-comment docs/standup --mention alice
agent-coexist read-brief docs/onboarding --entry 'parser entry point'
```

Use `check-state` for the selected directory, inherited declarations,
and its subtree. `check path/to/state` inspects only that one file.
Read relevant intent before writing even when the CLI would permit it.

## What brief states establish

- **fresh:** the target hash matches and any required regex matches.
- **shifted:** the target hash changed; any required regex still matches.
- **stale:** the target is missing, or the regex is invalid or does not match.
- **unchecked:** verification could not complete; read the reason.

None establishes that a note is accurate, a design is sound, a test ran,
or a deliverable was accepted. The hash covers the target file, not just
the regex excerpt. Read changed evidence and correct outdated notes
before refreshing hashes; never refresh merely to make warnings vanish.

## Do not mistake partial reading for absence

Inspect `warn`, `busy`, `skipped`, and `partial` rows. Resolve missing
coverage when it matters to your decision. `--include` cannot reopen an
ignore-pruned directory; use `--no-ignore` only deliberately, within the
intended scope. Ordinary file tools can read the Markdown, but they do
not replace the CLI's format and reference checks.

For unread messages and change waits, continue with `guide wait`.
For smaller, clearer artifacts, use `guide write`.
