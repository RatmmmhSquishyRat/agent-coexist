# 信息命令：guide、help 与版本

这些命令只打印内置文本，不读项目文件、不写任何文件；无效的 AGENT_COEXIST_NOW 或 AGENT_COEXIST_LOCK_WAIT_MS 不影响它们。同一命令的 --help、-h 与 `help <命令>` 输出同一份文本，每个帮助页本页只出现一次。

阅读约定：
- 输出与命令都是真实运行的逐字节记录：stdout、stderr 分开原样展示，控制字符与非 UTF-8 字节用标注的转义视图。
- 示例都在固定时钟下运行（`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`），输出里的时间以它为准；个别命令前缀显式设置该变量时以那个值为准，日常使用不需要设置它。
- 除说明明确前后关系外，相邻示例不表示连续操作。
- 命令按 POSIX shell 写出（在 Windows 上可用 Git Bash 执行）。
- 正常结束（退出码 0）与空输出不逐条标注；非零退出码单独列出。

<a id="info-guide-pages"></a>

## guide：内置指南的五个主题页

guide 向 stdout 打印一份内置 Markdown 页，不带主题时是首页。这五页的正文就是仓库里已发布的指南文档，下面每个命令的输出与所链接文件逐字节相同，因此直接阅读链接的文档，这里不重复打印。主题是精确的小写单词（start、read、write、wait）；主题写错是一条 stderr 用法错误（规则见[命令参考](../usage.md)）。

```sh
agent-coexist guide
```

说明：不带主题：首页。

以上命令的 stdout 与产品内置指南文件 [crates/cli/guide/index.md](../../crates/cli/guide/index.md) 的内容逐字节相同，本页不重复贴出，直接阅读该文件即可。

```sh
agent-coexist guide start
```

说明：start：选哪种文件。

以上命令的 stdout 与产品内置指南文件 [crates/cli/guide/start.md](../../crates/cli/guide/start.md) 的内容逐字节相同，本页不重复贴出，直接阅读该文件即可。

```sh
agent-coexist guide read
```

说明：read：找并核对别人留下的内容。

以上命令的 stdout 与产品内置指南文件 [crates/cli/guide/read.md](../../crates/cli/guide/read.md) 的内容逐字节相同，本页不重复贴出，直接阅读该文件即可。

```sh
agent-coexist guide write
```

说明：write：写别人能接手的帖子、导读、档案和状态。

以上命令的 stdout 与产品内置指南文件 [crates/cli/guide/write.md](../../crates/cli/guide/write.md) 的内容逐字节相同，本页不重复贴出，直接阅读该文件即可。

```sh
agent-coexist guide wait
```

说明：wait：先补读，再等回复或变化。

以上命令的 stdout 与产品内置指南文件 [crates/cli/guide/wait.md](../../crates/cli/guide/wait.md) 的内容逐字节相同，本页不重复贴出，直接阅读该文件即可。

<a id="info-help-top"></a>

## 顶层帮助

不带子命令的 --help（-h、help 同输出）列出全部命令和全局选项。

```sh
agent-coexist --help
```

stdout：

```text
Local markdown collaboration files for agents: posts, briefs, agent-profiles and directory state

Usage: agent-coexist <COMMAND>

Commands:
  guide              Print a built-in guide page on when and how to use coexist
  list               Recursively list collaboration files under DIR
  check              Check one file: format, diagnostics and a preview
  profile            Create or update an agent profile
  post               Append a comment, or edit your own
  subscribe          Join or leave a post's subscriber list
  brief              Create or update a brief and its entries
  state              Declare or remove directory state (locks, observations)
  read-notification  List mentions, replies, subscriptions and state changes since a time
  lookfor-comment    Find comments by keyword, author, mention or number
  read-brief         Print a brief's description, notes and target text
  check-state        Show the state declarations for DIR, its ancestors and its subtree
  wait               Wait for any change to one collaboration file
  wait-post          Wait for a comment that mentions or replies to you
  wait-state         Wait for changes to DIR/state.agent-besties.md
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

New here? 'agent-coexist guide' explains when and how to use coexist. Exact flags for one command: 'agent-coexist <command> --help'.

Arguments:
  PATH      a collaboration file; the '.agent-besties.md' suffix may be omitted
  DIR       a directory (default: the current directory); scans are recursive
  TIME      an RFC 3339 time, e.g. 2026-09-17T09:12:00Z; any offset is converted to UTC and fractional seconds are dropped
  DURATION  e.g. 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero
  types     post, brief, agent-profile (alias: profile), state
  values    a text value (-m, --keyword, --entry, --target, --regex, --include, --exclude) that starts with '-' must be attached with '=', as in -m='- item' or --keyword=-x; a separate word starting with '-' is read as an option. Nicks, status and section names, types, times, durations and comment numbers never start with '-'

Which read command:
  check        one file: format, diagnostics and a preview; no brief target text
  read-brief   a brief with its target text
  check-state  state declarations for a directory, its ancestors and its subtree

Exit codes of the business commands (all except guide and help):
  0    first line 'ok' or 'warn'; 'warn' also exits 0, so read the first line and rows for conflicts and incomplete scans
  1    first line 'error'; also when a command could not write its report to stdout (a write command may already have changed the file: check it before retrying)
  2    usage error: one 'error: ...' line on stderr, nothing on stdout
  3    busy or locked (E-LOCKED, E-INUSE, E-FLOCKED); the report's 'fix' row says what to do next
  130  wait, wait-post or wait-state cancelled with Ctrl-C; no stdout
guide, help, --help and --version print information with no ok/warn/error line and exit 0; usage errors still exit 2, and a failed guide write exits 1.

Scan progress ('scanning dirs: N, files: M...') appears on stderr only when stderr is a terminal. No progress line does not mean success or failure; judge the result by the report and the exit code.
Examples and 'fix' commands use POSIX shell quoting (Git Bash on Windows); adjust the quoting for cmd.exe or PowerShell.
```

<a id="info-help-write"></a>

## 写命令的帮助

profile、post、subscribe、brief、state 五种写命令的帮助页。

```sh
agent-coexist profile --help
```

stdout：

```text
Create or update an agent profile

Usage: agent-coexist profile <PATH> [--as <nick>] [--section <name> (-m <content> | --remove)]

Arguments:
  <PATH>  the agent-profile file; the suffix may be omitted

Options:
      --as <nick>       your nick; not written, only exempts your own 'locked' declarations
      --section <name>  the section to set or remove
  -m <content>          the section body, stored verbatim in a code fence
      --remove          delete the whole --section
  -h, --help            Print help

--section <name> -m <content> sets that section's body, stored verbatim in a code fence; --section <name> --remove deletes the section. Without --section, creates the file (heading only) or reports 'unchanged'. Content that starts with '-' must be attached: -m='- item'.
--as is your nick. It is not written; it only exempts your own 'locked' declarations. A valid 'locked' by another nick on the file's directory or an ancestor refuses the write with E-FLOCKED (exit 3); without --as, your own 'locked' refuses it too.
If another write holds the file, fails after about 200ms with E-LOCKED (exit 3); retry shortly. On E-FLOCKED, retry after the time shown in 'reason' or ask the holder; do not retry in a loop.
Example: agent-coexist profile agents/alice --section role -m 'Rust implementer. Owns crates/core.'
```

```sh
agent-coexist post --help
```

stdout：

```text
Append a comment, or edit your own

Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]

Arguments:
  <PATH>  the post file; the suffix may be omitted

Options:
      --as <nick>  the author nick written in the comment heading
  -m <content>     the comment body, stored verbatim in a code fence
      --edit <n>   replace the whole body of your own comment #n (its author must be --as)
  -h, --help       Print help

-m is the comment body, stored verbatim in a code fence. In it, @nick mentions someone and @#n replies to comment n. A body that starts with '-', such as a Markdown list, must be attached: -m='- item'. --as is the author nick written in the comment heading.
--edit <n> replaces the whole body of your own comment #n (its author must be --as), so resend everything you want to keep.
The report's 'comment' row names the comment written.
Waits up to 5s for the file lock (AGENT_COEXIST_LOCK_WAIT_MS overrides); if still busy: E-LOCKED, exit 3; retry the same command shortly.
Example: agent-coexist post docs/standup --as bob -m '@#1 parser done, see src/parser/mod.rs'
```

```sh
agent-coexist subscribe --help
```

stdout：

```text
Join or leave a post's subscriber list

Usage: agent-coexist subscribe <PATH> --as <nick> [--cancel]

Arguments:
  <PATH>  the post file; the suffix may be omitted

Options:
      --as <nick>  the nick added to (or, with --cancel, removed from) the list
      --cancel     remove --as from the subscriber list instead of adding it
  -h, --help       Print help

Adds --as to the post's subscriber list, so read-notification --as <nick> lists every new comment in that post. Nothing is pushed to subscribers, and wait-post does not use the list.
--cancel removes --as from the list. Subscribing twice, or cancelling a nick that is not subscribed, changes nothing ('unchanged <nick>').
Locking is as for post: waits up to 5s (AGENT_COEXIST_LOCK_WAIT_MS overrides), then E-LOCKED, exit 3.
Example: agent-coexist subscribe docs/standup --as alice
```

```sh
agent-coexist brief --help
```

stdout：

```text
Create or update a brief and its entries

Usage: agent-coexist brief <PATH> [--as <nick>]
   or: agent-coexist brief <PATH> [--as <nick>] -m <description>
   or: agent-coexist brief <PATH> [--as <nick>] --target <path> [--entry <title>] [--regex <pattern>] [-m <note>]
   or: agent-coexist brief <PATH> [--as <nick>] --entry <title> -m <note>
   or: agent-coexist brief <PATH> [--as <nick>] --entry <title> --remove
   or: agent-coexist brief <PATH> [--as <nick>] [--entry <title>] --refresh

Arguments:
  <PATH>  the brief file; the suffix may be omitted

Options:
      --as <nick>        your nick; not written, only exempts your own 'locked' declarations
  -m <content>           the description (no --target/--entry) or the entry's note (with --target/--entry)
      --target <path>    the file this entry points to, relative to the brief's directory
      --entry <title>    the entry title; one space at either end is ignored, while two spaces in a row anywhere (including at an end) or a control character such as a tab or CR is rejected (titles in the file match with whitespace runs read as one space)
      --regex <pattern>  a regex the target must still match when the brief is read ('' deletes it; needs --target)
      --remove           delete the whole --entry
      --refresh          recompute stored hashes from the targets' current bytes
  -h, --help             Print help

The usage forms, in order: create the brief (or report 'unchanged'); set the description; add or update an entry for a target; replace one entry's note; remove an entry; refresh hashes (every entry, or only --entry). Other combinations are rejected.
-m is the description without --target/--entry, and the entry's note with them. Text that starts with '-' must be attached: -m='- item', and likewise --regex='-x'.
--target is relative to the brief's directory, not the current directory. Without --entry, the title is the target's file name.
--regex: when the brief is checked or read, the target must still match it, or the entry is 'stale'; writing only checks that it compiles. --regex '' deletes the entry's regex lines. Use \r or \n in a pattern to match line breaks; --regex and --target cannot contain raw line breaks.
--refresh accepts the targets' current bytes: read what changed and update the note first.
--as is your nick. It is not written; it only exempts your own 'locked' declarations. A valid 'locked' by another nick on the file's directory or an ancestor refuses the write with E-FLOCKED (exit 3); without --as, your own 'locked' refuses it too.
If another write holds the file, fails after about 200ms with E-LOCKED (exit 3); retry shortly. On E-FLOCKED, retry after the time shown in 'reason' or ask the holder; do not retry in a loop.
Example: agent-coexist brief docs/onboarding --target ../src/parser/mod.rs --entry 'parser entry point'
```

```sh
agent-coexist state --help
```

stdout：

```text
Declare or remove directory state (locks, observations)

Usage: agent-coexist state [DIR] --as <nick> --status <name> [--for <duration>]
   or: agent-coexist state [DIR] --remove <nick> [--as <nick>] [--status <name>]

Arguments:
  [DIR]  the directory (default: the current directory); must already exist

Options:
      --as <nick>       declare form: your nick; remove form: who removes (default: the removed nick)
      --status <name>   the status name; required in the declare form; in the remove form, a filter (omitted: every status)
      --for <duration>  how long the declaration lasts (declare form only; default 2h for locked, 8h otherwise)
      --remove <nick>   remove this nick's declarations instead of declaring
  -h, --help            Print help

Declare: --as <nick> --status <name> records that <nick> holds <name> on DIR and everything below it until the end time. Declaring again replaces your entries under that status with one from now, except an entry on a line that also has unparseable fragments (W-STATE-ENTRY): that line is kept, and the latest end time still wins; check-state shows the result.
Remove: --remove <nick> removes that nick's declarations in DIR. Add --status to remove only that status; without it every status goes, including a live 'locked'.
Built-in statuses: 'locked' = please do not change this directory or its subdirectories until the end time; 'observing' = I am watching, changes are welcome. Other names are stored and shown but have no built-in meaning.
--for defaults to 2h for 'locked' and 8h otherwise. In the remove form --as defaults to the removed nick; removing another nick's declaration adds a 'removed-other' row and a 'warn' first line.
DIR must already exist; it is never created.
'locked' is a request between agents, not access control, and another nick's 'locked' never blocks a state write. Locks overlapping yours are listed as 'conflict' rows with a 'warn' first line (exit 0); talk to the holder in a post.
The conflict check reads the state files of DIR and its ancestors and the state files below DIR that ignore rules do not skip; state has no --no-ignore, so to see locks inside ignored directories run check-state DIR --no-ignore.
If another write holds the state file, fails after about 200ms with E-LOCKED (exit 3); retry shortly.
Example: agent-coexist state src --as alice --status locked --for 1h30m
```

<a id="info-help-read"></a>

## 读命令的帮助

list、check、lookfor-comment、read-brief、read-notification、check-state 六种查询命令的帮助页。

```sh
agent-coexist list --help
```

stdout：

```text
Recursively list collaboration files under DIR

Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]

Arguments:
  [DIR]  directory to scan (default: the current directory; the scan is recursive)

Options:
      --type <type>     only these file types; repeatable, union (post, brief, agent-profile, state; profile is an alias)
      --include <glob>  keep only files matching this gitignore-style glob, relative to DIR (repeatable; does not reach into ignored directories without --no-ignore)
      --exclude <glob>  skip files matching this glob, relative to DIR (repeatable)
      --no-ignore       apply no ignore rules (.gitignore, .ignore, .git/info/exclude, global gitignore)
  -h, --help            Print help

Lists the collaboration files under DIR as a tree with a type column.
--type keeps only the given types (repeatable: post, brief, agent-profile, state; 'profile' is an alias).
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Type column 'busy': the file lock or a stable read was not available within 100ms; retry. On Windows it can also mean another program holds the file open without sharing; close that program, then retry.
Type column '?': unreadable (permissions, not UTF-8, invalid H1, or over 16 MiB).
'busy' and '?' files still show under any --type and make the first line 'warn'; the command does not fail on them.
Example: agent-coexist list docs --type post --type brief
```

```sh
agent-coexist check --help
```

stdout：

```text
Check one file: format, diagnostics and a preview

Usage: agent-coexist check <PATH>

Arguments:
  <PATH>  the collaboration file; the '.agent-besties.md' suffix may be omitted

Options:
  -h, --help  Print help

Reports one file: its 'file' and 'type' rows, diagnostics in line order, then a preview (topic and latest comment for posts, entry states for briefs, sections for profiles, declarations for state).
Brief targets are checked but not printed; use read-brief for their text. For every declaration that applies to a directory, use check-state.
Waits up to 1s for a shared lock; if the file stays busy: E-LOCKED, exit 3; retry the same command shortly.
Example: agent-coexist check docs/standup
```

```sh
agent-coexist lookfor-comment --help
```

stdout：

```text
Find comments by keyword, author, mention or number

Usage: agent-coexist lookfor-comment <PATH> [--keyword <text>] [--by <nick>] [--mention <nick>] [--reply-to <n>] [--comment <n>]

Arguments:
  <PATH>  the post file; the suffix may be omitted

Options:
      --keyword <text>  case-insensitive text in the comment body or heading, matched as typed
      --by <nick>       the comment author
      --mention <nick>  the body mentions @nick
      --reply-to <n>    the body replies to comment n (@#n)
      --comment <n>     the comment numbered n
  -h, --help            Print help

Lists matching comments with line, number, author, time and an excerpt. Without filters it lists every comment; with several, a comment must match all of them.
--keyword is a case-insensitive substring of the raw body or heading line, matched exactly as typed (spaces are not folded).
--by is the comment's author (check lists a post's 'authors'). --mention matches @nick in the body, --reply-to matches @#n in the body, and --comment selects comment #n.
Waits up to 1s for a shared lock; if still busy: E-LOCKED, exit 3; retry shortly.
Example: agent-coexist lookfor-comment docs/standup --by bob
```

```sh
agent-coexist read-brief --help
```

stdout：

```text
Print a brief's description, notes and target text

Usage: agent-coexist read-brief <PATH> [--entry <title>]

Arguments:
  <PATH>  the brief file; the suffix may be omitted

Options:
      --entry <title>  print only this entry; one space at either end is ignored, while two spaces in a row anywhere (including at an end) or a control character such as a tab or CR is rejected (titles in the file match with whitespace runs read as one space)
  -h, --help           Print help

Prints the description, then each entry's state, note and target text. With a regex, only the lines of its first match are printed; without one, the whole target.
An entry with no path, a missing or unreadable target, a failing regex, or a target over 64 MiB prints no text; its row says why.
--entry prints only that entry. For entry states without target text, use check.
Waits up to 1s for a shared lock; if still busy: E-LOCKED, exit 3; retry shortly.
Example: agent-coexist read-brief docs/onboarding --entry 'parser entry point'
```

```sh
agent-coexist read-notification --help
```

stdout：

```text
List mentions, replies, subscriptions and state changes since a time

Usage: agent-coexist read-notification [DIR] --as <nick> --since <time|duration> [--include <glob>]... [--exclude <glob>]... [--no-ignore]

Arguments:
  [DIR]  directory to scan (default: the current directory; recursive, plus ancestor state files)

Options:
      --as <nick>              your nick: whose mentions, replies and subscriptions to list
      --since <time|duration>  RFC 3339 time or a duration back from now
      --include <glob>         keep only files matching this gitignore-style glob, relative to DIR (repeatable; does not reach into ignored directories without --no-ignore)
      --exclude <glob>         skip files matching this glob, relative to DIR (repeatable)
      --no-ignore              apply no ignore rules (.gitignore, .ignore, .git/info/exclude, global gitignore)
  -h, --help                   Print help

Lists, oldest first, what happened since --since that concerns the --as nick: comments that mention it, reply to its comments, or appear in posts it subscribes to; state declarations by others; locks that overlap its own; its own declarations that end soon or have ended.
Scans every post and state file under DIR that ignore rules do not skip (default: the current directory) and also reads ancestor state files. It never writes files and keeps no read cursor; for files inside ignored directories add --no-ignore.
--since is an RFC 3339 time (e.g. 2026-09-17T09:12:00Z) or a duration back from now (e.g. 1h30m). The window ends before the current second, so a comment written this second shows up in a later scan.
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Each file waits at most 100ms for a shared lock. Busy or unparseable files become 'skipped' or 'partial' rows with a 'warn' first line; read them directly before concluding nothing is new.
Example: agent-coexist read-notification --as alice --since 1h
```

```sh
agent-coexist check-state --help
```

stdout：

```text
Show the state declarations for DIR, its ancestors and its subtree

Usage: agent-coexist check-state [DIR] [--include <glob>]... [--exclude <glob>]... [--no-ignore]

Arguments:
  [DIR]  directory to scan (default: the current directory; recursive, plus ancestors)

Options:
      --include <glob>  keep only files matching this gitignore-style glob, relative to DIR (repeatable; does not reach into ignored directories without --no-ignore)
      --exclude <glob>  skip files matching this glob, relative to DIR (repeatable)
      --no-ignore       apply no ignore rules (.gitignore, .ignore, .git/info/exclude, global gitignore)
  -h, --help            Print help

Shows the declarations that apply to DIR: 'inherited' rows from ancestor directories (valid ones only), DIR's own declarations, then one group per state file below DIR that ignore rules do not skip (add --no-ignore to see declarations in skipped directories).
A row ends with 'conflict' when it is a valid 'locked' and another nick holds a valid 'locked' shown in this same report on an overlapping directory (the same, an ancestor or a descendant; both rows are marked), or with 'expired' when a declaration in DIR or below has ended. Either mark makes the first line 'warn' (exit 0). Only this report's declarations are compared: an inherited lock can still conflict with a lock in another branch below that ancestor; check-state on the ancestor shows it.
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Each state file waits at most 100ms for a shared lock. Busy or unparseable ones become 'skipped' or 'partial' rows with a 'warn' first line.
For one state file with its diagnostics, use check <DIR>/state.
Example: agent-coexist check-state src
```

<a id="info-help-wait"></a>

## 等待命令的帮助

wait、wait-post、wait-state 三个等待命令的帮助页，写明就绪行、采样节奏和文件被锁或独占打开时的结束方式。

```sh
agent-coexist wait --help
```

stdout：

```text
Wait for any change to one collaboration file

Usage: agent-coexist wait <PATH>

Arguments:
  <PATH>  the file to watch; the suffix may be omitted

Options:
  -h, --help  Print help

Waits for the next change to this collaboration file (any type), prints one report of what changed, and exits 0. It never writes.
Once it has recorded its starting point, it writes one line 'waiting  wait  <object>' to stderr. A change made before that line is not reported, so read the file again after the line appears.
The file is sampled about once a second. A missing file is a valid start: the wait reports its creation (its directory must exist), so a mistyped path waits forever.
Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3); on Windows, a program holding the file open without sharing ends it with E-INUSE (exit 3). However a wait ends — a busy file, an unreadable file, or an invalid sample — a restarted wait starts from a new baseline: read the file again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait docs/onboarding
```

```sh
agent-coexist wait-post --help
```

stdout：

```text
Wait for a comment that mentions or replies to you

Usage: agent-coexist wait-post <PATH> [--as <nick>]

Arguments:
  <PATH>  the post file to watch; the suffix may be omitted

Options:
      --as <nick>  the nick whose mentions and replies end the wait; without it, any change counts
  -h, --help       Print help

Waits for a comment in this post that mentions @<nick> or replies to a comment by <nick> and is new, edited or newly relevant; prints those comments and exits 0. With --as, subscriber changes and deleted comments do not count; without --as, any change to the post counts.
Once it has recorded its starting point, it writes one line 'waiting  wait-post  <object>' to stderr. A comment written before that line is not reported, so read the post again after the line appears.
The post is sampled about once a second; a missing post is a valid start. Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3), or on Windows E-INUSE (exit 3). However a wait ends — a busy post, an unreadable post, or an invalid sample — a restarted wait starts from a new baseline: read the post again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait-post docs/standup --as alice
```

```sh
agent-coexist wait-state --help
```

stdout：

```text
Wait for changes to DIR/state.agent-besties.md

Usage: agent-coexist wait-state [DIR] [--status <name>]

Arguments:
  [DIR]  the directory (default: the current directory); only its state file is watched

Options:
      --status <name>  count only changes to declarations under this status ('locked', 'observing' or another name)
  -h, --help           Print help

Watches only DIR/state.agent-besties.md (DIR defaults to the current directory), not ancestors or subdirectories.
--status (locked, observing or another status name) waits for a declaration under that status to be added, removed or renewed; without it, any change to the file counts. A declaration that simply expires does not count; use check-state for current declarations.
Once it has recorded its starting point, it writes one line 'waiting  wait-state  <object>' to stderr. A change made before that line is not reported, so run check-state again after the line appears.
The file is sampled about once a second; a missing state file is a valid start. Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3), or on Windows E-INUSE (exit 3). However a wait ends — a busy file, an unreadable file, or an invalid sample — a restarted wait starts from a new baseline: run check-state on the directory again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait-state src --status locked
```

<a id="info-help-guide-and-help"></a>

## guide 与 help 自身的帮助；help 不接受帮助选项

guide 的帮助说明主题取值与静态页性质。help 子命令自己的帮助很短；help 后面跟 --help 或 -h 不会被当作帮助选项，而是当作未知命令报告。

```sh
agent-coexist guide --help
```

stdout：

```text
Print a built-in guide page on when and how to use coexist

Usage: agent-coexist guide [start|read|write|wait]

Arguments:
  [start|read|write|wait]  start, read, write or wait; omit for the index page

Options:
  -h, --help  Print help

Prints one built-in Markdown page to stdout. Without a topic it prints the index page: what to do when you arrive in a project, and a first exchange between two agents.
Topics: start (which file type to use and where), read (find and check what others left), write (posts, briefs, profiles and state others can act on), wait (catch up, then wait for a reply or change).
Topics are exact lowercase words. Pages are static text; the command does not read your project.
Example: agent-coexist guide start
```

```sh
agent-coexist help help
```

说明：help help：help 自身的帮助。

stdout：

```text
Print this message or the help of the given subcommand(s)

Usage: agent-coexist help [COMMAND]...

Arguments:
  [COMMAND]...  Print help for the subcommand(s)
```

```sh
agent-coexist help --help
```

说明：help --help 被当作未知子命令；-h 同样不接受。

stderr：

```text
error: unrecognized subcommand '--help'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：2

<a id="info-version"></a>

## 版本号：--version 与 -V

--version 与 -V 是程序本身的选项，写在命令之前，输出一行版本；后面的命令不会执行。version 不是命令，写在命令之后的 -V 也不被接受——两种写错都按参数错误报告（规则见[命令参考](../usage.md)）。

```sh
agent-coexist --version
```

说明：--version（-V 同输出）。

stdout：

```text
agent-coexist 0.1.0
```
