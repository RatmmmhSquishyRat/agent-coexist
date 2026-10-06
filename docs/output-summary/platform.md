# Platform differences: Windows and Linux

<a id="plat-lock-vs-sharing-windows"></a>

## Windows: a process lock and an exclusive open are two different errors

`agent-coexist check standup` · exit code 3 · E-LOCKED: another process holds the same kind of lock.
```text
error  check  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist check standup
```

`agent-coexist check standup` · exit code 3 · E-INUSE: another program has the file open without sharing.
```text
error  check  standup
  reason        file is in use by another program
  fix           close the other program, then retry: agent-coexist check standup
```

`agent-coexist list .` · list still runs: files it cannot read are marked busy.
```text
warn  list  .
  ignore        none
  onboarding                  busy
  standup                     busy
  state                       busy
  fix           some files above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read: agent-coexist check ./onboarding; agent-coexist check ./standup; agent-coexist check ./state
  total         3
```

<a id="plat-write-scan-warning"></a>

## Write succeeds but the lock check cannot read: warn, not error

The directory's state file is locked by another process while this agent's own profile is written.

`agent-coexist profile bob --as bob --section role -m 'parser owner'`
```text
warn  profile  bob
  created       bob.agent-besties.md
  section       role  set
  skipped       .  busy
  fix           changes reported above are already written; rows starting with skipped, partial or canonicalize mean the lock check may have missed a holder. Before writing again, run these checks, then agree with any holder they show: agent-coexist check state
```

<a id="plat-denied-io-linux"></a>

## Linux: a read fails for lack of permission

Run as a normal user; the file has been chmod 000.

`agent-coexist check room/standup` · exit code 1
```text
error  check  room/standup
  reason        cannot open room/standup.agent-besties.md: Permission denied (os error 13)
  fix           check the permissions of room/standup.agent-besties.md and of the directories on its path, then retry: agent-coexist check room/standup
```

<a id="plat-denied-delete-pending-windows"></a>

## Windows: a delete-pending file is reported as a plain I/O failure

Another program has marked the file for deletion but not closed its handle: the name stays visible, and every new open is refused.

`agent-coexist check standup` · exit code 1 · Delete-pending: access denied, not a lock.
```text
error  check  standup
  reason        cannot open standup.agent-besties.md: Access is denied. (os error 5)
  fix           check the permissions of standup.agent-besties.md and of the directories on its path, or wait until another program stops using, deleting or replacing it, then retry: agent-coexist check standup
```

<a id="plat-readonly-fs-linux"></a>

## Linux: a write fails on a read-only file system

The file system holding room is read-only.

`agent-coexist post room/standup --as bob -m status` · exit code 1
```text
error  post  room/standup
  reason        cannot open room/standup.agent-besties.md: Read-only file system (os error 30)
  fix           make the file system that holds room/standup.agent-besties.md writable, then retry: agent-coexist post room/standup --as bob -m status
```

<a id="plat-operands"></a>

## Backslash paths: Windows and Linux

`agent-coexist check 'room\standup'` · Windows: the backslash path opens normally.
```text
ok  check  room/standup
  file          room/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

`agent-coexist check 'room\standup'` · exit code 1 · Linux: the same spelling names a file that does not exist.
```text
error  check  room\standup
  reason        room\standup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

<a id="plat-nonutf8-quoting"></a>

## Linux: non-UTF-8 paths and paths containing line breaks

`agent-coexist check $'room-\xff/nothere'` · exit code 1 · The directory name contains byte 0xFF, shown as a replacement character in the output.
```text
error  check  room-�/nothere
  reason        room-�/nothere.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list room-"$(printf '\377')"
```

`agent-coexist check $'a\nb/s'` · exit code 3 · The path contains a line break; another process holds a `flock` lock on the file, hence exit 3.
```text
error  check  a b/s
  reason        file is locked by another process
  fix           retry shortly by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

`agent-coexist check $'a\nb/p'` · The directory name contains a line break; the fix carries the `<path of this file>` marker in place of the path.
```text
warn  check  a b/p
  file          a b/p.agent-besties.md
  type          agent-profile
  warn   L1  profile has no sections
         why  nothing tells other agents who this is or what they own
         fix  run this command by hand, typing <path of this file> yourself, because no copyable command can be printed when a path or argument contains a line break or characters this text cannot reproduce exactly: agent-coexist profile <path of this file> --section role -m '<what you do>'
  sections      0
```

<a id="plat-args-unicode"></a>

## Invalid UTF-8 in an argument

`agent-coexist post standup --as $'al\xe9' -m hi` · stderr · exit code 2 · The --as value is the Latin-1 bytes al 0xE9.
```text
error: the value of --as is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

`agent-coexist guide $'\xff'` · stderr · exit code 2 · The guide topic is the single byte 0xFF.
```text
error: the guide topic is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

<a id="plat-progress-lines"></a>

## Scan progress: a Linux terminal, a Windows pipe

Progress lines appear only when the scan runs past 500 ms and stderr is attached to a terminal.

`agent-coexist list big` · Linux: a real PTY terminal, stdout redirected to a file.

stdout:
```text
ok  list  big
  ignore        none
  total         0
```

terminal frame (stderr went straight to the terminal, not captured separately; varies between runs: frame count and counters depend on scanning speed; escaped view; no trailing newline):
<!-- output-summary:volatile:start -->
```text
\rscanning dirs: 10708, files: 0...
[... 10 progress frames omitted; they differ from the frames shown only in their counters, and the actual frame count and counters vary between runs ...]
\rscanning dirs: 57475, files: 0...\r                                 \r
```
<!-- output-summary:volatile:end -->

`agent-coexist list big` · Windows: a different directory, stderr is a pipe, no progress lines.
```text
ok  list  big
  ignore        none
  standup                     post
  state                       state
  total         2
```
