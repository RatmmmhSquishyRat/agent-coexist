# 等待：wait、wait-post、wait-state

<a id="wait-ready-and-hit"></a>

## wait-post：就绪行与命中报告

bob 在就绪后先发无关评论，再回复 #1。

`agent-coexist wait-post standup --as alice`

stdout：
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

stderr：
```text
waiting  wait-post  standup
```

<a id="wait-file-created-removed"></a>

## wait：文件被创建与被删除

wait 可在文件不存在时启动（上级目录必须存在）。

`agent-coexist wait notes` · 第一个 wait：文件创建命中。

stdout：
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

stderr：
```text
waiting  wait  notes
```

`agent-coexist wait notes` · 第二个 wait：文件删除命中。

stdout：
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

stderr：
```text
waiting  wait  notes
```

<a id="wait-state-hit"></a>

## wait-state：过滤外的变化不结束，解除命中

src 里 bob 已声明 locked；就绪后 carol 声明 observing（过滤外，不结束），bob 移除声明后命中。

`agent-coexist wait-state src --status locked`

stdout：
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

stderr：
```text
waiting  wait-state  src
```

<a id="wait-modified-report"></a>

## wait：对已有内容的修改报告修改前后

就绪后 bob 编辑了自己那条已回复过 alice 的评论。

`agent-coexist wait-post standup --as alice`

stdout：
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

stderr：
```text
waiting  wait-post  standup
```

<a id="wait-byte-only"></a>

## wait：只改字节不改内容

就绪后在文件末尾追加一个空行（手工改动）。

`agent-coexist wait docs/s`

stdout：
```text
ok  wait  docs/s
  bytes         changed
  semantic      unchanged
```

stderr：
```text
waiting  wait  docs/s
```

<a id="wait-cancel"></a>

## 在真实终端按 Ctrl-C 取消等待

就绪后按 Ctrl-C。

`agent-coexist wait standup` · 退出码 130 · Windows：帧内仅控制序列，无可见文字。

stderr：
```text
waiting  wait  standup
```

终端帧（转义视图；末尾没有换行符）：
```text
\x1b[?9001h\x1b[?1004h
```

`agent-coexist wait standup` · 退出码 130 · Linux 终端帧。

终端帧（stderr 直连终端，未单独捕获；转义视图；末尾没有换行符）：
```text
waiting  wait  standup\r
^C
```

<a id="wait-busy-windows"></a>

## Windows：采样时遇到锁或独占打开，以退出 3 结束

就绪后另一进程以同种独占锁锁住文件；或另一程序以不共享方式打开。

`agent-coexist wait standup` · 退出码 3 · E-LOCKED：另一进程持有同种锁。

stdout：
```text
error  wait  standup
  reason        file is locked by another process
  fix           a retried wait starts from a new baseline and does not report changes made while the file was busy, so after its ready line, read the file again; retry shortly: agent-coexist wait standup
```

stderr：
```text
waiting  wait  standup
```

`agent-coexist wait standup` · 退出码 3 · E-INUSE：另一程序以不共享方式打开。

stdout：
```text
error  wait  standup
  reason        file is in use by another program
  fix           a retried wait starts from a new baseline and does not report changes made while the file was busy, so after its ready line, read the file again; close the other program, then retry: agent-coexist wait standup
```

stderr：
```text
waiting  wait  standup
```

<a id="wait-restore-after-ready"></a>

## 就绪后路径被破坏：三种修复指令

`agent-coexist wait-post room/standup --as alice` · 退出码 1 · 上级目录被删除（Windows）。

stdout：
```text
error  wait-post  room/standup
  reason        cannot open room/standup.agent-besties.md: 系统找不到指定的路径。 (os error 3)
  fix           a retried wait starts from a new baseline and does not report changes made while the file could not be read, so after its ready line, read the file again; restore the intended path of room/standup.agent-besties.md: if the directory on it is missing, recreate it, or move it back if it was renamed away; if a file now occupies the directory's place, move that file aside, keeping it; if room/standup.agent-besties.md is a dangling link, restore the file it points at, then retry: agent-coexist wait-post room/standup --as alice
```

stderr：
```text
waiting  wait-post  room/standup
```

`agent-coexist wait-post room/standup --as alice` · 退出码 1 · 上级目录被普通文件占用（Linux）。

stdout：
```text
error  wait-post  room/standup
  reason        cannot open room/standup.agent-besties.md: Not a directory (os error 20)
  fix           a retried wait starts from a new baseline and does not report changes made while the file could not be read, so after its ready line, read the file again; a directory on the path of room/standup.agent-besties.md is now a file; move that file aside, keeping it, and recreate the directory it replaced, then retry: agent-coexist wait-post room/standup --as alice
```

stderr：
```text
waiting  wait-post  room/standup
```

`agent-coexist wait standup` · 退出码 1 · 被监视文件换成同名目录（Linux）。

stdout：
```text
error  wait  standup
  reason        cannot open standup.agent-besties.md: path does not name a regular file
  fix           a retried wait starts from a new baseline and does not report changes made while the file could not be read, so after its ready line, read the file again; the path of standup.agent-besties.md itself no longer names a regular file; move whatever now occupies it aside, keeping it; the watch also accepts the file absent, so nothing needs to be created in its place, then retry: agent-coexist wait standup
```

stderr：
```text
waiting  wait  standup
```
