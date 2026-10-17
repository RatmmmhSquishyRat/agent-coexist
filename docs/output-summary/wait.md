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

<a id="wait-file-lifecycle"></a>

## wait: file creation, removal, and semantic changes

A wait may start while the file does not exist yet (the parent directory must exist).

`agent-coexist wait notes` · First wait: the file's creation is the hit.

stdout:
```text
ok  wait  notes
  bytes         changed
  semantic      changed
  created       notes.agent-coexist.md
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
  removed       notes.agent-coexist.md
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

`agent-coexist wait docs/s` · Bytes change but content does not: semantic unchanged.

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

<a id="wait-busy-windows"></a>

## Windows: a conflicting exclusive lock found at sampling time ends with exit 3

After the ready line, another process locks the file with a conflicting exclusive lock.

`agent-coexist wait standup` · exit code 3 · E-LOCKED: another process holds a conflicting exclusive lock.

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

<a id="wait-restore-after-ready"></a>

## The path breaks after the ready line: parent directory replaced by a file (Linux)

`agent-coexist wait-post room/standup --as alice` · exit code 1 · A regular file now occupies the parent directory's place (Linux).

stdout:
```text
error  wait-post  room/standup
  reason        cannot open room/standup.agent-coexist.md: Not a directory (os error 20)
  fix           a directory on the path of room/standup.agent-coexist.md is now a file; move that file aside, keeping it, and recreate the directory it replaced; restart with this command, then read the file again after its ready line (the new wait cannot report changes made while it was stopped): agent-coexist wait-post room/standup --as alice
```

stderr:
```text
waiting  wait-post  room/standup
```
