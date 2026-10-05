# 平台差异：Windows 与 Linux

<a id="plat-lock-vs-sharing-windows"></a>

## Windows：进程锁与独占打开是两种错误

`agent-coexist check standup` · 退出码 3 · E-LOCKED：另一进程持有同种锁。
```text
error  check  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist check standup
```

`agent-coexist check standup` · 退出码 3 · E-INUSE：另一程序以不共享方式打开。
```text
error  check  standup
  reason        file is in use by another program
  fix           close it, then retry: agent-coexist check standup
```

`agent-coexist list .` · list 继续：读不到的叶子标为 busy。
```text
warn  list  .
  ignore        none
  onboarding                  busy
  standup                     busy
  state                       busy
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read: agent-coexist check ./onboarding; agent-coexist check ./standup; agent-coexist check ./state
  total         3
```

<a id="plat-write-scan-warning"></a>

## 写入成功但写后核对读不到：warn 而不是 error

目录的 state 文件被另一进程锁住时写入自己的档案。

`agent-coexist profile bob --as bob --section role -m 'parser owner'`
```text
warn  profile  bob
  created       bob.agent-besties.md
  section       role  set
  skipped       .  busy
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check state
```

<a id="plat-denied-io-linux"></a>

## Linux：权限不足的读取失败

普通用户运行；文件已 chmod 000。

`agent-coexist check room/standup` · 退出码 1
```text
error  check  room/standup
  reason        cannot open room/standup.agent-besties.md: Permission denied (os error 13)
  fix           check the permissions of room/standup.agent-besties.md and of the directories on its path, then retry: agent-coexist check room/standup
```

<a id="plat-denied-delete-pending-windows"></a>

## Windows：待删除的文件报告为普通 I/O 失败

另一程序对文件设置删除标记但句柄未关闭：文件名仍可见，新打开都被拒绝。

`agent-coexist check standup` · 退出码 1 · 待删除状态：拒绝访问，不是锁。
```text
error  check  standup
  reason        cannot open standup.agent-besties.md: 拒绝访问。 (os error 5)
  fix           check the permissions of standup.agent-besties.md and of the directories on its path, or wait until another program stops using, deleting or replacing it, then retry: agent-coexist check standup
```

<a id="plat-readonly-fs-linux"></a>

## Linux：只读文件系统上的写入失败

room 所在文件系统只读。

`agent-coexist post room/standup --as bob -m status` · 退出码 1
```text
error  post  room/standup
  reason        cannot open room/standup.agent-besties.md: Read-only file system (os error 30)
  fix           make the file system that holds room/standup.agent-besties.md writable, then retry: agent-coexist post room/standup --as bob -m status
```

<a id="plat-operands"></a>

## 反斜杠路径：Windows 与 Linux

`agent-coexist check 'room\standup'` · Windows：反斜杠路径正常打开。
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

`agent-coexist check 'room\standup'` · 退出码 1 · Linux：同一写法指向不存在的文件。
```text
error  check  room\standup
  reason        room\standup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

<a id="plat-nonutf8-quoting"></a>

## Linux：非 UTF-8 与含换行的路径

`agent-coexist check $'room-\xff/nothere'` · 退出码 1 · 目录名含字节 0xFF，输出中显示为替换字符。
```text
error  check  room-�/nothere
  reason        room-�/nothere.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list room-"$(printf '\377')"
```

`agent-coexist check $'a\nb/s'` · 退出码 3 · 路径含换行；另一进程正持有 flock，故退出 3。
```text
error  check  a b/s
  reason        file is locked by another process
  fix           retry shortly by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

`agent-coexist check $'a\nb/p'` · 目录名含换行；fix 中路径为 `<path of this file>` 标记。
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

## 参数或环境变量不是有效 Unicode / UTF-8

`agent-coexist post standup --as $'al\xe9' -m hi` · stderr · 退出码 2 · --as 的值是 Latin-1 字节 al 0xE9。
```text
error: the value of --as is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

<a id="plat-progress-lines"></a>

## 扫描进度：Linux 终端、Windows 管道

扫描超过 500 ms 且 stderr 接终端时才出现进度行。

`agent-coexist list big` · Linux：真实 PTY 终端，stdout 重定向到文件。

stdout：
```text
ok  list  big
  ignore        none
  total         0
```

终端帧（stderr 直连终端，未单独捕获；随时序变化：进度帧的条数和计数取决于扫描速度；转义视图；末尾没有换行符）：
<!-- runtime-examples:volatile:start -->
```text
\rscanning dirs: 28982, files: 0...\rscanning dirs: 40380, files: 0...\rscanning dirs: 51300, files: 0...\r                                 \r
```
<!-- runtime-examples:volatile:end -->

`agent-coexist list big` · Windows：另一目录，stderr 是管道，无进度行。
```text
ok  list  big
  ignore        none
  standup                     post
  state                       state
  total         2
```
