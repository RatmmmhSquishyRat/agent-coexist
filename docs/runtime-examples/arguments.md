# 参数与入口失败

命令还没碰到任何文件时，CLI 先检查写法：命令名、选项、值和环境变量。参数错误是 stderr 上的一行 error 加提示（stdout 为空、退出码 2）；PATH 给了目录或文件不存在时的诊断在 stdout、退出码 1。本页每组情况给一个真实输出；所有选项的取值规则、组合约束和环境变量的完整说明见 [docs/usage.md](../usage.md)，入门教程运行 agent-coexist guide。

阅读约定：
- 输出与命令都是真实运行的逐字节记录：stdout、stderr 分开原样展示，控制字符与非 UTF-8 字节用标注的转义视图。
- 示例都在固定时钟下运行（`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`），输出里的时间以它为准；个别命令前缀显式设置该变量时以那个值为准，日常使用不需要设置它。
- 除说明明确前后关系外，相邻示例不表示连续操作。
- 命令按 POSIX shell 写出（在 Windows 上可用 Git Bash 执行）。
- 正常结束（退出码 0）与空输出不逐条标注；非零退出码单独列出。

<a id="arg-no-command"></a>

## 只输入程序名，没有命令

CLI 要求至少一个子命令。只运行 agent-coexist 时，错误里直接列出全部子命令。

```sh
agent-coexist
```

说明：只运行程序名，错误列出所有子命令

stderr：

```text
error: 'agent-coexist' requires a subcommand but one was not provided   [subcommands: guide, list, check, profile, post, subscribe, brief, state, read-notification, lookfor-comment, read-brief, check-state, wait, wait-post, wait-state, help]  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：2

<a id="arg-unknown-command"></a>

## 命令名拼错

子命令名不存在时，若有相近的名字会给出建议；没有相近者就只列用法。help 后面跟不存在的命令、或在真实命令后面多写一个词，也是同样的「未知子命令」错误。

```sh
agent-coexist lsit
```

说明：拼成 lsit，提示最接近的是 list

stderr：

```text
error: unrecognized subcommand 'lsit'    tip: a similar subcommand exists: 'list'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：2

```sh
agent-coexist frobnicate
```

说明：完全不相近的名字没有建议

stderr：

```text
error: unrecognized subcommand 'frobnicate'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：2

<a id="arg-unknown-option"></a>

## 选项不被这个命令接受

不认识的选项按「意外参数」报告，提示说明原因：拼错的给出相近选项、别的命令的选项点明所属命令、只写一个短横线的长选项单独说明。选项里夹着看不见的字符时按 `<U+XXXX>` 转义显示。命令名之前也不能给选项。

```sh
agent-coexist list --bogus
```

说明：--bogus 不是 list 的选项；提示指向 list --help，并说明若它其实是目录名要写到 -- 之后。拼错的选项（如 --typ）则提示最接近的 --type

stderr：

```text
error: unexpected argument '--bogus' found    tip: 'list' has no option --bogus; 'agent-coexist list --help' lists its options; only if '--bogus' is the DIR itself, write it last, after '--'  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：2

<a id="arg-missing-arguments"></a>

## 缺参数或缺选项值

必需的位置参数没给、选项后面缺值，都在改动任何文件之前被拦下，错误列出缺什么。缺必需选项（如 post 的 --as 和 -m）是同样的形式。

```sh
agent-coexist check
```

说明：check 缺 PATH

stderr：

```text
error: the following required arguments were not provided:   <PATH>  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

退出码：2

```sh
agent-coexist list --type
```

说明：--type 后面没有跟值

stderr：

```text
error: a value is required for '--type <type>' but none was supplied  For more information, try '--help'. 
```

退出码：2

<a id="arg-enum-values"></a>

## 固定取值的词写错

--type 这类只收固定几个词的选项，错误会列出全部合法值。guide 主题同理，必须是小写的主题名。

```sh
agent-coexist list --type posts
```

说明：类型词只有四个，错误全部列出

stderr：

```text
error: invalid value 'posts' for '--type <type>': 'posts' is not a valid type; expected post, brief, agent-profile (alias: profile) or state  For more information, try '--help'. 
```

退出码：2

<a id="arg-name-values"></a>

## 名称不符合书写规则

nick、状态名、节名各有固定写法，错误原样回显收到的值并复述规则；回显里的控制字符和不可见字符按 `<U+XXXX>` 显示。写 @昵称 时错误会直接说去掉 @。完整规则见 [docs/usage.md](../usage.md)。

```sh
agent-coexist post standup --as 'alice smith' -m hello
```

说明：nick 里不能有空格，错误复述完整规则

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice smith')
```

退出码：2

```sh
agent-coexist lookfor-comment standup --mention @bob
```

说明：--mention 写昵称本身，不带 @

stderr：

```text
error: --mention must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '@bob'); write 'bob' without the '@'
```

退出码：2

<a id="arg-numbers"></a>

## 编号不是数字

--edit、--reply-to、--comment 这些编号选项只收数字，写错时错误回显值并说明解析失败的原因；为空、太大也是同一形式。

```sh
agent-coexist lookfor-comment standup --reply-to first
```

说明：--reply-to 写成 first

stderr：

```text
error: invalid value 'first' for '--reply-to <n>': invalid digit found in string  For more information, try '--help'. 
```

退出码：2

<a id="arg-time-durations"></a>

## 时间与时长写错

--for 的时长按 Nd/Nh/Nm/Ns 顺序写、不重复、大于零；--since 还接受 RFC 3339 时刻。数字合法但算出的时间超出可表示范围（公元 1 年到 9999 年）时另有说明，并写明默认时长。全部格式见 [docs/usage.md](../usage.md)。

```sh
agent-coexist state --as alice --status locked --for 30m1h
```

说明：单位顺序颠倒 30m1h

stderr：

```text
error: --for must be a duration such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '30m1h')
```

退出码：2

```sh
agent-coexist state --as alice --status locked --for 3000000d
```

说明：结束时间超过 9999 年；括号里写明默认时长

stderr：

```text
error: the declaration would end after 9999-12-31T23:59:59Z, the latest representable time; use a shorter --for (without --for, 'locked' lasts 2h and other statuses 8h)
```

退出码：2

<a id="arg-glob-regex"></a>

## 过滤模式与正则写错

--include/--exclude 的 glob 在解析参数时就检查，错误点明模式与原因，分支类的还给出转义写法；模式错误先于目录读取报告（目录不存在时也一样先报模式错）。--regex 按 Rust 正则语法编译并限制编译后大小，也不能含原始换行或回车（要匹配换行写 \n 转义）。

```sh
agent-coexist list --include 'docs/[ab'
```

说明：glob 的 [ 没有闭合

stderr：

```text
error: --include 'docs/[ab' is not a valid glob: unclosed character class; missing ']'
```

退出码：2

```sh
agent-coexist brief onboarding --target src/main.rs --regex 'fn main('
```

说明：正则分组没有闭合，无法编译

stderr：

```text
error: --regex 'fn main(' does not compile: unclosed group
```

退出码：2

<a id="arg-target-required"></a>

## 缺少配套选项

有的选项必须成对出现：--regex 描述的是 --target 指向的文件，单独给出时被拒绝。

```sh
agent-coexist brief onboarding --entry 'parser entry point' --regex 'fn main'
```

说明：--regex 要配 --target

stderr：

```text
error: --regex requires --target <path>
```

退出码：2

<a id="arg-conflicting-flags"></a>

## 多余的参数与冲突的组合

多余的位置参数、同一选项给两次、开关带值、命令名前写 --，以及各命令内部互斥的选项组合（如 --refresh 独占、--remove 与 -m/--target 互斥、profile/state 的形式要求），都在执行前被拦下，错误说明缺什么或哪两个不能同用。各命令的完整约束见 [docs/usage.md](../usage.md)。

```sh
agent-coexist check standup notes
```

说明：check 只收一个 PATH

stderr：

```text
error: unexpected argument 'notes' found  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

退出码：2

```sh
agent-coexist brief onboarding --refresh -m note
```

说明：--refresh 独占，错误列出全部不兼容选项

stderr：

```text
error: --refresh cannot be combined with --target, --regex, -m or --remove
```

退出码：2

<a id="arg-env-overrides"></a>

## 环境变量 AGENT_COEXIST_NOW 与 AGENT_COEXIST_LOCK_WAIT_MS

这两个变量在每个真实命令运行前解析（guide、help 等信息命令不受影响）。值无效时错误复述变量格式并回显值，提示可以 unset 回到默认。两个变量都有各自的取值规则，多处错误同时存在时语法错误最先、环境变量次之。

```sh
AGENT_COEXIST_NOW='2026-09-17 10:00:00Z' agent-coexist check standup
```

说明：日期和时间之间的 T 写成了空格

stderr：

```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got '2026-09-17 10:00:00Z'); separate the date and time with 'T', not a space, or unset it to use the system clock
```

退出码：2

<a id="arg-dash-values"></a>

## 以短横线开头的值

分开写的值以 - 开头会被当成选项。自由文本（-m 的正文）可以用 = 连接传入，错误会给出连接写法；本身不允许 - 开头的值（状态名、nick、时间、编号）就是值写错了。[docs/usage.md](../usage.md) 记录了所有支持 = 连接的选项。

```sh
agent-coexist post standup --as alice -m '- parser done'
```

说明：正文以 - 开头：提示里给出 -m='- parser done' 这个连接写法，照它重跑即可通过

stderr：

```text
error: unexpected argument '- ' found    tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, attach it with '=': -m='- parser done'  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：2

<a id="arg-invisible-whitespace"></a>

## 看不见的空白混进了值

从别处粘贴的值常带着看不见的空白。错误原样回显收到的值，回车按 `<U+000D>`、BOM 按 `<U+FEFF>` 这样转义显示，便于照着找到；能靠去掉首尾空白修好的，错误会直接点明位置。

```sh
agent-coexist post standup --as $'alice\r' -m hello
```

说明：末尾带回车，按 `<U+000D>` 回显

stderr：

```text
error: --as must not contain control characters (got 'alice<U+000D>'); remove the whitespace before or after 'alice'
```

退出码：2

<a id="arg-path-lookup"></a>

## PATH 给了目录

需要单个协作文件的命令收到目录路径时，说明该路径是目录、这条命令需要单个协作文件，并给出列出该目录协作文件的命令。后缀 .agent-besties.md 可以省略。这类诊断打印在 stdout，退出码 1。

```sh
agent-coexist lookfor-comment docs
```

说明：给了目录 docs：说明需要单个文件，并给出列目录的命令

stdout：

```text
error  lookfor-comment  docs
  reason        docs.agent-besties.md does not exist; docs is a directory, and this command needs one collaboration file
  fix           rerun with the path of one collaboration file (the .agent-besties.md suffix may be omitted); collaboration files in that directory: agent-coexist list docs
```

退出码：1
