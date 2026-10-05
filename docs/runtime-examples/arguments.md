# 参数与入口失败

<a id="arg-no-command"></a>

## 只运行程序名

`agent-coexist` · stderr · 退出码 2
```text
error: 'agent-coexist' requires a subcommand but one was not provided   [subcommands: guide, list, check, profile, post, subscribe, brief, state, read-notification, lookfor-comment, read-brief, check-state, wait, wait-post, wait-state, help]  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

<a id="arg-unknown-command"></a>

## 命令名拼错

`agent-coexist lsit` · stderr · 退出码 2
```text
error: unrecognized subcommand 'lsit'    tip: a similar subcommand exists: 'list'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

`agent-coexist frobnicate` · stderr · 退出码 2
```text
error: unrecognized subcommand 'frobnicate'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

<a id="arg-unknown-option"></a>

## 不认识的选项

`agent-coexist list --bogus` · stderr · 退出码 2
```text
error: unexpected argument '--bogus' found    tip: 'list' has no option --bogus; 'agent-coexist list --help' lists its options; only if '--bogus' is the DIR itself, write it last, after '--'  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

<a id="arg-missing-arguments"></a>

## 缺参数或缺选项值

`agent-coexist check` · stderr · 退出码 2
```text
error: the following required arguments were not provided:   <PATH>  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

`agent-coexist list --type` · stderr · 退出码 2
```text
error: a value is required for '--type <type>' but none was supplied  For more information, try '--help'. 
```

<a id="arg-enum-values"></a>

## 固定取值的词写错

`agent-coexist list --type posts` · stderr · 退出码 2
```text
error: invalid value 'posts' for '--type <type>': 'posts' is not a valid type; expected post, brief, agent-profile (alias: profile) or state  For more information, try '--help'. 
```

<a id="arg-name-values"></a>

## 名称写法不合法

`agent-coexist post standup --as 'alice smith' -m hello` · stderr · 退出码 2
```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice smith')
```

`agent-coexist lookfor-comment standup --mention @bob` · stderr · 退出码 2
```text
error: --mention must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '@bob'); write 'bob' without the '@'
```

<a id="arg-numbers"></a>

## 编号不是数字

`agent-coexist lookfor-comment standup --reply-to first` · stderr · 退出码 2
```text
error: invalid value 'first' for '--reply-to <n>': invalid digit found in string  For more information, try '--help'. 
```

<a id="arg-time-durations"></a>

## 时间与时长写错

`agent-coexist state --as alice --status locked --for 30m1h` · stderr · 退出码 2
```text
error: --for must be a duration such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '30m1h')
```

`agent-coexist state --as alice --status locked --for 3000000d` · stderr · 退出码 2
```text
error: the declaration would end after 9999-12-31T23:59:59Z, the latest representable time; use a shorter --for (without --for, 'locked' lasts 2h and other statuses 8h)
```

<a id="arg-glob-regex"></a>

## 过滤模式与正则写错

`agent-coexist list --include 'docs/[ab'` · stderr · 退出码 2
```text
error: --include 'docs/[ab' is not a valid glob: unclosed character class; missing ']'
```

`agent-coexist brief onboarding --target src/main.rs --regex 'fn main('` · stderr · 退出码 2
```text
error: --regex 'fn main(' does not compile: unclosed group
```

<a id="arg-target-required"></a>

## 缺少配套选项

`agent-coexist brief onboarding --entry 'parser entry point' --regex 'fn main'` · stderr · 退出码 2
```text
error: --regex requires --target <path>
```

<a id="arg-conflicting-flags"></a>

## 多余与冲突的参数

`agent-coexist check standup notes` · stderr · 退出码 2
```text
error: unexpected argument 'notes' found  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

`agent-coexist brief onboarding --refresh -m note` · stderr · 退出码 2
```text
error: --refresh cannot be combined with --target, --regex, -m or --remove
```

<a id="arg-env-overrides"></a>

## 环境变量值无效

`AGENT_COEXIST_NOW='2026-09-17 10:00:00Z' agent-coexist check standup` · stderr · 退出码 2
```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got '2026-09-17 10:00:00Z'); separate the date and time with 'T', not a space, or unset it to use the system clock
```

<a id="arg-dash-values"></a>

## 以短横线开头的值

`agent-coexist post standup --as alice -m '- parser done'` · stderr · 退出码 2
```text
error: unexpected argument '- ' found    tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, attach it with '=': -m='- parser done'  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

<a id="arg-invisible-whitespace"></a>

## 看不见的空白

`agent-coexist post standup --as $'alice\r' -m hello` · stderr · 退出码 2
```text
error: --as must not contain control characters (got 'alice<U+000D>'); remove the whitespace before or after 'alice'
```

<a id="arg-path-lookup"></a>

## PATH 给了目录

`agent-coexist lookfor-comment docs` · 退出码 1
```text
error  lookfor-comment  docs
  reason        docs.agent-besties.md does not exist; docs is a directory, and this command needs one collaboration file
  fix           rerun with the path of one collaboration file (the .agent-besties.md suffix may be omitted); collaboration files in that directory: agent-coexist list docs
```
