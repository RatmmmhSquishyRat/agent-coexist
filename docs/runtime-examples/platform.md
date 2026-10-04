# 平台差异：Windows 与 Linux

同样的系统条件在两个平台上各有自己的报告：进程锁与独占打开的区分（E-LOCKED、E-INUSE）、权限不足与待删除文件的普通 I/O 失败、路径写法与大小写、非 UTF-8 名称的显示与恢复命令写法、非 Unicode 参数的拒绝，以及扫描进度行；其余规则由[命令参考](../usage.md)承载。每节输出均为实际捕获；标注 Windows 或 Linux 的情形只在该平台出现。

阅读约定：
- 输出与命令都是真实运行的逐字节记录：stdout、stderr 分开原样展示，控制字符与非 UTF-8 字节用标注的转义视图。
- 示例都在固定时钟下运行（`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`），输出里的时间以它为准；个别命令前缀显式设置该变量时以那个值为准，日常使用不需要设置它。
- 除说明明确前后关系外，相邻示例不表示连续操作。
- 命令按 POSIX shell 写出（在 Windows 上可用 Git Bash 执行）。
- 正常结束（退出码 0）与空输出不逐条标注；非零退出码单独列出。

<a id="plat-lock-vs-sharing-windows"></a>

## Windows：进程锁与独占打开是两种错误

另一进程持有与 agent-coexist 写入时相同的独占锁（LockFileEx）：读写命令以 E-LOCKED 失败（退出 3）。编辑器等程序以不共享方式打开文件则是 E-INUSE。两种情况的 fix 都要求处理后重试。目录扫描不受影响：读不到的文件标为 busy（或 skipped），fix 给出逐个重读的命令；锁释放或程序关闭后重试即恢复。

```sh
agent-coexist check standup
```

说明：E-LOCKED：另一进程持有同一种锁。

stdout：

```text
error  check  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist check standup
```

退出码：3

```sh
agent-coexist check standup
```

说明：E-INUSE：另一程序以不共享方式打开。

stdout：

```text
error  check  standup
  reason        file is in use by another program
  fix           close it, then retry: agent-coexist check standup
```

退出码：3

```sh
agent-coexist list .
```

说明：list 继续：读不到的叶子标为 busy。

stdout：

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

目录的 state 文件被锁时写入自己的档案：写入本身成功（created/section 行在前），写后的 locked 核对读不到 state 文件，报告降为 warn 并建议在再次写入前先核对那里有没有别人的 locked 声明。

```sh
agent-coexist profile bob --as bob --section role -m 'parser owner'
```

说明：profile 部分完成：写入成功，locked 核对标为 busy。

stdout：

```text
warn  profile  bob
  created       bob.agent-besties.md
  section       role  set
  skipped       .  busy
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check state
```

<a id="plat-denied-io-linux"></a>

## Linux：权限不足的读取失败

普通用户运行时权限位生效：文件 chmod 000 后 check 以 Permission denied 失败（退出 1），fix 要求检查该文件及路径上各目录的权限；挡住访问的是路径上的目录时错误由 cannot open 变为 cannot read，扫描把目录标为 skipped unreadable。恢复权限后按 fix 重试即成功。

```sh
agent-coexist check room/standup
```

（这条输出在 Linux 上捕获）

说明：文件失去全部权限：Permission denied。

stdout：

```text
error  check  room/standup
  reason        cannot open room/standup.agent-besties.md: Permission denied (os error 13)
  fix           check the permissions of room/standup.agent-besties.md and of the directories on its path, then retry: agent-coexist check room/standup
```

退出码：1

<a id="plat-denied-delete-pending-windows"></a>

## Windows：待删除的文件报告为普通 I/O 失败

另一程序已对文件设置删除标记但句柄未关闭：文件名仍可见，任何新打开都被拒绝。这不是锁——不会给出 E-LOCKED，而是 os error 5 的普通 I/O 失败，fix 要求检查权限或等别的程序停止使用、删除或替换它。句柄关闭后删除完成，重试时按不存在处理。

```sh
agent-coexist check standup
```

说明：待删除状态：拒绝访问，不是锁。

stdout：

```text
error  check  standup
  reason        cannot open standup.agent-besties.md: 拒绝访问。 (os error 5)
  fix           check the permissions of standup.agent-besties.md and of the directories on its path, or wait until another program stops using, deleting or replacing it, then retry: agent-coexist check standup
```

退出码：1

<a id="plat-readonly-fs-linux"></a>

## Linux：只读文件系统上的写入失败

文件所在的文件系统被重新挂载为只读（此处用进程私有的临时挂载模拟，进程结束即消失）：post 以 Read-only file system 失败，fix 要求让持有该文件的文件系统可写后重试。文件大小上限、磁盘已满、符号链接循环等其他系统错误同样以 reason 带出系统信息、fix 给出对应处理，形式相同。

（这条输出在 Linux 上捕获）

说明：实际操作：在私有挂载命名空间里把 room 重新挂成只读，再运行 agent-coexist post room/standup --as bob -m status。

stdout：

```text
error  post  room/standup
  reason        cannot open room/standup.agent-besties.md: Read-only file system (os error 30)
  fix           make the file system that holds room/standup.agent-besties.md writable, then retry: agent-coexist post room/standup --as bob -m status
```

退出码：1

<a id="plat-operands"></a>

## 路径写法与大小写：Windows 接受，Linux 是文件名的一部分

Windows 接受 room\standup 这样的反斜杠路径，文件系统也不区分大小写（ROOM/STANDUP 打开同一文件；文件名按磁盘实际拼写与 H1 比较，不告警）。Linux 上同样的写法是另外的文件名：check 报告文件不存在，fix 给出列出该目录实际文件的命令。

```sh
agent-coexist check 'room\standup'
```

说明：Windows：反斜杠路径正常打开（大小写不同的路径同样正常）。

stdout：

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

```sh
agent-coexist check 'room\standup'
```

（这条输出在 Linux 上捕获）

说明：Linux：同一写法指向不存在的文件。

stdout：

```text
error  check  room\standup
  reason        room\standup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

退出码：1

<a id="plat-nonutf8-quoting"></a>

## Linux：非 UTF-8 名字的显示、恢复命令与手工分支

目录名含非 UTF-8 字节（显示为替换字符 U+FFFD）时，错误报告里的 fix 用 `$(printf '\377')` 写法保留原字节，在 POSIX shell 里执行即可还原出原路径。路径含换行时无法写出任何可复制的单行命令，fix 改为手工说明（no copyable command can be printed…），分两类：重试类只说明按原来的命令手工重试，不打印命令；动作类保留完整的命令和选项，路径写成 `<path of this file>` 标记，由读者自己键入。

（这条输出在 Linux 上捕获）

说明：实际命令：`agent-coexist check $'room-\xff/nothere'`（目录名含字节 0xFF，显示为替换字符）；fix 里的命令用 printf 写法保留原字节。

stdout：

```text
error  check  room-�/nothere
  reason        room-�/nothere.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list room-"$(printf '\377')"
```

退出码：1

（这条输出在 Linux 上捕获）

说明：重试类 fix 的手工形式：路径含换行（shell 中写作 `$'a\nb'`）且另一进程正对这份帖子持有 flock，所以退出 3；fix 不打印命令也不打印路径标记，只说明按原来的命令手工重试。实际命令：`agent-coexist check $'a\nb/s'`。

stdout：

```text
error  check  a b/s
  reason        file is locked by another process
  fix           retry shortly by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

退出码：3

（这条输出在 Linux 上捕获）

说明：带完整命令的手工形式：修复动作和选项原样保留，只有路径要读者自己键入（标记 `<path of this file>`）。实际命令：`agent-coexist check $'a\nb/p'`（目录名含换行）。

stdout：

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

只有 PATH 和 DIR 参数接受不是有效 Unicode 的文件名；其他参数和两个环境变量遇到无效文本时在访问任何文件之前报错（退出 2），提示重新键入或把来源文本重编码为 Unicode 后重跑。Linux 上无效文本是参数里的非 UTF-8 字节；Windows 上是 PowerShell 传入的孤立 UTF-16 代理项，错误文本与此完全相同。

（这条输出在 Linux 上捕获）

说明：实际操作：agent-coexist post standup --as 含 Latin-1 字节的昵称 -m hi。

stderr：

```text
error: the value of --as is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

退出码：2

<a id="plat-progress-lines"></a>

## 扫描进度行：只在 stderr 是终端时出现

目录扫描超过 500 ms 且 stderr 连着终端时，每 200 ms 用回车重写一行 scanning dirs: N, files: M...，结束时用空格擦除；作为对照，另一个 Windows 示例对一个包含上万个子目录和两个协作文件的目录运行扫描、stderr 是管道：没有输出任何进度（重定向收集输出时不会混入进度行）。Linux 的 PTY 只把 LF 换成 CRLF，进度行里的回车重写和空格擦除就是 CLI 写到 stderr 的字节；Windows 的 ConPTY 按屏幕内容重新绘制，帧里表现为光标定位后的新计数。

（这条输出在 Linux 上捕获）

说明：Linux：在分配了真实 PTY 的终端里运行 agent-coexist list big（stdout 重定向到文件）。

stdout：

```text
ok  list  big
  ignore        none
  total         0
```

stderr 连接在终端上，未单独捕获；见下面的终端帧。

<!-- runtime-examples:volatile:start -->
终端帧（随时序变化：进度帧的条数和计数取决于扫描速度）（转义视图，末尾没有换行符）：

```text
\rscanning dirs: 28982, files: 0...\rscanning dirs: 40380, files: 0...\rscanning dirs: 51300, files: 0...\r                                 \r
```
<!-- runtime-examples:volatile:end -->

```sh
agent-coexist list big
```

说明：对照（Windows，另一个示例，目录包含上万个子目录和两个协作文件）：stderr 是管道时没有进度行。

stdout：

```text
ok  list  big
  ignore        none
  standup                     post
  state                       state
  total         2
```
