# 等待：wait、wait-post、wait-state

三个等待命令各监视一个文件，报告启动之后第一条符合条件的变化，然后退出；就绪行 waiting … 写在 stderr，它出现之前写入的变化不会被报告，重启的等待也从新的起点开始，所以等待开始前后各读一次文件。取消用 Ctrl-C（退出码 130）。各命令参数、过滤与报告字段的完整规则见[命令参考](../usage.md)。

阅读约定：
- 输出与命令都是真实运行的逐字节记录：stdout、stderr 分开原样展示，控制字符与非 UTF-8 字节用标注的转义视图。
- 示例都在固定时钟下运行（`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`），输出里的时间以它为准；个别命令前缀显式设置该变量时以那个值为准，日常使用不需要设置它。
- 除说明明确前后关系外，相邻示例不表示连续操作。
- 命令按 POSIX shell 写出（在 Windows 上可用 Git Bash 执行）。
- 正常结束（退出码 0）与空输出不逐条标注；非零退出码单独列出。

<a id="wait-ready-and-hit"></a>

## wait-post：就绪行与命中报告

alice 在帖子里请 bob 回复，然后在后台启动 wait-post standup --as alice。就绪行出现在 stderr 后，bob 先发一条与她无关的评论（不提及、不回复），等待继续；随后 bob 回复她的 #1，等待以报告结束。报告的 add comment 形式也是 wait/wait-state 命中新内容时的基本形式。

```sh
agent-coexist wait-post standup --as alice
```

说明：后台运行的 wait-post；就绪行在 stderr，命中后 stdout 输出报告。

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

wait 可以在文件尚不存在时启动（上级目录必须存在）：命中时报告 created 和新文件里的评论。第二个 wait 就绪后文件被删除，报告 removed，并按删除前的内容列出被移除的评论。

```sh
agent-coexist wait notes
```

说明：第一个 wait：文件创建命中。

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

```sh
agent-coexist wait notes
```

说明：第二个 wait：文件删除命中。

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

src 里 bob 已声明 locked。wait-state src --status locked 就绪后，carol 声明 observing——不在过滤内，等待继续；bob 随后移除自己的声明，locked 解除，等待以 remove locked 行命中。

```sh
agent-coexist wait-state src --status locked
```

说明：后台运行的 wait-state --status locked；命中报告列出被移除的声明及其起止时间。

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

就绪后 bob 编辑了自己那条已经回复过 alice 的评论：报告不是新增，而是对 #2 的 update，before/after 列出修改前后的正文（edited 时间从 none 变为修改时刻）。续期、条目说明、H1 名称等修改都用这一报告形式。

```sh
agent-coexist wait-post standup --as alice
```

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

就绪后在文件末尾追加一个空行（手工改动）：字节变了，可解析的内容没变。

```sh
agent-coexist wait docs/s
```

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

就绪行出现后键入 Ctrl-C：进程以退出码 130 结束，stdout 为空——取消不是变化报告，也不打印状态行；stderr 里只有就绪行。三个等待命令行为相同（这里捕获的是 wait standup）。Linux 终端帧里的 ^C 是行规程对键入的回显、就绪行 LF 前的回车是 PTY 输出处理加上的，都不是 CLI 写出的字节；Windows 控制台的终端帧只含 ConPTY 的控制序列，输出与此相同。

说明：Windows 控制台：wait standup 被 Ctrl-C 取消，退出 130。

stderr：

```text
waiting  wait  standup
```

终端帧（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h
```

退出码：130

（这条输出在 Linux 上捕获）

说明：Linux 终端：同一取消的终端帧。

stderr 连接在终端上，未单独捕获；见下面的终端帧。

终端帧（转义视图，末尾没有换行符）：

```text
waiting  wait  standup\r
^C
```

退出码：130

<a id="wait-busy-windows"></a>

## Windows：采样时遇到锁或独占打开，以退出 3 结束

wait 每次采样最多等 1 秒共享锁。就绪后另一进程以独占锁锁住文件：锁等待到期，wait 以 E-LOCKED 结束；换成不共享打开（如编辑器独占）则以 E-INUSE 结束。两种 fix 都说明重试的等待从新起点开始、不报告文件忙期间的变化，要求重试并在就绪后重读文件。

```sh
agent-coexist wait standup
```

说明：E-LOCKED：另一进程持有同一种锁。

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

退出码：3

```sh
agent-coexist wait standup
```

说明：E-INUSE：另一程序以不共享方式打开。

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

退出码：3

<a id="wait-restore-after-ready"></a>

## 就绪后路径被破坏：三种修复指令

就绪后打不开被监视的文件时，wait 以错误结束，不把它当作删除命中；fix 都先声明重试的等待从新起点开始、就绪后要重读，再按破坏方式给出不同的恢复指令。三种指令：上级目录被整个删除（重建或移回，Windows 报 os error 3）；上级目录被普通文件占用（把文件挪到一边留下、移回原目录，Linux 报 Not a directory）；被监视的文件本身被换成同名目录（把占用者挪开即可——watch 也接受文件缺失，不必在原路径再放文件）。

```sh
agent-coexist wait-post room/standup --as alice
```

说明：上级目录被删除后的下一次采样（Windows）。

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

退出码：1

```sh
agent-coexist wait-post room/standup --as alice
```

（这条输出在 Linux 上捕获）

说明：上级目录被普通文件占用（Linux）。

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

退出码：1

```sh
agent-coexist wait standup
```

（这条输出在 Linux 上捕获）

说明：被监视的文件换成同名目录（Linux）。

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

退出码：1
