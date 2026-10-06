# Waiting: wait, wait-post, and wait-state

<a id="wait-ready-and-hit"></a>

## wait-post: the ready line and the hit report

After the ready line, bob posts an unrelated comment first, then replies to #1.

`agent-coexist wait-post standup --as alice`

stdout:
```text
ok  wait-post  standup
  bytes         changed
  semantic      changed
  add     comment  #3
  author
    | bob
  created
    | 2026-09-17T10:00:00Z
  edited
    | none
  body
    | @#1 parser done
```

stderr:
```text
waiting  wait-post  standup
```

<a id="wait-file-created-removed"></a>

## wait: a file created and a file removed

A wait may start while the file does not exist yet (the parent directory must exist).

`agent-coexist wait notes` · First wait: the file's creation is the hit.

stdout:
```text
ok  wait  notes
  bytes         changed
  semantic      changed
  created       notes.agent-besties.md
  add     comment  #1
  author
    | alice
  created
    | 2026-09-17T10:00:00Z
  edited
    | none
  body
    | first note
```

stderr:
```text
waiting  wait  notes
```

`agent-coexist wait notes` · Second wait: the file's removal is the hit.

stdout:
```text
ok  wait  notes
  bytes         changed
  semantic      changed
  removed       notes.agent-besties.md
  remove  comment  #1
  author
    | alice
  created
    | 2026-09-17T10:00:00Z
  edited
    | none
  body
    | first note
```

stderr:
```text
waiting  wait  notes
```

<a id="wait-state-hit"></a>

## wait-state: out-of-filter changes do not end it; a removal does

src has bob's `locked` declaration; carol's `observing` declaration is added after the ready line (outside the filter), then bob's declaration is removed.

`agent-coexist wait-state src --status locked`

stdout:
```text
ok  wait-state  src
  bytes         changed
  semantic      changed
  remove  locked  bob
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr:
```text
waiting  wait-state  src
```

<a id="wait-modified-report"></a>

## wait-post: a comment edit shows before and after

After the ready line, bob edits his own comment, the one that had replied to alice.

`agent-coexist wait-post standup --as alice`

stdout:
```text
ok  wait-post  standup
  bytes         changed
  semantic      changed
  update  comment  #2  edited
  before
    | none
  after
    | 2026-09-17T10:05:00Z
  update  comment  #2  body
  before
    | @#1 parser done
  after
    | @#1 parser done; tests pass
```

stderr:
```text
waiting  wait-post  standup
```

<a id="wait-brief-entry-added"></a>

## wait: a brief gains an entry

After the ready line, alice adds the `parser entry point` entry to `onboarding`, pointing to `src/parser/mod.rs` with a regex and note.

`agent-coexist wait onboarding`

stdout:
```text
ok  wait  onboarding
  bytes         changed
  semantic      changed
  add     entry  parser entry point
  path
    | src/parser/mod.rs
  hash
    | c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  regex
    | fn parse_document
  note
    | Start at parse_document.
```

stderr:
```text
waiting  wait  onboarding
```

<a id="wait-profile-section-updated"></a>

## wait: a profile section changes

After the ready line, alice updates the `role` section in `agents/alice` to include ownership of `crates/cli`.

`agent-coexist wait agents/alice`

stdout:
```text
ok  wait  agents/alice
  bytes         changed
  semantic      changed
  update  section  role  body
  before
    | Rust implementer. Owns crates/core.
  after
    | Rust implementer. Owns crates/core and crates/cli.
```

stderr:
```text
waiting  wait  agents/alice
```

<a id="wait-byte-only"></a>

## wait: bytes change, content does not

After the ready line, a blank line is appended at the end of the file (a manual edit).

`agent-coexist wait docs/s`

stdout:
```text
ok  wait  docs/s
  bytes         changed
  semantic      unchanged
```

stderr:
```text
waiting  wait  docs/s
```

<a id="wait-cancel"></a>

## Cancel a wait with Ctrl-C in a real terminal

Ctrl-C is pressed after the ready line.

`agent-coexist wait standup` · exit code 130 · Windows: the frame holds control sequences only, with no visible text.

stderr:
```text
waiting  wait  standup
```

terminal frame (escaped view; no trailing newline):
```text
\x1b[?9001h\x1b[?1004h
```

`agent-coexist wait standup` · exit code 130 · Linux terminal frame.

terminal frame (stderr went straight to the terminal, not captured separately; escaped view; no trailing newline):
```text
waiting  wait  standup\r
^C
```

<a id="wait-busy-windows"></a>

## Windows: a lock or an exclusive open found at sampling time ends with exit 3

After the ready line, another process locks the file with the same kind of exclusive lock; or another program opens it without sharing.

`agent-coexist wait standup` · exit code 3 · E-LOCKED: another process holds the same kind of lock.

stdout:
```text
error  wait  standup
  reason        file is locked by another process
  fix           wait briefly; restart with this command, then read the file again after its ready line (the new wait cannot report changes made while it was stopped): agent-coexist wait standup
```

stderr:
```text
waiting  wait  standup
```

`agent-coexist wait standup` · exit code 3 · E-INUSE: another program has the file open without sharing.

stdout:
```text
error  wait  standup
  reason        file is in use by another program
  fix           close the other program; restart with this command, then read the file again after its ready line (the new wait cannot report changes made while it was stopped): agent-coexist wait standup
```

stderr:
```text
waiting  wait  standup
```

<a id="wait-restore-after-ready"></a>

## The path breaks after the ready line: three fix directives

`agent-coexist wait-post room/standup --as alice` · exit code 1 · Windows: the parent junction is moved away after ready; the post remains in its backing directory.

stdout:
```text
error  wait-post  room/standup
  reason        cannot open room/standup.agent-besties.md: The system cannot find the path specified. (os error 3)
  fix           restore the intended path of room/standup.agent-besties.md; if the directory on it is missing, recreate it, or move it back if it was renamed away; if a file now occupies the directory's place, move that file aside, keeping it; if room/standup.agent-besties.md is a dangling link, restore the file it points at; restart with this command, then read the file again after its ready line (the new wait cannot report changes made while it was stopped): agent-coexist wait-post room/standup --as alice
```

stderr:
```text
waiting  wait-post  room/standup
```

`agent-coexist wait-post room/standup --as alice` · exit code 1 · A regular file now occupies the parent directory's place (Linux).

stdout:
```text
error  wait-post  room/standup
  reason        cannot open room/standup.agent-besties.md: Not a directory (os error 20)
  fix           a directory on the path of room/standup.agent-besties.md is now a file; move that file aside, keeping it, and recreate the directory it replaced; restart with this command, then read the file again after its ready line (the new wait cannot report changes made while it was stopped): agent-coexist wait-post room/standup --as alice
```

stderr:
```text
waiting  wait-post  room/standup
```

`agent-coexist wait standup` · exit code 1 · The watched file is now a directory of the same name (Linux).

stdout:
```text
error  wait  standup
  reason        cannot open standup.agent-besties.md: path does not name a regular file
  fix           the path of standup.agent-besties.md itself no longer names a regular file; move whatever now occupies it aside, keeping it; the watch also accepts the file absent, so nothing needs to be created in its place; restart with this command, then read the file again after its ready line (the new wait cannot report changes made while it was stopped): agent-coexist wait standup
```

stderr:
```text
waiting  wait  standup
```
