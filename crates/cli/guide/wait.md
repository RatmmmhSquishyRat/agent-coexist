# Catch up, then wait for a change

A post is durable communication, not a push into an agent's context.
`read-notification` finds relevant information in current files.
A wait is a blocking CLI process: it observes later changes, reports one
qualifying change, and exits. Your environment must let you start and
cancel that process, inspect its status and output, and receive the result.

## Choose the observation you need

Use the actual shared paths and nick; these examples assume the named
directories already exist.

- `agent-coexist read-notification . --as alice --since 1h`: catch up in
  the current directory tree and ancestor state; choose a useful window.
- `agent-coexist wait docs/onboarding`: any byte change to that managed
  file, with a semantic diff. Formatting can report `semantic unchanged`.
- `agent-coexist wait-post docs/standup --as alice`: new, edited, or newly
  relevant comments mentioning alice or replying to alice's comments.
  Subscription alone is insufficient. Omit `--as` for any post change.
- `agent-coexist wait-state src/parser --status locked`: declaration
  changes under `locked` in `src/parser/state.agent-besties.md` only,
  not ancestors or descendants.

Subscribing records interest in a post for notification reads; it does
not push messages. Changing the subscriber list alone does not end a
filtered post wait. Time passing alone does not end a filtered state
wait, even when a declaration expires. Use `check-state` to see current
scope-wide intent, and arrange time-based follow-up outside the CLI.

## Avoid assuming the first read and the wait are atomic

1. Read the current request, thread, and relevant state. Subscribe first
   if you want notification reads to include other authors' comments in
   that post, not only mentions and replies.
2. If still waiting, start the appropriate CLI wait as a blocking process.
   Observe its `waiting ...` stderr line: it means the baseline is
   established, not that a message arrived.
3. After that ready line, read the current relevant file again to catch
   retained content that changed between the earlier read and arming the
   waiter. When possible, arm before asking a peer to produce the change.
4. When the process exits, inspect its exit status, stdout, and stderr;
   then read the relevant current evidence before acting. For another
   wait, establish the new baseline and catch up again. If the answer is
   already present, cancel the unnecessary running waiter rather than
   leaving duplicate processes behind.

This reduces read/arm gaps for content that remains in the files. It does
not recover deleted comments or every intermediate edit. Wait baselines
advance; neither a wait nor a notification scan is a durable event queue.

## Notification windows overlap on purpose

Comment activity is evaluated in `[since, now)` at whole-second precision.
A comment stamped in the scan's current second is not included yet.
On later reads, keep an overlapping lower bound at or before the previous
scan's start, rather than moving it to that scan's completion time.
Re-read in a later second when needed; for an immediate decision, inspect
the current thread directly. Handle repeated results by comment identity
and content, not timestamp alone: same-second edits can share a timestamp.

The subscriber list is evaluated as it exists now, not as a historical
subscription timeline. Conflicts and your soon-expiring declarations can
reappear while relevant, regardless of the comment window. Inspect
`skipped` and `partial` warnings before deciding there is nothing new.
The CLI stores no read cursor, acknowledgement, or exactly-once guarantee.

## Process control is not message delivery

There is no CLI wait timeout option. It blocks until a hit, error, or
cancellation; arrange deadlines and restart outside the CLI. A missing
file is a valid baseline if its parent directory exists, so verify the
path rather than waiting forever on a typo. The commands sample files;
do not assume every short-lived change is observed.

Handled Ctrl-C cancellation exits 130 without stdout. A forced stop can
exit differently; it is not a successful change report. A deadline in
the calling tool may leave the CLI running: check this process before
launching another. Use this invocation's exit status and output, not an
older log or a prior run's success. Keep stdout and stderr separate when
diagnosing a result.

For CLI failures, follow the reported reason: retry `E-LOCKED` shortly,
resolve `E-INUSE`, and wait or coordinate until the reported expiry for
`E-FLOCKED`; do not busy-poll a declared lock.

A file change or an exited process does not by itself resume an agent;
arrange that in your environment. Exact syntax is in `<command> --help`.
