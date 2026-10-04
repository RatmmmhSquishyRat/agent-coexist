# 查询与写入报告

查询与写入命令的真实报告输出，按要做的事分组；同一模板只展示一次，参数与用法的完整说明见[命令参考](../usage.md)。

阅读约定：
- 输出与命令都是真实运行的逐字节记录：stdout、stderr 分开原样展示，控制字符与非 UTF-8 字节用标注的转义视图。
- 示例都在固定时钟下运行（`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`），输出里的时间以它为准；个别命令前缀显式设置该变量时以那个值为准，日常使用不需要设置它。
- 除说明明确前后关系外，相邻示例不表示连续操作。
- 命令按 POSIX shell 写出（在 Windows 上可用 Git Bash 执行）。
- 正常结束（退出码 0）与空输出不逐条标注；非零退出码单独列出。

<a id="list-dir"></a>

## list：查看目录下有什么

工作区里有讨论、导读、档案和状态文件，部分目录被 ignore 规则跳过。

```sh
agent-coexist list
```

说明：ignore 行列出实际生效的来源；被跳过的目录不出现在树里、不计入 total。

stdout：

```text
ok  list  .
  ignore        .gitignore, .ignore, exclude
  agents/
    alice                     agent-profile
  docs/
    Retro                     post
    onboarding                brief
    standup                   post
  src/
    parser/
      state                   state
  state                       state
  total         6
```

<a id="list-unknown-leaves"></a>

## list：类型读不出的文件

目录里有非 UTF-8、空文件或首行不是标题的文件，类型无法判断。

```sh
agent-coexist list
```

说明：? 叶子保留并计数、首行 warn；fix 行给出逐个查看的只读命令。

stdout：

```text
warn  list  .
  ignore        none
  docs/
    binary                    ?
    empty                     ?
    noheading                 ?
    standup                   post
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read: agent-coexist check ./docs/binary; agent-coexist check ./docs/empty; agent-coexist check ./docs/noheading
  total         4
```

<a id="list-include-empty"></a>

## list：--include 结果为空

--include 的 glob 写错，或匹配到的目录被 ignore 跳过时，list 以 warn 报告空结果。

```sh
agent-coexist list --include 'build/**'
```

说明：include 行说明原因：检查 glob 相对 DIR 的写法，被 ignore 跳过的目录要 --no-ignore 才能重开。

stdout：

```text
warn  list  .
  ignore        .gitignore
  include       build/**  no file is left after the filters; check that this glob is relative to DIR; if the files are in a directory that ignore rules skip, --include cannot reach them: add --no-ignore
  total         0
```

<a id="check-previews"></a>

## check：单份文件的概览

check 只读地给出一份文件的概览。

```sh
agent-coexist check docs/standup
```

说明：有评论的讨论：topic 是第一条评论，latest 是最后一条。

stdout：

```text
ok  check  docs/standup
  file          docs/standup.agent-besties.md
  type          post
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Daily standup. @bob please post your status here.
  latest        #3  carol  2026-09-17T09:10:00Z  @alice can you review the lexer change?
  subscribers   alice, bob
```

```sh
agent-coexist check docs/onboarding
```

说明：导读：条目按 fresh/shifted/stale 计数。

stdout：

```text
ok  check  docs/onboarding
  file          docs/onboarding.agent-besties.md
  type          brief
  description   How to read this project.
  entries       1
  fresh         1
  shifted       0
  stale         0
  entry         parser entry point  fresh  ../src/parser.rs  Read this first.
```

<a id="lookfor-comment"></a>

## lookfor-comment：查找评论

在一份讨论里按条件筛评论；文件类型不对或评论头损坏时命令拒绝执行。

```sh
agent-coexist lookfor-comment docs/standup
```

说明：摘录超长截断、以 ... 结尾；正文为空的评论没有摘录列。

stdout：

```text
ok  lookfor-comment  docs/standup
  L5  #1  alice  2026-09-17T08:10:00Z  Daily standup. @bob please post your status here.
  L11  #2  bob  2026-09-17T09:05:00Z  @#1 @alice parser done, see src/parser/mod.rs
  L17  #3  alice  2026-09-17T09:12:00Z  @bob thanks, merging the Parser now.
  L23  #4  carol  2026-09-17T09:20:00Z  The lexer is also done, and this first line is deliberately long enough to be...
  L30  #5  bob  2026-09-17T09:30:00Z
  total         5
```

```sh
agent-coexist lookfor-comment docs/onboarding
```

说明：类型不对时整份拒绝；fix 给出该目录下 post 的只读 list。

stdout：

```text
error  lookfor-comment  docs/onboarding
  reason        file type is brief, not post
  fix           this command needs a file of type post; rerun it with such a path; files of that type under this directory: agent-coexist list docs --type=post
```

退出码：1

```sh
agent-coexist lookfor-comment docs/t
```

说明：评论头损坏时整份拒绝、退出 1；read-notification 对同一文件只记 partial。

stdout：

```text
error  lookfor-comment  docs/t
  file          docs/t.agent-besties.md
  type          post
  error  L9  "## #2 bob 2026-09-17T09:20:00Z" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (2026-09-17T09:20:00Z)"
  warn   L15  comment numbers jump from #1 to #3
         why  no comment in this post has number #2, so @#2 cannot be resolved; a malformed comment header before this one may be that comment
         fix  repair the malformed header first; if the gap remains, leave the numbering as is, because other files may already reference #3
```

退出码：1

<a id="read-brief"></a>

## read-brief：阅读导读

read-brief 展开导读的说明、条目与目标内容；条目状态由本次运行计算，不存储在文件里。

```sh
agent-coexist read-brief docs/onboarding
```

说明：带 regex 的条目只显示匹配覆盖的行，哈希仍覆盖整个目标文件。

stdout：

```text
ok  read-brief  docs/onboarding
  description
    | How to read this project in ten minutes.
  entry         parser entry point  fresh  ../src/parser.rs  L3..L3
  group         name  parse_document
  note
    | Read this first; spans are byte-precise.
  content
    | pub fn parse_document(input: &str) -> Doc {
  entry         lexer  fresh  ../src/lexer.rs  L1..L1
  content
    | pub fn tokens() {}
```

```sh
agent-coexist read-brief docs/states
```

说明：stale 与无路径、不可读的条目不输出内容；首行 warn、退出 0。

stdout：

```text
warn  read-brief  docs/states
  entry         fresh one  fresh  ../src/b.rs  L1..L1
  note
    | Unchanged helper.
  content
    | fn helper() {}
  entry         empty target  fresh  ../src/empty.rs
  content
  entry         shifted one  shifted  ../src/a.rs  L1..L1
  note
    | Notes written against version one.
  content
    | version two
  entry         missing  stale  ../src/gone.rs  target missing
  entry         bad regex  stale  ../src/b.rs  regex invalid
  entry         regex gone  stale  ../src/b.rs  regex not found
  entry         no path  unchecked  ?  no path
  entry         no hash  unchecked  ../src/b.rs  no hash  L1..L1
  content
    | fn helper() {}
  entry         dir target  unchecked  ../src/dir  target unreadable
```

```sh
agent-coexist read-brief docs/big
```

说明：超过 64 MiB 的目标省略 content 并标注 target too large。

stdout：

```text
warn  read-brief  docs/big
  entry         with regex  unchecked  ../data/big.txt  target too large
  entry         hash only  fresh  ../data/big.txt  target too large
```

```sh
agent-coexist read-brief docs/onboarding --entry parser
```

说明：标题要完整匹配；fix 指向该导读的 check。

stdout：

```text
error  read-brief  docs/onboarding
  reason        no entry titled "parser"
  fix           pick an existing title from: agent-coexist check docs/onboarding.agent-besties.md
```

退出码：1

<a id="read-notification"></a>

## read-notification：收通知

read-notification 汇总窗口内与你相关的活动：回复、提及、订阅的讨论、他人声明与冲突。

```sh
agent-coexist read-notification --as alice --since 2026-09-17T08:00:00Z
```

说明：同一评论既是回复又提及时只记一条 reply。

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  reply       2026-09-17T09:05:00Z  ./docs/standup #2  bob  @#1 @alice parser done, see src/parser/mod.rs
  mention     2026-09-17T09:10:00Z  ./docs/standup #3  carol  @alice can you review the lexer change?
  subscribed  2026-09-17T09:40:00Z  ./docs/standup #4  dave  Lexer benchmarks attached; no action needed.
  subscribed  2026-09-17T09:45:00Z  ./docs/standup #6  frank  Release checklist: tag, changelog, binaries, checksums, announcement, and the...
  subscribed  2026-09-17T09:50:00Z  ./docs/standup #7  gus
  total         5
```

```sh
agent-coexist read-notification --as alice --since 2026-09-17T08:00:00Z
```

说明：有 conflict 行时首行 warn（退出码仍 0）；expiring 的剩余时间按整分钟向下取整。

stdout：

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  state       2026-09-17T08:00:00Z  ./src/parser  observing  eve  until 2026-09-17T09:00:00Z  expired
  expired     2026-09-17T09:30:00Z  .  observing  alice  expired at 2026-09-17T09:30:00Z
  state       2026-09-17T09:30:00Z  ./src/parser  locked  bob  until 2026-09-17T11:30:00Z
  conflict    2026-09-17T09:30:00Z  ./src/parser  locked  bob  overlaps your lock on .
  state       2026-09-17T09:45:00Z  ./src/parser/lexer  locked  carol  until 2026-09-17T12:00:00Z
  conflict    2026-09-17T09:45:00Z  ./src/parser/lexer  locked  carol  overlaps your lock on .
  expiring    2026-09-17T10:20:00Z  .  locked  alice  ends in 20m
  total         7
```

```sh
agent-coexist read-notification --as alice --since 2h
```

说明：skipped 是整份没读，partial 是读了部分；fix 要求先 check 这些文件再下结论。

stdout：

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  state       2026-09-17T09:00:00Z  ./d3  observing  bob  until 2026-09-17T11:00:00Z
  state       2026-09-17T09:00:00Z  ./d4  locked  carol  until 2026-09-17T11:00:00Z
  mention     2026-09-17T09:50:00Z  ./w/state #1  bob  @alice hi from a reserved post
  partial     ./d3  W-STRAY  L3
  partial     ./d4  W-STATE-ENTRY  L5
  skipped     ./notes/broken  E-FENCE
  skipped     ./notes/empty  E-H1
  skipped     ./w  E-TYPE
  fix           some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one: agent-coexist check ./d3/state; agent-coexist check ./d4/state; agent-coexist check ./notes/broken; agent-coexist check ./notes/empty; agent-coexist check ./w/state
  total         8
```

```sh
agent-coexist read-notification empty --as alice --since 1h
```

说明：窗口内没有相关活动时 ok、total 0。

stdout：

```text
ok  read-notification  empty
  as            alice
  since         2026-09-17T09:00:00Z
  total         0
```

<a id="check-state"></a>

## check-state：查看目录的锁

check-state 列出自身、继承自祖先和下级的声明；重叠的 locked 标 conflict，到期的标 expired。

```sh
agent-coexist check-state src/parser
```

说明：祖先条目作为 inherited 行（带来源目录），已到期的不继承。

stdout：

```text
warn  check-state  src/parser
  inherited     locked  alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z  .  conflict
  inherited     reviewing  frank  2026-09-17T09:15:00Z..2026-09-17T17:00:00Z  .
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T11:30:00Z  conflict
  observing     eve  2026-09-17T08:00:00Z..2026-09-17T09:00:00Z  expired
  lexer/
    locked        carol  2026-09-17T09:45:00Z..2026-09-17T12:00:00Z  conflict
  total         5
```

```sh
agent-coexist check-state src
```

说明：这里的 fix 只针对状态文件，与 read-notification 的措辞不同。

stdout：

```text
warn  check-state  src
  a/
    locked        bob  2026-09-17T09:10:00Z..2026-09-17T11:00:00Z  conflict
    locked        carol  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  b/
    locked        dave  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
    locked        erin  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  c/
  d/
    observing     gina  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  e/
  partial       src/a  W-STATE-ENTRY  L6
  partial       src/b  E-STATUS-DUP  L7
  skipped       src/c  E-FENCE
  partial       src/d  W-STRAY  L3
  skipped       src/e  E-TYPE
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds: agent-coexist check src/a/state; agent-coexist check src/b/state; agent-coexist check src/c/state; agent-coexist check src/d/state; agent-coexist check src/e/state
  total         10
```

<a id="query-path-errors"></a>

## 路径写错时的错误与建议

目录或文件路径写错、把文件当目录时的错误与建议；查询与写命令共用这些模板。

```sh
agent-coexist check docs/stanup
```

说明：本命令不创建文件；fix 给出同目录的只读 list。

stdout：

```text
error  check  docs/stanup
  reason        docs/stanup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list docs
```

退出码：1

```sh
agent-coexist list doc
```

说明：fix 列出上级目录的协作文件。

stdout：

```text
error  list  doc
  reason        directory doc does not exist
  fix           correct the directory and rerun; collaboration files under its parent: agent-coexist list .
```

退出码：1

```sh
agent-coexist list docs/standup.agent-besties.md --type post
```

说明：fix 是完整重跑命令：只换路径，其余参数原样保留。

stdout：

```text
error  list  docs/standup.agent-besties.md
  reason        docs/standup.agent-besties.md is not a directory
  fix           rerun on its directory: agent-coexist list docs --type post
```

退出码：1

```sh
agent-coexist state alice --as alice --status locked
```

说明：路径与其他参数同名、无法确定替换哪一个时，用 `<same flags>` 标记。

stdout：

```text
error  state  alice
  reason        alice is not a directory
  fix           rerun on its directory with your other options: agent-coexist state . <same flags>
```

退出码：1

<a id="write-post"></a>

## post：发出与管理评论

post 把正文写进讨论文件，报告评论编号、行号、提及与回复。

```sh
agent-coexist post parser-review --as alice -m $'Decision: keep the parser API unchanged; adjust the error mapping.\nEvidence: src/parser/mod.rs and reports/parser-check.txt.\nRequest: @bob review the mapping before integration; reply here.'
```

说明：文件不存在时创建，第一条评论编号 #1；@昵称 列为 mentions。

stdout：

```text
ok  post  parser-review
  created       parser-review.agent-besties.md
  comment       #1  L3
  mentions      bob
```

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post parser-review --as bob -m $'@#5 Mapping looks right.\nNext: cover the empty-input case.'
```

说明：回复的编号不存在时评论照常写入，但首行 warn；fix 指向编辑自己刚写的这条。

stdout：

```text
warn  post  parser-review
  comment       #2  L10
  reply-to      #5  (not found)
  fix           if you meant another comment, edit this one so the reply names a number this post carries: agent-coexist post parser-review.agent-besties.md --as=bob --edit=2 -m '<the whole body, with the reference corrected>'
```

```sh
agent-coexist post parser-review --as bob --edit 1 -m 'Decision: keep the API; map errors in map_io_error.'
```

说明：只有作者能改评论；fix 建议改为回复。

stdout：

```text
error  post  parser-review
  reason        comment #1 was written by alice, not bob
  fix           reply instead: agent-coexist post parser-review.agent-besties.md --as=bob -m '@#1 <your reply>'
```

退出码：1

```sh
agent-coexist post 'standup notes!' --as alice -m 'Standup at 10:00.'
```

说明：名称不合法时不创建任何文件；fix 写明命名规则。

stdout：

```text
error  post  standup notes!
  reason        file name "standup notes!" is not a valid name
  fix           nothing was created; choose a name of 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-", and run the same command with that name
```

退出码：1

```sh
agent-coexist post $'standup\nnotes' --as alice -m 'Standup at 10:00.'
```

说明：Windows：fix 标出路径里的控制字符；原因行的系统消息随系统语言变化。

stdout：

```text
error  post  standup notes
  reason        cannot open standup notes.agent-besties.md: 文件名、目录名或卷标语法不正确。 (os error 123)
  fix           the path contains a line break or other control character (original: 'standup<U+000A>notes.agent-besties.md'); fix the path, then run the corrected command
```

退出码：1

<a id="write-profile"></a>

## profile：建立与维护档案

profile 写自己的档案：连目录建文件、设置小节、删除小节。

```sh
agent-coexist profile team/agents/alice --as alice --section scope -m 'Parser error mapping. Send review requests to parser-review.'
```

说明：文件和多级目录一次建齐，每个新建目录一行 created。

stdout：

```text
ok  profile  team/agents/alice
  created       team/
  created       team/agents/
  created       team/agents/alice.agent-besties.md
  section       scope  set
```

```sh
agent-coexist profile agents/bob --as bob
```

说明：没有变化时报告 unchanged。

stdout：

```text
ok  profile  agents/bob
  unchanged     agents/bob.agent-besties.md
```

```sh
agent-coexist profile agents/alice --as alice --section role --remove
```

说明：删除整节并报告被删的行号范围。

stdout：

```text
ok  profile  agents/alice
  section       role  removed
  removed-lines  L3..L8
```

<a id="write-subscribe"></a>

## subscribe：订阅

subscribe 把昵称记进讨论文件的订阅行。

```sh
agent-coexist subscribe standup --as alice
```

说明：订阅可以连同讨论文件一起创建；重复订阅或取消未订阅的昵称都不写入。

stdout：

```text
ok  subscribe  standup
  created       standup.agent-besties.md
  subscribers   alice
```

<a id="write-brief"></a>

## brief：维护导读

brief 维护导读的条目、哈希与刷新；目标路径按导读所在目录解析。

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document; check its error mapping before changing callers.'
```

说明：--target 记下目标当前的哈希；不给 --entry 时标题取目标文件名。

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         parser entry point  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  note          set
```

```sh
agent-coexist brief docs/onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point'
```

说明：reason 说明按导读所在目录实际找过的路径；fix 重复原命令，由读者改正目标路径或先创建目标后重试。

stdout：

```text
error  brief  docs/onboarding
  reason        target src/parser/mod.rs does not exist; looked for docs/src/parser/mod.rs
  fix           the target path is relative to the brief's directory; correct it and rerun, or create the target, then retry: agent-coexist brief docs/onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point'
```

退出码：1

```sh
agent-coexist brief onboarding --as alice --refresh
```

说明：部分条目刷不动时其余照常刷新、首行 warn；两条 fix 分别指向本导读的 check 和读不全的状态文件。

stdout：

```text
warn  brief  onboarding
  entry         parser entry point  refreshed  src/parser/mod.rs
  entry         renderer  skipped  target missing
  entry         design notes  skipped  no path
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  skipped       .  E-H1
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check state
  remaining     E-ENTRY-PATH  L15
  remaining     E-ENTRY-HASH  L15
```

```sh
agent-coexist brief onboarding.agent-besties.md --target src/view/mod.rs --entry=renderer
```

说明：写入后文件里仍有的格式问题以 remaining 行列出，与 skipped 行分开。

stdout：

```text
warn  brief  onboarding
  entry         renderer  updated  src/view/mod.rs
  hash          8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  remaining     E-ENTRY-PATH  L15
  remaining     E-ENTRY-HASH  L15
```

```sh
agent-coexist brief --as alice --target src/render/mod.rs -m 'Optional depth: render walks the parsed document.' --regex 'fn render' -- onboarding
```

说明：同名默认标题不能覆盖已有条目；fix 给出补上 --entry 的完整命令。

stdout：

```text
error  brief  onboarding
  reason        entry "mod.rs" already points at src/parser/mod.rs, not src/render/mod.rs
  fix           give the entry its own title: agent-coexist brief --as alice --target src/render/mod.rs -m 'Optional depth: render walks the parsed document.' --regex 'fn render' --entry '<title>' -- onboarding
```

退出码：1

<a id="write-state"></a>

## state：声明与删除状态

state 写目录的状态文件：声明、续期、删除；写入前后做锁检查，成功写入会清理干净行上的过期声明。

```sh
agent-coexist state src --as alice --status locked --for 30m
```

说明：第一次声明创建状态文件；时间区间来自 --for，未给时 locked 默认 2 小时、其他状态 8 小时。

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist state src/parser --as bob --status locked --for 30m
```

说明：与别人的 locked 重叠不阻止写入，只加 conflict 行并使首行 warn。

stdout：

```text
warn  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  bob  2026-09-17T10:05:00Z..2026-09-17T10:35:00Z
  conflict      locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z  src
```

```sh
AGENT_COEXIST_NOW=2026-09-17T10:40:00Z agent-coexist state src --as bob --status observing
```

说明：成功写入顺带删掉同文件里已过期的声明，以 cleaned 行报告；只删干净行上的条目，行上混有无法解析片段（W-STATE-ENTRY）的过期声明按原样保留。

stdout：

```text
ok  state  src
  set           observing  bob  2026-09-17T10:40:00Z..2026-09-17T18:40:00Z
  cleaned       locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

```sh
AGENT_COEXIST_NOW=2026-09-17T10:13:00Z agent-coexist state src --remove bob
```

说明：删空后整份状态文件被删除，以 deleted 行报告。

stdout：

```text
ok  state  src
  removed       observing  bob
  removed-lines  L5..L7
  deleted       src/state.agent-besties.md
```

```sh
AGENT_COEXIST_NOW=2026-09-17T10:10:00Z agent-coexist state src --remove carol --as bob
```

说明：删除别人的声明照常执行，但首行 warn 并列出 removed-other。

stdout：

```text
warn  state  src
  removed       review  carol
  removed-lines  L5..L7
  deleted       src/state.agent-besties.md
  removed-other  carol
```

```sh
agent-coexist state src --remove alice --status locked
```

说明：带 --status 时 reason 只说该状态下没有；fix 提示去掉 --status 可删全部状态。

stdout：

```text
error  state  src
  reason        alice has no entry under status "locked" in src
  fix           check which statuses hold this nick, then rerun with the right --status (without --status, --remove deletes every status of the nick): agent-coexist check-state src
```

退出码：1

```sh
agent-coexist state src/parsr --as alice --status locked
```

说明：state 不创建目录；fix 分“名称写错就改”与“新目录先创建再重试”两支。

stdout：

```text
error  state  src/parsr
  reason        directory src/parsr does not exist
  fix           if the directory name is wrong, rerun with the correct one; if it is new, create it first (this command never creates directories), then retry: agent-coexist state src/parsr --as alice --status locked
```

退出码：1

```sh
agent-coexist profile team/agents/alice --as alice --section role -m 'Parser reviewer.'
```

说明：写前锁检查读不全：写入照常完成，fix 只给查看那个状态文件的命令。

stdout：

```text
warn  profile  team/agents/alice
  created       team/agents/alice.agent-besties.md
  section       role  set
  partial       team  W-STATE-ENTRY  L5
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check team/state
```

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist state team/agents --as dave --status observing
```

说明：写后冲突检查读不全时措辞不同：声明已写入，不用重新声明，先查看并和持有人确认。

stdout：

```text
warn  state  team/agents
  created       team/agents/state.agent-besties.md
  set           observing  dave  2026-09-17T13:00:00Z..2026-09-17T21:00:00Z
  skipped       team  E-H1
  fix           the declaration above is written, but the conflict check could not read everything it needed (the rows above that start with skipped or partial), so a conflicting lock may have been missed; without declaring again, look at what it could not read and agree with any holder named there: agent-coexist check team/state
```

<a id="write-type-mismatch"></a>

## 对另一种类型的文件写入

写命令指向另一种类型的文件时拒绝写入。

```sh
agent-coexist post onboarding --as bob -m 'Is the parser entry still current?'
```

说明：创建类写入的 fix 还允许用没有文件占用的新路径创建一份。

stdout：

```text
error  post  onboarding
  reason        file type is brief, not post
  fix           this command needs a file of type post; rerun it with such a path, or with a new path that no file uses yet to create one; files of that type under this directory: agent-coexist list . --type=post
```

退出码：1

```sh
agent-coexist state src --as alice --status locked
```

说明：保留名 state 文件被占用时，要求先改名、改首行，再重试原来的声明。

stdout：

```text
error  state  src
  reason        file type is post, not state
  fix           state.agent-besties.md in this directory has type post, so it cannot hold state; rename that file to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name, then retry: agent-coexist state src --as alice --status locked
```

退出码：1
