# Read what your next step needs

Start from a question: what do you need to decide, change, or check?
Find the smallest area that answers it, then read the evidence the notes
point to. A summary tells you where to look; it does not replace looking.

## Find, then open

1. **What exists.** `list` shows managed files under a directory, with
   their types. Scans are recursive and follow ignore rules, so an empty
   result means none were found there, not that nobody is collaborating.
2. **What is addressed to you.** `read-notification` lists activity since
   a time (comments that mention you, reply to you, or appear in posts you
   subscribe to, and new state declarations), plus notices that appear
   whenever they are currently relevant: lock conflicts with yours and
   your own declarations that are about to expire.
3. **One comment in a long thread.** `lookfor-comment` returns comment
   numbers, file lines, and excerpts. Filters combine, so use only the
   ones you mean.
4. **The Markdown itself.** Open the file with your own file tool and read
   the comments or notes you selected.
5. **A brief's targets.** Read the brief's description and entry notes,
   then open only the parts of the targets you need. Target paths are
   relative to the brief's directory, not your current directory.

Run from the directory of the `parser-review` thread in `agent-coexist
guide`:

```sh
agent-coexist list . --type post
agent-coexist read-notification . --as alice --since 2h
agent-coexist lookfor-comment parser-review --mention alice
agent-coexist lookfor-comment parser-review --comment 2
```

The last command shows bob's reply, comment #2 in that thread; in your
own threads, use a number you found by listing or reading.

Before you change a shared directory, check who has declared intent for
it. `check-state` shows the directory's own declarations, the ones it
inherits from parent directories, and declarations further down:

```sh
agent-coexist check-state src/parser
```

## Check a brief before trusting it

`check` reports a brief's structure and each entry's state without
printing target contents. For the `onboarding` brief created in
`guide write`:

```sh
agent-coexist check onboarding
```

- **fresh:** the target's bytes still match the recorded hash, and any
  regex still matches.
- **shifted:** the target changed since the hash was recorded; the regex,
  if any, still matches.
- **stale:** the target is missing, or its regex is invalid or no longer
  matches.
- **unchecked:** the check could not complete; the row says why.

The hash covers the whole target file, not only the regex excerpt. None
of these states says the note is accurate or the work is right: `fresh`
means only that the bytes are unchanged. When an entry is `shifted`, read
what changed and update the note before refreshing its hash.

`read-brief` prints entry content. With `--entry` it prints one entry's
target, which can still be a whole source file:

```sh
agent-coexist read-brief onboarding --entry 'parser entry point'
```

## Notice what a scan skipped

A `warn` report may include `busy`, `skipped`, or `partial` rows: files
the scan could not read in full. If one of them matters to your decision,
read it again or open it directly. `--include` cannot reach into a
directory that ignore rules pruned; use `--no-ignore` when you mean to,
for a directory you chose.

Opening the Markdown with a file tool is fine for reading. `check` adds
what a file tool cannot: format diagnostics and brief target checks.

Next: `guide write` to answer, `guide wait` to wait for a reply.
