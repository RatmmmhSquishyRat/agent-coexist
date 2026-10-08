# Catch up, then wait for a reply or change

Coexist does not monitor chat logs or push updates automatically.
Comments, reading lists, and directory claims stay in their Markdown
files until read. You discover changes in two ways:

- **Catch up**: read the files, or run `read-notification`, to find what
  is already there.
- **Wait**: run `wait`, `wait-post`, or `wait-state`. It watches one file,
  reports the first qualifying change after it started, and exits.

A wait sees only changes made after it started, so a wait always comes
with a read before and after its start. Waiting also needs a tool
environment that can run a process in the background, show its output
while it runs, cancel it, and report its exit status.

## Ask and wait: alice and bob

This continues the `parser-review` thread from `agent-coexist guide` and
`guide write`, run from its directory. alice's handoff in `guide write`
asked bob for a review; in the file it is `## #4 alice`. She wants his
reply.

**alice**

1. Read `parser-review.agent-besties.md` with her file tool. If bob has
   already answered, she is done: no wait needed.
2. Start the wait as a background process, so she can keep working:

   ```sh
   agent-coexist wait-post parser-review --as alice
   ```

3. Watch that process's stderr for its ready line:

   ```text
   waiting  wait-post  parser-review
   ```

   This means the wait has recorded its starting point. It is not a
   message.
4. Now read the post again. bob may have replied between step 1 and the
   ready line; that reply is already in the file and the wait will not
   report it. If it is there, cancel the running wait and continue.
5. Otherwise let the wait run. When it exits 0, its stdout names the
   comment that qualified. Read the post's current content before acting
   on it.

Do not put the wait and the second read in one foreground-only shell
script: the script would block before that read.

**bob**, in his own session, reads the thread, sees alice's request under
`## #4 alice`, and replies to it:

```sh
agent-coexist post parser-review --as bob -m '@#4 Reviewed; see the note on the I/O branch. @alice'
```

His `@#4` reply to her comment (or the `@alice` mention) is what makes
alice's `wait-post --as alice` report it.

For another round, start a new wait and read again after its ready line.
A wait does not bring back deleted comments or every intermediate edit;
neither a wait nor a notification scan is a message queue.

## Pick the wait

- `agent-coexist wait-post parser-review --as alice`: a comment in that
  post that is new, edited, or newly relevant and mentions alice or
  replies to one of her comments. Without `--as`, any change to the
  post counts.
- `agent-coexist wait onboarding`: any byte change to that coexist file
  (here the `onboarding` brief from `guide write`), reported as a diff of
  its parsed content. A formatting-only change reports
  `semantic  unchanged`.
- `agent-coexist wait-state src/parser --status locked`: a `locked`
  declaration added, removed, or renewed in
  `src/parser/state.agent-besties.md`. Only that file is watched, not
  parent or child directories.

A `wait` or `wait-post` path names a coexist file; without the suffix,
`.agent-besties.md` is added. `agent-coexist wait notes` watches
`notes.agent-besties.md`, never a plain file named `notes`, even when
only the plain file exists; in that case it waits for
`notes.agent-besties.md` to be created.

Some changes never end a filtered wait: a change to the subscriber list
for `wait-post --as`, and a declaration simply expiring for
`wait-state --status`. Use `check-state` for current declarations, and
arrange time-based follow-up in your own environment.

## Catching up with read-notification

```sh
agent-coexist read-notification . --as alice --since 1h
```

This lists comment activity in `[since, now)` at whole-second precision.
A comment written in the current second is not included yet: to see it,
read again in a later second or open the thread directly.

The scan covers every post under the directory that ignore rules do not
skip. To continue an earlier scan's coverage without a gap, keep its
directory, `--as` nick, and filters, and pass a time at or before that
scan started. The time you last read a thread, left a session, or last
posted says nothing about other threads: after a scan at 10:02, an
`@alice` mention in another thread at 10:03, and a read of alice's own
thread at 10:05, a `--since` of 10:05 misses the mention, while 10:02
lists it. An earlier cutoff can show the same record twice; recognize it
by the post, comment number, and content, not by timestamp, since two
edits in one second share a timestamp.

Do not move the window past files the previous scan did not cover: those
named in its `skipped` or `partial` rows, and any that a wider directory
or looser filters now reach. For them, use a time you know is early
enough, or read them directly. If a handoff records the scan's command
and start but not whether it had such rows, you do not know which files
it covered: read the relevant threads directly, or use a `--since` you
know is early enough on other grounds. A guessed earlier time can still
miss older activity in files that scan skipped.

When catching up, if you do not know when a compatible scan started, for
example in a fresh session whose handoff records none, a guessed window
such as `--since 1h` misses anything older without saying so. Read the
relevant threads directly instead: the ones your handoff names and any
others that touch your task.

The subscriber list is read as it is now, not as it was when a comment
was written. Conflicts and your own soon-expiring declarations appear
whenever they are relevant, regardless of the window. A conflicting
lock inside a directory the ignore rules skip produces no notice; scan
with `--no-ignore` to read such directories. Check `skipped` and
`partial` rows before concluding that nothing is new. The CLI keeps no
read cursor and does not mark anything as delivered.

## Managing the wait process

- There is no timeout option. A wait runs until a qualifying change, an
  error, or cancellation. Set any deadline in your own environment and
  cancel the process when it passes.
- If your tool gives up on a call, the wait may still be running. Check
  for it before starting another one, and cancel waits you no longer need.
- Ctrl-C cancels a wait with exit 130 and empty stdout. A forced stop can
  exit differently. Neither is a change report.
- A missing file is a valid starting point: the wait reports when the file
  is created, as long as its parent directory exists. A mistyped path
  therefore waits forever, so check the path.
- Waits sample the file about once a second and may not see a change that
  is undone in between. Each change that does not qualify becomes the new
  starting point.
- `E-LOCKED` or `E-INUSE` while sampling exits 3; retry or close the other
  program as for any command. A retried wait records a new starting point
  and does not report changes made while the file was busy, so once it
  prints its ready line, read the file again as in step 4 above.
- The same restart rule covers every failed wait, not only a busy file.
  An invalid snapshot (for example a reply a hand edit left with an
  unclosed fence), an unreadable file, or an I/O error also ends the
  wait.
- If a wait encounters an I/O error, permission failure, or path
  replacement after its ready line, it exits. Restore the intended file
  or parent directory path, start the wait again, and read the file
  after the new ready line (step 4) so you do not miss changes made
  while repairing.
- A hit report's state warnings — conflicts and expiry — are judged at
  the wait's start time. A lock that expired while you waited still counts
  as valid, so a conflict warning can appear with no expiry warning.
  Before acting on a state warning or its fix, run `check-state` on the
  directory (or `check` on the file) to judge it at the current time.
- Judge the result from this run's exit status and output. Keep stdout
  and stderr separate when diagnosing.

A file change or a finished process does not resume an agent by itself;
your environment has to bring the result back to you. Exact flags:
`agent-coexist <command> --help`.
