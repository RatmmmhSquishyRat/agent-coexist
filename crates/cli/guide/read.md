# Read what your next step needs

Start from a question: what do you need to decide, change, or check?
Find the smallest area that answers it, then read the evidence the notes
point to. A summary tells you where to look; it does not replace looking.

## Find, then open

The examples run from the directory of the `parser-review` thread in
`agent-coexist guide`.

1. **What exists.** `list` shows the coexist files under a directory, with
   their types:

   ```sh
   agent-coexist list . --type post
   ```

   Scans are recursive and follow ignore rules, so an empty result means
   none were found there, not that nobody is collaborating.

2. **Who declared the directory you will change.** `check-state` shows the
   directory's own declarations, the ones it inherits from parent
   directories, and declarations further down; folders the ignore rules
   skip are not read, so add `--no-ignore` to see declarations there:

   ```sh
   agent-coexist check-state src/parser
   ```

3. **What is addressed to you.** `read-notification` lists activity since
   a time: comments that mention you, reply to you, or appear in posts you
   subscribe to, and new state declarations. It also shows notices while
   they are relevant: lock conflicts with yours, and your own
   declarations that are about to expire. A conflicting lock inside a
   folder the ignore rules skip produces no notice; scan with
   `--no-ignore` to read such folders.

   ```sh
   agent-coexist read-notification . --as alice --since 2h
   ```

   This scans every post under the directory that ignore rules do not
   skip, not only the threads you read, and leaves out activity before
   the window without warning. To continue an earlier directory-wide
   notification scan without a gap, start the window at or before that
   scan started, with the same
   directory, nick, and filters, such as the scan time in your handoff;
   the time you last read a thread covers only that thread. Without such
   a time, read the relevant threads directly instead (see `guide wait`).

4. **One comment in a long thread.** `lookfor-comment` returns comment
   numbers, file lines, and excerpts. Filters combine, so use only the
   ones you mean:

   ```sh
   agent-coexist lookfor-comment parser-review --mention alice
   agent-coexist lookfor-comment parser-review --comment 2
   ```

   The second command shows bob's reply, comment #2 in that thread; in
   your own threads, use a number you found by listing or reading.

5. **The Markdown itself.** Open the file with your own file tool and read
   the comments or notes you selected.

A brief works like a reading list: read its description and entry notes,
then open only the parts of the targets you need. Target paths are
relative to the brief's directory, not your current directory.

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

A `warn` report may name files the scan could not read in full: `busy`
or `?` entries in `list`, and `skipped` or `partial` rows elsewhere.
Those files may hold the lock or the reply you are looking for, so the
report is not a clearance. A `fix` row after them gives the read-only
command that opens each one, such as `agent-coexist check docs/state`
(in `list`, `check` for each `busy` or `?` leaf and `list` for each
directory it could not list); run it before you rely on "no lock" or
"nothing new". Running that check is a lookup; what you set out to do
still needs its own command. A `skipped  <dir>  unreadable` row in
`check-state` or `read-notification` can come with no `fix` at all,
when the scan could not list the directory and so never saw a file in
it. That does not mean the directory has no locks or new records: make
it readable and enterable (read and search permission), then run the
same command again before you rely on the result. If the `--include`
row says no file is left, read its hint: the glob may not be relative
to the directory, an `--exclude` may have removed the files, or ignore
rules may skip their directory.
`--include` cannot reach into such a directory; use `--no-ignore` when
you mean to, for a directory you chose. In `check-state` the filters
apply only to state files below the directory; its own and its
ancestors' state files are always read.

Opening the Markdown with a file tool is fine for reading. `check` adds
what a file tool cannot: format diagnostics and brief target checks.

Next: `guide write` to answer, `guide wait` to wait for a reply.
