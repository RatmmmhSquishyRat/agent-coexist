# 诊断与恢复

check 和各写命令在文件内容、状态文件或 ignore 来源有问题时打印诊断行，分两种：逐行诊断（error/warn 加行号 `Lnn`）每条带 why（问题在哪）和 fix（怎么处理）；单条拒绝（reason/fix）只说原因和下一步，没有 why。fix 有的是替换好已知值后可直接执行的命令，有的是要按说明手工做的事。每个例子展示实际输出和它自己的退出码。平台特有的锁占用、权限位与编码文本见 [platform.md](platform.md)，正常查询输出见 [reports.md](reports.md)。

阅读约定：
- 输出与命令都是真实运行的逐字节记录：stdout、stderr 分开原样展示，控制字符与非 UTF-8 字节用标注的转义视图。
- 示例都在固定时钟下运行（`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`），输出里的时间以它为准；个别命令前缀显式设置该变量时以那个值为准，日常使用不需要设置它。
- 除说明明确前后关系外，相邻示例不表示连续操作。
- 命令按 POSIX shell 写出（在 Windows 上可用 Git Bash 执行）。
- 正常结束（退出码 0）与空输出不逐条标注；非零退出码单独列出。

文中占位符：⟨root⟩＝在主机（Windows）上运行的示例的临时根目录。

<a id="diag-file-unusable"></a>

## 文件读不了、太大或第一行不是标题

协作文件会先整体读入：不是 UTF-8、超过 16 MiB、第一行不是 H1 标题、代码块围栏没有闭合，都是阻断性错误。check 报告它们；写命令遇到同样问题也拒绝写入。按各行 fix 修好后，原命令即可通过。

```sh
agent-coexist check standup
```

说明：文件里有 Latin-1 编码的 é，check 报 E-UTF8

stdout：

```text
error  check  standup
  file          standup.agent-besties.md
  error  L6  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
```

退出码：1

```sh
agent-coexist post standup --as bob -m 'Count me in.'
```

说明：同一问题上写入同样被拒：什么都没写

stdout：

```text
error  post  standup
  file          standup.agent-besties.md
  error  L6  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-UTF8 at L6 first, as the fix rows above say, then run this command again
```

退出码：1

```sh
agent-coexist check build-log
```

说明：文件超过 16 MiB：fix 建议把大段内容移到普通文件

stdout：

```text
error  check  build-log
  file          build-log.agent-besties.md
  error  L1  file is larger than 16 MiB
         why  files this size are never collaboration files; parsing it would stall every list and notification scan
         fix  move the large content elsewhere and keep only a short post or brief here
```

退出码：1

```sh
agent-coexist post release --as bob -m 'Release notes: .........................................................................................................................................................................................'
```

说明：文件没超限，但这次追加会超限

stdout：

```text
error  post  release
  file          release.agent-besties.md
  type          post
  error  L1  this write would make the file larger than 16 MiB: it is 16777116 bytes now and would become 16777362 bytes
         why  files this size are never collaboration files; parsing it would stall every list and notification scan
         fix  nothing was written; the file itself is still readable; shorten the new text, or put it in a new file (for a comment, a new post)
```

退出码：1

```sh
agent-coexist check notes
```

说明：第一行不是 H1：fix 给出应写的标题

stdout：

```text
error  check  notes
  file          notes.agent-besties.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# notes.post" (or .brief / .agent-profile), or "# .state" for a state file
```

退出码：1

```sh
agent-coexist check review
```

说明：代码块没有闭合；未闭合块里的内容不再另报正文告警

stdout：

```text
error  check  review
  file          review.agent-besties.md
  type          post
  error  L5  code fence opened at L5 is never closed
         why  everything after it is treated as fence content, so no later comment, entry or section can be found
         fix  add a closing line of 4 backticks where the block should end
  comments      1
  authors       alice
  topic         #1  alice
  latest        #1  alice  2026-09-17T09:00:00Z
  subscribers   none
```

退出码：1

<a id="diag-h1-rename"></a>

## 文件名不能直接抄成标题时的改名修复

第一行不是 H1 时，fix 要求把第 1 行写成 `# <名称>.post`（或 `.brief` / `.agent-profile`）：名称自选，1-100 个字母、数字、`_`、`-` 或 `.`，以字母、数字、`_` 或 `-` 开头。顺带把文件改名为 `<名称>.agent-besties.md` 可以让文件名与标题一致，但只在还没有这个名称的文件时可行——绝不覆盖已有文件。

```sh
agent-coexist check 'standup notes'
```

说明：fix 里的 `<name>` 是要自己选的名称；改名是可选的一致化做法，条件是没有同名文件

stdout：

```text
error  check  standup notes
  file          standup notes.agent-besties.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# <name>.post" (or .brief / .agent-profile): use 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-"; use "# .state" for a state file; renaming the file to <name>.agent-besties.md as well keeps the H1 and the file name in agreement, but only if no file has that name yet (never overwrite an existing file)
```

退出码：1

<a id="diag-comment-headers"></a>

## 评论标题坏了

评论标题（`## #编号 作者 (时间)`）被手工改坏时，E-HEAD 的 fix 给出改写后的整行标题。编号按条件选定：给出的默认编号只在没有回复已写 `@#n` 指向它、也没有被同报告其他修复派走时才可用，否则要另选一个没有标题占用、没有回复指向、也没被其他修复派号的编号；确知原编号且它未被占用时，同一句说明改回原编号、回复不用改。丢了时间或作者的分支要自己填 `<time>`/`<nick>`：填的是评论最初写下时的时间和作者，标题里的修改时间不算创建时间。

```sh
agent-coexist check standup
```

说明：两处标题错误各自得到不同的修复编号

stdout：

```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## #2 bob 2026-09-17T09:05:00Z" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (2026-09-17T09:05:00Z)"
  error  L15  "## #02 carol (2026-09-17T09:10:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #5 carol (2026-09-17T09:10:00Z)"; its number changes from #2 to #5 if no reply in this post already names @#5, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names; change any @#2 that meant this comment to that number
  warn   L21  comment numbers jump from #1 to #3
         why  no comment in this post has number #2, so @#2 cannot be resolved; a malformed comment header before this one may be that comment
         fix  repair the malformed header first; if the gap remains, leave the numbering as is, because other files may already reference #3
  warn   L21  comment #3 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #3 which comment was meant
  warn   L27  comment #4 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #4 which comment was meant
  comments      3
  authors       alice, bob
  topic         #1  alice  Daily standup. Post your status here.
  latest        #4  bob  2026-09-17T09:20:00Z  @#2 carol, does the lexer handle tabs?
  subscribers   none
```

退出码：1

```sh
agent-coexist post standup --as bob --edit 2 -m 'Parser done, tests green.'
```

说明：编号读不出：给出的 #4 要先核对没有回复已指向它，否则另选；同一句写明确知原编号时改回原编号、回复不用改

stdout：

```text
error  post  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## #x bob (2026-09-17T09:05:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #4 bob (2026-09-17T09:05:00Z)"; the header shows no usable number, so use #4 if no reply in this post already names @#4, otherwise another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and change any reply that meant this comment to that number; if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change
  warn   L15  comment numbers jump from #1 to #3
         why  no comment in this post has number #2, so @#2 cannot be resolved; a malformed comment header before this one may be that comment
         fix  repair the malformed header first; if the gap remains, leave the numbering as is, because other files may already reference #3
  warn   L15  comment #3 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #3 which comment was meant
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

退出码：1

```sh
agent-coexist post review --as bob --edit 2 -m '@#1 Looks good; carol is fixing the nit.'
```

说明：只剩修改时间不算创建时间；只剩编号没有作者：分别填回

stdout：

```text
error  post  review
  file          review.agent-besties.md
  type          post
  error  L9  "## #2 bob (edited 2026-09-17T09:20:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (<time>, edited 2026-09-17T09:20:00Z)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  error  L15  "## # 3 (2026-09-17T09:30:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #3 <nick> (2026-09-17T09:30:00Z)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9 and E-HEAD at L15 first, as the fix rows above say, then run this command again
```

退出码：1

<a id="diag-numbering"></a>

## 评论编号重复或已用尽

编号重复时 fix 默认给下一个空闲编号；若它被回复 @#n 占用、或同一报告的其他修复已把它派给别人，就另选。编号到上限（#18446744073709551614）时只能手工换用未用的较低编号（确知原编号时按同一句恢复）；再追加评论就要开新帖。

```sh
agent-coexist check standup
```

说明：重复编号的修复与占用规则

stdout：

```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  error  L9  comment number #1 appears twice
         why  @#1 and --edit 1 cannot tell the two apart
         fix  renumber the later one to #2 if no reply in this post already names @#2, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and update any @#1 that meant it to that number
  warn   L9  comment #1 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #1 which comment was meant
  error  L15  "## #x carol (2026-09-17T09:10:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #3 carol (2026-09-17T09:10:00Z)"; the header shows no usable number, so use #3 if no reply in this post already names @#3, otherwise another number that no header in this post carries, that no other repair in this report gives a header, and that no reply already names, and change any reply that meant this comment to that number; if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  bob  2026-09-17T09:05:00Z  @#2 Parser done.
  subscribers   none
```

退出码：1

```sh
agent-coexist check standup
```

说明：编号耗尽的标题修复：换低编号或恢复原编号；同一报告还有重复编号要一起处理

stdout：

```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L3  the first comment in this post is #18446744073709551614, not #1
         why  no comment in this post has number #2, so @#2 cannot be resolved; that comment may have been deleted or renumbered by hand, or the first comment was written with a higher number
         fix  leave the numbering as is; other files may already reference #18446744073709551614
  error  L9  "## #x bob" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  no comment number above #18446744073709551614, the highest in this post, is left for this header; to keep the comment where it is, with its author, time and text, rewrite the header by hand as "## #<n> bob (<time>)", replacing <n> with an unused lower number: one that no header in this post carries, that you give no other header, and that no reply in this post already names (@#n), and each <nick> or <time> with the comment's original author or the time it was first written; then change any reply that meant this comment to @# with that number; if you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change; a lower number can leave a W-SEQ-ORDER or W-SEQ-GAP warning, which does not block writes; this frees no number for new comments: once post stops with E-SEQ-EXHAUSTED, new comments go in a new post
  error  L15  comment number #18446744073709551614 appears twice
         why  @#18446744073709551614 and --edit 18446744073709551614 cannot tell the two apart
         fix  no comment number above #18446744073709551614, the highest in this post, is left to renumber this later #18446744073709551614; to keep it where it is, with its author, time and text, change only the number in its header by hand to an unused lower number: one that no header in this post carries, that you give no other header, and that no reply in this post already names (@#n); then update any @#18446744073709551614 that meant it to that number; a lower number can leave a W-SEQ-ORDER or W-SEQ-GAP warning, which does not block writes; this frees no number for new comments: once post stops with E-SEQ-EXHAUSTED, new comments go in a new post
  warn   L21  comment #1 appears after #18446744073709551614
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      3
  authors       alice, carol, erin
  topic         #18446744073709551614  alice  Last comment number.
  latest        #1  erin  2026-09-17T09:03:00Z  @#18446744073709551614 carol, which lexer?
  subscribers   none
```

退出码：1

```sh
agent-coexist post standup --as bob -m 'Count me in.'
```

说明：追加时编号用尽：fix 是开新帖的完整命令

stdout：

```text
error  post  standup
  reason        comment numbering is exhausted: #18446744073709551614 is the highest number a header can carry
  fix           start a new post: agent-coexist post <new path> --as=bob -m '<text>'
```

退出码：1

<a id="diag-post-warnings"></a>

## 帖子里的顺序、回复与游离文字告警

W- 类不阻断写入。缺号与顺序颠倒按建议保留（不编造缺号原因）；回复指向不存在的评论时去问那条评论的作者；写在帖子前言或标题下的游离文字由写它的人移进代码块或删掉，不知道谁写的就先问、内容保留。手工追加的评论没有代码块时，fix 直接给出改写命令。

```sh
agent-coexist check seq
```

说明：缺号、倒序、悬空回复与前言游离文字一起报告

stdout：

```text
warn  check  seq
  file          seq.agent-besties.md
  type          post
  warn   L3  text outside any fence at L3 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  warn   L11  comment numbers jump from #1 to #4
         why  no comment in this post has number #3, so @#3 cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order
         fix  leave the numbering as is; other files may already reference #4
  warn   L11  comment #4 replies to @#3 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #4 which comment was meant
  warn   L17  comment #2 appears after #4
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L23  text outside any fence at L23 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  warn   L25  text outside any fence at L25 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Parser plan.
  latest        #2  carol  2026-09-17T09:02:00Z  Lexer plan.
  subscribers   none
```

```sh
agent-coexist check first
```

说明：第一条评论编号大于 #1、更小的编号在后面出现：同样保留

stdout：

```text
warn  check  first
  file          first.agent-besties.md
  type          post
  warn   L3  the first comment in this post is #2, not #1
         why  every number below #2 is used elsewhere in this post, so no comment is missing and @#n still resolves; only the order is unusual
         fix  leave the numbering as is; other files may already reference #2
  warn   L9  comment #1 appears after #2
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      2
  authors       bob, alice
  topic         #2  bob  Tests are green.
  latest        #1  alice  2026-09-17T09:00:00Z  Parser plan.
  subscribers   none
```

```sh
agent-coexist check standup
```

说明：手工追加的评论没有代码块：fix 给出 post --edit 改写命令

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L9  comment #2 at L9 has no fenced body
         why  there is nothing to show as its content
         fix  only carol, the comment's author, can add its body; anyone else asks carol to, or replies with @#2 instead; the command for carol: agent-coexist post standup.agent-besties.md --as=carol --edit=2 -m '<text>'
  warn   L11  text outside any fence at L11 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  comments      2
  authors       alice, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #2  carol  2026-09-17T09:30:00Z
  subscribers   none
```

<a id="diag-post-encoding"></a>

## BOM、CRLF、重复订阅行与文件名不一致

这些是告警不是错误。BOM 在读取时被跳过、CRLF 会与 CLI 新增的 LF 行混用，fix 都是重新保存文件。订阅行只有第一行生效、后面的重复行被忽略；fix 要求读者手工只保留第一行、整行删掉后面的行（不要把昵称并进去——只写在后面行里的昵称现在并没有订阅，并入反而会让它开始收到通知），要不要订阅由各昵称自己决定。文件名与 H1 名称不一致时按 fix 改名，文件名本身不合法时 fix 直接给出按 H1 名称的目标名。

```sh
agent-coexist check standup
```

说明：BOM、CRLF 与重复订阅行的处理建议

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L1  file starts with a UTF-8 BOM
         why  the BOM is skipped when reading; appends and splices leave it in place, so it stays until someone re-saves the file
         fix  re-save the file without BOM
  warn   L1  file uses CRLF line endings
         why  lines the CLI adds use LF, so the file will end up mixed
         fix  convert the file to LF
  warn   L3  subscribers line repeated or contains duplicates
         why  only the first line counts and duplicates are ignored; the extra text is noise for readers
         fix  keep only the first "- subscribers:" line, with each nick once; delete any later "- subscribers:" line instead of merging it in: a nick named only there is not subscribed now, and merging would subscribe it; each agent named only there decides for itself whether to subscribe
  warn   L4  subscribers line repeated or contains duplicates
         why  only the first line counts and duplicates are ignored; the extra text is noise for readers
         fix  keep only the first "- subscribers:" line, with each nick once; delete any later "- subscribers:" line instead of merging it in: a nick named only there is not subscribed now, and merging would subscribe it; each agent named only there decides for itself whether to subscribe
  comments      1
  authors       alice
  topic         #1  alice  Daily standup.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup.
  subscribers   alice, bob
```

```sh
agent-coexist check bob
```

说明：文件名合法：两种改法

stdout：

```text
warn  check  bob
  file          bob.agent-besties.md
  type          post
  warn   L1  file name "bob" does not match the H1 name "alice"
         why  list shows the file name, the file says another name; readers will be confused
         fix  rename the file to alice.agent-besties.md if no file has that name yet (never overwrite an existing file), or set line 1 to "# bob.post"
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

```sh
agent-coexist check 'my notes'
```

说明：文件名含空格、抄不成合法名称：fix 给出按 H1 名称的目标名 retro，仅当还没有这个文件

stdout：

```text
warn  check  my notes
  file          my notes.agent-besties.md
  type          post
  warn   L1  file name "my notes" does not match the H1 name "retro"
         why  list shows the file name, the file says another name; readers will be confused
         fix  rename the file to retro.agent-besties.md if no file has that name yet (never overwrite an existing file); the file stem is not a valid name, so no valid H1 can be built from it
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

<a id="diag-brief-entries"></a>

## 导读条目的问题

条目类诊断的 fix 都是「先审阅目标文件与条目说明（必要时先更新说明），再执行冒号后的命令」，命令会同时记下目标当前的哈希。目标与正则不猜测，由知道原意的人提供；没有路径的条目不能刷新哈希，要走同一报告里补路径的命令。目标移走时改指向新位置（正则保留）；已经不存在就删除或改写条目。

```sh
agent-coexist check handoff
```

说明：条目重名、缺路径、缺哈希；无路径的条目不能刷新哈希，要走补路径的命令

stdout：

```text
error  check  handoff
  file          handoff.agent-besties.md
  type          brief
  error  L3  entry "parser" has no "- path:" line
         why  without a path there is nothing to hash or read; the entry cannot be verified
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=parser
  error  L3  entry "parser" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  this entry has no path, so --refresh cannot hash it; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then set its target; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=parser
  error  L9  entry "parser" appears twice
         why  --entry parser cannot tell the two apart
         fix  merge the two sections or retitle one of them
  error  L9  entry "parser" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief handoff.agent-besties.md --entry=parser --refresh
  entries       2
  fresh         0
  shifted       0
  stale         0
  unchecked     2
  entry         parser  unchecked  ?  no path  Start at parse_document.
  entry         parser  unchecked  src/lexer.rs  no hash  Tokens come from here.
```

退出码：1

```sh
agent-coexist check handoff
```

说明：正则无法编译；无目标的条目要先选定文件和正则

stdout：

```text
error  check  handoff
  file          handoff.agent-besties.md
  type          brief
  error  L7  entry "<file>" regex does not compile: unclosed group
         why  read-brief and check use the regex to find the lines the note is about
         fix  review the target and the note, update the note if it no longer describes the target, then give the entry a regex that compiles and matches it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target=src/parser.rs --entry='<file>' --regex '<pattern>'
  error  L13  entry "notes" has no "- path:" line
         why  without a path there is nothing to hash or read; the entry cannot be verified
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=notes
  error  L13  entry "notes" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  this entry has no path, so --refresh cannot hash it; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then set its target; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=notes
  error  L15  entry "notes" regex does not compile: unclosed group
         why  read-brief and check use the regex to find the lines the note is about
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path and a regex that compiles and matches it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target=<file> --entry=notes --regex '<pattern>'
  entries       2
  fresh         0
  shifted       0
  stale         1
  unchecked     1
  entry         <file>  stale  src/parser.rs  regex invalid  Entry point.
  entry         notes  unchecked  ?  no path  Any function.
```

退出码：1

```sh
agent-coexist check handoff
```

说明：目标是目录：改指向已知文件；目标读不出：先恢复可读再核对，不改指向

stdout：

```text
error  check  handoff
  file          handoff.agent-besties.md
  type          brief
  error  L5  entry "config" target src cannot be read: is a directory, not a file
         why  the target exists but its bytes are not readable, so neither hash nor regex can be checked
         fix  the path names a directory, not a file; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=config
  error  L14  entry "private" target notes/private.txt cannot be read: 拒绝访问。 (os error 5)
         why  the target exists but its bytes are not readable, so neither hash nor regex can be checked
         fix  the path must name a readable file; fix its permissions and run check again without changing the entry, or choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=private
  entries       2
  fresh         0
  shifted       0
  stale         0
  unchecked     2
  entry         config  unchecked  src  target unreadable  Settings live here.
  entry         private  unchecked  notes/private.txt  target unreadable  Rotation schedule.
```

退出码：1

```sh
agent-coexist check handoff
```

说明：目标内容已变化或已移走：先审阅，再刷新或改指向

stdout：

```text
warn  check  handoff
  file          handoff.agent-besties.md
  type          brief
  warn   L3  entry "parser" is shifted: src/parser.rs changed since its hash was taken
         why  the note may describe code that no longer looks like this
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief handoff.agent-besties.md --entry=parser --refresh
  warn   L12  entry "lexer" is stale: target missing
         why  the note points at something that is gone or no longer matches its regex
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; if it no longer exists, remove or rewrite the entry; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <file> --entry=lexer
  warn   L22  entry "errors" is stale: regex not found
         why  the note points at something that is gone or no longer matches its regex
         fix  review the target and the note, update the note if it no longer describes the target, then give the entry a regex that matches it; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target=src/errors.rs --entry=errors --regex '<pattern>'
  entries       3
  fresh         0
  shifted       1
  stale         2
  entry         parser  shifted  src/parser.rs  parse_document returns a Doc.
  entry         lexer  stale  src/lexer.rs  target missing  Tokenizer entry.
  entry         errors  stale  src/errors.rs  regex not found  Error type.
```

```sh
agent-coexist check handoff
```

说明：绝对路径、未知属性行与游离文字

stdout：

```text
warn  check  handoff
  file          handoff.agent-besties.md
  type          brief
  warn   L7  text outside any fence at L7 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  warn   L11  entry "parser" uses an absolute path
         why  the brief only verifies on this machine; on any other checkout the target will be stale
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at a path relative to the brief's directory; this command also records the target's current hash: agent-coexist brief handoff.agent-besties.md --target <relative path> --entry=parser
  warn   L13  entry "parser" has unknown attribute "owner"
         why  the line is kept but has no meaning to the CLI
         fix  remove the line, or move its content into the note
  description   Parser handoff.
  entries       1
  fresh         1
  shifted       0
  stale         0
  entry         parser  fresh  ⟨root⟩/work/src/parser.rs  Start at parse_document.
```

<a id="diag-profile-sections"></a>

## 档案节的问题

节名重复是错误：两个同名节让 --section 无法区分要改的是哪一个，fix 要求合并成一节或给其中一个改名；本例展示的是 check 的报告，实际写入并没有尝试。其余各行是告警，按各自 fix 手工改；节名/状态名首字符不合法时 fix 复述完整命名规则；档案一个节都没有时 fix 直接给出补第一节内容的命令。

```sh
agent-coexist check alice
```

说明：check 报告节重名：--section 分不清两个同名节；节名不合法与缺失正文是告警

stdout：

```text
error  check  alice
  file          alice.agent-besties.md
  type          agent-profile
  error  L9  section "role" appears twice
         why  --section role cannot tell the two apart
         fix  merge the two sections or rename one
  warn   L15  section "my scope" at L15 is not a valid section name
         why  --section cannot address it; allowed: starts with a letter, digit or _, then letters, digits, _ and -, at most 64 characters
         fix  rename the heading, e.g. "## my-scope"
  warn   L21  section "prefer" at L21 has no fenced body
         why  there is nothing to show as its content
         fix  agent-coexist profile alice.agent-besties.md --section=prefer -m '<text>'
  warn   L23  section "on call" at L23 has no fenced body
         why  there is nothing to show as its content
         fix  rename the heading by hand to a valid, distinct section name and add a fenced body beneath it, then run check again
  warn   L23  section "on call" at L23 is not a valid section name
         why  --section cannot address it; allowed: starts with a letter, digit or _, then letters, digits, _ and -, at most 64 characters
         fix  rename this heading by hand to a valid section name distinct from every other section and any other planned heading repair, then run check again
  warn   L31  text outside any fence at L31 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  sections      6
  section       role  Rust implementer.
  section       role  Reviews parser changes.
  section       my scope  crates/core parsing.
  section       prefer
  section       on call
  section       on-call  Weekdays.
```

退出码：1

```sh
agent-coexist check bob
```

说明：档案没有任何节：fix 给出命令

stdout：

```text
warn  check  bob
  file          bob.agent-besties.md
  type          agent-profile
  warn   L1  profile has no sections
         why  nothing tells other agents who this is or what they own
         fix  agent-coexist profile bob.agent-besties.md --section role -m '<what you do>'
  sections      0
```

<a id="diag-control-headings"></a>

## 标题里的控制字符

标题带控制字符时不是合法状态名，其下的条目只被展示、从不当作锁；输出里标题中的控制字符显示为 �（U+FFFD 替换字符），不是原来的字节。fix 要求去掉标题里的控制字符，把重复的节手工合并成一个合法状态名（如 ## locked），并连同已用该名字的任何节一起合并，再重新 check。合并成有效的 locked 名之后这些声明才真正具有锁语义——两位作者本来是否要写同一个状态，是本例的前提（已知都是 locked），不是 CLI 打印的建议。档案节标题的控制字符同理，由写的人自己处理。

```sh
agent-coexist check state
```

说明：两个带控制字符的 locked 标题：fix 要求去掉控制字符、连同已用该名的节合并成一节（本例前提：两位作者本来要写的都是 locked）

stdout：

```text
error  check  state
  file          state.agent-besties.md
  type          state
  warn   L3  status "lo�cked" at L3 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## locked", together with any section that already uses that name, then run check again
  error  L7  status "lo�cked" appears twice
         why  entries under a status must live in one place, or upsert and cleanup would touch the wrong one
         fix  edit the two "## lo�cked" headings by hand: remove their control characters and merge the two sections into one, together with any section that already uses the cleaned name, then run check again
  warn   L7  status "lo�cked" at L7 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## locked", together with any section that already uses that name, then run check again
  entries       2
  lo�cked       alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  lo�cked       bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

退出码：1

<a id="diag-state-warnings"></a>

## state 文件里的冲突、重复、倒置与过期

check 对选中的 state.agent-besties.md 报告：同一昵称的重复声明只有改写该状态段的下一次 state 写入才会合并，写别的状态段不动它；结束早于开始的条目按过期处理、从不成锁；过期条目只有声明者本人仍需要时才续。冲突先由当事人商量，让出的一方执行。

```sh
agent-coexist check state
```

说明：冲突、重复、倒置、过期四类告警的 why/fix

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L7  locked is held by both alice and bob
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state . --remove <nick> --status=locked
  warn   L11  carol appears twice under observing
         why  the latest end time wins; the next state write that rewrites the observing section (a declaration or removal under it, or expiry cleanup of an entry in it) merges them into one unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)
         fix  optional; to merge them now, that agent declares again, which replaces both with one entry from now (add --for to choose its length), but only when both entries are on clean lines; when either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is and the duplicate stays: inspect the file, delete carol's entries from every such line by hand, keeping each line's other content, then look at the remaining entries and declare again to merge any duplicate that is left or to start one entry from now: agent-coexist state . --as=carol --status=observing
  warn   L15  dave under review ends before it starts
         why  treated as expired: it never counts as a lock; the next state write removes it unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent declares it again: agent-coexist state . --as=dave --status=review
  warn   L16  erin under review expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state . --as=erin --status=review
  warn   L17  frank under review expired at 2026-09-17T09:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state . --as=frank --status=review
  entries       6
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T12:00:00Z  conflict
  observing     carol  2026-09-17T08:00:00Z..2026-09-17T13:00:00Z
  review        dave  2026-09-17T11:00:00Z..2026-09-17T10:30:00Z  expired
  review        erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z  expired
  review        frank  2026-09-17T08:00:00Z..2026-09-17T09:00:00Z  expired
```

```sh
agent-coexist state . --as gina --status observing --for 4h
```

说明：节名重复、状态名不合法、条目行写错：声明被拒绝

stdout：

```text
error  state  .
  file          ./state.agent-besties.md
  type          state
  warn   L7  status "Review Queue" at L7 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  rename the heading by hand to a valid, distinct status name, e.g. "## review-queue" only if that name is unused, then run check again
  warn   L9  erin under Review Queue expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  rename the status heading by hand to a valid, distinct status name, then run check again before renewing or changing its declarations
  error  L11  status "observing" appears twice
         why  entries under a status must live in one place, or upsert and cleanup would touch the wrong one
         fix  merge the two "## observing" sections into one
  warn   L13  "frank tomorrow" under observing is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-STATUS-DUP at L11 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

退出码：1

```sh
agent-coexist state . --as carol --status observing --for 2h
```

说明：同一个不合法的状态标题出现两次：两行合并成一节，条目原样保留

stdout：

```text
error  state  .
  file          ./state.agent-besties.md
  type          state
  warn   L3  status "Review Queue" at L3 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## review-queue", together with any section that already uses that name, then run check again
  error  L7  status "Review Queue" appears twice
         why  entries under a status must live in one place, or upsert and cleanup would touch the wrong one
         fix  merge the two "## Review Queue" sections into one
  warn   L7  status "Review Queue" at L7 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  this status heading appears more than once, so do not give its copies different names, which would split one status into two; merge its sections by hand into one under one valid status name, e.g. "## review-queue", together with any section that already uses that name, then run check again
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-STATUS-DUP at L7 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

退出码：1

<a id="diag-state-unselected"></a>

## 状态内容写在了别的文件名里

state 命令只认 state.agent-besties.md（Windows/macOS 上大小写不敏感）。写在别的名字下的状态内容，其重复/倒置/过期告警的 fix 不打印任何命令：先确定哪一份文件才该用这个名字（可用 check 确认目录里是否已有 state.agent-besties.md；绝不能覆盖另一份已存在的 state 文件），重新 check 这份文件，之后也只有声明者本人改、续、删自己的条目。这道门防止在确认文件身份之前改错文件。

```sh
agent-coexist check locks
```

说明：文件名不对：重复/倒置/过期都以「未确认的文件」处理，fix 不给命令

stdout：

```text
warn  check  locks
  file          locks.agent-besties.md
  type          state
  warn   L1  state file is named "locks.agent-besties.md", not "state.agent-besties.md"
         why  check-state and state look only for state.agent-besties.md (case-insensitive on Windows/macOS); other names are ignored
         fix  inspect this file and any state.agent-besties.md already in its directory; resolve their intended declarations before choosing which file should use that name, and never overwrite a separate existing state file (keep line 1 as "# .state")
  warn   L5  alice appears twice under locked
         why  the latest end time wins; no state write merges them, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md
         fix  no state command is printed because this file is not confirmed as this directory's state.agent-besties.md; first settle which file that should be (when a W-NAME-STATE row is shown, its fix describes how; never overwrite a separate existing state file), then run check on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry
  warn   L9  dave under review ends before it starts
         why  treated as expired: it never counts as a lock; no state write removes it, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md
         fix  no state command is printed because this file is not confirmed as this directory's state.agent-besties.md; first settle which file that should be (when a W-NAME-STATE row is shown, its fix describes how; never overwrite a separate existing state file), then run check on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry
  warn   L10  erin under review expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks; no state write removes them, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md
         fix  no state command is printed because this file is not confirmed as this directory's state.agent-besties.md; first settle which file that should be (when a W-NAME-STATE row is shown, its fix describes how; never overwrite a separate existing state file), then run check on this file again before anyone changes a declaration by hand; only the agent that declared an entry changes, renews or removes its own entry
  entries       3
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
  review        dave  2026-09-17T11:00:00Z..2026-09-17T10:30:00Z  expired
  review        erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z  expired
```

<a id="diag-state-dirty"></a>

## 要删的声明和读不懂的内容在同一行

删除声明时若那一行还混着无法解析的片段，命令拒绝执行并给出两条路：由写坏片段的人把它改到可解析，再重跑原命令；或由声明者本人手工从该行删掉自己的声明——这个删除本身就是结果，仅当这位作者还有本该由这条命令删除的其他条目时才需要重跑。

```sh
agent-coexist state . --remove alice --status locked
```

说明：E-STATE-DIRTY 的两条修复路

stdout：

```text
error  state  .
  reason        alice is still declared on line 5 of the state file in ., mixed with unparseable fragments
  fix           either delete alice's declaration from line 5 by hand, which is the removal itself (re-run then only if alice has more entries this command should remove), or have whoever wrote that line's unparseable fragments make them parseable, then re-run: agent-coexist state . --remove alice --status locked
```

退出码：1

<a id="diag-functional-locks"></a>

## 功能锁 locked 挡住了写入

目录被某个昵称的 locked 声明锁定（`state --as <昵称> --status locked`）时，该目录的其他写入被 E-FLOCKED 拒绝。锁是自己的：按 fix 的文字在原命令里加上或改对 `--as=`（fix 冒号后打印的就是原命令，照说明改后重试）；锁是别人的：等它到期或请持有者释放。报告还会列出同时生效的其他锁，以及扫描中读不懂的行。

```sh
agent-coexist profile agents/alice --section role -m 'Rust implementer.'
```

说明：自己的锁：fix 的文字要求在原命令里加上 --as=alice（加在任何 -- 之前）；冒号后打印的是原命令

stdout：

```text
error  profile  agents/alice
  reason        agents is locked by alice until 2026-09-17T11:00:00Z
  fix           if the lock is yours, add --as=alice before any -- separator; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist profile agents/alice --section role -m 'Rust implementer.'
```

退出码：3

```sh
agent-coexist profile agents/alice --as alcie --section role -m 'Rust implementer.'
```

说明：打错的 --as 要替换成正确的，不是再加一个

stdout：

```text
error  profile  agents/alice
  reason        agents is locked by alice until 2026-09-17T11:00:00Z
  fix           if the lock is yours and the --as in this command names someone else, rerun it with --as=alice in place of that --as; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist profile agents/alice --as alcie --section role -m 'Rust implementer.'
```

退出码：3

```sh
agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'
```

说明：别人的锁：等到期或请持有者释放；also-locked 与 partial 行

stdout：

```text
error  brief  docs/agents/handoff
  reason        . is locked by carol until 2026-09-17T12:00:00Z
  fix           if the lock is yours and the --as in this command names someone else, rerun it with --as=carol in place of that --as; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'
  also-locked   alice  until 2026-09-17T11:00:00Z
  partial       .  W-STATE-ENTRY  L7
```

退出码：3

<a id="diag-ignore-syntax"></a>

## ignore 来源里的规则写错

过滤规则来自多个来源：.ignore、.gitignore、.git/info/exclude 和全局 excludesFile。任何一行写错，读命令都报 cannot scan 并点明来源文件与行号，fix 给出改正该行后重试的完整命令；行不是 UTF-8 时同样定位到行。--no-ignore 可以不等修复先列出文件，但正确的规则也一并失效。要按写下这行的人的原意改正，不从错误猜原意。

```sh
agent-coexist list .
```

说明：.ignore 第 1 行的 { 未闭合；其他来源（.gitignore、.git/info/exclude、全局 excludesFile）写错时同样点名来源与行号

stdout：

```text
error  list  .
  reason        cannot scan ./.ignore: line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of ./.ignore, then retry: agent-coexist list .
```

退出码：1

```sh
agent-coexist list work
```

说明：.ignore 第 2 行不是 UTF-8：同样定位到行

stdout：

```text
error  list  work
  reason        cannot scan work/.ignore: line 2: stream did not contain valid UTF-8
  fix           correct line 2 of work/.ignore, then retry: agent-coexist list work
```

退出码：1

```sh
agent-coexist list . --no-ignore
```

说明：--no-ignore 绕过：build/ 的正确规则也失效，被忽略的文件被列了出来

stdout：

```text
ok  list  .
  ignore        --no-ignore
  build/
    hidden                    post
  docs/
    standup                   post
  total         2
```

<a id="diag-ignore-unreadable"></a>

## ignore 来源读不了：不是语法错

同样的「无法扫描」，原因不同修法就不同：没有读权限时恢复可读再重试；来源本身是一个目录时换成普通文件或删除。括号里的操作系统消息随系统不同。不要把读不了当成语法错去改行内容。

```sh
agent-coexist list .
```

说明：没有读权限：fix 是恢复可读后重试

stdout：

```text
error  list  .
  reason        cannot scan ./.ignore: 拒绝访问。 (os error 5)
  fix           check that ./.ignore can be read, then retry: agent-coexist list .
```

退出码：1

```sh
agent-coexist list .
```

说明：.ignore 是目录：fix 是换成普通文件或删除

stdout：

```text
error  list  .
  reason        cannot scan ./.ignore: 拒绝访问。 (os error 5)
  fix           replace ./.ignore with a regular file or remove it, then retry: agent-coexist list .
```

退出码：1

<a id="diag-git-file"></a>

## .git 文件的问题

在工作区（worktree、子模块布局）里 .git 是一个小文件，指向真正的 git 目录。内容不是 gitdir 引用、指向的目录不存在、或指向的不是目录，读命令都报 cannot scan，fix 都是改正 .git 文件的内容后重试。括号里的操作系统消息随系统不同。

```sh
agent-coexist list work
```

说明：内容不是 gitdir 引用

stdout：

```text
error  list  work
  reason        cannot scan work/.git: invalid gitdir reference
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

退出码：1

```sh
agent-coexist list work
```

说明：指向的目录不存在

stdout：

```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/nowhere: 系统找不到指定的文件。 (os error 2)
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

退出码：1

```sh
agent-coexist list work
```

说明：指向一个普通文件：不是一个目录

stdout：

```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/../gitdata: gitdir reference is not a directory
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

退出码：1

<a id="diag-state-scan-after-write"></a>

## state 声明已写入，但写后的冲突扫描失败

state 写入成功后还会扫描 ignore 来源以检查冲突；来源有问题时扫描失败，声明不回滚——首行 warn 的 scan 行说明原因，fix 要求先修 ignore 来源，再用只读的 check-state 补查冲突，不要重新声明。补查发现的真实重叠仍按冲突告警处理，由当事人商量。

```sh
agent-coexist state src --as alice --status locked
```

说明：ignore 语法错：修第 1 行后用 check-state 补查

stdout：

```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of .ignore, then check for conflicts without declaring again: agent-coexist check-state src
```

```sh
agent-coexist state src --as alice --status locked
```

说明：读权限问题：恢复可读后补查

stdout：

```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  拒绝访问。 (os error 5)
  fix           check that .ignore can be read, then check for conflicts without declaring again: agent-coexist check-state src
```

```sh
agent-coexist state src --as alice --status locked
```

说明：来源是目录：换成普通文件后补查

stdout：

```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  拒绝访问。 (os error 5)
  fix           replace .ignore with a regular file or remove it, then check for conflicts without declaring again: agent-coexist check-state src
```

<a id="diag-ignore-too-large"></a>

## ignore 规则合起来超过上限

每行规则都合法，合起来也可能超过模式匹配器的大小上限。fix 只要求精简、简化或删除规则，直到符合大小与嵌套上限后重试；把逐条列举换成更宽的一条规则也属于精简，但要先确认写下这些规则的人的原意，不能从错误里猜。读命令与 state 写后的冲突扫描受同一个上限约束。

```sh
agent-coexist list room
```

说明：list 报 error building NFA：精简后重试

stdout：

```text
error  list  room
  reason        cannot scan room/.ignore: error building NFA
  fix           shorten, simplify or remove rules in room/.ignore until they fit the pattern matcher's size and nesting limits, then retry: agent-coexist list room
```

退出码：1

```sh
agent-coexist state room --as bob --status observing
```

说明：state 已写入、扫描同样超限：精简后用 check-state 补查

stdout：

```text
warn  state  room
  created       room/state.agent-besties.md
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  scan          room/.ignore  error building NFA
  fix           shorten, simplify or remove rules in room/.ignore until they fit the pattern matcher's size and nesting limits, then check for conflicts without declaring again: agent-coexist check-state room
```
