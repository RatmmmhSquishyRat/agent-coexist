# 写入文件的说明文字

CLI 新建导读或状态文件时自动写进文件、给后续读者看的固定说明只有下面两处；用户自己写的正文不属于这类文字。

阅读约定：
- 输出与命令都是真实运行的逐字节记录：stdout、stderr 分开原样展示，控制字符与非 UTF-8 字节用标注的转义视图。
- 示例都在固定时钟下运行（`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`），输出里的时间以它为准；个别命令前缀显式设置该变量时以那个值为准，日常使用不需要设置它。
- 除说明明确前后关系外，相邻示例不表示连续操作。
- 命令按 POSIX shell 写出（在 Windows 上可用 Git Bash 执行）。
- 正常结束（退出码 0）与空输出不逐条标注；非零退出码单独列出。

<a id="brief-new-file"></a>

## brief：新建空导读

目录里还没有导读时，brief 创建文件并写入这行说明；再次运行同一命令不改动文件。

```sh
agent-coexist brief onboarding --as alice
```

说明：创建报告只有一行 created。

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
```

写入后文件 onboarding.agent-besties.md 的内容：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.
```

说明：给读者的固定说明：下面的路径相对本文件所在目录，哈希是目标字节的 SHA-256。

<a id="state-new-file"></a>

## state：新建状态文件

第一次对某目录声明状态时创建 state 文件并写入这行说明；目录必须已存在。

```sh
agent-coexist state src/parser --as alice --status locked --for 30m
```

说明：set 行报告状态、昵称与时间区间。

stdout：

```text
ok  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

写入后文件 src/parser/state.agent-besties.md 的内容：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

alice 2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

说明：给读者的固定说明：locked 与 observing 的含义；条目按“昵称 起..止”写在状态节下。
