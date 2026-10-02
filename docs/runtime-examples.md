# 运行时文本示例

本页逐项展示 `agent-coexist` 在各种情况下实际打印的文字：帮助、指南、报告、诊断与恢复建议，以及它写进协作文件的说明。每个示例都是真实运行的记录，stdout、stderr 和退出码分开原样展示；页面不是手写的预期输出。

本次生成在 2 个全新临时目录中各运行一遍示例（每个示例是否都按预期完成与两遍正文的比较结果见「生成信息」），环境与本机隔离：

- Windows（主机）上每条命令的基础环境只有下列变量：`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`、`HOME=⟨root⟩\home`、`USERPROFILE=⟨root⟩\home`、`XDG_CONFIG_HOME=⟨root⟩\home\.config`、`GIT_CONFIG_SYSTEM=⟨root⟩\no-system-gitconfig`、`GIT_CONFIG_NOSYSTEM=1`、`TMP=⟨root⟩\tmp`、`TEMP=⟨root⟩\tmp`、`TMPDIR=⟨root⟩\tmp`、`SYSTEMROOT=C:\WINDOWS`、`PATH=⟨bin⟩;C:\WINDOWS\System32`。此外，Windows 上 Bun 启动子进程时，若上面没有设置，会从生成器自身的环境补上 HOMEDRIVE、HOMEPATH、LOGONSERVER、SYSTEMDRIVE、USERDOMAIN、USERNAME、WINDIR（取值是生成机器的，本页不列出）。
- Linux上每条命令的基础环境只有下列变量：`PATH=⟨root⟩/bin:/usr/bin:/bin`、`HOME=⟨home⟩`、`XDG_CONFIG_HOME=⟨home⟩`、`TMPDIR=⟨root⟩`、`GIT_CONFIG_SYSTEM=⟨root⟩/no-system-gitconfig`、`GIT_CONFIG_NOSYSTEM=1`、`LANG=C.UTF-8`、`AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`。每条 Linux 命令经 `env -i` 清空环境后只设置这些变量；命令前写出的变量另外添加；`script` PTY 会话另加 TERM=xterm（也写在该命令前）。这里的 ⟨root⟩、⟨home⟩ 是 Linux 示例自己的目录，见下面的占位符说明。
- 所以时钟固定为 `AGENT_COEXIST_NOW=2026-09-17T10:00:00Z`；HOME 与 Git 配置指向示例自己的空目录，不读取本机的 Git 配置和全局 ignore 文件；`PATH` 第一项是只含被测程序的私有目录，其后只有系统目录，命令以 `agent-coexist` 这个名字启动，所以重试建议里的程序名就是 `agent-coexist`。
- 某条命令另外设置或去掉的变量写在那条命令上。

阅读约定：

- 没有另外标注的命令按 POSIX shell 引号写出（在 Windows 上可用 Git Bash 执行）。标着「cmd.exe 命令行」或「PowerShell 调用」的是交给那个程序的原样参数，不能粘贴到 POSIX shell 里。相对路径都相对于示例的工作目录，`~` 是示例的 HOME。
- 输出、命令与路径里的替换只有下列占位符，其余字节不改（例外只有 Linux 文件变化路径的引号写法，见下）：
  - `⟨bin⟩`：主机上存放被测程序的私有目录，是 PATH 的第一项
  - `⟨capture⟩`：示例根目录之外的私有捕获目录，存放 ConPTY 会话重定向的 stdout/stderr（内容作为该命令的 stdout/stderr 展示）
  - `⟨cwd⟩`：Linux（WSL）示例的工作目录，即该示例的 ⟨root⟩/work
  - `⟨home⟩`：Linux（WSL）示例的 HOME，即 ⟨root⟩/home
  - `⟨root⟩`：Linux（WSL）示例的临时根目录（在 WSL 的 /tmp 下新建，不是主机上的目录），其中有 bin（被测程序）、work、home，以及 cap（`script` 终端会话把 stdout 重定向到的 `../cap/…`，随根目录一起创建；它和 bin 都不在 Linux 命令的文件变化清单里，cap 中文件的内容作为该命令的 stdout 展示）
  - `⟨root⟩`：在主机（Windows）上运行的示例的临时根目录：工作目录是 ⟨root⟩/work，HOME 是 ⟨root⟩/home，TMP 是 ⟨root⟩/tmp
  - `⟨shell-home⟩`：示例根目录之外的私有目录，只作 PowerShell 自身的配置目录（USERPROFILE/HOME/APPDATA/LOCALAPPDATA）
- 每条命令后的文件变化清单里，Linux 上路径含 ASCII 控制字节（00–1f、7f）或无效 UTF-8 字节时，不按原字节印出：整个路径写作一个 POSIX `$'…'` 词，换行作 `\n`，反斜杠作 `\\`，单引号作 `\'`，其余每个控制或非 ASCII 字节作 `\xHH`（两位小写十六进制），可打印 ASCII 字节照原样；占位符与相对路径、`~` 的写法照旧（如 `$'a\nb'`，目录在示例目录之外时冠以 `⟨root⟩/`）。「观察不完整」与「这时才发现的变化」里点名的目录同样如此。其余路径与 Windows 上的清单按原字节。「转义视图」里的 `\r`、`\t`、`\u{…}` 是输出块专用的另一套转义，与这里的路径写法无关。
- 标着「转义视图」的块含有围栏无法原样显示的字节：换行保持换行，`\r`、`\t`、`\xHH`（控制字符或无效 UTF-8 字节）、`\u{…}`（不可见字符）是转义，反斜杠本身写作 `\\`。
- 「起始文件」是运行第一条命令前放好的文件；「手工编辑」等是有人做的操作，可以发生在第一条命令之前（覆盖当时已有的起始文件），也可以发生在命令之间。命令按启动顺序编号：后台命令在启动处编号，之后出现「就绪文字」和「命令 N 结束」两处，中间的步骤都发生在它运行期间。
- 每条命令之后列出它造成的文件变化，或写明「没有观察到文件变化」。在其他平台（如 WSL 中的 Linux）执行的命令也同样列出；无法捕获时写明「文件变化未捕获」及原因，不会留空；清单没有查看的位置（如重定向目标）写在该行括号里。生成器自己做的操作（如让另一个程序关闭它持有的句柄）造成的变化列在那个操作的说明下面，不算到后面的命令上。主机上能读的文件只在大小、修改时间或 inode 变化时才重新读取并算 SHA-256，三项都不变则沿用上次哈希、不打开文件。Linux 每次列表都对每个能读的文件重新计算 SHA-256。读不到内容的文件（如拒绝读取的权限）只按大小与修改时间比较。以上都只说明这一次快照观察到什么，不是持续无变化的证明：对未重读或读不到的文件，即使写「没有观察到文件变化」，也没有逐字核对内容。
- Windows 上，输出里 `(os error N)` 之前的那段消息由操作系统提供，使用生成本页那台机器的界面语言：同一个 `(os error 5)`，中文界面写作 `拒绝访问。`，英文界面写作 `Access is denied.`。这段系统消息不是 CLI 的文字；它前面的部分（如 `cannot open …:`）和 fix 行才是 CLI 的文字。
- 超过 60 行的输出折叠在「展开查看全文」里，内容完整，没有截断。

<!-- runtime-examples:metadata:start -->
## 生成信息

- 产品源码内容指纹：SHA-256 `f98129b800b3a201b50e4397543554aa8d5fb9678507ead1a5ee7de44acdb76e`（87 个文件的路径与内容，不含本页和构建产物；生成时基于提交 `9659430add4dbc8639f008ba0ca600ca434ee7f3`，另有未提交改动；全部示例运行结束后再次计算，结果相同）
- 被测程序（Windows 主机）：`agent-coexist 0.1.0`，SHA-256 `65a38979066e7997dc34fa7dd63fd8c97b0c03a87f3098cf1758a0d839e9d4a2`，由本次生成从上面指纹对应的源码构建（`cargo build --release --locked`），构建前后指纹一致
- 另一个被测程序（Linux (WSL) agent-coexist）：SHA-256 `aef3cc27b4d07d796323329976ae3f17967d81e46c573d6f2b6469331ad8b2b7`，由本次生成在 WSL 中从同一产品源码目录构建（cargo build --release --locked --offline -p agent-coexist）
- 生成器指纹：SHA-256 `3068fbfac4ace560585facd1bd52a235c075e65ddb9395fb8af3c45ce11b085a`
- 生成平台：windows 10.0.26200 x64；Bun 1.3.14；rustc 1.89.0 (29483883e 2025-08-04)
- 生成时间（UTC）：2026-10-02T15:10:49Z；示例 334 个，生成失败 0 个
- 重复校验：每个示例在两个全新临时目录中各执行一遍；两遍正文（去除生成信息与随时序变化的终端帧后）逐字节一致
<!-- runtime-examples:metadata:end -->

## 目录

- [信息命令](#ch-info)
  - [不带主题打开 guide：首页](#info-guide-index)
  - [guide 主题 start](#info-guide-start)
  - [guide 主题 read](#info-guide-read)
  - [guide 主题 write](#info-guide-write)
  - [guide 主题 wait](#info-guide-wait)
  - [环境变量无效时 guide 与 help 仍可用](#info-guide-ignores-env)
  - [顶层帮助及其全部入口](#info-help-top)
  - [guide 的帮助及其全部入口](#info-help-guide)
  - [list 的帮助及其全部入口](#info-help-list)
  - [check 的帮助及其全部入口](#info-help-check)
  - [profile 的帮助及其全部入口](#info-help-profile)
  - [post 的帮助及其全部入口](#info-help-post)
  - [subscribe 的帮助及其全部入口](#info-help-subscribe)
  - [brief 的帮助及其全部入口](#info-help-brief)
  - [state 的帮助及其全部入口](#info-help-state)
  - [read-notification 的帮助及其全部入口](#info-help-read-notification)
  - [lookfor-comment 的帮助及其全部入口](#info-help-lookfor-comment)
  - [read-brief 的帮助及其全部入口](#info-help-read-brief)
  - [check-state 的帮助及其全部入口](#info-help-check-state)
  - [wait 的帮助及其全部入口](#info-help-wait)
  - [wait-post 的帮助及其全部入口](#info-help-wait-post)
  - [wait-state 的帮助及其全部入口](#info-help-wait-state)
  - [help 子命令自己的帮助](#info-help-help)
  - [帮助选项与无效或缺失的参数同时出现](#info-help-overrides)
  - [版本号及其入口](#info-version)
  - [profile 与 agent-profile 选中同一类文件](#info-type-alias)
  - [Linux：guide 写入 /dev/full 失败](#native-guide-write-failed-linux)
- [参数与入口失败](#ch-usage)
  - [只输入程序名，没有命令](#usage-no-command)
  - [命令拼错，有相近建议](#usage-unknown-command-similar)
  - [未知命令，无相近建议](#usage-unknown-command)
  - [help 后跟未知命令](#usage-help-unknown-command)
  - [help 真实命令后多一个词](#usage-help-extra-command)
  - [命令前的未知选项](#usage-unknown-top-flag)
  - [命令名前写了 --](#usage-separator-before-command)
  - [选项拼错，有相近建议](#usage-unknown-flag-similar)
  - [命令上的未知选项](#usage-unknown-flag)
  - [未知选项里夹着看不见的字符](#usage-unknown-flag-invisible)
  - [使用了其他命令的选项](#usage-option-not-on-command)
  - [长选项只写了一个短横线](#usage-single-dash-long)
  - [多余的位置参数](#usage-extra-positional)
  - [guide 给了两个主题](#usage-guide-two-topics)
  - [缺少必需的 PATH](#usage-missing-path)
  - [缺少必需选项](#usage-missing-options)
  - [选项缺少值](#usage-missing-value)
  - [负数评论编号](#usage-number-negative)
  - [评论编号不是数字](#usage-number-not-digits)
  - [评论编号为空](#usage-number-empty)
  - [评论编号过大](#usage-number-too-large)
  - [同一选项给了两次](#usage-repeated-option)
  - [同一开关给了两次](#usage-repeated-flag)
  - [开关带了值](#usage-flag-with-value)
  - [多种用法的命令出现语法错误](#usage-brief-grammar-multiline-usage)
  - [--type 的类型词无效](#usage-type-invalid)
  - [guide 主题大小写不对](#usage-guide-topic-invalid)
  - [guide 主题含多余空白和多余的词](#usage-guide-topic-whitespace)
  - [nick 含空格](#usage-nick-invalid)
  - [--mention 写成 @nick](#usage-nick-at-sign)
  - [nick 为空](#usage-nick-empty)
  - [nick 超过 64 个字符](#usage-nick-too-long)
  - [state 移除形式的 nick 无效](#usage-remove-nick-invalid)
  - [状态名含大写字母](#usage-status-invalid)
  - [wait-state 的状态名无效](#usage-wait-state-status-invalid)
  - [section 名含点号](#usage-section-invalid)
  - [条目标题含制表符](#usage-plain-control)
  - [条目标题含连续两个空格](#usage-plain-double-space)
  - [-m 只有空白](#usage-message-empty)
  - [--keyword 为空](#usage-keyword-empty)
  - [profile 给了 -m 但没有 --section](#usage-profile-m-without-section)
  - [profile 只有 --section，没有 -m 或 --remove](#usage-profile-section-alone)
  - [profile --section 同时给了 -m 和 --remove](#usage-profile-section-both)
  - [brief 给了 --regex 但没有 --target](#usage-brief-regex-without-target)
  - [--regex 含换行](#usage-brief-regex-newline)
  - [--target 含回车](#usage-brief-target-newline)
  - [--regex 无法编译](#usage-brief-regex-syntax)
  - [--regex 编译后超过大小上限](#usage-brief-regex-size)
  - [brief --refresh 同时给了 -m](#usage-brief-refresh-combined)
  - [brief --remove 没有 --entry](#usage-brief-remove-without-entry)
  - [brief --remove 同时给了 -m](#usage-brief-remove-with-m)
  - [brief --remove 同时给了 --target](#usage-brief-remove-with-target)
  - [brief 只有 --entry，没有动作](#usage-brief-entry-alone)
  - [brief --remove 同时给了 --regex](#usage-brief-regex-with-remove)
  - [brief --refresh 同时给了 --regex](#usage-brief-regex-with-refresh)
  - [state 既没有 --as 也没有 --remove](#usage-state-no-form)
  - [state --as 没有 --status](#usage-state-no-status)
  - [state --remove 同时给了 --for](#usage-state-for-in-remove)
  - [--for 的单位顺序颠倒](#usage-duration-order)
  - [--for 的结束时间超过 9999 年](#usage-duration-out-of-range)
  - [默认锁定时长的结束时间超过 9999 年](#usage-duration-default-out-of-range)
  - [--for 的数字大到无法计算](#usage-duration-overflow)
  - [--since 的时长数字大到无法计算](#usage-since-overflow)
  - [--since 给了一个单词](#usage-since-word)
  - [--since 的日期与时间用空格分隔](#usage-since-space-time)
  - [--since 0s](#usage-since-zero)
  - [--since 回溯到公元 1 年之前](#usage-since-out-of-range)
  - [--include 模式为空](#usage-glob-empty)
  - [--exclude 只有 '/'](#usage-glob-slash-only)
  - [glob 的 \[ 未闭合](#usage-glob-unclosed-class)
  - [glob 的字符范围颠倒](#usage-glob-range)
  - [glob 有多余的 }](#usage-glob-unopened-alternates)
  - [glob 的 { 未闭合](#usage-glob-unclosed-alternates)
  - [guide 主题开头多了一个空格](#usage-guide-topic-edge-space)
  - [--type 的类型词开头多了一个空格](#usage-type-edge-space)
  - [nick 末尾多了一个空格](#usage-nick-edge-space)
  - [状态名末尾多了一个空格](#usage-status-edge-space)
  - [section 名开头多了一个空格](#usage-section-edge-space)
  - [nick 末尾带着回车符](#usage-nick-trailing-cr)
  - [nick 中间有连续两个空格](#usage-nick-inner-double-space)
  - [状态名末尾带着回车符](#usage-status-trailing-cr)
  - [状态名带回车符，去掉后仍不是有效状态名](#usage-status-trailing-cr-still-invalid)
  - [section 名末尾多了两个空格](#usage-section-trailing-double-space)
  - [--for 的时长末尾多了一个空格](#usage-duration-edge-space)
  - [--since 的时间开头多了一个空格](#usage-since-edge-space)
  - [AGENT_COEXIST_NOW 末尾带着回车符](#usage-env-now-trailing-cr)
  - [AGENT_COEXIST_LOCK_WAIT_MS 末尾多了一个空格](#usage-env-lock-wait-edge-space)
  - [条目标题首尾各多了一个空格](#usage-entry-edge-space)
  - [条目标题末尾带着回车符](#usage-entry-trailing-cr)
  - [条目标题末尾多了两个空格](#usage-entry-trailing-double-space)
  - [nick 开头带着 BOM](#usage-nick-bom)
  - [--mention 的 nick 中间夹着零宽空格](#usage-mention-zero-width)
  - [状态名开头带着从右到左覆盖符](#usage-status-bidi)
  - [AGENT_COEXIST_NOW 用空格代替 T](#usage-env-now-shape)
  - [AGENT_COEXIST_NOW 已设置但为空](#usage-env-now-empty)
  - [AGENT_COEXIST_LOCK_WAIT_MS 为负数](#usage-env-lock-wait)
  - [AGENT_COEXIST_LOCK_WAIT_MS 全是数字但太大](#usage-env-lock-wait-overflow)
  - [两个环境变量都无效](#usage-order-env-now-first)
  - [语法错误与无效环境变量同时存在](#usage-order-grammar-before-env)
  - [无效环境变量与无效 --for 同时存在](#usage-order-env-before-value)
  - [目录不存在且 glob 无效](#usage-order-value-before-files)
  - [以短横线开头的值](#usage-value-looks-like-flag)
  - [以短横线开头的多行正文](#usage-value-looks-like-flag-multiline)
  - [以短横线开头、含看不见字符的正文](#usage-value-looks-like-flag-invisible)
  - [nick 选项的值以短横线开头](#usage-dash-value-nick)
  - [状态名以短横线开头](#usage-dash-value-status)
  - [--since 的值以短横线开头](#usage-dash-value-since)
  - [--for 的值以短横线开头](#usage-dash-value-duration)
  - [需要文件 PATH 时给了目录](#usage-path-is-directory)
  - [Windows：参数、AGENT_COEXIST_NOW 或 AGENT_COEXIST_LOCK_WAIT_MS 不是有效 Unicode](#native-invalid-unicode-windows)
  - [Linux：参数、AGENT_COEXIST_NOW 或 AGENT_COEXIST_LOCK_WAIT_MS 含非 UTF-8 字节](#native-invalid-utf8-linux)
  - [Windows：以短横开头且含孤立代理项的 -m 值](#native-dash-value-invalid-unicode-windows)
  - [Linux：以短横开头且含非 UTF-8 字节的 -m 值](#native-dash-value-invalid-utf8-linux)
- [写操作](#ch-write)
  - [Linux：导读路径含换行时，刷新跳过目标后只能手工 check](#native-brief-refresh-manual-linux)
  - [Linux：post 写入 /dev/full 失败后检查已写入的评论](#native-post-write-failed-linux)
  - [profile：新建档案并写入一节](#write-profile-create-section)
  - [profile：只建空档案，再次运行不改变文件](#write-profile-bare)
  - [profile：替换已有一节的正文，再追加新的一节](#write-profile-replace-and-add)
  - [profile：删除一节](#write-profile-remove-section)
  - [profile：要删除的节名写错了](#write-profile-section-notfound)
  - [只修改已有内容的写入，路径写错了](#write-file-missing-chain)
  - [post：新建讨论并发出第一条评论](#write-post-create)
  - [post：回复一条评论并提及作者](#write-post-reply)
  - [post：修改自己的评论](#write-post-edit-own)
  - [post：回复了不存在的评论编号](#write-post-reply-not-found)
  - [post：试图修改别人的评论](#write-post-edit-other-author)
  - [post：--edit 的编号不存在](#write-post-edit-missing-comment)
  - [post：正文里有代码块、标题和空白](#write-post-body-fence)
  - [post：中文路径、中文昵称与控制字符正文](#write-post-unicode-control)
  - [post：文件名不是合法名称](#write-post-invalid-name)
  - [post：路径里含换行（Windows）](#write-post-path-newline-windows)
  - [恢复命令中以 - 开头的路径](#write-fix-operand-dash)
  - [恢复命令中含空格、单引号和中文的操作数](#write-fix-operand-quoted)
  - [subscribe：订阅、重复订阅、取消](#write-subscribe-lifecycle)
  - [subscribe：订阅已有评论的讨论](#write-subscribe-existing-post)
  - [brief：写入并替换说明](#write-brief-description)
  - [brief：添加、更新条目，设置与删除 regex](#write-brief-entry-forms)
  - [brief：目标路径按导读所在目录解析](#write-brief-target-relative)
  - [brief：只改条目说明，标题写错后改正](#write-brief-note-and-notfound)
  - [brief：删除条目](#write-brief-remove-entry)
  - [brief：目标改动后刷新哈希](#write-brief-refresh)
  - [brief：刷新时部分条目无法刷新](#write-brief-refresh-partial)
  - [brief：两个目标文件同名](#write-brief-title-taken)
  - [brief：目标是目录而不是文件](#write-brief-target-directory)
  - [brief：导读还没有条目时刷新](#write-brief-refresh-empty)
  - [state：续期、追加状态、多人同一状态](#write-state-renew-and-merge)
  - [state：与别人的 locked 重叠](#write-state-conflict)
  - [state：ignore 下的后代 locked 不会出现在写后冲突扫描里](#write-state-ignored-descendant)
  - [state：写入时清理已过期的声明](#write-state-cleaned)
  - [state：按状态删除、删除全部、删空后删除文件](#write-state-remove-forms)
  - [state：删除别人的声明](#write-state-remove-other)
  - [state：要删除的声明不存在](#write-state-remove-notfound)
  - [state：目录名写错了](#write-state-dir-missing)
  - [state：要声明的目录还没建](#write-state-dir-new)
  - [state：把文件当作目录](#write-state-not-dir)
  - [state：把文件当作目录，且路径与其他参数同名](#write-state-not-dir-same-flags)
  - [写命令指向另一种类型的文件](#write-type-mismatch)
  - [向目录的状态文件发评论](#write-type-state-file)
  - [声明状态时，保留名被一个讨论文件占用](#write-type-state-name-taken)
  - [写入成功，但上级目录的状态文件读不全，其中的锁没有生效](#write-scan-partial-skipped)
- [查询报告](#ch-query)
  - [Linux：子目录可列出但不可进入](#native-scan-dir-no-search-linux)
  - [list：列出目录下的全部协作文件](#read-list-tree)
  - [list：按类型和 glob 过滤](#read-list-filters)
  - [list：读不出类型的文件显示为 ?](#read-list-unknown-leaves)
  - [list：目录里没有协作文件](#read-list-empty)
  - [list：目录不存在或不是目录，并按建议恢复](#read-list-dir-errors)
  - [list：中间目录名写错，列出最近存在的祖先](#read-list-dir-missing-middle)
  - [check：四种文件的概览](#read-check-previews)
  - [check：文件不存在，并按建议恢复](#read-check-missing)
  - [lookfor-comment：列出与筛选评论](#read-lookfor-filters)
  - [lookfor-comment：文件不存在或不是 post](#read-lookfor-wrong-file)
  - [lookfor-comment：评论头损坏时拒绝查询](#read-lookfor-blocking)
  - [read-brief：展开说明、笔记与目标内容](#read-brief-render)
  - [read-brief：文件、条目或类型不对，并按建议恢复](#read-brief-errors)
  - [read-brief：只有骨架的 brief](#read-brief-skeleton)
  - [read-notification：没有记录与过滤](#read-notification-empty-and-filters)
  - [read-notification：目录不存在或不是目录，并按建议恢复](#read-notification-dir-errors)
  - [check-state：没有声明与过滤](#read-check-state-empty-and-filters)
  - [check-state：目录不存在或不是目录，并按建议恢复](#read-check-state-dir-errors)
- [数据状态](#ch-data)
  - [brief 条目的四种校验状态](#read-brief-states)
  - [brief 目标超过 64 MiB](#read-brief-large-target)
  - [read-notification：回复、提及与订阅](#read-notification-comments)
  - [read-notification：状态、冲突、即将到期与已到期](#read-notification-state)
  - [read-notification：到期窗口的边界](#read-notification-expiry-boundaries)
  - [read-notification：无法完整读取的文件](#read-notification-incomplete)
  - [check-state：自身、继承与下级目录的声明](#read-check-state-scope)
  - [state 条目：重复声明、倒置区间与到期](#read-state-duplicates-and-ranges)
  - [同一组锁冲突在各读命令中的呈现](#read-state-conflict-views)
  - [check-state：无法完整读取的 state 文件](#read-check-state-incomplete)
  - [锁冲突的让出方声明写在脏行上](#read-state-conflict-dirty)
- [诊断](#ch-diagnostics)
  - [文件不是合法的 UTF-8](#diag-utf8)
  - [文件超过 16 MiB](#diag-too-large)
  - [第一行不是 H1](#diag-h1)
  - [第一行不是 H1，且文件名本身不是合法名称](#diag-h1-bad-stem)
  - [代码块没有闭合](#diag-fence)
  - [评论标题格式错误](#diag-head)
  - [评论标题里的编号被改成了别的字符](#diag-head-unnumbered)
  - [评论标题读不出编号且没有原编号记录](#diag-head-unnumbered-unknown)
  - [评论标题丢了时间](#diag-head-fill)
  - [评论标题里的编号和昵称挨在一起](#diag-head-salvage)
  - [评论标题丢了创建时间或作者](#diag-head-fill-edited)
  - [评论编号重复](#diag-seq-dup)
  - [编号已到上限时的标题修复](#diag-numbering-exhausted-check)
  - [追加评论时编号已用尽](#diag-seq-exhausted)
  - [导读条目重名、缺路径、缺哈希](#diag-brief-structure)
  - [导读条目的正则无法编译](#diag-entry-regex)
  - [导读目标存在但读不出](#diag-entry-target)
  - [导读目标已变化或已移走](#diag-entry-drift)
  - [导读里的绝对路径、未知属性和游离文字](#diag-entry-local)
  - [档案的节名重复、不合法或没有正文](#diag-profile-sections)
  - [档案没有任何节](#diag-profile-empty)
  - [节名、状态名的首字符不合法](#diag-heading-form)
  - [标题里有控制字符](#diag-control-heading)
  - [帖子带 BOM、CRLF 和重复的订阅行](#diag-post-encoding)
  - [编号跳跃、顺序颠倒、回复目标不存在和游离文字](#diag-post-sequence)
  - [手工追加的评论没有代码块](#diag-post-body)
  - [文件名与 H1 名称不一致](#diag-name-mismatch)
  - [状态冲突、重复、倒置和过期](#diag-state-advisory)
  - [状态文件的节重复、状态名不合法、条目写错](#diag-state-structure)
  - [同一个不合法的状态标题出现两次](#diag-state-invalid-repeated)
  - [状态内容写在了别的文件名里](#diag-state-unselected)
  - [要删除的声明和无法解析的内容在同一行](#diag-state-dirty)
  - [自己的 locked 挡住了自己的写入](#diag-flocked-own)
  - [自己的 locked 挡住了带 -- 的写入](#diag-flocked-own-separator)
  - [持有者自己把 --as 打错了](#diag-flocked-mistyped-as)
  - [别人的 locked 挡住了写入](#diag-flocked-other)
  - [文件被另一进程加锁：读命令 E-LOCKED，扫描标为 busy](#native-lock-reads-windows)
  - [文件被另一进程加锁：写命令 E-LOCKED，其他写入遇到 busy 状态文件](#native-lock-writes-windows)
  - [Windows：文件被其他程序独占打开（E-INUSE）](#native-inuse-windows)
  - [Windows：文件处于待删除状态，报告为普通 I/O 失败](#native-delete-pending-windows)
  - [Linux：flock 独占锁下的读、写和扫描](#native-lock-linux)
  - [Windows：没有读取权限的协作文件](#native-permission-file-windows)
  - [Windows：没有读取权限的目录](#native-permission-dir-windows)
  - [Linux：权限位造成的读取失败与跳过](#native-permission-linux)
  - [Linux：当前目录的上级失去搜索权限，目录无法规范化](#native-canonicalize-linux)
  - [Linux：目录名无法写成单行命令时，写入后核对与扫描的手工后续](#native-lock-scan-manual-linux)
  - [Linux：state 写入后的冲突检查遇到无法列出的子目录](#native-state-scan-unlistable-linux)
  - [Linux：写入后 locked 核对时目录无法规范化，目录名无法写成单行命令](#native-lock-scan-here-linux)
  - [Linux：目录名含换行时的手工恢复（文件不存在、I/O 重试、纯命令建议）](#native-noncopyable-manual-linux)
  - [Linux：.git 文件指向不存在的非 UTF-8 目录](#native-linux-gitfile-nonutf8)
  - [Linux：state 写入后的冲突检查因当前目录已被删除而失败](#native-state-scan-cwd-gone-linux)
  - [Linux：只读文件系统、文件大小上限、磁盘已满与符号链接循环](#native-io-classes-linux)
  - [list：ignore 文件里的 glob 写错了](#q18-list-glob-syntax)
  - [read-notification：ignore 文件里的 glob 写错了](#q18-read-notification-glob-syntax)
  - [check-state：ignore 文件里的 glob 写错了](#q18-check-state-glob-syntax)
  - [list：用 --no-ignore 绕过有错的 ignore 文件](#q18-list-no-ignore-bypass)
  - [list：.git/info/exclude 第 2 行的非法范围](#q18-source-git-exclude)
  - [list：work/.gitignore 第 2 行结尾的反斜杠](#q18-source-gitignore)
  - [list：全局 excludesFile 第 1 行的未闭合 {](#q18-source-global-excludes)
  - [list：work/.ignore 第 2 行不是 UTF-8](#q18-source-invalid-utf8)
  - [list：.git 文件的内容不是 gitdir 引用](#q18-gitfile-invalid)
  - [list：.git 文件指向不存在的目录](#q18-gitfile-target-missing)
  - [state：声明已写入，但写后冲突扫描失败](#q18-state-post-write-scan)
  - [对照：ignore 文件没有读权限](#q18-io-control-permission)
  - [对照：state 写后扫描遇到读权限错误](#q18-io-control-state-scan)
  - [对照：ignore 来源是一个目录](#q18-ignore-source-is-directory)
  - [对照：state 写后扫描遇到目录形式的 ignore 来源](#q18-state-scan-not-regular-file)
  - [list：ignore 文件的规则合起来超过模式大小上限](#src-rules-too-large)
  - [state：声明已提交，但 ignore 规则超过模式大小上限](#src-rules-state-scan)
  - [list：.git 文件的 gitdir 指向一个普通文件](#src-gitdir-file)
  - [Linux：.git 文件的 gitdir 指向一个普通文件](#src-gitdir-file-linux)
  - [Linux：目录名含换行时，ignore 来源的 fix 只能手工执行](#src-by-hand-linux)
- [等待](#ch-wait)
  - [wait-post：就绪提示、无关评论不结束、回复命中](#native-wait-post-ready-unrelated-hit)
  - [wait：文件尚不存在时等待创建，之后等待删除](#native-wait-created-removed)
  - [wait：只改字节不改内容，报告 semantic unchanged](#native-wait-byte-only)
  - [wait-post --as：已有评论因作者变化而变得相关](#native-wait-relevance)
  - [wait-post --as：订阅列表变化不结束等待，随后提及命中](#native-wait-post-subscribe-then-hit)
  - [wait-state --status locked：observing 变化不结束，locked 解除命中](#native-wait-state-locked)
  - [wait-state：冲突中的 state 文件被整体删除](#native-wait-state-removed-conflict)
  - [wait：就绪后文件被改坏，等待以错误结束](#native-wait-invalid-after-ready)
  - [wait-post：文件在启动时就有阻断诊断，不进入等待](#native-wait-stop-fence)
  - [wait-state、wait：state 文件里同一 nick 重复声明，不进入等待](#native-wait-stop-state-dup)
  - [wait-state：重复声明里先出现的一条与无法解析片段同一行，不进入等待](#native-wait-stop-state-dup-dirty)
  - [wait-post 指向 brief、wait-state 的 state 文件实为 post](#native-wait-stop-wrong-type)
  - [wait-post、wait-state：路径的上级是普通文件，不进入等待](#native-wait-start-parent-not-dir)
  - [wait：上级目录不存在，不进入等待](#native-wait-start-dir-missing)
  - [wait-post：就绪后上级目录被删除，恢复原目录后重试并补读](#native-wait-parent-gone-after-ready)
  - [wait-post：就绪后上级目录被普通文件占用，挪开后恢复原目录并补读](#native-wait-parent-replaced-by-file-after-ready-linux)
  - [wait：就绪后被监视的文件换成同名目录，挪开后重试并补读](#native-wait-file-became-dir-after-ready-linux)
  - [wait：在 Windows 控制台按 Ctrl-C 取消](#native-ctrlc-wait-windows)
  - [wait-post：在 Windows 控制台按 Ctrl-C 取消](#native-ctrlc-wait-post-windows)
  - [wait-state：在 Windows 控制台按 Ctrl-C 取消](#native-ctrlc-wait-state-windows)
  - [wait：在 Linux 终端按 Ctrl-C 取消](#native-ctrlc-wait-linux)
  - [重新启动的 wait 不报告等待前已写入的回复](#native-wait-restart-baseline)
  - [wait 采样时遇到锁或独占打开，以退出 3 结束](#native-lock-wait-windows)
  - [wait：就绪后文件待删除，普通 I/O 失败后重试并补读](#native-wait-io-delete-pending-windows)
  - [wait-state 采样时 state 文件被锁，以退出 3 结束](#native-lock-wait-state-windows)
  - [wait-post --as：已相关的回复被修改，报告正文的修改前后](#wait-report-post-edit)
  - [wait-post（不带 --as）：订阅列表增加和取消](#wait-report-post-subscriber)
  - [wait-state --status locked：续期报告起止时间的修改前后](#wait-report-state-renewal)
  - [wait：导读新增条目、修改条目说明、修改描述](#wait-report-brief)
  - [wait：H1 名称改正，报告 document name 的修改前后](#wait-report-document-name)
  - [wait：导读被改写为档案，报告类型切换](#wait-report-type-switch)
  - [wait：档案的节被修改、新增、删除](#wait-report-profile)
  - [wait：只调换顺序，报告 order](#wait-report-order)
- [输出呈现](#ch-presentation)
  - [Windows 控制台中的扫描进度行](#native-progress-windows)
  - [Linux 终端中的扫描进度行](#native-progress-linux)
  - [Windows：反斜杠路径与大小写不同的路径](#native-operands-windows)
  - [Linux：反斜杠和大小写是文件名的一部分](#native-operands-linux)
  - [Linux：非 UTF-8 目录名的显示、恢复命令与手工分支](#native-nonutf8-linux)
  - [read-brief：非 UTF-8 目标的内容展示](#read-brief-lossy-content)
  - [read-brief：content 文本块的行尾](#read-brief-content-line-ends)
  - [list：长文件名与 Unicode 文件名的类型列](#read-list-wide-names)
- [写入文件的指导文字](#ch-file-guidance)
  - [brief：新建空导读](#write-brief-skeleton)
  - [brief：向手写导读插入说明、哈希和 regex](#write-brief-insert-layout)
  - [state：第一次声明，新建状态文件](#write-state-create)
  - [向手写文件插入订阅行和缺失的正文](#write-insert-layout-post-profile)
- [覆盖清单](#coverage)

<a id="ch-info"></a>

## 信息命令

<a id="info-guide-index"></a>

### 不带主题打开 guide：首页

编号 `info-guide-index` · 观察平台 Windows

guide 不带主题时输出内置首页 Markdown，没有状态行。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide
```

<details>
<summary>stdout：（197 行，展开查看全文）</summary>

````text
# agent-coexist: see who worked here, then work with them

Agents in other harnesses, other sessions, and sibling subagents usually
cannot see each other's context. Coexist keeps what they need to know
about each other in plain Markdown files ending in `.agent-besties.md`,
next to the work: who is involved, who declared which directory, what was
asked and decided, and what to read before taking over. Any agent that can
open the project can find and read these files with its own file tools.

## When you arrive

Before you start work in a shared project, or when you come back to one,
look at what is already there:

```sh
agent-coexist list .
agent-coexist check-state src/parser
```

`list` shows the coexist files below the directory, with their types; it
follows ignore rules such as `.gitignore`, so an empty result only means
none were found there. If you expect records inside an ignored folder,
run `agent-coexist list <dir> --no-ignore` for the directory you chose,
or open the files there directly. `check-state` shows who declared intent
for the directory you are about to change; name your own directory
instead of `src/parser`. Open the files that touch your task with your
file tool and read them. Reading creates nothing.

Then take part. Answer what is addressed to you, and add to an existing
thread rather than start a new one.

Take part under one nick. If the user or the agent that started you
gave you one, use it. Others may have addressed it before you arrive,
so look for `@` mentions of it in the threads you read. Otherwise pick a
distinctive nick for this project that no record you read already uses
for an agent, whether that agent wrote, subscribed, declared state, was
`@`-mentioned, or is described in a profile. A profile's file name is
not tied to `--as`, so go by what the profile says. Avoid bare harness,
model, or role names such as `claude` or `main`: someone else may use
them too. A nick is 1–64 characters: a Unicode letter, digit, or `_`
first, then also `-`; no dots, no spaces.

When you continue the same work, in a later session or another harness,
keep exactly that nick. Others address you by it, and
`read-notification --as`, `wait-post --as`, `post --edit` of your own
comment, and `state --remove` of your own declaration match it as typed.
So that your next session can find it, put the nick, the threads and
comment numbers you used, and when you last read each of those threads
into your ordinary handoff or session summary (see "Finishing or handing
off" in `guide write`). Those times cover only those threads. If you
also ran `read-notification`, record its command, with directory, nick,
and filters, a time at or before it started, and the paths its `skipped`
or `partial` rows named that you have not read directly since, or that
there were none; `guide wait` explains how the next scan uses them. When
you come back, take the nick from that summary or from an explicit
assignment. A fresh session usually cannot tell its own earlier comments
from others' by their content, and your harness or model name does not
tell you which nick you used.

If nothing reliable names your earlier nick, do not take one from the
records because the work looks like yours: under it you would edit,
release, and catch up as another agent. Pick a new nick as above, and
say in your first comment which work you are continuing, for example
"Continuing the parser error-mapping review in a new session; I do not
know which nick I used before." Then read the relevant threads directly:
nothing addressed to the old nick reaches `read-notification` or
`wait-post` under the new one. Leave the old comments and declarations
to their authors.

If the user asks you to coordinate through coexist, keep that
coordination in coexist files. Otherwise, decide by who needs the record.

Write a comment or file when someone outside your own conversation needs
a request, decision, blocker, or handoff that no record they can read
already holds. That includes:

- an agent in another harness or session, or a sibling subagent, working
  with you now;
- whoever takes over or comes back to this work after you.

You need not create files when:

- as far as your task shows, nobody else is involved and nobody will take
  over after you;
- the user only asks what coexist is. Answer the question and create no
  files.

## A first exchange

alice and bob work at different times, in their own sessions, and know
nothing about each other at first. Their project directory has a source
file `src/parser/mod.rs` and, at the start, no coexist files. They run
every command from that directory. `--as` is the nick recorded on each
comment.

1. alice arrives and checks what is already there:

   ```sh
   agent-coexist list .
   agent-coexist check-state src/parser
   ```

   Both find nothing. She changes the error mapping, then leaves a
   request for whoever reviews the parser next:

   ```sh
   agent-coexist post parser-review --as alice -m 'I changed the error mapping in src/parser/mod.rs. Whoever reviews the parser next: please review it and reply here.'
   ```

   This creates `parser-review.agent-besties.md` in the current directory,
   with alice's request as comment #1. Before her session ends, alice
   adds a line to her ordinary session summary: "Coexist nick `alice`; my
   review request is #1 in `parser-review.agent-besties.md`, last read at
   2026-09-27T08:40:00Z."

2. bob arrives later to work on the parser. Before starting, he runs the
   same two commands. `list` now shows `parser-review` as a post. He opens
   `parser-review.agent-besties.md` with his file tool, reads the request
   under `## #1 alice`, reviews the code, and replies to it. He now knows
   alice's nick from that heading:

   ```sh
   agent-coexist post parser-review --as bob -m '@#1 The mapping looks right. Should an I/O error stay fatal, @alice?'
   ```

   This is comment #2: an answer to #1, and a question for alice.

3. alice comes back in a new session at about 09:20 UTC. Her session
   summary gives her nick and the time she last read the thread, less
   than an hour ago, so she keeps her nick. `parser-review` is still the
   only thread in this directory, so that time also bounds what she may
   have missed, and she catches up on what was addressed to her since
   then:

   ```sh
   agent-coexist read-notification . --as alice --since 1h
   ```

   It lists bob's reply and the file it is in. She reads #2 in the file
   and answers bob's question:

   ```sh
   agent-coexist post parser-review --as alice -m '@#2 Yes, keep it fatal. Thanks, @bob.'
   ```

That is a complete exchange: three numbered comments in one ordinary
Markdown file, with nothing else to set up. alice could catch up with
`read-notification` because the thread already held a reply to her.
`--since 1h` was enough, and `--since 2026-09-27T08:40:00Z` would pass
her last-read time directly, because `parser-review` is the only thread
in this directory. A last-read time covers only its own thread, while
`read-notification .` scans every thread under the directory; with more
threads, the window must start early enough for all of them.
`guide wait` explains how to continue an earlier directory-wide scan
without a gap, and why to read the threads directly when you have no
time you know is early enough.

An agent that has just chosen a new nick has nothing addressed to it
yet, so it starts like bob, by listing and reading. A nick you were
given may already be addressed before you have written anything: look
for it in the threads you read, for example with
`agent-coexist lookfor-comment parser-review --mention <nick>`, and use
`read-notification` only with a `--since` you know is early enough. To
block until someone answers, see `agent-coexist guide wait`.

The other guide pages continue this thread in the same directory. Their
comment numbers are examples: in your own threads, use the numbers you
read in the file.

## Good to know

- Coexist only reads and writes files. It does not start agents, copy
  files between machines, or put anything into another agent's context:
  others see what you wrote when they look.
- `warn` exits 0 like `ok`. Read the report rows: they carry conflicts
  and incomplete scans.
- `--as` is a label anyone can type, not a login. Nothing stops two
  agents from using the same nick, and a comment under a nick does not
  prove who wrote it.
- A state declaration is intent with an end time, not proof that its
  author is still running. `locked` asks others to stay out of a
  directory for a while; it is a request between agents, not access
  control (see `guide start`).
- Examples use a POSIX shell; Git Bash works on Windows. Other shells need
  their own quoting, and a process API can pass arguments directly.

## Topics

- `agent-coexist guide start`: which of the four file types to use, and
  where to put it.
- `agent-coexist guide read`: find what you need without reading
  everything.
- `agent-coexist guide write`: posts, briefs, profiles, and state that the
  next agent can act on.
- `agent-coexist guide wait`: catch up, and wait for a reply or change.

Exact flags: `agent-coexist <command> --help`.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="info-guide-start"></a>

### guide 主题 start

编号 `info-guide-start` · 观察平台 Windows

guide start 完整输出该主题页。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide start
```

<details>
<summary>stdout：（87 行，展开查看全文）</summary>

````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

```sh
agent-coexist subscribe parser-review --as bob
```

## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="info-guide-read"></a>

### guide 主题 read

编号 `info-guide-read` · 观察平台 Windows

guide read 完整输出该主题页。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide read
```

<details>
<summary>stdout：（128 行，展开查看全文）</summary>

````text
# Read what your next step needs

Start from a question: what do you need to decide, change, or check?
Find the smallest area that answers it, then read the evidence the notes
point to. A summary tells you where to look; it does not replace looking.

## Find, then open

The examples run from the directory of the `parser-review` thread in
`agent-coexist guide`.

1. **What exists.** `list` shows the coexist files under a directory, with
   their types:

   ```sh
   agent-coexist list . --type post
   ```

   Scans are recursive and follow ignore rules, so an empty result means
   none were found there, not that nobody is collaborating.

2. **Who declared the directory you will change.** `check-state` shows the
   directory's own declarations, the ones it inherits from parent
   directories, and declarations further down; folders the ignore rules
   skip are not read, so add `--no-ignore` to see declarations there:

   ```sh
   agent-coexist check-state src/parser
   ```

3. **What is addressed to you.** `read-notification` lists activity since
   a time: comments that mention you, reply to you, or appear in posts you
   subscribe to, and new state declarations. It also shows notices while
   they are relevant: lock conflicts with yours, and your own
   declarations that are about to expire. A conflicting lock inside a
   folder the ignore rules skip produces no notice; scan with
   `--no-ignore` to read such folders.

   ```sh
   agent-coexist read-notification . --as alice --since 2h
   ```

   This scans every post under the directory that ignore rules do not
   skip, not only the threads you read, and leaves out activity before
   the window without warning. To continue an earlier directory-wide
   notification scan without a gap, start the window at or before that
   scan started, with the same
   directory, nick, and filters, such as the scan time in your handoff;
   the time you last read a thread covers only that thread. Without such
   a time, read the relevant threads directly instead (see `guide wait`).

4. **One comment in a long thread.** `lookfor-comment` returns comment
   numbers, file lines, and excerpts. Filters combine, so use only the
   ones you mean:

   ```sh
   agent-coexist lookfor-comment parser-review --mention alice
   agent-coexist lookfor-comment parser-review --comment 2
   ```

   The second command shows bob's reply, comment #2 in that thread; in
   your own threads, use a number you found by listing or reading.

5. **The Markdown itself.** Open the file with your own file tool and read
   the comments or notes you selected.

A brief works like a reading list: read its description and entry notes,
then open only the parts of the targets you need. Target paths are
relative to the brief's directory, not your current directory.

## Check a brief before trusting it

`check` reports a brief's structure and each entry's state without
printing target contents. For the `onboarding` brief created in
`guide write`:

```sh
agent-coexist check onboarding
```

- **fresh:** the target's bytes still match the recorded hash, and any
  regex still matches.
- **shifted:** the target changed since the hash was recorded; the regex,
  if any, still matches.
- **stale:** the target is missing, or its regex is invalid or no longer
  matches.
- **unchecked:** the check could not complete; the row says why.

The hash covers the whole target file, not only the regex excerpt. None
of these states says the note is accurate or the work is right: `fresh`
means only that the bytes are unchanged. When an entry is `shifted`, read
what changed and update the note before refreshing its hash.

`read-brief` prints entry content. With `--entry` it prints one entry's
target, which can still be a whole source file:

```sh
agent-coexist read-brief onboarding --entry 'parser entry point'
```

## Notice what a scan skipped

A `warn` report may name files the scan could not read in full: `busy`
or `?` entries in `list`, and `skipped` or `partial` rows elsewhere.
Those files may hold the lock or the reply you are looking for, so the
report is not a clearance. A `fix` row after them gives the read-only
command that opens each one, such as `agent-coexist check docs/state`
(in `list`, `check` for each `busy` or `?` leaf and `list` for each
directory it could not list); run it before you rely on "no lock" or
"nothing new". Running that check is a lookup; what you set out to do
still needs its own command. A `skipped  <dir>  unreadable` row in
`check-state` or `read-notification` can come with no `fix` at all,
when the scan could not list the directory and so never saw a file in
it. That does not mean the directory has no locks or new records: make
it readable and enterable (read and search permission), then run the
same command again before you rely on the result. If the `--include`
row says no file is left, read its hint: the glob may not be relative
to the directory, an `--exclude` may have removed the files, or ignore
rules may skip their directory.
`--include` cannot reach into such a directory; use `--no-ignore` when
you mean to, for a directory you chose. In `check-state` the filters
apply only to state files below the directory; its own and its
ancestors' state files are always read.

Opening the Markdown with a file tool is fine for reading. `check` adds
what a file tool cannot: format diagnostics and brief target checks.

Next: `guide write` to answer, `guide wait` to wait for a reply.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="info-guide-write"></a>

### guide 主题 write

编号 `info-guide-write` · 观察平台 Windows

guide write 完整输出该主题页。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide write
```

<details>
<summary>stdout：（269 行，展开查看全文）</summary>

````text
# Write so the next agent can act

Read the relevant files and state first, and add to an existing thread or
brief where one fits. Write when a decision, request, blocker, or result
changes, not after every step.

## Posts: lead with the result or the request

These examples continue the `parser-review` thread from
`agent-coexist guide`, run from the same directory. After that exchange
alice finishes the change and hands it back with a message another agent
can act on without asking back:

```sh
agent-coexist post parser-review --as alice -m 'Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.'
```

Those labels are one useful shape, not a required form. A short question
needs only enough context and a clear request, for example this `-m`
body (not posted in this thread):

```text
Does parser-check.txt include the empty-input case? @alice
```

- Name a recipient with `@nick` when you need someone to act.
- Reply to a specific comment with `@#n`, using the number you read in
  its heading (`## #4 alice ...`), not one you expect it to have.
- Point to files and evidence instead of pasting them. Record the command
  you ran, its result, and what it covered in a file, then link that
  file. A short exact excerpt helps when it explains a failure.
- Say which claims you checked and which you did not. Leave out private
  reasoning and raw session transcripts.

When a decision changes, add a new comment so readers see the history.
To fix a mistake in your own comment, such as a misspelled path, edit it
in place. Suppose alice's handoff above appears in the file as
`## #4 alice`, but she had typed the report as `reports/parser-chek.txt`.
`--edit` replaces the whole body, so she resends all of it with only the
name corrected; the Decision line stays as it was:

```sh
agent-coexist post parser-review --as alice --edit 4 -m 'Decision: keep the parser API unchanged; adjust the error mapping.
Evidence: src/parser/mod.rs and reports/parser-check.txt.
Limit: the report covers malformed input, not the integration path.
Request: @bob review the mapping before integration; reply here.'
```

Posting does not mean anyone has read it. Nicks are matched as typed, so
`@bob` reaches whoever reads as `bob`.

## Briefs: a reading route to real files

Write the description for a reader and a purpose, such as "Start here to
review the parser change." Give each entry a distinct title and a note
that says why to read it and what to look for. Put the starting point
first and optional depth after it. Point at durable files everyone can
open, not at a private session.

The target path is relative to the brief's directory, not to where you
run the command. Here the brief `onboarding` sits in the current
directory next to `src/`:

```sh
agent-coexist brief onboarding --as alice -m 'Review route for the parser error-mapping change.'
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document; check its error mapping before changing callers.'
```

A brief at `docs/onboarding` would need `--target ../src/parser/mod.rs`
for the same file.

Reading an entry without a `--regex` prints its whole target, so avoid
pointing routine entries at large logs. With a `--regex`, `read-brief`
prints only the whole lines of the pattern's first match, marked
`L<a>..L<b>`, which suits a stable line in a long file. Either way the
hash covers the whole file. An entry with no path, a missing or
unreadable target, or a regex that fails prints no content, and neither
does a target over 64 MiB; the entry row says why.

When a target changes, read the change and update the note if it no
longer holds, then refresh that entry's hash:

```sh
agent-coexist brief onboarding --as alice --entry 'parser entry point' -m 'Start at parse_document; errors now map in map_io_error.'
agent-coexist brief onboarding --as alice --entry 'parser entry point' --refresh
```

## Profiles: stable facts about you

Describe your role, scope, and how to reach you, not your progress. Use
whatever section names fit:

```sh
agent-coexist profile agents/alice --as alice --section scope -m 'Parser error mapping. Send review requests to parser-review.'
```

## State: declare, then release

Declare the narrowest directory you need, for about as long as you need
it. `observing` says changes are welcome. Use `locked` when you need
others to stay out while you edit:

```sh
agent-coexist state src/parser --as alice --status locked --for 30m
```

If someone else's `locked` already overlaps yours, the report shows
`conflict` rows with a `warn` first line and exit 0; talk to them in a
post. The conflict check after the write reads the state files of that
directory and its ancestors and the state files below it that ignore
rules do not skip; `state` has no `--no-ignore`, so to see locks inside
ignored folders run `check-state <dir> --no-ignore`. Do not remove
another nick's declaration to get past it. If the report instead shows
`skipped` or `partial` rows and a `fix`, your
declaration is written but some state file was not read, so a lock
there may have been missed. Do not declare again: do what the `fix`
says. It may first ask you to make a directory it could not read both
readable and enterable; the read-only command it then names is `check`
on a file or `check-state` on a directory. Run that command and talk to
any holder it shows. A `profile` or `brief` write can end the same way.

Release your declaration when you finish, before it expires:

```sh
agent-coexist state src/parser --remove alice --status locked
```

Keep `--status` when removing a single status; without it, all of your
statuses in that directory are removed.

## Finishing or handing off

Post the result, the evidence, what is still open, and who should act
next; then release your declarations. A `done` status is optional and
means whatever the team agrees.

If your own work may continue in another session, whether you resume it
or a fresh context picks it up, also put your nick, the coexist threads
and comment numbers you used, and when you last read each of those
threads into the handoff or session summary that session will read,
such as your final report or task notes. For example: "Coexist nick
`alice`; my handoff is #4 in `parser-review.agent-besties.md`, and bob's
review will arrive there; last read at 2026-09-27T10:05:00Z." That time
covers only that thread. If you also ran `read-notification`, add the
command as you ran it, a time at or before it started, and the paths its
`skipped` or `partial` rows named that you have not read directly since,
or that there were none, for example: "Scan started at 2026-09-27T10:02:00Z:
`agent-coexist read-notification . --as alice --since 1h`; no skipped or
partial rows." `guide wait` explains how the next scan uses that start
time and those paths. The comments alone do
not say which nick was yours. A session that finds no such note and has
no assigned nick takes a new one, says which work it continues, and reads
the threads directly, as "When you arrive" in `agent-coexist guide`
describes; it does not adopt a nick from the records.

After an interruption, look at the actual files and state before
renewing or cleaning up: an expired declaration only ran out of time,
and reading never cleans files.

## When a write reports a problem

Most report rows give a problem's message, or a `reason` and a `fix`,
with no ID field; only a few, such as a scan's `skipped` or `partial`
rows, have one. Explanatory text may still mention some other ID, so an
`E-` or `W-` name in the prose does not identify the problem. Match the
message you see against the quoted text below; the IDs are the names
the command reference (`docs/usage.md` in the agent-coexist repository)
uses.

- `warn` exits 0. Read its rows before moving on.
- `file is locked by another process` (`E-LOCKED`): the CLI could not
  get the file lock or a stable read in time; retry the same command
  shortly.
- `file is in use by another program` (`E-INUSE`, Windows only): another
  program opened the file without allowing shared access. Having it open
  in an editor does not cause this unless that program denies sharing.
  Close that program's handle and retry.
- `<dir> is locked by <holder> until <time>` (`E-FLOCKED`): a `locked`
  declaration covers this `profile` or `brief` write. If you left out
  `--as`, even a lock under your own nick can block it. Wait until the
  reported time or coordinate; do not retry in a tight loop.
- A diagnostic's `fix` line is a command, an instruction such as a hand
  edit (a heading to copy), or an instruction, a colon, and a command.
  Read it before acting, and carry out instructions yourself. The command
  part starts at the program name. In the combined form the words before
  the colon say what the command is for. Common cases, not a complete
  list:
  - Repair, then retry, as in `retry shortly: agent-coexist post
    parser-review …` or `correct line 1 of room/.ignore, then retry: …`.
    Do the instruction first (wait, fix the file), then run the command;
    it repeats your original command, including the program path you
    typed.
  - Look up, then correct, as in `pick an existing title from:
    agent-coexist check onboarding.agent-besties.md` or `find the number
    of the comment to edit, then rerun --edit with it: agent-coexist
    lookfor-comment …`. Run the command first; it only reads. Use its
    output to correct your original command, then run that. Its `ok`
    is only the lookup's result: your original command still has to run.
  - Check after a write that already happened, as in `correct line 1 of
    room/.ignore, then check for conflicts without declaring again:
    agent-coexist check-state room`. Do the repair, then run the check;
    do not repeat the write.
  - Do something else instead, as in `reply instead: agent-coexist post
    … -m '@#3 <your reply>'` or `start a new post: …`. The command
    replaces your original one and writes something else; run it in
    place of the original, not after it.
  - A command for a named agent, as in `if still needed, that agent
    renews it: …`, `the command for carol: …`, or `agree who continues,
    then the other runs the removal; …`. Run it only if you are that agent, under
    your own nick; otherwise tell that agent in a post. Typing someone
    else's nick into `--as`, or into `--remove`, where `--as` defaults to
    the removed nick, changes their records as if they had done it. On a
    lock conflict, agree in a post who yields; only the agent that yields
    runs `--remove` with its own nick.

  When the words before the colon fit none of these, read them as a
  description of the command and any condition for running it; if they
  name another agent or leave you unsure, ask in a post instead of
  running it.
- Angle-bracket words that the CLI prints as markers, such as `<file>`,
  `<nick>`, or `<text>`, stand for a value you choose. Fill them in
  before running a command, even when quoted and even when the CLI knows
  a current value: `--target <file>` on a stale entry asks which file the
  entry should point at now. A value taken from your files is real even
  if it looks like a marker: for an entry titled `<file>`, the
  `--entry='<file>'` in its fix names that entry and stays as printed.
  Compare with the title, nick, or path the report shows to tell the two
  apart.

## Editing the Markdown by hand

Pass bodies with `-m`; the CLI wraps them in a code fence, so headings or
code inside a body stay body text. A body that starts with `-`, such as
a Markdown list, must be attached with `=` (`-m='- item'`); written as a
separate word after `-m`, it is read as an option. Hand edits are fine
if you keep the type heading, comment and entry headings, and fences
intact; the exact schema is in the file format reference
(`docs/file-format.md` in the agent-coexist repository). Change one
comment or note at a time rather than rewriting the whole shared file.

When the CLI is available, prefer `post` and `brief` for comments and
brief entries, since they keep numbers, times, and hashes up to date.
After a hand edit, run `agent-coexist check <file>`; it only validates
the file and changes nothing.

Whenever you write a comment header by hand, copy the shape of the
entries already in the file or the reference. A new comment goes at the
end with the next number (highest existing plus one), your own nick, and
the current UTC time, such as `## #6 bob (2026-09-27T10:15:00Z)`, then
its body in an `md` fence. Open the fence with at least three backticks
and more than any run of backticks in the body, and close it with a line
of the same number; a body that quotes a three-backtick block needs a
four-backtick fence, so do not reuse the other comments' fence without
counting. A fence that closes too early cuts the body short and can
leave an unclosed fence, which makes CLI writes refuse the file. When
you edit your own comment, keep its created time and add or update
`, edited <current UTC time>` before the `)`. Take the time from your
environment's clock or from the user, never from another comment:
`read-notification` finds comments by these times. Do not guess a brief
entry's hash either; the reference gives its rules. If you cannot get
the current time or are unsure of the format, leave the file unchanged
and say what you would have written. Without the CLI nothing checks such
an edit, so re-read the file afterwards.

Exact flags: `agent-coexist <command> --help`. For waiting on replies:
`guide wait`.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="info-guide-wait"></a>

### guide 主题 wait

编号 `info-guide-wait` · 观察平台 Windows

guide wait 完整输出该主题页。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide wait
```

<details>
<summary>stdout：（184 行，展开查看全文）</summary>

````text
# Catch up, then wait for a reply or change

Nothing is pushed into an agent's context. Comments stay in their files,
and reading them removes nothing. You learn what changed in two ways:

- **Catch up**: read the files, or run `read-notification`, to find what
  is already there.
- **Wait**: run `wait`, `wait-post`, or `wait-state`. It watches one file,
  reports the first qualifying change after it started, and exits.

A wait sees only changes made after it started, so a wait always comes
with a read before and after its start. Waiting also needs a tool
environment that can run a process in the background, show its output
while it runs, cancel it, and report its exit status.

## Ask and wait: alice and bob

This continues the `parser-review` thread from `agent-coexist guide` and
`guide write`, run from its directory. alice's handoff in `guide write`
asked bob for a review; in the file it is `## #4 alice`. She wants his
reply.

**alice**

1. Read `parser-review.agent-besties.md` with her file tool. If bob has
   already answered, she is done: no wait needed.
2. Start the wait as a background process, so she can keep working:

   ```sh
   agent-coexist wait-post parser-review --as alice
   ```

3. Watch that process's stderr for its ready line:

   ```text
   waiting  wait-post  parser-review
   ```

   This means the wait has recorded its starting point. It is not a
   message.
4. Now read the post again. bob may have replied between step 1 and the
   ready line; that reply is already in the file and the wait will not
   report it. If it is there, cancel the running wait and continue.
5. Otherwise let the wait run. When it exits 0, its stdout names the
   comment that qualified. Read the post's current content before acting
   on it.

Do not put the wait and the second read in one foreground-only shell
script: the script would block before that read.

**bob**, in his own session, reads the thread, sees alice's request under
`## #4 alice`, and replies to it:

```sh
agent-coexist post parser-review --as bob -m '@#4 Reviewed; see the note on the I/O branch. @alice'
```

His `@#4` reply to her comment (or the `@alice` mention) is what makes
alice's `wait-post --as alice` report it.

For another round, start a new wait and read again after its ready line.
A wait does not bring back deleted comments or every intermediate edit;
neither a wait nor a notification scan is a message queue.

## Pick the wait

- `agent-coexist wait-post parser-review --as alice`: a comment in that
  post that mentions alice or replies to one of her comments is new,
  edited, or newly relevant. Without `--as`, any change to the post counts.
- `agent-coexist wait onboarding`: any byte change to that coexist file
  (here the `onboarding` brief from `guide write`), reported as a diff of
  its parsed content. A formatting-only change reports
  `semantic  unchanged`.
- `agent-coexist wait-state src/parser --status locked`: a `locked`
  declaration added, removed, or renewed in
  `src/parser/state.agent-besties.md`. Only that file is watched, not
  parent or child directories.

A `wait` or `wait-post` path names a coexist file; without the suffix,
`.agent-besties.md` is added. `agent-coexist wait notes` watches
`notes.agent-besties.md`, never a plain file named `notes`, even when
only the plain file exists; in that case it waits for
`notes.agent-besties.md` to be created.

Some changes never end a filtered wait: a change to the subscriber list
for `wait-post --as`, and a declaration simply expiring for
`wait-state --status`. Use `check-state` for current declarations, and
arrange time-based follow-up in your own environment.

## Catching up with read-notification

```sh
agent-coexist read-notification . --as alice --since 1h
```

This lists comment activity in `[since, now)` at whole-second precision.
A comment written in the current second is not included yet: to see it,
read again in a later second or open the thread directly.

The scan covers every post under the directory that ignore rules do not
skip. To continue an earlier scan's coverage without a gap, keep its
directory, `--as` nick, and filters, and pass a time at or before that
scan started. The time you
last read a thread, left a session, or last posted says nothing about
other threads: after a scan at 10:02, an
`@alice` mention in another thread at 10:03, and a read of alice's own
thread at 10:05, a `--since` of 10:05 misses the mention, while 10:02
lists it. An earlier cutoff can show the same record twice; recognize it
by the post, comment number, and content, not by timestamp, since two
edits in one second share a timestamp.

Do not move the window past files the previous scan did not cover: those
named in its `skipped` or `partial` rows, and any that a wider directory
or looser filters now reach. For them, use a time you know is early
enough, or read them directly. If a handoff records the scan's command
and start but not whether it had such rows, you do not know which files
it covered: read the relevant threads directly, or use a `--since` you
know is early enough on other grounds. A guessed earlier time can still
miss older activity in files that scan skipped.

When catching up, if you do not know when a compatible scan started, for
example in a fresh session whose handoff records none, a guessed window
such as `--since 1h` misses anything older without saying so. Read the
relevant threads directly instead: the ones your handoff names and any
others that touch your task.

The subscriber list is read as it is now, not as it was when a comment
was written. Conflicts and your own soon-expiring declarations appear
whenever they are relevant, regardless of the window. A conflicting
lock inside a folder the ignore rules skip produces no notice; scan
with `--no-ignore` to read such folders. Check `skipped` and `partial`
rows before concluding that nothing is new. The CLI keeps no
read cursor and does not mark anything as delivered.

## Managing the wait process

- There is no timeout option. A wait runs until a qualifying change, an
  error, or cancellation. Set any deadline in your own environment and
  cancel the process when it passes.
- If your tool gives up on a call, the wait may still be running. Check
  for it before starting another one, and cancel waits you no longer need.
- Ctrl-C cancels a wait with exit 130 and empty stdout. A forced stop can
  exit differently. Neither is a change report.
- A missing file is a valid starting point: the wait reports when the file
  is created, as long as its parent directory exists. A mistyped path
  therefore waits forever, so check the path.
- Waits sample the file about once a second and may not see a change that
  is undone in between. Each change that does not qualify becomes the new
  starting point.
- `E-LOCKED` or `E-INUSE` while sampling exits 3; retry or close the other
  program as for any command. A retried wait records a new starting point
  and does not report changes made while the file was busy, so after its
  ready line, read the file again as in step 4 above.
- The same restart rule covers every failed wait, not only a busy file.
  An invalid snapshot (for example a reply a hand edit left with an
  unclosed fence), an unreadable file, or an I/O error also ends the
  wait. If that I/O error is after the ready line and the intended
  path no longer resolves (its parent missing, renamed away or taken
  by a file) or, on Linux and macOS, the path itself is no longer a
  regular file, the `fix` says how to restore that path (recreate or
  move it back; move aside whatever replaced it, keeping it) and then
  prints the original wait as the command to retry; a path that was
  wrong from the start still says to correct it, with no retry. On
  Windows a watched file replaced by a directory after the ready line
  is reported as access denied and gets the generic remedy — check
  permissions, or wait until another program stops using, deleting or
  replacing the file — which cannot tell a replacement from a busy
  file; once you establish that a directory now occupies the path,
  move it aside, keep it, start the original wait again, and read the
  file after its ready line. Once you fix the cause and start the wait
  again, its new starting point already contains everything then in the file,
  including the change you were waiting for, so read the file again
  after the new ready line as in step 4 above.
- A hit report's state warnings — conflicts and expiry — are judged at
  the wait's start time. A lock that expired while you waited still counts
  as valid, so a conflict warning can appear with no expiry warning.
  Before acting on a state warning or its fix, run `check-state` on the
  directory (or `check` on the file) to judge it at the current time.
- Judge the result from this run's exit status and output. Keep stdout
  and stderr separate when diagnosing.

A file change or a finished process does not resume an agent by itself;
your environment has to bring the result back to you. Exact flags:
`agent-coexist <command> --help`.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="info-guide-ignores-env"></a>

### 环境变量无效时 guide 与 help 仍可用

编号 `info-guide-ignores-env` · 观察平台 Windows

guide 和 --help 不读取 AGENT_COEXIST_NOW / AGENT_COEXIST_LOCK_WAIT_MS；同样环境下业务命令失败作为对照。

起始状态：空的工作目录。

**命令 1**

```sh
AGENT_COEXIST_NOW=yesterday AGENT_COEXIST_LOCK_WAIT_MS=soon agent-coexist guide wait
```

<details>
<summary>stdout：（184 行，展开查看全文）</summary>

````text
# Catch up, then wait for a reply or change

Nothing is pushed into an agent's context. Comments stay in their files,
and reading them removes nothing. You learn what changed in two ways:

- **Catch up**: read the files, or run `read-notification`, to find what
  is already there.
- **Wait**: run `wait`, `wait-post`, or `wait-state`. It watches one file,
  reports the first qualifying change after it started, and exits.

A wait sees only changes made after it started, so a wait always comes
with a read before and after its start. Waiting also needs a tool
environment that can run a process in the background, show its output
while it runs, cancel it, and report its exit status.

## Ask and wait: alice and bob

This continues the `parser-review` thread from `agent-coexist guide` and
`guide write`, run from its directory. alice's handoff in `guide write`
asked bob for a review; in the file it is `## #4 alice`. She wants his
reply.

**alice**

1. Read `parser-review.agent-besties.md` with her file tool. If bob has
   already answered, she is done: no wait needed.
2. Start the wait as a background process, so she can keep working:

   ```sh
   agent-coexist wait-post parser-review --as alice
   ```

3. Watch that process's stderr for its ready line:

   ```text
   waiting  wait-post  parser-review
   ```

   This means the wait has recorded its starting point. It is not a
   message.
4. Now read the post again. bob may have replied between step 1 and the
   ready line; that reply is already in the file and the wait will not
   report it. If it is there, cancel the running wait and continue.
5. Otherwise let the wait run. When it exits 0, its stdout names the
   comment that qualified. Read the post's current content before acting
   on it.

Do not put the wait and the second read in one foreground-only shell
script: the script would block before that read.

**bob**, in his own session, reads the thread, sees alice's request under
`## #4 alice`, and replies to it:

```sh
agent-coexist post parser-review --as bob -m '@#4 Reviewed; see the note on the I/O branch. @alice'
```

His `@#4` reply to her comment (or the `@alice` mention) is what makes
alice's `wait-post --as alice` report it.

For another round, start a new wait and read again after its ready line.
A wait does not bring back deleted comments or every intermediate edit;
neither a wait nor a notification scan is a message queue.

## Pick the wait

- `agent-coexist wait-post parser-review --as alice`: a comment in that
  post that mentions alice or replies to one of her comments is new,
  edited, or newly relevant. Without `--as`, any change to the post counts.
- `agent-coexist wait onboarding`: any byte change to that coexist file
  (here the `onboarding` brief from `guide write`), reported as a diff of
  its parsed content. A formatting-only change reports
  `semantic  unchanged`.
- `agent-coexist wait-state src/parser --status locked`: a `locked`
  declaration added, removed, or renewed in
  `src/parser/state.agent-besties.md`. Only that file is watched, not
  parent or child directories.

A `wait` or `wait-post` path names a coexist file; without the suffix,
`.agent-besties.md` is added. `agent-coexist wait notes` watches
`notes.agent-besties.md`, never a plain file named `notes`, even when
only the plain file exists; in that case it waits for
`notes.agent-besties.md` to be created.

Some changes never end a filtered wait: a change to the subscriber list
for `wait-post --as`, and a declaration simply expiring for
`wait-state --status`. Use `check-state` for current declarations, and
arrange time-based follow-up in your own environment.

## Catching up with read-notification

```sh
agent-coexist read-notification . --as alice --since 1h
```

This lists comment activity in `[since, now)` at whole-second precision.
A comment written in the current second is not included yet: to see it,
read again in a later second or open the thread directly.

The scan covers every post under the directory that ignore rules do not
skip. To continue an earlier scan's coverage without a gap, keep its
directory, `--as` nick, and filters, and pass a time at or before that
scan started. The time you
last read a thread, left a session, or last posted says nothing about
other threads: after a scan at 10:02, an
`@alice` mention in another thread at 10:03, and a read of alice's own
thread at 10:05, a `--since` of 10:05 misses the mention, while 10:02
lists it. An earlier cutoff can show the same record twice; recognize it
by the post, comment number, and content, not by timestamp, since two
edits in one second share a timestamp.

Do not move the window past files the previous scan did not cover: those
named in its `skipped` or `partial` rows, and any that a wider directory
or looser filters now reach. For them, use a time you know is early
enough, or read them directly. If a handoff records the scan's command
and start but not whether it had such rows, you do not know which files
it covered: read the relevant threads directly, or use a `--since` you
know is early enough on other grounds. A guessed earlier time can still
miss older activity in files that scan skipped.

When catching up, if you do not know when a compatible scan started, for
example in a fresh session whose handoff records none, a guessed window
such as `--since 1h` misses anything older without saying so. Read the
relevant threads directly instead: the ones your handoff names and any
others that touch your task.

The subscriber list is read as it is now, not as it was when a comment
was written. Conflicts and your own soon-expiring declarations appear
whenever they are relevant, regardless of the window. A conflicting
lock inside a folder the ignore rules skip produces no notice; scan
with `--no-ignore` to read such folders. Check `skipped` and `partial`
rows before concluding that nothing is new. The CLI keeps no
read cursor and does not mark anything as delivered.

## Managing the wait process

- There is no timeout option. A wait runs until a qualifying change, an
  error, or cancellation. Set any deadline in your own environment and
  cancel the process when it passes.
- If your tool gives up on a call, the wait may still be running. Check
  for it before starting another one, and cancel waits you no longer need.
- Ctrl-C cancels a wait with exit 130 and empty stdout. A forced stop can
  exit differently. Neither is a change report.
- A missing file is a valid starting point: the wait reports when the file
  is created, as long as its parent directory exists. A mistyped path
  therefore waits forever, so check the path.
- Waits sample the file about once a second and may not see a change that
  is undone in between. Each change that does not qualify becomes the new
  starting point.
- `E-LOCKED` or `E-INUSE` while sampling exits 3; retry or close the other
  program as for any command. A retried wait records a new starting point
  and does not report changes made while the file was busy, so after its
  ready line, read the file again as in step 4 above.
- The same restart rule covers every failed wait, not only a busy file.
  An invalid snapshot (for example a reply a hand edit left with an
  unclosed fence), an unreadable file, or an I/O error also ends the
  wait. If that I/O error is after the ready line and the intended
  path no longer resolves (its parent missing, renamed away or taken
  by a file) or, on Linux and macOS, the path itself is no longer a
  regular file, the `fix` says how to restore that path (recreate or
  move it back; move aside whatever replaced it, keeping it) and then
  prints the original wait as the command to retry; a path that was
  wrong from the start still says to correct it, with no retry. On
  Windows a watched file replaced by a directory after the ready line
  is reported as access denied and gets the generic remedy — check
  permissions, or wait until another program stops using, deleting or
  replacing the file — which cannot tell a replacement from a busy
  file; once you establish that a directory now occupies the path,
  move it aside, keep it, start the original wait again, and read the
  file after its ready line. Once you fix the cause and start the wait
  again, its new starting point already contains everything then in the file,
  including the change you were waiting for, so read the file again
  after the new ready line as in step 4 above.
- A hit report's state warnings — conflicts and expiry — are judged at
  the wait's start time. A lock that expired while you waited still counts
  as valid, so a conflict warning can appear with no expiry warning.
  Before acting on a state warning or its fix, run `check-state` on the
  directory (or `check` on the file) to judge it at the current time.
- Judge the result from this run's exit status and output. Keep stdout
  and stderr separate when diagnosing.

A file change or a finished process does not resume an agent by itself;
your environment has to bring the result back to you. Exact flags:
`agent-coexist <command> --help`.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_NOW=yesterday AGENT_COEXIST_LOCK_WAIT_MS=soon agent-coexist --help
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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
AGENT_COEXIST_NOW=yesterday AGENT_COEXIST_LOCK_WAIT_MS=soon agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got 'yesterday'); unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化。

说明：同样的环境变量用于业务命令：在访问文件之前就报告环境变量错误。

<a id="info-help-top"></a>

### 顶层帮助及其全部入口

编号 `info-help-top` · 观察平台 Windows

--help、-h 与 help 子命令输出同一份顶层帮助。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-guide"></a>

### guide 的帮助及其全部入口

编号 `info-help-guide` · 观察平台 Windows

guide --help、guide -h 与 help guide。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist guide -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help guide
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-list"></a>

### list 的帮助及其全部入口

编号 `info-help-list` · 观察平台 Windows

list --help、list -h 与 help list。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help list
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-check"></a>

### check 的帮助及其全部入口

编号 `info-help-check` · 观察平台 Windows

check --help、check -h 与 help check。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help check
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-profile"></a>

### profile 的帮助及其全部入口

编号 `info-help-profile` · 观察平台 Windows

profile --help、profile -h 与 help profile。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist profile -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help profile
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-post"></a>

### post 的帮助及其全部入口

编号 `info-help-post` · 观察平台 Windows

post --help、post -h 与 help post。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help post
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-subscribe"></a>

### subscribe 的帮助及其全部入口

编号 `info-help-subscribe` · 观察平台 Windows

subscribe --help、subscribe -h 与 help subscribe。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist subscribe -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help subscribe
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-brief"></a>

### brief 的帮助及其全部入口

编号 `info-help-brief` · 观察平台 Windows

brief --help、brief -h 与 help brief。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help brief
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-state"></a>

### state 的帮助及其全部入口

编号 `info-help-state` · 观察平台 Windows

state --help、state -h 与 help state。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help state
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-read-notification"></a>

### read-notification 的帮助及其全部入口

编号 `info-help-read-notification` · 观察平台 Windows

read-notification --help、read-notification -h 与 help read-notification。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help read-notification
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-lookfor-comment"></a>

### lookfor-comment 的帮助及其全部入口

编号 `info-help-lookfor-comment` · 观察平台 Windows

lookfor-comment --help、lookfor-comment -h 与 help lookfor-comment。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist lookfor-comment -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help lookfor-comment
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-read-brief"></a>

### read-brief 的帮助及其全部入口

编号 `info-help-read-brief` · 观察平台 Windows

read-brief --help、read-brief -h 与 help read-brief。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-brief -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help read-brief
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-check-state"></a>

### check-state 的帮助及其全部入口

编号 `info-help-check-state` · 观察平台 Windows

check-state --help、check-state -h 与 help check-state。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check-state -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help check-state
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-wait"></a>

### wait 的帮助及其全部入口

编号 `info-help-wait` · 观察平台 Windows

wait --help、wait -h 与 help wait。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist wait -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help wait
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-wait-post"></a>

### wait-post 的帮助及其全部入口

编号 `info-help-wait-post` · 观察平台 Windows

wait-post --help、wait-post -h 与 help wait-post。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist wait-post -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help wait-post
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-wait-state"></a>

### wait-state 的帮助及其全部入口

编号 `info-help-wait-state` · 观察平台 Windows

wait-state --help、wait-state -h 与 help wait-state。

起始状态：空的工作目录。

**命令 1**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist wait-state -h
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help wait-state
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

<a id="info-help-help"></a>

### help 子命令自己的帮助

编号 `info-help-help` · 观察平台 Windows

help help 描述 clap 的 help 子命令；help 本身不接受 --help，也不接受 -h（命令 2、命令 3）。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist help help
```

stdout：

```text
Print this message or the help of the given subcommand(s)

Usage: agent-coexist help [COMMAND]...

Arguments:
  [COMMAND]...  Print help for the subcommand(s)
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist help --help
```

stdout：（空，0 字节）

stderr：

```text
error: unrecognized subcommand '--help'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist help -h
```

stdout：（空，0 字节）

stderr：

```text
error: unrecognized subcommand '-h'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

说明：短写法 -h 同样不被 help 接受。

<a id="info-help-overrides"></a>

### 帮助选项与无效或缺失的参数同时出现

编号 `info-help-overrides` · 观察平台 Windows

参数从左往右读。能抢在帮助之前报告的无效参数，只有命令行解析器自己就能发现的错误（例如不存在的选项、同一选项给了两次、开关带值、数字选项里的非数字、--type 与 guide 主题的取值不在许可列表内）。这类错误出现在帮助选项之前时，先报告这个错误，不输出帮助；帮助选项在前，或命令只是缺少必需参数时，输出帮助。CLI 自己的取值检查（nick、状态名、条目标题、时间、正则等）在帮助处理之后才运行：无效值写在 --help 之前也不被报告，照样输出帮助（命令 5）；去掉 --help，同一无效值在访问任何文件之前就被报告（命令 6）。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --help --bogus
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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 3。

这条命令没有观察到文件变化。

说明：--help 在 --bogus 之前：输出 list 的帮助。

**命令 2**

```sh
agent-coexist list --bogus --help
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '--bogus' found    tip: 'list' has no option --bogus; 'agent-coexist list --help' lists its options; only if '--bogus' is the DIR itself, write it last, after '--'  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

说明：顺序反过来：先读到无效的 --bogus，报告错误后就停止，后面的 --help 不起作用。

**命令 3**

```sh
agent-coexist list --help
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

说明：去掉无效的 --bogus 后得到同一份帮助。

**命令 4**

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

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 5。

这条命令没有观察到文件变化。

说明：post 平时必须带 PATH、--as 和 -m；缺少它们不影响输出帮助。

**命令 5**

```sh
agent-coexist post standup --as 'alice smith' --help
```

stdout、stderr 与退出码和命令 4 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

说明：无效的 nick 写在 --help 之前：nick 语法是 CLI 自己的取值检查，在帮助处理之后才运行，所以照样输出帮助。

**命令 6**

```sh
agent-coexist post standup --as 'alice smith' -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice smith')
```

退出码：`2`

这条命令没有观察到文件变化。

说明：去掉 --help 的对照：同一无效 nick 在访问任何文件之前被报告；该错误的单独场景见 usage-nick-invalid。

**命令 7**

```sh
agent-coexist guide start --help
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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：输出的是 guide 命令的帮助，而不是 start 主题页。

<a id="info-version"></a>

### 版本号及其入口

编号 `info-version` · 观察平台 Windows

-V 与 --version 输出版本；它们只属于程序本身，要写在命令之前。version 不是命令，写在命令之后的 -V 也不被接受；两种写错都改用 agent-coexist -V（或 --version）。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist --version
```

stdout：

```text
agent-coexist 0.1.0
```

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2、命令 3、命令 6。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist -V
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist -V list
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

说明：-V 出现在命令之前时，后面的命令不会执行。

**命令 4**

```sh
agent-coexist version
```

stdout：（空，0 字节）

stderr：

```text
error: unrecognized subcommand 'version'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

说明：version 不是命令；要查看版本，用命令 1 的 --version 或命令 2 的 -V。

**命令 5**

```sh
agent-coexist list -V
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-V' found    tip: --version (-V) is accepted only before a command: agent-coexist --version; after 'list' it is an unknown option  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

说明：-V 放在命令之后不被接受。

**命令 6**

```sh
agent-coexist --version
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

说明：按错误给出的命令，把版本选项写在命令之前，得到原本要查看的版本号。

<a id="info-type-alias"></a>

### profile 与 agent-profile 选中同一类文件

编号 `info-type-alias` · 观察平台 Windows

list --type profile 与 --type agent-profile 输出相同；重复 --type 取并集。

起始文件 `alice.agent-besties.md`：

```text
# alice.agent-profile
```

起始文件 `standup.agent-besties.md`：

```text
# standup.post
```

起始文件 `state.agent-besties.md`：

```text
# .state
```

**命令 1**

```sh
agent-coexist list --type agent-profile
```

stdout：

```text
ok  list  .
  ignore        none
  alice                       agent-profile
  total         1
```

stderr：（空，0 字节）

退出码：`0`

以下入口的输出与本条逐字节相同：命令 2。

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list --type profile
```

stdout、stderr 与退出码和命令 1 逐字节相同，不重复展示。

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist list --type profile --type post
```

stdout：

```text
ok  list  .
  ignore        none
  alice                       agent-profile
  standup                     post
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="native-guide-write-failed-linux"></a>

### Linux：guide 写入 /dev/full 失败

编号 `native-guide-write-failed-linux` · 观察平台 Linux（平台专属场景）

在 WSL 中运行 `agent-coexist guide >/dev/full`。/dev/full 对每次写入返回 ENOSPC。guide.rs 把 stdout 的写入或 flush 失败映射为退出码 1，不打印状态行、不写 stderr、不改任何文件。这是帮助里「a failed guide write exits 1」的实跑，不是故障注入。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist guide >/dev/full'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：把 guide 首页写到 /dev/full。退出 1、stdout 和 stderr 都空、工作树无变化，是 guide.rs 对写入失败的处理；生成器没有改写输出，也没有注入产品故障。

<a id="ch-usage"></a>

## 参数与入口失败

<a id="usage-no-command"></a>

### 只输入程序名，没有命令

编号 `usage-no-command` · 观察平台 Windows

不带任何参数运行 agent-coexist。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist
```

stdout：（空，0 字节）

stderr：

```text
error: 'agent-coexist' requires a subcommand but one was not provided   [subcommands: guide, list, check, profile, post, subscribe, brief, state, read-notification, lookfor-comment, read-brief, check-state, wait, wait-post, wait-state, help]  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-unknown-command-similar"></a>

### 命令拼错，有相近建议

编号 `usage-unknown-command-similar` · 观察平台 Windows

与真实命令相近的未知命令会得到建议。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist lsit
```

stdout：（空，0 字节）

stderr：

```text
error: unrecognized subcommand 'lsit'    tip: a similar subcommand exists: 'list'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-unknown-command"></a>

### 未知命令，无相近建议

编号 `usage-unknown-command` · 观察平台 Windows

没有相近名称的未知命令。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist frobnicate
```

stdout：（空，0 字节）

stderr：

```text
error: unrecognized subcommand 'frobnicate'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-help-unknown-command"></a>

### help 后跟未知命令

编号 `usage-help-unknown-command` · 观察平台 Windows

`help <名称>`，名称不是命令。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist help frobnicate
```

stdout：（空，0 字节）

stderr：

```text
error: unrecognized subcommand 'frobnicate'  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-help-extra-command"></a>

### help 真实命令后多一个词

编号 `usage-help-extra-command` · 观察平台 Windows

help list extra：第二个词被当作嵌套子命令查找。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist help list extra
```

stdout：（空，0 字节）

stderr：

```text
error: unrecognized subcommand 'extra'  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-unknown-top-flag"></a>

### 命令前的未知选项

编号 `usage-unknown-top-flag` · 观察平台 Windows

无法识别的顶层选项。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist --bogus
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '--bogus' found  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-separator-before-command"></a>

### 命令名前写了 --

编号 `usage-separator-before-command` · 观察平台 Windows

clap 说明命令存在，但 -- 把它变成了普通值。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist -- list
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument 'list' found    tip: subcommand 'list' exists; to use it, remove the '--' before it  Usage: agent-coexist <COMMAND>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-unknown-flag-similar"></a>

### 选项拼错，有相近建议

编号 `usage-unknown-flag-similar` · 观察平台 Windows

与真实长选项相近的未知选项。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --typ post
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '--typ' found    tip: a similar argument exists: '--type'  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-unknown-flag"></a>

### 命令上的未知选项

编号 `usage-unknown-flag` · 观察平台 Windows

list 没有 --bogus 这个选项，整条命令被拒绝，什么也没扫描。打错或多写了选项时，对照 list --help 去掉或改正它，再运行原来的 list。只有 --bogus 本身就是要扫描的目录名时，才把它写在最后、放到 -- 之后。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --bogus
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '--bogus' found    tip: 'list' has no option --bogus; 'agent-coexist list --help' lists its options; only if '--bogus' is the DIR itself, write it last, after '--'  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list
```

stdout：

```text
ok  list  .
  ignore        none
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉不存在的选项后，原来的 list 完成。

<a id="usage-unknown-flag-invisible"></a>

### 未知选项里夹着看不见的字符

编号 `usage-unknown-flag-invisible` · 观察平台 Windows

选项 `--bad` 末尾多了一个从右到左覆盖符 U+202E。原样打印时它会把这一行后半段倒过来显示；tip 要把它写成 `<U+202E>`，整行保持原来的顺序。post 没有这个选项，PATH 已经给出，所以 tip 只指向 post --help。去掉这个多余的选项，原来的评论写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup $'--bad\xe2\x80\xae' -m hello --as alice
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '--bad<U+202E>' found    tip: 'post' has no option --bad<U+202E>; 'agent-coexist post --help' lists its options  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup -m hello --as alice
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hello
  ```
  ````

说明：去掉不存在的选项，原来的评论写入。

<a id="usage-option-not-on-command"></a>

### 使用了其他命令的选项

编号 `usage-option-not-on-command` · 观察平台 Windows

check 只检查文件，没有 -m；-m 属于 post 等写命令。PATH 已经给出，把 -m 放到 -- 之后只会多出一个位置参数。要检查文件就去掉 -m；要发评论则改用 post。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
agent-coexist check standup -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-m' found    tip: 'check' has no option -m; 'agent-coexist check --help' lists its options  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup thread. @bob please post your status here.
  latest        #2  bob  2026-09-17T09:20:00Z  @#1 parser done, lexer next.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉 -m，完成原来的检查。

<a id="usage-single-dash-long"></a>

### 长选项只写了一个短横线

编号 `usage-single-dash-long` · 观察平台 Windows

-as 被读成短选项 -a，而 post 没有 -a。长选项要写两个短横线：--as。PATH 已经给出，把 -a 放到 -- 之后不是修法。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup -as alice -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-a' found    tip: '-as' has one '-', so it was read as the short option -a; the long option is written with two: --as  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice -m hello
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hello
  ```
  ````

说明：把 -as 改成 --as，原来的评论写入成功。

<a id="usage-extra-positional"></a>

### 多余的位置参数

编号 `usage-extra-positional` · 观察平台 Windows

check 只接受一个 PATH。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist check standup notes
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument 'notes' found  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-guide-two-topics"></a>

### guide 给了两个主题

编号 `usage-guide-two-topics` · 观察平台 Windows

guide 最多接受一个主题。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide start read
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument 'read' found  Usage: agent-coexist guide [start|read|write|wait]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-missing-path"></a>

### 缺少必需的 PATH

编号 `usage-missing-path` · 观察平台 Windows

需要 PATH 的命令没有给出 PATH。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist check
```

stdout：（空，0 字节）

stderr：

```text
error: the following required arguments were not provided:   <PATH>  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-missing-options"></a>

### 缺少必需选项

编号 `usage-missing-options` · 观察平台 Windows

post 缺 --as 和 -m，两项一起列出。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup
```

stdout：（空，0 字节）

stderr：

```text
error: the following required arguments were not provided:   --as <nick>   -m <content>  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-missing-value"></a>

### 选项缺少值

编号 `usage-missing-value` · 观察平台 Windows

--type 位于行尾。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --type
```

stdout：（空，0 字节）

stderr：

```text
error: a value is required for '--type <type>' but none was supplied  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-number-negative"></a>

### 负数评论编号

编号 `usage-number-negative` · 观察平台 Windows

--edit -1 被当作选项；tip 说明 --edit 只接受数字，写成 --edit=-1 也不会成为有效编号。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as alice -m fixed --edit -1
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-1' found    tip: '-1' after --edit starts with '-', so it was read as an option; --edit takes a comment number made of digits only (such as --edit 3), which never starts with '-'  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-number-not-digits"></a>

### 评论编号不是数字

编号 `usage-number-not-digits` · 观察平台 Windows

--edit / --reply-to / --comment 的 u64 解析失败。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist lookfor-comment standup --reply-to first
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value 'first' for '--reply-to <n>': invalid digit found in string  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-number-empty"></a>

### 评论编号为空

编号 `usage-number-empty` · 观察平台 Windows

数字选项给了空字符串。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as alice -m fixed --edit ''
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value '' for '--edit <n>': cannot parse integer from empty string  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-number-too-large"></a>

### 评论编号过大

编号 `usage-number-too-large` · 观察平台 Windows

超出 u64 范围的数字。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist lookfor-comment standup --comment 99999999999999999999
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value '99999999999999999999' for '--comment <n>': number too large to fit in target type  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-repeated-option"></a>

### 同一选项给了两次

编号 `usage-repeated-option` · 观察平台 Windows

单值选项重复出现。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as alice --as bob -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: the argument '--as <nick>' cannot be used multiple times  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-repeated-flag"></a>

### 同一开关给了两次

编号 `usage-repeated-flag` · 观察平台 Windows

布尔开关重复出现。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --no-ignore --no-ignore
```

stdout：（空，0 字节）

stderr：

```text
error: the argument '--no-ignore' cannot be used multiple times  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-flag-with-value"></a>

### 开关带了值

编号 `usage-flag-with-value` · 观察平台 Windows

--no-ignore=yes。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --no-ignore=yes
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected value 'yes' for '--no-ignore' found; no more were expected  Usage: agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-grammar-multiline-usage"></a>

### 多种用法的命令出现语法错误

编号 `usage-brief-grammar-multiline-usage` · 观察平台 Windows

brief 有六行用法，全部并入单行 stderr。brief 没有 --bogus 这个选项；PATH 已经给出，所以 tip 只指向 brief --help，不提供 -- 写法。从 Usage 中选要用的形式，去掉 --bogus。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --bogus
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '--bogus' found    tip: 'brief' has no option --bogus; 'agent-coexist brief --help' lists its options  Usage: agent-coexist brief <PATH> [--as <nick>]    or: agent-coexist brief <PATH> [--as <nick>] -m <description>    or: agent-coexist brief <PATH> [--as <nick>] --target <path> [--entry <title>] [--regex <pattern>] [-m <note>]    or: agent-coexist brief <PATH> [--as <nick>] --entry <title> -m <note>    or: agent-coexist brief <PATH> [--as <nick>] --entry <title> --remove    or: agent-coexist brief <PATH> [--as <nick>] [--entry <title>] --refresh  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief onboarding
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.
  ```

说明：去掉 --bogus，按 Usage 的第一种形式新建空导读。

<a id="usage-type-invalid"></a>

### --type 的类型词无效

编号 `usage-type-invalid` · 观察平台 Windows

--type 的自定义取值解析。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --type posts
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value 'posts' for '--type <type>': 'posts' is not a valid type; expected post, brief, agent-profile (alias: profile) or state  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-guide-topic-invalid"></a>

### guide 主题大小写不对

编号 `usage-guide-topic-invalid` · 观察平台 Windows

主题必须是完全一致的小写词，不做大小写折叠。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide Start
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value 'Start' for '[start|read|write|wait]': 'Start' is not a valid guide topic; expected start, read, write or wait  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-guide-topic-whitespace"></a>

### guide 主题含多余空白和多余的词

编号 `usage-guide-topic-whitespace` · 观察平台 Windows

' start  page ' 除了首尾和中间的空白，还多了一个词 page，去掉空白也不是有效主题。错误原样回显这个值，读者能看出是哪些字符不对；只差首尾空白的情况见 usage-guide-topic-edge-space。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide ' start  page '
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value ' start  page ' for '[start|read|write|wait]': ' start  page ' is not a valid guide topic; expected start, read, write or wait  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist guide start
```

<details>
<summary>stdout：（87 行，展开查看全文）</summary>

````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

```sh
agent-coexist subscribe parser-review --as bob
```

## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：只写主题词 start，打开原本要看的主题页。

<a id="usage-nick-invalid"></a>

### nick 含空格

编号 `usage-nick-invalid` · 观察平台 Windows

nick 语法错误；同一模板也用于 --remove、--by、--mention。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as 'alice smith' -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice smith')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-nick-at-sign"></a>

### --mention 写成 @nick

编号 `usage-nick-at-sign` · 观察平台 Windows

--mention 接受不带 @ 的 nick；正文里的 @bob 写法被带进了选项值。错误指出去掉 @ 的写法，照做后原来的查找完成，找到提及 bob 的 #1。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
agent-coexist lookfor-comment standup --mention @bob
```

stdout：（空，0 字节）

stderr：

```text
error: --mention must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '@bob'); write 'bob' without the '@'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist lookfor-comment standup --mention bob
```

stdout：

```text
ok  lookfor-comment  standup
  L3  #1  alice  2026-09-17T09:12:00Z  Daily standup thread. @bob please post your status here.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按提示去掉 @，原来的查找完成。

<a id="usage-nick-empty"></a>

### nick 为空

编号 `usage-nick-empty` · 观察平台 Windows

--as= 后面没有内容。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist subscribe standup --as=
```

stdout：（空，0 字节）

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-nick-too-long"></a>

### nick 超过 64 个字符

编号 `usage-nick-too-long` · 观察平台 Windows

长度超限使用同一语法提示。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-remove-nick-invalid"></a>

### state 移除形式的 nick 无效

编号 `usage-remove-nick-invalid` · 观察平台 Windows

nick 模板中的选项名为 --remove。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --remove 'alice smith'
```

stdout：（空，0 字节）

stderr：

```text
error: --remove must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice smith')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-status-invalid"></a>

### 状态名含大写字母

编号 `usage-status-invalid` · 观察平台 Windows

状态名语法错误。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status Locked
```

stdout：（空，0 字节）

stderr：

```text
error: --status must be a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-' (got 'Locked')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-wait-state-status-invalid"></a>

### wait-state 的状态名无效

编号 `usage-wait-state-status-invalid` · 观察平台 Windows

wait-state 在开始监视前用同一语法校验 --status。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist wait-state --status Locked
```

stdout：（空，0 字节）

stderr：

```text
error: --status must be a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-' (got 'Locked')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-section-invalid"></a>

### section 名含点号

编号 `usage-section-invalid` · 观察平台 Windows

section 名语法错误。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile alice --section role.v2 -m 'Rust implementer.'
```

stdout：（空，0 字节）

stderr：

```text
error: --section must be a section name: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'role.v2')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-plain-control"></a>

### 条目标题含制表符

编号 `usage-plain-control` · 观察平台 Windows

--entry 拒绝控制字符，不论出现在哪里。制表符在标题中间，去掉首尾空白也不会让它通过，所以错误只回显原值，不提示去掉首尾空白。末尾回车符的对照见 usage-nick-trailing-cr、usage-entry-trailing-cr：这两个值去掉回车符后本身有效，错误才同时指出去掉首尾空白即可；去掉回车符后仍不合语法的值不给这条提示，见 usage-status-trailing-cr-still-invalid。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --entry $'parser\tentry' -m note
```

stdout：（空，0 字节）

stderr：

```text
error: --entry must not contain control characters (got 'parser<U+0009>entry')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-plain-double-space"></a>

### 条目标题含连续两个空格

编号 `usage-plain-double-space` · 观察平台 Windows

标题中有连续两个空格就在匹配之前被拒绝，不论在中间还是首尾。这里两个空格在中间，错误回显原值；去掉首尾空白也不会让它通过，所以不提示去掉首尾空白。首尾各只多一个空格时不报错（usage-entry-edge-space）；末尾多两个空格见 usage-entry-trailing-double-space。改成单个空格后找到条目。

起始文件 `src/main.rs`：

```text
fn main() {}
```

起始文件 `onboarding.agent-besties.md`：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/main.rs
- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

**命令 1**

```sh
agent-coexist read-brief onboarding --entry 'parser  entry point'
```

stdout：（空，0 字节）

stderr：

```text
error: --entry must not contain two consecutive spaces (got 'parser  entry point')
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-brief onboarding --entry 'parser entry point'
```

stdout：

```text
ok  read-brief  onboarding
  entry         parser entry point  fresh  src/main.rs  L1..L1
  content
    | fn main() {}
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：改成单个空格，原来的阅读完成。

<a id="usage-message-empty"></a>

### -m 只有空白

编号 `usage-message-empty` · 观察平台 Windows

-m 在 CRLF 折叠和去除末尾空白后为空。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as alice -m $' \r\n '
```

stdout：（空，0 字节）

stderr：

```text
error: -m text is empty after normalization (CRLF folded, trailing whitespace removed)
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-keyword-empty"></a>

### --keyword 为空

编号 `usage-keyword-empty` · 观察平台 Windows

lookfor-comment 拒绝空关键词，并说明不加 --keyword 就列出全部评论。照做后原来的查找完成，列出 standup 的两条评论。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
agent-coexist lookfor-comment standup --keyword ''
```

stdout：（空，0 字节）

stderr：

```text
error: --keyword must not be empty; omit --keyword to list every comment
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist lookfor-comment standup
```

stdout：

```text
ok  lookfor-comment  standup
  L3  #1  alice  2026-09-17T09:12:00Z  Daily standup thread. @bob please post your status here.
  L9  #2  bob  2026-09-17T09:20:00Z  @#1 parser done, lexer next.
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按提示去掉 --keyword，列出每一条评论。

<a id="usage-profile-m-without-section"></a>

### profile 给了 -m 但没有 --section

编号 `usage-profile-m-without-section` · 观察平台 Windows

-m / --remove 需要 --section。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile alice -m 'Rust implementer.'
```

stdout：（空，0 字节）

stderr：

```text
error: -m and --remove require --section <name>
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-profile-section-alone"></a>

### profile 只有 --section，没有 -m 或 --remove

编号 `usage-profile-section-alone` · 观察平台 Windows

--section 需要一个动作。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile alice --section role
```

stdout：（空，0 字节）

stderr：

```text
error: --section requires exactly one of -m <content> or --remove
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-profile-section-both"></a>

### profile --section 同时给了 -m 和 --remove

编号 `usage-profile-section-both` · 观察平台 Windows

两个动作互斥。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile alice --section role -m 'Rust implementer.' --remove
```

stdout：（空，0 字节）

stderr：

```text
error: --section takes exactly one of -m <content> or --remove, not both
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-regex-without-target"></a>

### brief 给了 --regex 但没有 --target

编号 `usage-brief-regex-without-target` · 观察平台 Windows

--regex 只用于 target 形式。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --entry 'parser entry point' --regex 'fn main'
```

stdout：（空，0 字节）

stderr：

```text
error: --regex requires --target <path>
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-regex-newline"></a>

### --regex 含换行

编号 `usage-brief-regex-newline` · 观察平台 Windows

--regex 拒绝原始 CR/LF：这里的模式想跨行匹配 main 所在行和下一行的 run，却把真正的换行符传了进去。错误回显的值里换行符显示成可见的转义，让看不见的换行有了位置；修法不是去掉空白——那会改变模式匹配的内容——而是在模式里写 \n。照做后条目写入，regex 原样保存为 \n；read-brief 显示匹配的两行。

起始文件 `src/main.rs`：

```text
fn main() {
    run();
}
```

**命令 1**

```sh
agent-coexist brief onboarding --target src/main.rs --regex $'main.*\n.*run'
```

stdout：（空，0 字节）

stderr：

```text
error: --regex must not contain raw CR or LF (got 'main.*<U+000A>.*run'); write \r or \n in the pattern to match them
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief onboarding --target src/main.rs --regex 'main.*\n.*run'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         main.rs  added  src/main.rs
  hash          4e429d42b4b4fa03c0f6ca98872cde62c219f04d2dd9b7b0a4e2f3f6c27ac528
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## main.rs

  - path: src/main.rs
  - hash: 4e429d42b4b4fa03c0f6ca98872cde62c219f04d2dd9b7b0a4e2f3f6c27ac528
  - regex: main.*\n.*run
  ```

说明：按提示把换行写成模式里的 \n（反斜杠加 n 两个字符），原来的条目连同 regex 写入。

**命令 3**

```sh
agent-coexist read-brief onboarding
```

stdout：

```text
ok  read-brief  onboarding
  entry         main.rs  fresh  src/main.rs  L1..L2
  content
    | fn main() {
    |     run();
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：读这个条目：regex 跨两行匹配，只显示 L1..L2 两行。

<a id="usage-brief-target-newline"></a>

### --target 含回车

编号 `usage-brief-target-newline` · 观察平台 Windows

从 CRLF 换行的脚本读入的目标路径末尾多了回车符。--target 拒绝原始 CR/LF；错误回显的值里回车符显示成可见的转义，并指出去掉首尾空白就是修法。去掉末尾的回车符后，原来的条目写入。

起始文件 `src/main.rs`：

```text
fn main() {}
```

**命令 1**

```sh
agent-coexist brief onboarding --target $'src/main.rs\r'
```

stdout：（空，0 字节）

stderr：

```text
error: --target must not contain raw CR or LF (got 'src/main.rs<U+000D>'); remove the whitespace before or after 'src/main.rs'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief onboarding --target src/main.rs
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         main.rs  added  src/main.rs
  hash          536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## main.rs

  - path: src/main.rs
  - hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
  ```

说明：去掉末尾的回车符，原来的条目写入；没有 --entry 时标题取目标文件名 main.rs。

<a id="usage-brief-regex-syntax"></a>

### --regex 无法编译

编号 `usage-brief-regex-syntax` · 观察平台 Windows

regex 库的语法错误。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --target src/main.rs --regex 'fn main('
```

stdout：（空，0 字节）

stderr：

```text
error: --regex 'fn main(' does not compile: unclosed group
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-regex-size"></a>

### --regex 编译后超过大小上限

编号 `usage-brief-regex-size` · 观察平台 Windows

能解析但超出 regex 大小上限的模式。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --target src/main.rs --regex 'a{99999999}'
```

stdout：（空，0 字节）

stderr：

```text
error: --regex 'a{99999999}' does not compile: the compiled pattern exceeds the 10485760-byte size limit; use a shorter or less repetitive pattern
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-refresh-combined"></a>

### brief --refresh 同时给了 -m

编号 `usage-brief-refresh-combined` · 观察平台 Windows

--refresh 排斥 --target、-m 和 --remove。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --refresh -m note
```

stdout：（空，0 字节）

stderr：

```text
error: --refresh cannot be combined with --target, --regex, -m or --remove
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-remove-without-entry"></a>

### brief --remove 没有 --entry

编号 `usage-brief-remove-without-entry` · 观察平台 Windows

--remove 需要指明要删的条目。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --remove
```

stdout：（空，0 字节）

stderr：

```text
error: --remove requires --entry <title>
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-remove-with-m"></a>

### brief --remove 同时给了 -m

编号 `usage-brief-remove-with-m` · 观察平台 Windows

-m 不能与 --remove 同用。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --entry 'parser entry point' --remove -m note
```

stdout：（空，0 字节）

stderr：

```text
error: -m cannot be combined with --remove
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-remove-with-target"></a>

### brief --remove 同时给了 --target

编号 `usage-brief-remove-with-target` · 观察平台 Windows

--target 不能与 --remove 同用。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --entry 'parser entry point' --remove --target src/main.rs
```

stdout：（空，0 字节）

stderr：

```text
error: --target cannot be combined with --remove
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-brief-entry-alone"></a>

### brief 只有 --entry，没有动作

编号 `usage-brief-entry-alone` · 观察平台 Windows

单独的 --entry 需要 -m、--target、--remove 或 --refresh。这里本来要把条目 parser entry point 改指向 src/lib.rs，却漏写了 --target。补上 --target 后条目改指新目标；改目标不需要 -m。

起始文件 `src/main.rs`：

```text
fn main() {}
```

起始文件 `onboarding.agent-besties.md`：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/main.rs
- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

起始文件 `src/lib.rs`：

```text
pub fn parse() {}
```

**命令 1**

```sh
agent-coexist brief onboarding --entry 'parser entry point'
```

stdout：（空，0 字节）

stderr：

```text
error: --entry requires -m (note), --target <path>, --remove or --refresh
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief onboarding --entry 'parser entry point' --target src/lib.rs
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  updated  src/lib.rs
  hash          e15aa76c42eb11b315dd032de35f539026f74214d64c1c90d531f811644a08e5
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/lib.rs
  - hash: e15aa76c42eb11b315dd032de35f539026f74214d64c1c90d531f811644a08e5
  ```

说明：补上 --target，原来的改目标完成：条目 parser entry point 改指 src/lib.rs。

<a id="usage-brief-regex-with-remove"></a>

### brief --remove 同时给了 --regex

编号 `usage-brief-regex-with-remove` · 观察平台 Windows

删除条目时用不到 --regex。错误应直接指出 --regex 不能和 --remove 一起用，而不是让读者去补一个同样不能和 --remove 一起用的 --target。去掉 --regex 后删除完成。

起始文件 `src/main.rs`：

```text
fn main() {}
```

起始文件 `onboarding.agent-besties.md`：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/main.rs
- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

**命令 1**

```sh
agent-coexist brief onboarding --entry 'parser entry point' --remove --regex 'fn main'
```

stdout：（空，0 字节）

stderr：

```text
error: --regex cannot be combined with --remove
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief onboarding --entry 'parser entry point' --remove
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  removed
  removed-lines  L5..L8
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```

说明：去掉 --regex，原来的删除完成。

<a id="usage-brief-regex-with-refresh"></a>

### brief --refresh 同时给了 --regex

编号 `usage-brief-regex-with-refresh` · 观察平台 Windows

--refresh 只按目标当前的字节重算哈希，不接受 --regex。错误应直接指出 --regex 不能和 --refresh 一起用，而不是让读者去补一个同样不能和 --refresh 一起用的 --target。去掉 --regex 后刷新完成。

起始文件 `src/main.rs`：

```text
fn main() {}
```

起始文件 `onboarding.agent-besties.md`：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/main.rs
- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

**命令 1**

```sh
agent-coexist brief onboarding --refresh --regex 'fn main'
```

stdout：（空，0 字节）

stderr：

```text
error: --refresh cannot be combined with --target, --regex, -m or --remove
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief onboarding --refresh
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  refreshed  src/main.rs
  unchanged     onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉 --regex，原来的刷新完成。

<a id="usage-state-no-form"></a>

### state 既没有 --as 也没有 --remove

编号 `usage-state-no-form` · 观察平台 Windows

没有选定任何用法形式。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state
```

stdout：（空，0 字节）

stderr：

```text
error: state requires --as <nick> --status <name>, or the remove form --remove <nick>
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-state-no-status"></a>

### state --as 没有 --status

编号 `usage-state-no-status` · 观察平台 Windows

声明形式需要 --status。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice
```

stdout：（空，0 字节）

stderr：

```text
error: --status <name> is required in the declare form
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-state-for-in-remove"></a>

### state --remove 同时给了 --for

编号 `usage-state-for-in-remove` · 观察平台 Windows

--for 只属于声明形式。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --remove alice --for 1h
```

stdout：（空，0 字节）

stderr：

```text
error: --for is only valid in the declare form (state --as <nick> --status <name>)
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-duration-order"></a>

### --for 的单位顺序颠倒

编号 `usage-duration-order` · 观察平台 Windows

DURATION 单位须按 d > h > m > s 递减。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status locked --for 30m1h
```

stdout：（空，0 字节）

stderr：

```text
error: --for must be a duration such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '30m1h')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-duration-out-of-range"></a>

### --for 的结束时间超过 9999 年

编号 `usage-duration-out-of-range` · 观察平台 Windows

合法的时长，但结束时间无法表示。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status locked --for 3000000d
```

stdout：（空，0 字节）

stderr：

```text
error: the declaration would end after 9999-12-31T23:59:59Z, the latest representable time; use a shorter --for (without --for, 'locked' lasts 2h and other statuses 8h)
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-duration-default-out-of-range"></a>

### 默认锁定时长的结束时间超过 9999 年

编号 `usage-duration-default-out-of-range` · 观察平台 Windows

未给 --for；当前时间（AGENT_COEXIST_NOW）是 9999-12-31T23:00:00Z，locked 的默认 2h 窗口会结束在可表示的最晚时间之后。显式给一个更短的 --for，让结束时间落在范围内，原来的声明就能写入。

起始状态：空的工作目录。

**命令 1**

```sh
AGENT_COEXIST_NOW=9999-12-31T23:00:00Z agent-coexist state --as alice --status locked
```

stdout：（空，0 字节）

stderr：

```text
error: the declaration would end after 9999-12-31T23:59:59Z, the latest representable time; use a shorter --for (without --for, 'locked' lasts 2h and other statuses 8h)
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_NOW=9999-12-31T23:00:00Z agent-coexist state --as alice --status locked --for 30m
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  9999-12-31T23:00:00Z..9999-12-31T23:30:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 9999-12-31T23:00:00Z..9999-12-31T23:30:00Z
  ```

说明：同一时钟下显式写 --for 30m：结束于 9999-12-31T23:30:00Z，声明写入。

<a id="usage-duration-overflow"></a>

### --for 的数字大到无法计算

编号 `usage-duration-overflow` · 观察平台 Windows

99999999999999999999d 符合 DURATION 的写法，只是数字太大，算不出秒数。这与写法错误（usage-duration-order）是不同的问题：写法本身没有错，要改的只是大小。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status locked --for 99999999999999999999d
```

stdout：（空，0 字节）

stderr：

```text
error: --for '99999999999999999999d' is too long: the declaration would end after 9999-12-31T23:59:59Z, the latest representable time; use a shorter --for
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status locked --for 2h
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：换成一个实际的时长，原来的声明写入。

<a id="usage-since-overflow"></a>

### --since 的时长数字大到无法计算

编号 `usage-since-overflow` · 观察平台 Windows

--since 的值符合 DURATION 写法，但数字超出可计算范围；与 --since 写错（usage-since-word）不是同一个问题。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist read-notification --as alice --since 99999999999999999999d
```

stdout：（空，0 字节）

stderr：

```text
error: --since '99999999999999999999d' reaches back before year 0001; use a shorter duration or an absolute time
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 2h
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：换成一个实际的回溯时长，原来的查询完成。

<a id="usage-since-word"></a>

### --since 给了一个单词

编号 `usage-since-word` · 观察平台 Windows

既不是 RFC 3339 时间，也不是 DURATION。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist read-notification --as alice --since yesterday
```

stdout：（空，0 字节）

stderr：

```text
error: --since must be an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got 'yesterday')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-since-space-time"></a>

### --since 的日期与时间用空格分隔

编号 `usage-since-space-time` · 观察平台 Windows

用空格代替 T 的时间被拒绝，错误指出日期与时间之间要写 T。照做后原来的查询完成：bob 在 09:20 回复了 alice 的 #1，出现在通知里。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
agent-coexist read-notification --as alice --since '2026-09-17 09:00:00Z'
```

stdout：（空，0 字节）

stderr：

```text
error: --since must be an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '2026-09-17 09:00:00Z'); separate the date and time with 'T', not a space
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 2026-09-17T09:00:00Z
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T09:00:00Z
  reply       2026-09-17T09:20:00Z  ./standup #2  bob  @#1 parser done, lexer next.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按提示用 T 分隔日期与时间，原来的查询完成。

<a id="usage-since-zero"></a>

### --since 0s

编号 `usage-since-zero` · 观察平台 Windows

0s 形式上是时长，但总长为零；消息写出 DURATION 规则（总长须大于零）。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist read-notification --as alice --since 0s
```

stdout：（空，0 字节）

stderr：

```text
error: --since must be an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '0s')
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-since-out-of-range"></a>

### --since 回溯到公元 1 年之前

编号 `usage-since-out-of-range` · 观察平台 Windows

合法的时长，但把当前时间推到可表示范围之前。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist read-notification --as alice --since 800000d
```

stdout：（空，0 字节）

stderr：

```text
error: --since '800000d' reaches back before year 0001; use a shorter duration or an absolute time
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-glob-empty"></a>

### --include 模式为空

编号 `usage-glob-empty` · 观察平台 Windows

空 glob。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --include ''
```

stdout：（空，0 字节）

stderr：

```text
error: --include '' is not a valid glob: the pattern is empty
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-glob-slash-only"></a>

### --exclude 只有 '/'

编号 `usage-glob-slash-only` · 观察平台 Windows

去掉锚定和目录标记后模式为空。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist check-state --exclude /
```

stdout：（空，0 字节）

stderr：

```text
error: --exclude '/' is not a valid glob: nothing is left after removing the leading and trailing '/'
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-glob-unclosed-class"></a>

### glob 的 [ 未闭合

编号 `usage-glob-unclosed-class` · 观察平台 Windows

globset：字符类未闭合。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --include 'docs/[ab'
```

stdout：（空，0 字节）

stderr：

```text
error: --include 'docs/[ab' is not a valid glob: unclosed character class; missing ']'
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-glob-range"></a>

### glob 的字符范围颠倒

编号 `usage-glob-range` · 观察平台 Windows

globset：范围无效。命令 1 的端点是普通字符 z 与 a，原因原样可见。命令 2 的起点是从右到左覆盖符 U+202E：pattern 和原因里的这个字符都要写成 `<U+202E>`，不能原样进终端把这一行后半段倒过来。把范围改成合法的 [a-z] 后，原来的查询完成。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
agent-coexist read-notification --as alice --since 1h --include '[z-a]*'
```

stdout：（空，0 字节）

stderr：

```text
error: --include '[z-a]*' is not a valid glob: invalid range; 'z' > 'a'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 1h --include $'[\xe2\x80\xae-a]*'
```

stdout：（空，0 字节）

stderr：

```text
error: --include '[<U+202E>-a]*' is not a valid glob: invalid range; '<U+202E>' > 'a'
```

退出码：`2`

这条命令没有观察到文件变化。

说明：对照：范围起点是 U+202E。pattern 与原因都把这个字符写成 `<U+202E>`，整行保持原来的顺序。

**命令 3**

```sh
agent-coexist read-notification --as alice --since 1h --include '[a-z]*'
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T09:00:00Z
  reply       2026-09-17T09:20:00Z  ./standup #2  bob  @#1 parser done, lexer next.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：把颠倒的范围改成 [a-z]，原来的查询完成：bob 在 09:20 回复了 alice 的 #1，出现在通知里。

<a id="usage-glob-unopened-alternates"></a>

### glob 有多余的 }

编号 `usage-glob-unopened-alternates` · 观察平台 Windows

globset：候选组没有开头。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --exclude 'a,b}'
```

stdout：（空，0 字节）

stderr：

```text
error: --exclude 'a,b}' is not a valid glob: unopened alternate group; missing '{' (maybe escape '}' with '[}]'?)
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-glob-unclosed-alternates"></a>

### glob 的 { 未闭合

编号 `usage-glob-unclosed-alternates` · 观察平台 Windows

globset：候选组未闭合。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list --include '{docs,src'
```

stdout：（空，0 字节）

stderr：

```text
error: --include '{docs,src' is not a valid glob: unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-guide-topic-edge-space"></a>

### guide 主题开头多了一个空格

编号 `usage-guide-topic-edge-space` · 观察平台 Windows

' start' 去掉开头的空格就是有效主题 start。错误要原样回显这个值并指出多出的空格，而不是把它显示成 'start' 再说 start 无效。去掉空格后打开主题页。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist guide ' start'
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value ' start' for '[start|read|write|wait]': ' start' is not a valid guide topic; expected start, read, write or wait; remove the space before or after 'start'  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist guide start
```

<details>
<summary>stdout：（87 行，展开查看全文）</summary>

````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

```sh
agent-coexist subscribe parser-review --as bob
```

## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉开头空格，打开原本要看的 start 主题页。

<a id="usage-type-edge-space"></a>

### --type 的类型词开头多了一个空格

编号 `usage-type-edge-space` · 观察平台 Windows

' post' 去掉开头的空格就是有效类型 post。错误要原样回显这个值并指出多出的空格，而不是把它显示成 'post' 再说 post 无效。去掉空格后 list 只列出帖子。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

起始文件 `alice.agent-besties.md`：

```text
# alice.agent-profile
```

**命令 1**

```sh
agent-coexist list --type ' post'
```

stdout：（空，0 字节）

stderr：

```text
error: invalid value ' post' for '--type <type>': ' post' is not a valid type; expected post, brief, agent-profile (alias: profile) or state; remove the space before or after 'post'  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list --type post
```

stdout：

```text
ok  list  .
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉开头空格，原来的 list 完成，只列出帖子。

<a id="usage-nick-edge-space"></a>

### nick 末尾多了一个空格

编号 `usage-nick-edge-space` · 观察平台 Windows

'alice ' 末尾的空格让它不是有效的 nick；错误回显的值要能看出这个空格。去掉空格后评论写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as 'alice ' -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'alice '); remove the space before or after 'alice'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice -m hello
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hello
  ```
  ````

说明：去掉末尾空格，原来的评论写入。

<a id="usage-status-edge-space"></a>

### 状态名末尾多了一个空格

编号 `usage-status-edge-space` · 观察平台 Windows

'locked ' 末尾的空格让它不是有效的状态名；错误回显的值要能看出这个空格。去掉空格后声明写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status 'locked '
```

stdout：（空，0 字节）

stderr：

```text
error: --status must be a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-' (got 'locked '); remove the space before or after 'locked'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status locked
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：去掉末尾空格，原来的声明写入。

<a id="usage-section-edge-space"></a>

### section 名开头多了一个空格

编号 `usage-section-edge-space` · 观察平台 Windows

' role' 开头的空格让它不是有效的 section 名；错误回显的值要能看出这个空格。去掉空格后这一节写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile alice --section ' role' -m 'Rust implementer.'
```

stdout：（空，0 字节）

stderr：

```text
error: --section must be a section name: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got ' role'); remove the space before or after 'role'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist profile alice --section role -m 'Rust implementer.'
```

stdout：

```text
ok  profile  alice
  created       alice.agent-besties.md
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer.
  ```
  ````

说明：去掉开头空格，原来的写入完成。

<a id="usage-nick-trailing-cr"></a>

### nick 末尾带着回车符

编号 `usage-nick-trailing-cr` · 观察平台 Windows

从 CRLF 换行的脚本读入的 'alice\r' 含控制字符，在 nick 语法检查之前就被拒绝。错误要把回车符显示成可见的转义；去掉它就是有效的 nick alice，所以错误指出要去掉的是首尾空白。去掉回车符后评论写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as $'alice\r' -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: --as must not contain control characters (got 'alice<U+000D>'); remove the whitespace before or after 'alice'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice -m hello
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hello
  ```
  ````

说明：去掉末尾的回车符，原来的评论写入。

<a id="usage-nick-inner-double-space"></a>

### nick 中间有连续两个空格

编号 `usage-nick-inner-double-space` · 观察平台 Windows

对照：'ali  ce' 的两个空格在中间，去掉首尾空白也不能变成有效的 nick，所以错误直接报告 nick 语法并回显原值——规则本身就说明 nick 里没有空格；像 usage-nick-trailing-cr 那样去掉首尾空白就能修复的值，才报告控制字符并附上去掉首尾空白的提示。改成要用的 nick 后评论写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as 'ali  ce' -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'ali  ce')
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice -m hello
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hello
  ```
  ````

说明：写成本来的 nick alice，原来的评论写入。

<a id="usage-status-trailing-cr"></a>

### 状态名末尾带着回车符

编号 `usage-status-trailing-cr` · 观察平台 Windows

'locked\r' 含控制字符，在状态名语法检查之前就被拒绝。错误显示回车符的转义，并指出去掉首尾空白后就是有效的状态名 locked。去掉回车符后声明写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status $'locked\r'
```

stdout：（空，0 字节）

stderr：

```text
error: --status must not contain control characters (got 'locked<U+000D>'); remove the whitespace before or after 'locked'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status locked
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：去掉末尾的回车符，原来的声明写入。

<a id="usage-status-trailing-cr-still-invalid"></a>

### 状态名带回车符，去掉后仍不是有效状态名

编号 `usage-status-trailing-cr-still-invalid` · 观察平台 Windows

对照：'Locked\r' 去掉首尾空白后仍不是有效的状态名，所以错误不报告控制字符，而是直接报告状态名语法并回显原值，回车符显示成可见的转义。是否报告控制字符按去掉首尾空白后是否有效判断：'locked\r' 去掉回车符就是有效状态名（usage-status-trailing-cr）；同样的 Locked 作为 nick 是合法的。先去掉回车符，再按同一条语法规则改成小写。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status $'Locked\r'
```

stdout：（空，0 字节）

stderr：

```text
error: --status must be a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-' (got 'Locked<U+000D>')
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status Locked
```

stdout：（空，0 字节）

stderr：

```text
error: --status must be a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-' (got 'Locked')
```

退出码：`2`

这条命令没有观察到文件变化。

说明：只去掉回车符：还是同一条状态名语法，回显里不再有转义的回车符。

**命令 3**

```sh
agent-coexist state --as alice --status locked
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：按语法改成小写，原来的声明写入。

<a id="usage-section-trailing-double-space"></a>

### section 名末尾多了两个空格

编号 `usage-section-trailing-double-space` · 观察平台 Windows

'role  ' 末尾的两个空格是连续两个空格，在 section 名语法检查之前就被拒绝。错误回显这个值，让读者看出这两个空格，并指出去掉首尾空格后就是有效的 section 名 role。去掉空格后这一节写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile alice --section 'role  ' -m 'Rust implementer.'
```

stdout：（空，0 字节）

stderr：

```text
error: --section must not contain two consecutive spaces (got 'role  '); remove the space before or after 'role'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist profile alice --section role -m 'Rust implementer.'
```

stdout：

```text
ok  profile  alice
  created       alice.agent-besties.md
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer.
  ```
  ````

说明：去掉末尾空格，原来的写入完成。

<a id="usage-duration-edge-space"></a>

### --for 的时长末尾多了一个空格

编号 `usage-duration-edge-space` · 观察平台 Windows

'1h ' 去掉末尾的空格就是有效时长 1h。错误要原样回显这个值，让读者看出多出的空格，而不是显示成 '1h' 再说它不是时长。去掉空格后声明写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status locked --for '1h '
```

stdout：（空，0 字节）

stderr：

```text
error: --for must be a duration such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got '1h '); remove the space before or after '1h'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status locked --for 1h
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
  ```

说明：去掉末尾空格，原来的声明写入。

<a id="usage-since-edge-space"></a>

### --since 的时间开头多了一个空格

编号 `usage-since-edge-space` · 观察平台 Windows

' 2026-09-17T09:00:00Z' 去掉开头的空格就是有效的 RFC 3339 时间。错误要原样回显这个值，让读者看出多出的空格，而不是显示成和示例同一格式的时间再说它无效。去掉空格后查询完成。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist read-notification --as alice --since ' 2026-09-17T09:00:00Z'
```

stdout：（空，0 字节）

stderr：

```text
error: --since must be an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero (got ' 2026-09-17T09:00:00Z'); remove the space before or after '2026-09-17T09:00:00Z'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 2026-09-17T09:00:00Z
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T09:00:00Z
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉开头空格，原来的查询完成。

<a id="usage-env-now-trailing-cr"></a>

### AGENT_COEXIST_NOW 末尾带着回车符

编号 `usage-env-now-trailing-cr` · 观察平台 Windows

从 CRLF 换行的脚本或 env 文件读入时，值末尾会留下一个看不见的回车符（\r）。去掉它就是和错误里的示例完全相同的有效时间。错误要把这个控制字符显示成可见的转义，不能显示成有效时间再说它无效；读者要的是固定时钟，所以修法是去掉回车符重新设置，而不是取消变量改用系统时钟。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=$'2026-09-17T09:12:00Z\r' agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got '2026-09-17T09:12:00Z<U+000D>'); remove the whitespace before or after '2026-09-17T09:12:00Z', or unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:12:00Z agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup thread. @bob please post your status here.
  latest        #2  bob  2026-09-17T09:20:00Z  @#1 parser done, lexer next.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉末尾的回车符，仍用固定时钟 2026-09-17T09:12:00Z，原来的 check 完成。

<a id="usage-env-lock-wait-edge-space"></a>

### AGENT_COEXIST_LOCK_WAIT_MS 末尾多了一个空格

编号 `usage-env-lock-wait-edge-space` · 观察平台 Windows

'5000 ' 去掉末尾的空格就是有效的毫秒数 5000。错误要原样回显这个值，让读者看出多出的空格，而不是显示成和建议值相同的 5000 再说它无效。这个变量只设定 post/subscribe 写入排队等锁的时间，check 不用它；去掉空格重新设置后，check 不再被这个变量的校验挡住，原来的 check 完成。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
AGENT_COEXIST_LOCK_WAIT_MS='5000 ' agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_LOCK_WAIT_MS must be a whole number of milliseconds from 0 to 18446744073709551615 such as 5000, digits only (got '5000 '); remove the space before or after '5000', or unset it to use the default 5000
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_LOCK_WAIT_MS=5000 agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup thread. @bob please post your status here.
  latest        #2  bob  2026-09-17T09:20:00Z  @#1 parser done, lexer next.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉末尾空格重新设置，校验通过，原来的 check 完成（check 的等待时间不受这个变量影响）。

<a id="usage-entry-edge-space"></a>

### 条目标题首尾各多了一个空格

编号 `usage-entry-edge-space` · 观察平台 Windows

对照：--entry 不是名称语法，匹配前会折叠空白、去掉首尾空格，所以首尾各多一个空格的 ' parser entry point ' 照样找到条目，不报错。控制字符和连续两个空格不在此列：不论在中间还是首尾，都在匹配之前被拒绝（usage-plain-double-space、usage-entry-trailing-cr、usage-entry-trailing-double-space）。

起始文件 `src/main.rs`：

```text
fn main() {}
```

起始文件 `onboarding.agent-besties.md`：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/main.rs
- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

**命令 1**

```sh
agent-coexist read-brief onboarding --entry ' parser entry point '
```

stdout：

```text
ok  read-brief  onboarding
  entry         parser entry point  fresh  src/main.rs  L1..L1
  content
    | fn main() {}
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="usage-entry-trailing-cr"></a>

### 条目标题末尾带着回车符

编号 `usage-entry-trailing-cr` · 观察平台 Windows

从 CRLF 换行的脚本读入的 'parser entry point\r' 含控制字符，在匹配之前就被拒绝，即使它在末尾。错误要原样回显标题、把回车符显示成可见的转义；去掉它就是已保存的标题 parser entry point，所以错误指出要去掉的是首尾空白。去掉回车符后读到这个条目。

起始文件 `src/main.rs`：

```text
fn main() {}
```

起始文件 `onboarding.agent-besties.md`：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/main.rs
- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

**命令 1**

```sh
agent-coexist read-brief onboarding --entry $'parser entry point\r'
```

stdout：（空，0 字节）

stderr：

```text
error: --entry must not contain control characters (got 'parser entry point<U+000D>'); remove the whitespace before or after 'parser entry point'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-brief onboarding --entry 'parser entry point'
```

stdout：

```text
ok  read-brief  onboarding
  entry         parser entry point  fresh  src/main.rs  L1..L1
  content
    | fn main() {}
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉末尾的回车符，读到已保存的 parser entry point 条目，原来的阅读完成。

<a id="usage-entry-trailing-double-space"></a>

### 条目标题末尾多了两个空格

编号 `usage-entry-trailing-double-space` · 观察平台 Windows

'parser entry point  ' 末尾的两个空格是连续两个空格，在匹配之前就被拒绝；首尾各多一个空格才会在匹配时去掉（usage-entry-edge-space）。错误回显这个标题，让读者看出末尾的两个空格，并指出去掉首尾空格后就是已保存的标题。去掉空格后读到这个条目。

起始文件 `src/main.rs`：

```text
fn main() {}
```

起始文件 `onboarding.agent-besties.md`：

```text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/main.rs
- hash: 536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4
```

**命令 1**

```sh
agent-coexist read-brief onboarding --entry 'parser entry point  '
```

stdout：（空，0 字节）

stderr：

```text
error: --entry must not contain two consecutive spaces (got 'parser entry point  '); remove the space before or after 'parser entry point'
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-brief onboarding --entry 'parser entry point'
```

stdout：

```text
ok  read-brief  onboarding
  entry         parser entry point  fresh  src/main.rs  L1..L1
  content
    | fn main() {}
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉末尾空格，读到已保存的 parser entry point 条目，原来的阅读完成。

<a id="usage-nick-bom"></a>

### nick 开头带着 BOM

编号 `usage-nick-bom` · 观察平台 Windows

从带 BOM 的文件读入的 nick 开头多了 U+FEFF。它不是空白，也看不见，nick 语法照常拒绝；错误要把它显示成 `<U+FEFF>`，而不是显示成看起来有效的 alice 再说它无效。去掉 BOM，用本来的 nick alice 重跑，评论写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as $'\xef\xbb\xbfalice' -m hello
```

stdout：（空，0 字节）

stderr：

```text
error: --as must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '<U+FEFF>alice')
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice -m hello
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hello
  ```
  ````

说明：去掉开头的 BOM，用本来的 nick alice 重跑，原来的评论写入。

<a id="usage-mention-zero-width"></a>

### --mention 的 nick 中间夹着零宽空格

编号 `usage-mention-zero-width` · 观察平台 Windows

从网页或聊天记录复制来的 nick 在 b 和 ob 之间夹着一个零宽空格 U+200B，看起来就是 bob。它不是 nick 允许的字符；错误要把它显示成 `b<U+200B>ob`，读者才知道要删掉的是什么、在哪里。删掉它，用本来的 nick bob 重跑，原来的查找完成，找到提及 bob 的 #1。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
agent-coexist lookfor-comment standup --mention $'b\xe2\x80\x8bob'
```

stdout：（空，0 字节）

stderr：

```text
error: --mention must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got 'b<U+200B>ob')
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist lookfor-comment standup --mention bob
```

stdout：

```text
ok  lookfor-comment  standup
  L3  #1  alice  2026-09-17T09:12:00Z  Daily standup thread. @bob please post your status here.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：删掉零宽空格，用本来的 nick bob 重跑，原来的查找完成。

<a id="usage-status-bidi"></a>

### 状态名开头带着从右到左覆盖符

编号 `usage-status-bidi` · 观察平台 Windows

状态名开头多了 U+202E（RIGHT-TO-LEFT OVERRIDE）。原样打印时它会让终端把这一行的后半段倒过来显示；错误要把它显示成 `<U+202E>`，整行保持原来的顺序。状态名语法照常拒绝它。去掉这个字符，用本来的状态名 locked 重跑，声明写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status $'\xe2\x80\xaelocked'
```

stdout：（空，0 字节）

stderr：

```text
error: --status must be a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-' (got '<U+202E>locked')
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status locked
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：去掉开头的 U+202E，用本来的状态名 locked 重跑，原来的声明写入。

<a id="usage-env-now-shape"></a>

### AGENT_COEXIST_NOW 用空格代替 T

编号 `usage-env-now-shape` · 观察平台 Windows

时钟覆盖值要求严格的 UTC 时间形状 YYYY-MM-DDTHH:MM:SSZ；这里用空格代替了日期与时间之间的 T。错误指出把空格改成 T，而不是取消变量改用系统时钟。照做后固定时钟保留，原来的 check 完成。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW='2026-09-17 10:00:00Z' agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got '2026-09-17 10:00:00Z'); separate the date and time with 'T', not a space, or unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:00:00Z agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup thread. @bob please post your status here.
  latest        #2  bob  2026-09-17T09:20:00Z  @#1 parser done, lexer next.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：把日期与时间之间的空格换成 T：固定时钟保留，原来的 check 完成。

<a id="usage-env-now-empty"></a>

### AGENT_COEXIST_NOW 已设置但为空

编号 `usage-env-now-empty` · 观察平台 Windows

空值是无效值，不当作未设置。

起始状态：空的工作目录。

**命令 1**

```sh
AGENT_COEXIST_NOW='' agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got ''); unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-env-lock-wait"></a>

### AGENT_COEXIST_LOCK_WAIT_MS 为负数

编号 `usage-env-lock-wait` · 观察平台 Windows

锁等待覆盖值必须是十进制数字。这个变量只设定 post/subscribe 写入排队等锁的时间，check 不用它，但无效值在命令开始前就被拒绝，check 同样失败。

起始状态：空的工作目录。

**命令 1**

```sh
AGENT_COEXIST_LOCK_WAIT_MS=-5 agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_LOCK_WAIT_MS must be a whole number of milliseconds from 0 to 18446744073709551615 such as 5000, digits only (got '-5'); unset it to use the default 5000
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-env-lock-wait-overflow"></a>

### AGENT_COEXIST_LOCK_WAIT_MS 全是数字但太大

编号 `usage-env-lock-wait-overflow` · 观察平台 Windows

99999999999999999999 只有数字，符合「只写数字」的要求，问题只在于超过了可以接受的最大值。错误要说出上限，而不是重复「只写数字」。这个变量只设定 post/subscribe 写入排队等锁的时间，check 本身不用它；但无效值在任何业务命令开始前就被拒绝，所以 check 也被挡住。取消这个变量后，原来的 check 完成。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
AGENT_COEXIST_LOCK_WAIT_MS=99999999999999999999 agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_LOCK_WAIT_MS '99999999999999999999' is larger than 18446744073709551615, the largest accepted number of milliseconds; use a smaller value such as 5000, or unset it to use the default 5000
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup thread. @bob please post your status here.
  latest        #2  bob  2026-09-17T09:20:00Z  @#1 parser done, lexer next.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：不再设置 AGENT_COEXIST_LOCK_WAIT_MS，校验不再挡住 check，原来的 check 完成。

<a id="usage-order-env-now-first"></a>

### 两个环境变量都无效

编号 `usage-order-env-now-first` · 观察平台 Windows

先检查 AGENT_COEXIST_NOW，只打印一行。

起始状态：空的工作目录。

**命令 1**

```sh
AGENT_COEXIST_NOW=now AGENT_COEXIST_LOCK_WAIT_MS=soon agent-coexist check standup
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got 'now'); unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-order-grammar-before-env"></a>

### 语法错误与无效环境变量同时存在

编号 `usage-order-grammar-before-env` · 观察平台 Windows

clap 语法错误在读取环境变量之前报告。

起始状态：空的工作目录。

**命令 1**

```sh
AGENT_COEXIST_NOW=now agent-coexist check standup notes
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument 'notes' found  Usage: agent-coexist check <PATH>  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-order-env-before-value"></a>

### 无效环境变量与无效 --for 同时存在

编号 `usage-order-env-before-value` · 观察平台 Windows

环境变量先于业务参数值检查。

起始状态：空的工作目录。

**命令 1**

```sh
AGENT_COEXIST_NOW=now agent-coexist state --as alice --status locked --for soon
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got 'now'); unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-order-value-before-files"></a>

### 目录不存在且 glob 无效

编号 `usage-order-value-before-files` · 观察平台 Windows

参数值在访问任何文件之前校验，所以先报告 glob 错误而不是目录不存在。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist list missing-dir --include '[ab'
```

stdout：（空，0 字节）

stderr：

```text
error: --include '[ab' is not a valid glob: unclosed character class; missing ']'
```

退出码：`2`

这条命令没有观察到文件变化。

<a id="usage-value-looks-like-flag"></a>

### 以短横线开头的值

编号 `usage-value-looks-like-flag` · 观察平台 Windows

-m 后面的正文以 - 开头（Markdown 列表），被当作未知选项；tip 给出用 = 连接的写法，照做后原来的评论写入成功。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as alice -m '- parser done'
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '- ' found    tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, attach it with '=': -m='- parser done'  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice '-m=- parser done'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  - parser done
  ```
  ````

说明：按 tip 用 = 连接 -m 与正文。

<a id="usage-value-looks-like-flag-multiline"></a>

### 以短横线开头的多行正文

编号 `usage-value-looks-like-flag-multiline` · 观察平台 Windows

-m 的正文是两行 Markdown 列表，第一个字符是 -。这样的值没法写成一个可以直接复制的 shell 单词，tip 改为说明怎样把正文用 = 接在 -m 后面，而不是给出一条可复制的命令。照做后原来的评论写入成功，两行原样保存。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as alice -m $'- parser done\n- lexer next'
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '- ' found    tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, start that same argument with '-m=' followed directly by the whole text, line breaks included; it cannot be shown here as a one-line command  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice $'-m=- parser done\n- lexer next'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  - parser done
  - lexer next
  ```
  ````

说明：按 tip 把整段正文用 = 接在 -m 后面，作为同一个参数传入。

<a id="usage-value-looks-like-flag-invisible"></a>

### 以短横线开头、含看不见字符的正文

编号 `usage-value-looks-like-flag-invisible` · 观察平台 Windows

-m 的正文以 - 开头，还含一个终端看不见或会改变显示的字符：命令 1 是从右到左覆盖符 U+202E，命令 3 是 ANSI 颜色序列开头的 ESC（U+001B）。原样打印会让终端倒排这一行或吞掉、改色后面的文字，tip 因此不给可复制的命令，而是把正文写成 `<U+XXXX>` 形式，说明要用 = 把整段正文接在 -m 后面，并把每个 `<U+XXXX>` 键入成它所指的那一个字符。照做后两条评论都写入，正文的字节与原来要发的完全相同。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post standup --as alice -m $'-\xe2\x80\xaeitem'
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-<U+202E>' found    tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, start that same argument with '-m=' followed directly by the whole text '-<U+202E>item', typing each <U+XXXX> as the one character it names; it cannot be shown here as a copyable command  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice $'-m=-\xe2\x80\xaeitem'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`（转义视图）：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  -\u{202e}item
  ```
  ````

说明：按 tip 手工键入：-m= 后面紧接整段正文，`<U+202E>` 处键入 U+202E 这个字符本身。

**命令 3**

```sh
agent-coexist post standup --as alice -m $'-\x1b[31mitem'
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-<U+001B>' found    tip: the word after -m starts with '-', so it was read as an option; to pass it as the value of -m, start that same argument with '-m=' followed directly by the whole text '-<U+001B>[31mitem', typing each <U+XXXX> as the one character it names; it cannot be shown here as a copyable command  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist post standup --as alice $'-m=-\x1b[31mitem'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`（转义视图）：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  -\u{202e}item
  ```

  ## #2 alice (2026-09-17T10:00:00Z)

  ```md
  -\x1b[31mitem
  ```
  ````

说明：同样手工键入：`<U+001B>` 处键入 ESC 字符本身，后面的 [31m 照抄。

查看文件 `standup.agent-besties.md`（转义视图）：

````text
# standup.post

## #1 alice (2026-09-17T10:00:00Z)

```md
-\u{202e}item
```

## #2 alice (2026-09-17T10:00:00Z)

```md
-\x1b[31mitem
```
````

说明：在上面的文件内容里核对：#1 的正文是 -、U+202E、item，#2 的正文是 -、ESC、[31m、item，与命令 1、命令 3 原来要发的正文相同。

<a id="usage-dash-value-nick"></a>

### nick 选项的值以短横线开头

编号 `usage-dash-value-nick` · 观察平台 Windows

nick 的第一个字符只能是字母、数字或 _，所以 -bob 无论怎样写都不是有效的 nick；用 = 连接也没有用。去掉短横线，原来的查找完成。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@#1 parser done, lexer next.
```
````

**命令 1**

```sh
agent-coexist lookfor-comment standup --by -bob
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-b' found    tip: '-bob' after --by starts with '-', so it was read as an option; --by takes a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-', which never starts with '-'  Usage: agent-coexist lookfor-comment <PATH> [--keyword <text>] [--by <nick>] [--mention <nick>] [--reply-to <n>] [--comment <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist lookfor-comment standup --by bob
```

stdout：

```text
ok  lookfor-comment  standup
  L9  #2  bob  2026-09-17T09:20:00Z  @#1 parser done, lexer next.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉短横线，原来的查找完成。

<a id="usage-dash-value-status"></a>

### 状态名以短横线开头

编号 `usage-dash-value-status` · 观察平台 Windows

状态名以小写字母开头，-locked 永远无效，用 = 连接也不行。去掉短横线后声明写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status -locked
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-l' found    tip: '-locked' after --status starts with '-', so it was read as an option; --status takes a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-', which never starts with '-'  Usage: agent-coexist state [DIR] --as <nick> --status <name> [--for <duration>]    or: agent-coexist state [DIR] --remove <nick> [--as <nick>] [--status <name>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status locked
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：去掉短横线，原来的声明写入。

<a id="usage-dash-value-since"></a>

### --since 的值以短横线开头

编号 `usage-dash-value-since` · 观察平台 Windows

--since 的时间和时长都以数字开头；时长本来就表示「从现在往回」，不用写负号。去掉短横线，原来的查询完成。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist read-notification --as alice --since -2h
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-2' found    tip: '-2h' after --since starts with '-', so it was read as an option; --since takes an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m, which never starts with '-'  Usage: agent-coexist read-notification [DIR] --as <nick> --since <time|duration> [--include <glob>]... [--exclude <glob>]... [--no-ignore]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 2h
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉短横线，原来的查询完成。

<a id="usage-dash-value-duration"></a>

### --for 的值以短横线开头

编号 `usage-dash-value-duration` · 观察平台 Windows

--for 的时长以数字开头，不能为负。去掉短横线后声明写入。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state --as alice --status locked --for -1h
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '-1' found    tip: '-1h' after --for starts with '-', so it was read as an option; --for takes a duration such as 1h30m: Nd, Nh, Nm, Ns in that order, each at most once, total above zero, which never starts with '-'  Usage: agent-coexist state [DIR] --as <nick> --status <name> [--for <duration>]    or: agent-coexist state [DIR] --remove <nick> [--as <nick>] [--status <name>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state --as alice --status locked --for 1h
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
  ```

说明：去掉短横线，原来的声明写入。

<a id="usage-path-is-directory"></a>

### 需要文件 PATH 时给了目录

编号 `usage-path-is-directory` · 观察平台 Windows

lookfor-comment 等命令的 PATH 是一个协作文件；给出 . 或已存在的目录时，报告说明它是目录，并给出列出其中协作文件的 list 命令。报告本身不列文件；照 fix 运行 list 才看到文件，再用其中一个文件重跑原来的查找。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:12:00Z)

```md
Daily standup thread. @bob please post your status here.
```
````

**命令 1**

```sh
agent-coexist lookfor-comment .
```

stdout：

```text
error  lookfor-comment  .
  reason        ..agent-besties.md does not exist; . is a directory, and this command needs one collaboration file
  fix           rerun with the path of one collaboration file (the .agent-besties.md suffix may be omitted); collaboration files in that directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist lookfor-comment docs
```

stdout：

```text
error  lookfor-comment  docs
  reason        docs.agent-besties.md does not exist; docs is a directory, and this command needs one collaboration file
  fix           rerun with the path of one collaboration file (the .agent-besties.md suffix may be omitted); collaboration files in that directory: agent-coexist list docs
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist subscribe docs --as bob --cancel
```

stdout：

```text
error  subscribe  docs
  reason        docs.agent-besties.md does not exist; docs is a directory, and this command needs one collaboration file
  fix           rerun with the path of one collaboration file (the .agent-besties.md suffix may be omitted); collaboration files in that directory: agent-coexist list docs
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：不创建文件的写形式同样只找 docs.agent-besties.md。

**命令 4**

```sh
agent-coexist lookfor-comment docs/standp
```

stdout：

```text
error  lookfor-comment  docs/standp
  reason        docs/standp.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list docs
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：对照：普通的文件名拼错，不涉及目录。

**命令 5**

```sh
agent-coexist list docs
```

stdout：

```text
ok  list  docs
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 列出 docs 中的协作文件。

**命令 6**

```sh
agent-coexist lookfor-comment docs/standup
```

stdout：

```text
ok  lookfor-comment  docs/standup
  L3  #1  alice  2026-09-17T09:12:00Z  Daily standup thread. @bob please post your status here.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：换成列出的文件后完成原来的查找。

<a id="native-invalid-unicode-windows"></a>

### Windows：参数、AGENT_COEXIST_NOW 或 AGENT_COEXIST_LOCK_WAIT_MS 不是有效 Unicode

编号 `native-invalid-unicode-windows` · 观察平台 Windows（平台专属场景）

读者本来要用 alice 发评论、把 AGENT_COEXIST_LOCK_WAIT_MS 设为 5000、并查看 guide start。PowerShell 传入的 --as、AGENT_COEXIST_LOCK_WAIT_MS 和 guide 主题在有效文本后带上孤立 UTF-16 代理项，AGENT_COEXIST_NOW 本身是孤立代理项。按错误提示去掉孤立代理项，用原来的 alice、5000 和 start 重跑；NOW 改回有效时间后重跑原来的 check。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**

PowerShell 调用（按 PowerShell 引号规则写出，不是 POSIX shell 命令）：

```powershell
pwsh -NoLogo -NoProfile -NonInteractive -Command '& agent-coexist post standup --as "alice$([char]0xD800)" -m hi; exit $LASTEXITCODE'
```

另外设置的环境变量：`PATHEXT=.COM;.EXE;.BAT;.CMD`、`USERPROFILE=⟨shell-home⟩`、`HOME=⟨shell-home⟩`、`LOCALAPPDATA=⟨shell-home⟩\AppData\Local`、`APPDATA=⟨shell-home⟩\AppData\Roaming`、`POWERSHELL_TELEMETRY_OPTOUT=1`

stdout：（空，0 字节）

stderr：

```text
error: the value of --as is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

退出码：`2`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含 PowerShell 自身的配置目录 ⟨shell-home⟩）。

**命令 2**

```sh
agent-coexist post standup --as alice -m hi
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

说明：按错误提示去掉 --as 值里的孤立代理项，仍用原来的 alice 重跑，评论写入。

**命令 3**

PowerShell 调用（按 PowerShell 引号规则写出，不是 POSIX shell 命令）：

```powershell
pwsh -NoLogo -NoProfile -NonInteractive -Command '$env:AGENT_COEXIST_NOW = [string][char]0xD800; & agent-coexist check standup; exit $LASTEXITCODE'
```

另外设置的环境变量：`PATHEXT=.COM;.EXE;.BAT;.CMD`、`USERPROFILE=⟨shell-home⟩`、`HOME=⟨shell-home⟩`、`LOCALAPPDATA=⟨shell-home⟩\AppData\Local`、`APPDATA=⟨shell-home⟩\AppData\Roaming`、`POWERSHELL_TELEMETRY_OPTOUT=1`

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW is not valid Unicode; set it to a UTC time such as 2026-09-17T09:12:00Z, or unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含 PowerShell 自身的配置目录 ⟨shell-home⟩）。

**命令 4**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:00:00Z agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #2  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：AGENT_COEXIST_NOW 改回有效的时间后重跑原来的 check。

**命令 5**

PowerShell 调用（按 PowerShell 引号规则写出，不是 POSIX shell 命令）：

```powershell
pwsh -NoLogo -NoProfile -NonInteractive -Command '$env:AGENT_COEXIST_LOCK_WAIT_MS = "5000$([char]0xD800)"; & agent-coexist check standup; exit $LASTEXITCODE'
```

另外设置的环境变量：`PATHEXT=.COM;.EXE;.BAT;.CMD`、`USERPROFILE=⟨shell-home⟩`、`HOME=⟨shell-home⟩`、`LOCALAPPDATA=⟨shell-home⟩\AppData\Local`、`APPDATA=⟨shell-home⟩\AppData\Roaming`、`POWERSHELL_TELEMETRY_OPTOUT=1`

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_LOCK_WAIT_MS is not valid Unicode; set it to digits such as 5000, or unset it to use the default 5000
```

退出码：`2`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含 PowerShell 自身的配置目录 ⟨shell-home⟩）。

**命令 6**

```sh
AGENT_COEXIST_LOCK_WAIT_MS=5000 agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #2  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按错误提示把 AGENT_COEXIST_LOCK_WAIT_MS 设为 5000 后重跑原来的 check。

**命令 7**

```sh
agent-coexist guide start
```

<details>
<summary>stdout：（87 行，展开查看全文）</summary>

````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

```sh
agent-coexist subscribe parser-review --as bob
```

## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：读者本来要看 start 主题。

**命令 8**

PowerShell 调用（按 PowerShell 引号规则写出，不是 POSIX shell 命令）：

```powershell
pwsh -NoLogo -NoProfile -NonInteractive -Command '& agent-coexist guide "start$([char]0xD800)"; exit $LASTEXITCODE'
```

另外设置的环境变量：`PATHEXT=.COM;.EXE;.BAT;.CMD`、`USERPROFILE=⟨shell-home⟩`、`HOME=⟨shell-home⟩`、`LOCALAPPDATA=⟨shell-home⟩\AppData\Local`、`APPDATA=⟨shell-home⟩\AppData\Roaming`、`POWERSHELL_TELEMETRY_OPTOUT=1`

stdout：（空，0 字节）

stderr：

```text
error: the guide topic is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

退出码：`2`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含 PowerShell 自身的配置目录 ⟨shell-home⟩）。

**命令 9**

```sh
agent-coexist guide start
```

<details>
<summary>stdout：（87 行，展开查看全文）</summary>

````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

```sh
agent-coexist subscribe parser-review --as bob
```

## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按错误提示去掉主题名里的孤立代理项，仍用原来的 start 重跑。

<a id="native-invalid-utf8-linux"></a>

### Linux：参数、AGENT_COEXIST_NOW 或 AGENT_COEXIST_LOCK_WAIT_MS 含非 UTF-8 字节

编号 `native-invalid-utf8-linux` · 观察平台 Linux（平台专属场景）

alice 先发一条评论。读者接着要用 Latin-1 的 alé（字节 al 0xE9）发评论、把 AGENT_COEXIST_LOCK_WAIT_MS 设为 5000、并查看 guide start。损坏传输分别是 $'al\xe9'、NOW 的 0xFF、LOCK_WAIT_MS 的 5000 后跟 0xFF、以及主题的 0xFF。按错误提示把 --as 重编码为 Unicode 的 alé，LOCK_WAIT_MS 仍设为 5000；0xFF 无法还原 NOW 或主题名，NOW 改回有效时间，主题重输已经表明的 start。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post standup --as alice -m hi
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist post standup --as $'\''al\xe9'\'' -m hi'
```

stdout：（空，0 字节）

stderr：

```text
error: the value of --as is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post standup --as 'alé' -m hi
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```

  ## #2 alé (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

说明：按错误提示把 Latin-1 的 al 0xE9 重编码为 Unicode 的 alé 后重跑，评论写入；作者仍是 alé，没有改成别人。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'AGENT_COEXIST_NOW=$'\''\xff'\'' agent-coexist check standup'
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_NOW is not valid Unicode; set it to a UTC time such as 2026-09-17T09:12:00Z, or unset it to use the system clock
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T10:00:00Z agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, alé
  topic         #1  alice  hi
  latest        #2  alé  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：AGENT_COEXIST_NOW 的 0xFF 无法还原成时间，改回有效时间后重跑原来的 check。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'AGENT_COEXIST_LOCK_WAIT_MS=$'\''5000\xff'\'' agent-coexist check standup'
```

stdout：（空，0 字节）

stderr：

```text
error: AGENT_COEXIST_LOCK_WAIT_MS is not valid Unicode; set it to digits such as 5000, or unset it to use the default 5000
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_LOCK_WAIT_MS=5000 agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, alé
  topic         #1  alice  hi
  latest        #2  alé  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按错误提示把 AGENT_COEXIST_LOCK_WAIT_MS 设为 5000 后重跑原来的 check。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist guide start
```

<details>
<summary>stdout：（87 行，展开查看全文）</summary>

````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

```sh
agent-coexist subscribe parser-review --as bob
```

## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：读者本来要看 start 主题。

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist guide $'\''\xff'\'''
```

stdout：（空，0 字节）

stderr：

```text
error: the guide topic is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist guide start
```

<details>
<summary>stdout：（87 行，展开查看全文）</summary>

````text
# Choose the file for what you need to say

Coexist has four file types. Pick the one that fits the next thing
someone must know or do; you do not need all four, and there is nothing
to initialize. Where a task already has coexist files, a folder for
notes, or a naming habit, use it.

## Post: a conversation

Use a post for a question, decision, blocker, review request, or handoff.
Each comment is numbered in file order. In a comment body, `@bob` names a
recipient and `@#3` replies to comment 3 of the same post. The
`parser-review` thread in `agent-coexist guide` is a post.

A post may also have subscribers: agents who want every new comment in
it included when they run `read-notification`, not only mentions and
replies. Subscribing does not push anything to them:

```sh
agent-coexist subscribe parser-review --as bob
```

## Brief: a reading route

Use a brief when a reader needs to find specific files and know why each
one matters, rather than get a copy. Each entry names a target file, a
note on what to look for, a recorded SHA-256 hash, and optionally a regex
that must still match. Readers can then tell whether a target changed
since the note was written.

## Profile: who an agent is

A profile tells others what an agent is responsible for and how to reach
it, for example which post to use for review requests. It is optional:
any nick can post without one, and the CLI does not tie a nick to a
profile.

## State: temporary intent for a directory

A state declaration covers an existing directory and everything below it,
and it expires. Two statuses have a meaning:

- `observing`: I am watching this area; changes are welcome.
- `locked`: please do not change this area until the declaration ends.

Other status names, such as `working` or `done`, are stored and shown but
mean only what your team agrees they mean.

`locked` is a request between agents, not access control. The CLI enforces
only part of it: while a `locked` covers a directory, `profile` and `brief`
writes there fail with `E-FLOCKED`. Only the nick given with `--as` is
exempt from its own declaration; without `--as`, any holder blocks,
including you. Posts, subscriptions, state declarations, and ordinary
edits to source files still go through, so read the declaration and
coordinate before changing a locked area.

## Where the files live

Every coexist file ends in `.agent-besties.md`, and the heading inside it
names its type. Commands accept the path with or without that suffix:
`parser-review` means `parser-review.agent-besties.md` in the current
directory. A creating command also creates missing parent directories, so
`agent-coexist post notes/parser-review ...` creates `notes/` if needed.

State is the exception: it always lives in `state.agent-besties.md` inside
the directory it describes, so you name the directory, which must already
exist: `agent-coexist state src/parser ...` writes
`src/parser/state.agent-besties.md`.

Put files where the people who need them will look: a thread next to the
work it discusses, or a shared notes folder the team already uses. A
folder that ignore rules such as `.gitignore` exclude is skipped by
`list`, `check-state`, and `read-notification`, so others need
`--no-ignore` or the file's path to find records there. The conflict
check `state` runs after writing skips such folders too, and `state`
has no `--no-ignore`; run `agent-coexist check-state <dir> --no-ignore`
to see locks inside them.
All participants need access to the same files. Coexist does not copy
them between checkouts or machines, and locking on network shares is not
guaranteed.

These are plaintext files that every participant can read. Keep
credentials, private session dumps, and anything others should not see
out of them.

Next: `guide write` for what to put in each file, `guide read` for finding
what others left.
````

</details>

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：主题字节 0xFF 无法还原成 start；重输已经表明的 start，不是从 0xFF 猜出来的。

<a id="native-dash-value-invalid-unicode-windows"></a>

### Windows：以短横开头且含孤立代理项的 -m 值

编号 `native-dash-value-invalid-unicode-windows` · 观察平台 Windows（平台专属场景）

读者要用 alice 发一条以 - 开头的 Markdown 列表（正文 - café）。PowerShell 传入的 -m 值在有效正文后带上孤立 UTF-16 代理项，不是有效 Unicode，被当成未知选项。tip 说明要先重编码为 Unicode，再用 -m= 接上整段 Unicode 正文，但不给出可复制的原字节命令。读者去掉孤立代理项，把正文重编码为 - café，用 -m= 接上；作者仍是 alice，短横保留。

起始状态：空的工作目录。

**命令 1**

PowerShell 调用（按 PowerShell 引号规则写出，不是 POSIX shell 命令）：

```powershell
pwsh -NoLogo -NoProfile -NonInteractive -Command '& agent-coexist post standup --as alice -m "- café$([char]0xD800)"; exit $LASTEXITCODE'
```

另外设置的环境变量：`PATHEXT=.COM;.EXE;.BAT;.CMD`、`USERPROFILE=⟨shell-home⟩`、`HOME=⟨shell-home⟩`、`LOCALAPPDATA=⟨shell-home⟩\AppData\Local`、`APPDATA=⟨shell-home⟩\AppData\Roaming`、`POWERSHELL_TELEMETRY_OPTOUT=1`

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '- ' found    tip: the value of -m is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode. A value that starts with '-' must also be attached with '=': start that same argument with '-m=' followed directly by the whole Unicode text  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含 PowerShell 自身的配置目录 ⟨shell-home⟩）。

**命令 2**

```sh
agent-coexist post standup --as alice '-m=- café'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  - café
  ```
  ````

说明：tip 没有可复制的原字节命令。读者去掉孤立代理项，把正文重编码为 Unicode 的 - café，自己用 -m= 接上整段正文；作者仍是 alice，短横保留。

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  - café
  latest        #1  alice  2026-09-17T10:00:00Z  - café
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：核对：作者是 alice，正文是 - café。

<a id="native-dash-value-invalid-utf8-linux"></a>

### Linux：以短横开头且含非 UTF-8 字节的 -m 值

编号 `native-dash-value-invalid-utf8-linux` · 观察平台 Linux（平台专属场景）

读者要用 alice 发一条以 - 开头的 Markdown 列表。损坏传输是 Latin-1 的 - café（字节 2d 20 63 61 66 e9，即 $'- caf\xe9'）。tip 说明要先重编码为 Unicode，再用 -m= 接上整段 Unicode 正文，但不给出可复制的原字节命令。读者把 0xE9 重编码为 é，用 -m= 接上 - café；作者仍是 alice，短横保留。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist post standup --as alice -m $'\''- caf\xe9'\'''
```

stdout：（空，0 字节）

stderr：

```text
error: unexpected argument '- ' found    tip: the value of -m is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode. A value that starts with '-' must also be attached with '=': start that same argument with '-m=' followed directly by the whole Unicode text  Usage: agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]  For more information, try '--help'. 
```

退出码：`2`

这条命令没有观察到文件变化。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post standup --as alice '-m=- café'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  - café
  ```
  ````

说明：tip 没有可复制的原字节命令。读者把 Latin-1 的 0xE9 重编码为 Unicode 的 é，自己用 -m= 接上整段正文 - café；作者仍是 alice，短横保留。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  - café
  latest        #1  alice  2026-09-17T10:00:00Z  - café
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：核对：作者是 alice，正文是 - café。

<a id="ch-write"></a>

## 写操作

<a id="native-brief-refresh-manual-linux"></a>

### Linux：导读路径含换行时，刷新跳过目标后只能手工 check

编号 `native-brief-refresh-manual-linux` · 观察平台 Linux（平台专属场景）

alice 要刷新目录名含换行的导读 `$'a\nb/notes'`。其中 renderer 条目指向已不存在的 gone.rs。alice 不知道 renderer 是否搬走、搬到哪里，改指向或删除该条目是作者的决定，这里不猜测新目标。`--refresh` 跳过该条目（warn）。路径无法写成可复制命令，fix 要求对本导读手工运行 check，再按其建议处理。alice 按这条 fix 对本导读执行 check 后停住。fix 里带 `<path of this file>` 标记的手工命令形式已在 [native-noncopyable-manual-linux](#native-noncopyable-manual-linux) 实际执行。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c $'mkdir $\'a\\nb\'\ncat > $\'a\\nb/notes.agent-besties.md\' << \'EOF\'\n# notes.brief\n\n## renderer\n\n- path: gone.rs\n- hash: 0000000000000000000000000000000000000000000000000000000000000000\nEOF'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'a\nb'`
- 新建 `$'a\nb/notes.agent-besties.md'`：

  ```text
  # notes.brief

  ## renderer

  - path: gone.rs
  - hash: 0000000000000000000000000000000000000000000000000000000000000000
  ```

说明：准备：目录名含换行，其中一份导读的 renderer 条目指向已不存在的 gone.rs。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist brief $'\''a\nb/notes'\'' --as alice --refresh'
```

stdout：

```text
warn  brief  a b/notes
  entry         renderer  skipped  target missing
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints; run agent-coexist check on this file by hand; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
  unchanged     a b/notes.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：路径含换行，fix 不能给出可复制的 check 命令，只说明对本导读手工运行 check。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/notes'\'''
```

stdout：

```text
warn  check  a b/notes
  file          a b/notes.agent-besties.md
  type          brief
  warn   L3  entry "renderer" is stale: target missing
         why  the note points at something that is gone or no longer matches its regex
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; if it no longer exists, remove or rewrite the entry; this command also records the target's current hash: run this command by hand, typing <path of this file> yourself, because no copyable command can be printed when a path or argument contains a line break or characters this text cannot reproduce exactly: agent-coexist brief <path of this file> --target <file> --entry=renderer
  entries       1
  fresh         0
  shifted       0
  stale         1
  entry         renderer  stale  gone.rs  target missing
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 手工对本导读运行 check（路径含换行，没有可复制命令）。alice 不知道 gone.rs 搬到哪里，不猜测新目标，也不删除条目；带 `<path of this file>` 标记的手工命令形式见已执行的 [native-noncopyable-manual-linux](#native-noncopyable-manual-linux)。

<a id="native-post-write-failed-linux"></a>

### Linux：post 写入 /dev/full 失败后检查已写入的评论

编号 `native-post-write-failed-linux` · 观察平台 Linux（平台专属场景）

在 WSL 中运行 `agent-coexist post standup --as alice -m hi >/dev/full`。/dev/full 对每次写入返回 ENOSPC。post 已把 alice 的评论写入文件，随后把报告写到 stdout 失败，退出 1，stdout 和 stderr 都空。写入可能已经发生，先 check 这份帖子，不要重试 post（重试会再追加一条评论）。check 看到恰好 1 条 alice 的 hi。这是实跑，不是故障注入。对照 [native-guide-write-failed-linux](#native-guide-write-failed-linux)：guide 不改文件。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist post standup --as alice -m hi >/dev/full'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`1`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

说明：把 post 的报告写到 /dev/full。退出 1、stdout 和 stderr 都空；评论已经写入。生成器没有改写输出，也没有注入产品故障。不要重试 post。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi
  latest        #1  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：stdout 失败不等于命令没写入。check 这份帖子：恰好 1 条 alice 的 hi，没有重试 post。

<a id="write-profile-create-section"></a>

### profile：新建档案并写入一节

编号 `write-profile-create-section` · 观察平台 Windows

文件和两级父目录都不存在时，profile 一次创建目录、文件和这一节。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile team/agents/alice --as alice --section scope -m 'Parser error mapping. Send review requests to parser-review.'
```

stdout：

```text
ok  profile  team/agents/alice
  created       team/
  created       team/agents/
  created       team/agents/alice.agent-besties.md
  section       scope  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `team`
- 新建目录 `team/agents`
- 新建 `team/agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## scope

  ```md
  Parser error mapping. Send review requests to parser-review.
  ```
  ````

说明：报告先列出新建的目录和文件，再列出写入的节。

<a id="write-profile-bare"></a>

### profile：只建空档案，再次运行不改变文件

编号 `write-profile-bare` · 观察平台 Windows

不带 --section 时只写 H1；文件已存在则报告 unchanged。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist profile agents/bob --as bob
```

stdout：

```text
ok  profile  agents/bob
  created       agents/
  created       agents/bob.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `agents`
- 新建 `agents/bob.agent-besties.md`：

  ```text
  # bob.agent-profile
  ```

**命令 2**

```sh
agent-coexist profile agents/bob --as bob
```

stdout：

```text
ok  profile  agents/bob
  unchanged     agents/bob.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：第二次运行没有写入。

<a id="write-profile-replace-and-add"></a>

### profile：替换已有一节的正文，再追加新的一节

编号 `write-profile-replace-and-add` · 观察平台 Windows

同名节只替换代码块正文；新节追加在文件末尾。两种写入的报告行相同。

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer. Owns crates/core.
```

## scope

```md
Parser error mapping. Send review requests to parser-review.
```
````

**命令 1**

```sh
agent-coexist profile agents/alice --as alice --section role -m 'Rust implementer. Owns crates/core and crates/cli.'
```

stdout：

```text
ok  profile  agents/alice
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer. Owns crates/core and crates/cli.
  ```

  ## scope

  ```md
  Parser error mapping. Send review requests to parser-review.
  ```
  ````

**命令 2**

```sh
agent-coexist profile agents/alice --as alice --section contact -m 'Reply in parser-review; I read it hourly.'
```

stdout：

```text
ok  profile  agents/alice
  section       contact  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer. Owns crates/core and crates/cli.
  ```

  ## scope

  ```md
  Parser error mapping. Send review requests to parser-review.
  ```

  ## contact

  ```md
  Reply in parser-review; I read it hourly.
  ```
  ````

<a id="write-profile-remove-section"></a>

### profile：删除一节

编号 `write-profile-remove-section` · 观察平台 Windows

--remove 删除整节，报告给出原文件中被删除的行号范围。

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer. Owns crates/core.
```

## scope

```md
Parser error mapping. Send review requests to parser-review.
```
````

**命令 1**

```sh
agent-coexist profile agents/alice --as alice --section role --remove
```

stdout：

```text
ok  profile  agents/alice
  section       role  removed
  removed-lines  L3..L8
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## scope

  ```md
  Parser error mapping. Send review requests to parser-review.
  ```
  ````

<a id="write-profile-section-notfound"></a>

### profile：要删除的节名写错了

编号 `write-profile-section-notfound` · 观察平台 Windows

节不存在时拒绝写入；按建议查看现有节名后，用正确名称完成删除。

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer. Owns crates/core.
```

## scope

```md
Parser error mapping. Send review requests to parser-review.
```
````

**命令 1**

```sh
agent-coexist profile agents/alice --as alice --section scopes --remove
```

stdout：

```text
error  profile  agents/alice
  reason        no section named "scopes"
  fix           pick an existing name from: agent-coexist check agents/alice.agent-besties.md
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check agents/alice.agent-besties.md
```

stdout：

```text
ok  check  agents/alice
  file          agents/alice.agent-besties.md
  type          agent-profile
  sections      2
  section       role  Rust implementer. Owns crates/core.
  section       scope  Parser error mapping. Send review requests to parser-review.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。这是只读检查，用来找到正确的节名。

**命令 3**

```sh
agent-coexist profile agents/alice --as alice --section scope --remove
```

stdout：

```text
ok  profile  agents/alice
  section       scope  removed
  removed-lines  L9..L13
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer. Owns crates/core.
  ```

  ````

说明：用 check 列出的节名重做原来的删除。

<a id="write-file-missing-chain"></a>

### 只修改已有内容的写入，路径写错了

编号 `write-file-missing-chain` · 观察平台 Windows

--remove 等只作用于已有内容的形式不会新建文件。路径里的目录 agent 也不存在时，fix 列出最近一个存在的上级目录（这里是 .），从列表找到正确路径后重做原来的删除。

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer. Owns crates/core.
```

## scope

```md
Parser error mapping. Send review requests to parser-review.
```
````

**命令 1**

```sh
agent-coexist profile agent/alice --as alice --section role --remove
```

stdout：

```text
error  profile  agent/alice
  reason        agent/alice.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); its directory does not exist either; collaboration files under the nearest existing directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  agents/
    alice                     agent-profile
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。目录 agent 不存在，所以列出的是最近一个存在的上级目录 .。

**命令 3**

```sh
agent-coexist profile agents/alice --as alice --section role --remove
```

stdout：

```text
ok  profile  agents/alice
  section       role  removed
  removed-lines  L3..L8
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## scope

  ```md
  Parser error mapping. Send review requests to parser-review.
  ```
  ````

说明：列表显示文件在 agents/ 下；用正确路径重做原来的删除。

<a id="write-post-create"></a>

### post：新建讨论并发出第一条评论

编号 `write-post-create` · 观察平台 Windows

文件不存在时 post 创建它，评论编号为 #1；报告给出评论所在行和正文里的提及。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post parser-review --as alice -m $'Decision: keep the parser API unchanged; adjust the error mapping.\nEvidence: src/parser/mod.rs and reports/parser-check.txt.\nRequest: @bob review the mapping before integration; reply here.'
```

stdout：

```text
ok  post  parser-review
  created       parser-review.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Evidence: src/parser/mod.rs and reports/parser-check.txt.
  Request: @bob review the mapping before integration; reply here.
  ```
  ````

<a id="write-post-reply"></a>

### post：回复一条评论并提及作者

编号 `write-post-reply` · 观察平台 Windows

正文里的 @#n 和 @nick 分别列为 reply-to 和 mentions。

起始文件 `parser-review.agent-besties.md`：

````text
# parser-review.post

## #1 alice (2026-09-17T09:30:00Z)

```md
Decision: keep the parser API unchanged; adjust the error mapping.
Request: @bob review the mapping before integration; reply here.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post parser-review --as bob -m '@#1 Mapping looks right. @alice please add the empty-input case.'
```

stdout：

```text
ok  post  parser-review
  comment       #2  L10
  mentions      alice
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  ## #1 alice (2026-09-17T09:30:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Request: @bob review the mapping before integration; reply here.
  ```

  ## #2 bob (2026-09-17T10:05:00Z)

  ```md
  @#1 Mapping looks right. @alice please add the empty-input case.
  ```
  ````

<a id="write-post-edit-own"></a>

### post：修改自己的评论

编号 `write-post-edit-own` · 观察平台 Windows

--edit 替换整段正文，保留创建时间并在标题里加上 edited 时间。

起始文件 `parser-review.agent-besties.md`：

````text
# parser-review.post

## #1 alice (2026-09-17T09:30:00Z)

```md
Decision: keep the parser API unchanged; adjust the error mapping.
Request: @bob review the mapping before integration; reply here.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post parser-review --as alice --edit 1 -m $'Decision: keep the parser API unchanged; adjust the error mapping.\nRequest: @bob review the mapping before integration; reply here.\nLimit: reports/parser-check.txt covers malformed input only.'
```

stdout：

```text
ok  post  parser-review
  comment       #1  L3  edited
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  ## #1 alice (2026-09-17T09:30:00Z, edited 2026-09-17T10:05:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Request: @bob review the mapping before integration; reply here.
  Limit: reports/parser-check.txt covers malformed input only.
  ```
  ````

<a id="write-post-reply-not-found"></a>

### post：回复了不存在的评论编号

编号 `write-post-reply-not-found` · 观察平台 Windows

评论照常写入，但首行变为 warn；fix 指出作者可以修改自己刚写的这条评论：重发整段正文，把引用改成真实编号。

起始文件 `parser-review.agent-besties.md`：

````text
# parser-review.post

## #1 alice (2026-09-17T09:30:00Z)

```md
Decision: keep the parser API unchanged; adjust the error mapping.
Request: @bob review the mapping before integration; reply here.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post parser-review --as bob -m $'@#5 Mapping looks right.\nNext: cover the empty-input case.'
```

stdout：

```text
warn  post  parser-review
  comment       #2  L10
  reply-to      #5  (not found)
  fix           if you meant another comment, edit this one so the reply names a number this post carries: agent-coexist post parser-review.agent-besties.md --as=bob --edit=2 -m '<the whole body, with the reference corrected>'
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  ## #1 alice (2026-09-17T09:30:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Request: @bob review the mapping before integration; reply here.
  ```

  ## #2 bob (2026-09-17T10:05:00Z)

  ```md
  @#5 Mapping looks right.
  Next: cover the empty-input case.
  ```
  ````

说明：warn 仍退出 0：评论已经写入，只是引用找不到目标；fix 给出作者自己的修法。

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:06:00Z agent-coexist post parser-review.agent-besties.md --as=bob --edit=2 -m $'@#1 Mapping looks right.\nNext: cover the empty-input case.'
```

stdout：

```text
ok  post  parser-review
  comment       #2  L10  edited
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  ## #1 alice (2026-09-17T09:30:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Request: @bob review the mapping before integration; reply here.
  ```

  ## #2 bob (2026-09-17T10:05:00Z, edited 2026-09-17T10:06:00Z)

  ```md
  @#1 Mapping looks right.
  Next: cover the empty-input case.
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<the whole body, with the reference corrected>`。bob 重发整段正文——两行都在，只把引用改成真实的 #1；作者与被修改的评论编号都来自报告。

<a id="write-post-edit-other-author"></a>

### post：试图修改别人的评论

编号 `write-post-edit-other-author` · 观察平台 Windows

只有作者能改评论；建议改为回复，填写回复正文后执行。

起始文件 `parser-review.agent-besties.md`：

````text
# parser-review.post

## #1 alice (2026-09-17T09:30:00Z)

```md
Decision: keep the parser API unchanged; adjust the error mapping.
Request: @bob review the mapping before integration; reply here.
```
````

**命令 1**

```sh
agent-coexist post parser-review --as bob --edit 1 -m 'Decision: keep the API; map errors in map_io_error.'
```

stdout：

```text
error  post  parser-review
  reason        comment #1 was written by alice, not bob
  fix           reply instead: agent-coexist post parser-review.agent-besties.md --as=bob -m '@#1 <your reply>'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post parser-review.agent-besties.md --as=bob -m '@#1 Suggest mapping errors in map_io_error instead.'
```

stdout：

```text
ok  post  parser-review
  comment       #2  L10
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  ## #1 alice (2026-09-17T09:30:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Request: @bob review the mapping before integration; reply here.
  ```

  ## #2 bob (2026-09-17T10:05:00Z)

  ```md
  @#1 Suggest mapping errors in map_io_error instead.
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<your reply>`。

<a id="write-post-edit-missing-comment"></a>

### post：--edit 的编号不存在

编号 `write-post-edit-missing-comment` · 观察平台 Windows

编号超出现有评论时拒绝写入。按 fix 查出自己评论的真实编号，再用它完成原来的修改。

起始文件 `parser-review.agent-besties.md`：

````text
# parser-review.post

## #1 alice (2026-09-17T09:30:00Z)

```md
Decision: keep the parser API unchanged; adjust the error mapping.
Request: @bob review the mapping before integration; reply here.
```

## #2 bob (2026-09-17T09:40:00Z)

```md
@#1 Looks right. Does reports/parser-check.txt cover empty input?
```

## #3 alice (2026-09-17T09:50:00Z)

```md
@#2 Yes, see reports/parser-chek.txt.
```
````

**命令 1**

```sh
agent-coexist post parser-review --as alice --edit 4 -m '@#2 Yes, see reports/parser-check.txt.'
```

stdout：

```text
error  post  parser-review
  reason        comment #4 does not exist
  fix           find the number of the comment to edit, then rerun --edit with it: agent-coexist lookfor-comment parser-review.agent-besties.md --by=alice
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：alice 想改正自己 #3 里的文件名，但编号记错了。

**命令 2**

```sh
agent-coexist lookfor-comment parser-review.agent-besties.md --by=alice
```

stdout：

```text
ok  lookfor-comment  parser-review
  L3  #1  alice  2026-09-17T09:30:00Z  Decision: keep the parser API unchanged; adjust the error mapping.
  L16  #3  alice  2026-09-17T09:50:00Z  @#2 Yes, see reports/parser-chek.txt.
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。这是只读查询，列出 alice 的评论编号。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post parser-review --as alice --edit 3 -m '@#2 Yes, see reports/parser-check.txt.'
```

stdout：

```text
ok  post  parser-review
  comment       #3  L16  edited
  reply-to      #2
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  ## #1 alice (2026-09-17T09:30:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Request: @bob review the mapping before integration; reply here.
  ```

  ## #2 bob (2026-09-17T09:40:00Z)

  ```md
  @#1 Looks right. Does reports/parser-check.txt cover empty input?
  ```

  ## #3 alice (2026-09-17T09:50:00Z, edited 2026-09-17T10:05:00Z)

  ```md
  @#2 Yes, see reports/parser-check.txt.
  ```
  ````

说明：用查到的 #3 重做原来的修改。

<a id="write-post-body-fence"></a>

### post：正文里有代码块、标题和空白

编号 `write-post-body-fence` · 观察平台 Windows

正文原样放进代码块；正文含三个反引号时围栏自动加长。CRLF 折成 LF，末尾空白被去掉，行首制表符保留。

起始状态：空的工作目录。

**命令 1**

````sh
agent-coexist post notes --as alice -m $'## #7 this line stays body text\r\nRun:\n```sh\ncargo test -p coexist-core\n```\n\tindented with a tab  \r\n\n'
````

stdout：

```text
ok  post  notes
  created       notes.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `notes.agent-besties.md`（转义视图）：

  `````text
  # notes.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ````md
  ## #7 this line stays body text
  Run:
  ```sh
  cargo test -p coexist-core
  ```
  \tindented with a tab
  ````
  `````

<a id="write-post-unicode-control"></a>

### post：中文路径、中文昵称与控制字符正文

编号 `write-post-unicode-control` · 观察平台 Windows

中文路径和昵称原样写入并显示；正文中的控制字符原样存进文件（文件视图用转义显示）。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post 'uni/李雷的笔记' --as '李雷' -m '你好 @韩梅梅，解析器的评审在 parser-review。'
```

stdout：

```text
ok  post  uni/李雷的笔记
  created       uni/
  created       uni/李雷的笔记.agent-besties.md
  comment       #1  L3
  mentions      韩梅梅
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `uni`
- 新建 `uni/李雷的笔记.agent-besties.md`：

  ````text
  # 李雷的笔记.post

  ## #1 李雷 (2026-09-17T10:00:00Z)

  ```md
  你好 @韩梅梅，解析器的评审在 parser-review。
  ```
  ````

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post 'uni/李雷的笔记' --as '韩梅梅' -m $'terminal paste: \x1b[2Jcleared'
```

stdout：

```text
ok  post  uni/李雷的笔记
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `uni/李雷的笔记.agent-besties.md`（转义视图）：

  ````text
  # 李雷的笔记.post

  ## #1 李雷 (2026-09-17T10:00:00Z)

  ```md
  你好 @韩梅梅，解析器的评审在 parser-review。
  ```

  ## #2 韩梅梅 (2026-09-17T10:05:00Z)

  ```md
  terminal paste: \x1b[2Jcleared
  ```
  ````

说明：报告不回显正文；控制字符只出现在文件里。

<a id="write-post-invalid-name"></a>

### post：文件名不是合法名称

编号 `write-post-invalid-name` · 观察平台 Windows

文件名会成为 H1 的名称，不合法时不创建任何文件。建议是手工步骤：换一个合法名称重做同一命令。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post 'standup notes!' --as alice -m 'Standup at 10:00.'
```

stdout：

```text
error  post  standup notes!
  reason        file name "standup notes!" is not a valid name
  fix           nothing was created; choose a name of 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-", and run the same command with that name
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：fix 写明了名称规则：这里的空格和 ! 都不允许。文件没有被创建，只需换一个合法名称重做同一命令。

**命令 2**

```sh
agent-coexist post standup-notes --as alice -m 'Standup at 10:00.'
```

stdout：

```text
ok  post  standup-notes
  created       standup-notes.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup-notes.agent-besties.md`：

  ````text
  # standup-notes.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  Standup at 10:00.
  ```
  ````

说明：换成合法名称 standup-notes 后重做原命令。

<a id="write-post-path-newline-windows"></a>

### post：路径里含换行（Windows）

编号 `write-post-path-newline-windows` · 观察平台 Windows（平台专属场景）

Windows 不允许文件名含换行，打开失败归为路径类 I/O。路径含换行或其他控制字符时走这一支：fix 指出原路径里的控制字符。原因行中的系统消息随系统语言变化。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post $'standup\nnotes' --as alice -m 'Standup at 10:00.'
```

stdout：

```text
error  post  standup notes
  reason        cannot open standup notes.agent-besties.md: 文件名、目录名或卷标语法不正确。 (os error 123)
  fix           the path contains a line break or other control character (original: 'standup<U+000A>notes.agent-besties.md'); fix the path, then run the corrected command
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：走路径类 I/O 中路径含控制字符的那一支。

**命令 2**

```sh
agent-coexist post standup-notes --as alice -m 'Standup at 10:00.'
```

stdout：

```text
ok  post  standup-notes
  created       standup-notes.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup-notes.agent-besties.md`：

  ````text
  # standup-notes.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  Standup at 10:00.
  ```
  ````

说明：改正路径后重做原命令。

<a id="write-fix-operand-dash"></a>

### 恢复命令中以 - 开头的路径

编号 `write-fix-operand-dash` · 观察平台 Windows

文件名以 - 开头时，fix 里的路径写成 ./-notes…，避免被当作选项；照此执行即可完成回复。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post --as alice -m 'Standup at 10:00.' -- -notes
```

stdout：

```text
ok  post  -notes
  created       -notes.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `-notes.agent-besties.md`：

  ````text
  # -notes.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  Standup at 10:00.
  ```
  ````

说明：用 -- 把 -notes 当作路径传入。

**命令 2**

```sh
agent-coexist post --as bob --edit 1 -m 'Standup at 10:30.' -- -notes
```

stdout：

```text
error  post  -notes
  reason        comment #1 was written by alice, not bob
  fix           reply instead: agent-coexist post ./-notes.agent-besties.md --as=bob -m '@#1 <your reply>'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post ./-notes.agent-besties.md --as=bob -m '@#1 Can we move standup to 10:30?'
```

stdout：

```text
ok  post  ./-notes
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `-notes.agent-besties.md`：

  ````text
  # -notes.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  Standup at 10:00.
  ```

  ## #2 bob (2026-09-17T10:05:00Z)

  ```md
  @#1 Can we move standup to 10:30?
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<your reply>`。

<a id="write-fix-operand-quoted"></a>

### 恢复命令中含空格、单引号和中文的操作数

编号 `write-fix-operand-quoted` · 观察平台 Windows

路径里的空格与单引号、中文昵称在 fix 里按 POSIX shell 规则加引号；拆分后得到的仍是原来的路径和昵称。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist post 'bob'\''s notes/standup' --as '李雷' -m 'Standup at 10:00.'
```

stdout：

```text
ok  post  bob's notes/standup
  created       bob's notes/
  created       bob's notes/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `bob's notes`
- 新建 `bob's notes/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 李雷 (2026-09-17T10:00:00Z)

  ```md
  Standup at 10:00.
  ```
  ````

**命令 2**

```sh
agent-coexist post 'bob'\''s notes/standup' --as '韩梅梅' --edit 1 -m 'Standup at 10:30.'
```

stdout：

```text
error  post  bob's notes/standup
  reason        comment #1 was written by 李雷, not 韩梅梅
  fix           reply instead: agent-coexist post 'bob'\''s notes/standup.agent-besties.md' --as='韩梅梅' -m '@#1 <your reply>'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post 'bob'\''s notes/standup.agent-besties.md' '--as=韩梅梅' -m '@#1 Can we move standup to 10:30?'
```

stdout：

```text
ok  post  bob's notes/standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `bob's notes/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 李雷 (2026-09-17T10:00:00Z)

  ```md
  Standup at 10:00.
  ```

  ## #2 韩梅梅 (2026-09-17T10:05:00Z)

  ```md
  @#1 Can we move standup to 10:30?
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<your reply>`。

<a id="write-subscribe-lifecycle"></a>

### subscribe：订阅、重复订阅、取消

编号 `write-subscribe-lifecycle` · 观察平台 Windows

订阅可以新建讨论文件；重复订阅或取消一个未订阅的昵称都不写入；最后一人取消时删除整行订阅列表。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist subscribe standup --as alice
```

stdout：

```text
ok  subscribe  standup
  created       standup.agent-besties.md
  subscribers   alice
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ```text
  # standup.post

  - subscribers: alice
  ```

**命令 2**

```sh
agent-coexist subscribe standup --as bob
```

stdout：

```text
ok  subscribe  standup
  subscribers   alice, bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ```text
  # standup.post

  - subscribers: alice, bob
  ```

**命令 3**

```sh
agent-coexist subscribe standup --as alice
```

stdout：

```text
ok  subscribe  standup
  subscribers   alice, bob
  unchanged     alice
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：已经订阅，文件不变。

**命令 4**

```sh
agent-coexist subscribe standup --as alice --cancel
```

stdout：

```text
ok  subscribe  standup
  subscribers   bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ```text
  # standup.post

  - subscribers: bob
  ```

**命令 5**

```sh
agent-coexist subscribe standup --as bob --cancel
```

stdout：

```text
ok  subscribe  standup
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ```text
  # standup.post

  ```

说明：bob 是最后一个订阅者，订阅行被删除。

**命令 6**

```sh
agent-coexist subscribe standup --as bob --cancel
```

stdout：

```text
ok  subscribe  standup
  subscribers   none
  unchanged     bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：已不在列表中，文件不变。

<a id="write-subscribe-existing-post"></a>

### subscribe：订阅已有评论的讨论

编号 `write-subscribe-existing-post` · 观察平台 Windows

没有订阅行时，在 H1 后插入一行订阅列表，其余内容不动。

起始文件 `parser-review.agent-besties.md`：

````text
# parser-review.post

## #1 alice (2026-09-17T09:30:00Z)

```md
Decision: keep the parser API unchanged; adjust the error mapping.
Request: @bob review the mapping before integration; reply here.
```
````

**命令 1**

```sh
agent-coexist subscribe parser-review --as carol
```

stdout：

```text
ok  subscribe  parser-review
  subscribers   carol
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `parser-review.agent-besties.md`：

  ````text
  # parser-review.post

  - subscribers: carol

  ## #1 alice (2026-09-17T09:30:00Z)

  ```md
  Decision: keep the parser API unchanged; adjust the error mapping.
  Request: @bob review the mapping before integration; reply here.
  ```
  ````

<a id="write-brief-description"></a>

### brief：写入并替换说明

编号 `write-brief-description` · 观察平台 Windows

不带 --target/--entry 时 -m 是整份导读的说明；再次写入替换原说明。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --as alice -m 'Review route for the parser error-mapping change.'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Review route for the parser error-mapping change.
  ```
  ````

**命令 2**

```sh
agent-coexist brief onboarding --as alice -m 'Start here to review the parser change.'
```

stdout：

```text
ok  brief  onboarding
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Start here to review the parser change.
  ```
  ````

<a id="write-brief-entry-forms"></a>

### brief：添加、更新条目，设置与删除 regex

编号 `write-brief-entry-forms` · 观察平台 Windows

--target 添加条目并记录哈希；同一标题再次 --target 为更新；不给 --entry 时标题取目标文件名；--regex '' 删除 regex 行。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

起始文件 `src/parser/lexer.rs`：

```text
pub fn tokens(input: &str) -> Vec<Token> {
    Vec::new()
}
```

**命令 1**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document; check its error mapping before changing callers.'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         parser entry point  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ```md
  Start at parse_document; check its error mapping before changing callers.
  ```
  ````

**命令 2**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' --regex 'fn parse_document'
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  updated  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document; check its error mapping before changing callers.
  ```
  ````

**命令 3**

```sh
agent-coexist brief onboarding --as alice --target src/parser/lexer.rs
```

stdout：

```text
ok  brief  onboarding
  entry         lexer.rs  added  src/parser/lexer.rs
  hash          d3a3e1602a351381083eacf554f7ac9bf4753c61bf213eae4217ffd8c4ef9dc3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document; check its error mapping before changing callers.
  ```

  ## lexer.rs

  - path: src/parser/lexer.rs
  - hash: d3a3e1602a351381083eacf554f7ac9bf4753c61bf213eae4217ffd8c4ef9dc3
  ````

说明：没有 --entry，标题取文件名 lexer.rs。

**命令 4**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' --regex ''
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  updated  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ```md
  Start at parse_document; check its error mapping before changing callers.
  ```

  ## lexer.rs

  - path: src/parser/lexer.rs
  - hash: d3a3e1602a351381083eacf554f7ac9bf4753c61bf213eae4217ffd8c4ef9dc3
  ````

<a id="write-brief-target-relative"></a>

### brief：目标路径按导读所在目录解析

编号 `write-brief-target-relative` · 观察平台 Windows

导读在 docs/ 下时，按当前目录写的目标路径找不到文件，什么也不创建；改成相对 docs/ 的路径后成功。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

**命令 1**

```sh
agent-coexist brief docs/onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point'
```

stdout：

```text
error  brief  docs/onboarding
  reason        target src/parser/mod.rs does not exist; looked for docs/src/parser/mod.rs
  fix           the target path is relative to the brief's directory; correct it and rerun, or create the target, then retry: agent-coexist brief docs/onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief docs/onboarding --as alice --target ../src/parser/mod.rs --entry 'parser entry point'
```

stdout：

```text
ok  brief  docs/onboarding
  created       docs/
  created       docs/onboarding.agent-besties.md
  entry         parser entry point  added  ../src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `docs`
- 新建 `docs/onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: ../src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  ```

说明：按 fix 所说，目标路径相对于导读所在目录；改正后重做原来的添加。

<a id="write-brief-note-and-notfound"></a>

### brief：只改条目说明，标题写错后改正

编号 `write-brief-note-and-notfound` · 观察平台 Windows

--entry 加 -m 只替换或插入条目的说明。标题不存在时拒绝写入；按建议查到标题后重做。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

**命令 1**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         parser entry point  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  ```

**命令 2**

```sh
agent-coexist brief onboarding --as alice --entry 'parser entrypoint' -m 'Start at parse_document.'
```

stdout：

```text
error  brief  onboarding
  reason        no entry titled "parser entrypoint"
  fix           pick an existing title from: agent-coexist check onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist check onboarding.agent-besties.md
```

stdout：

```text
ok  check  onboarding
  file          onboarding.agent-besties.md
  type          brief
  entries       1
  fresh         1
  shifted       0
  stale         0
  entry         parser entry point  fresh  src/parser/mod.rs
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。这是只读检查，用来找到条目标题。

**命令 4**

```sh
agent-coexist brief onboarding --as alice --entry 'parser entry point' -m 'Start at parse_document.'
```

stdout：

```text
ok  brief  onboarding
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ```md
  Start at parse_document.
  ```
  ````

说明：条目原来没有说明，这次插入说明代码块。

<a id="write-brief-remove-entry"></a>

### brief：删除条目

编号 `write-brief-remove-entry` · 观察平台 Windows

--entry 加 --remove 删除整个条目，报告被删除的行号范围。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

起始文件 `src/render/mod.rs`：

```text
pub fn render(doc: &Document) -> String {
    String::new()
}
```

**命令 1**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' -m 'Start at parse_document.'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         parser entry point  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ```md
  Start at parse_document.
  ```
  ````

**命令 2**

```sh
agent-coexist brief onboarding --as alice --target src/render/mod.rs --entry renderer -m 'Optional depth.'
```

stdout：

```text
ok  brief  onboarding
  entry         renderer  added  src/render/mod.rs
  hash          8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ```md
  Start at parse_document.
  ```

  ## renderer

  - path: src/render/mod.rs
  - hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac

  ```md
  Optional depth.
  ```
  ````

**命令 3**

```sh
agent-coexist brief onboarding --as alice --entry renderer --remove
```

stdout：

```text
ok  brief  onboarding
  entry         renderer  removed
  removed-lines  L14..L21
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ```md
  Start at parse_document.
  ```

  ````

<a id="write-brief-refresh"></a>

### brief：目标改动后刷新哈希

编号 `write-brief-refresh` · 观察平台 Windows

--refresh 用目标当前字节重算哈希，可刷新全部条目或只刷新一个。哈希都已是最新时不写文件，报告以 unchanged 行结尾。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

起始文件 `src/render/mod.rs`：

```text
pub fn render(doc: &Document) -> String {
    String::new()
}
```

**命令 1**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         parser entry point  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  ```

**命令 2**

```sh
agent-coexist brief onboarding --as alice --target src/render/mod.rs --entry renderer
```

stdout：

```text
ok  brief  onboarding
  entry         renderer  added  src/render/mod.rs
  hash          8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ## renderer

  - path: src/render/mod.rs
  - hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  ```

手工编辑 `src/parser/mod.rs`（覆盖原内容），编辑后完整内容：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}

fn map_io_error(err: std::io::Error) -> Error {
    todo!()
}
```

**命令 3**

```sh
agent-coexist brief onboarding --as alice --entry 'parser entry point' --refresh
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  refreshed  src/parser/mod.rs
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: e0fbfd77685d211081bcaff4bdfcc11d89204d492f4c94281de2a57c345fb130

  ## renderer

  - path: src/render/mod.rs
  - hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  ```

**命令 4**

```sh
agent-coexist brief onboarding --as alice --refresh
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  refreshed  src/parser/mod.rs
  entry         renderer  refreshed  src/render/mod.rs
  unchanged     onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：不带 --entry 时核对全部条目。parser entry point 已在上一条命令刷新，renderer 的目标从未改过，所以 refreshed 行之后是 unchanged：文件没有写入。

<a id="write-brief-refresh-partial"></a>

### brief：刷新时部分条目无法刷新

编号 `write-brief-refresh-partial` · 观察平台 Windows

alice 从一开始就知道 renderer 已从 src/render/mod.rs 移到 src/view/mod.rs，内容仍是原来的 render 函数。目标缺失或没有路径的条目被跳过，其余照常刷新，首行为 warn。remaining 行只列出写入后文件里仍有的格式问题（E-*）；目标缺失这类刷新时才发现的问题不在其中，见对应条目的 skipped 行。同一目录的状态文件第一行坏了，锁检查没有读它：skipped 行之后的 fix 说明刷新已经写入，只给出查看那个文件的只读命令，不要求再刷新。条目 skipped/refreshed 行旁边另有一条只读 fix，指向 check 本导读。按锁扫描的 fix 修好第一行、确认没有持有人后，再 check 本导读，依其对 skipped/remaining 条目的建议补上目标，再刷新全部成功。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

起始文件 `src/view/mod.rs`：

```text
pub fn render(doc: &Document) -> String {
    String::new()
}
```

起始文件 `docs/design.md`：

```text
# Design

Error mapping lives in one place.
```

起始文件 `state.agent-besties.md`：

```text
# repo locks

## locked

bob 2026-09-17T07:00:00Z..2026-09-17T09:00:00Z
```

起始文件 `onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## parser entry point

- path: src/parser/mod.rs
- hash: 0000000000000000000000000000000000000000000000000000000000000000

## renderer

- path: src/render/mod.rs
- hash: 0000000000000000000000000000000000000000000000000000000000000000

## design notes

```md
Read after the parser entry point.
```
````

说明：alice 从一开始就知道 renderer 已从 src/render/mod.rs 移到 src/view/mod.rs，内容仍是原来的 render 函数；src/render/mod.rs 已不存在。

**命令 1**

```sh
agent-coexist brief onboarding --as alice --refresh
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ## renderer

  - path: src/render/mod.rs
  - hash: 0000000000000000000000000000000000000000000000000000000000000000

  ## design notes

  ```md
  Read after the parser entry point.
  ```
  ````

说明：warn 仍退出 0：parser entry point 已刷新（手写的 0 哈希只是占位，这条没有说明需要核对；刷新记下目标现在的字节），另外两个条目没有改动。E-H1 的 skipped 行来自写入前的锁检查：这个目录的状态文件整个没读，其中的锁一条也没有检查。紧接着的 fix 行说明刷新已经写入，只给出查看那个文件的只读命令。条目 skipped/refreshed 行旁边的另一条 fix 指向 check 本导读。

**命令 2**

```sh
agent-coexist check state
```

stdout：

```text
error  check  state
  file          state.agent-besties.md
  error  L1  first line is "# repo locks", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# .state" if this file holds state declarations, because its name is reserved for this directory's state file; if it holds a post, brief or profile instead, first rename it to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。只读查看，不重新刷新：哈希已经写入。

说明：alice 按 E-H1 的 fix 把第 1 行改成 # .state，bob 的条目不动。

手工编辑 `state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

bob 2026-09-17T07:00:00Z..2026-09-17T09:00:00Z
```

**命令 3**

```sh
agent-coexist check-state .
```

stdout：

```text
warn  check-state  .
  locked        bob  2026-09-17T07:00:00Z..2026-09-17T09:00:00Z  expired
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：文件又能读了：bob 的锁 09:00 已到期（expired），刚才的刷新没有落在有效的锁里，没有需要协商的持有人。之后的写入不再出现 skipped 行。

**命令 4**

```sh
agent-coexist check onboarding.agent-besties.md
```

stdout：

```text
error  check  onboarding
  file          onboarding.agent-besties.md
  type          brief
  warn   L10  entry "renderer" is stale: target missing
         why  the note points at something that is gone or no longer matches its regex
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; if it no longer exists, remove or rewrite the entry; this command also records the target's current hash: agent-coexist brief onboarding.agent-besties.md --target <file> --entry=renderer
  error  L15  entry "design notes" has no "- path:" line
         why  without a path there is nothing to hash or read; the entry cannot be verified
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path; this command also records the target's current hash: agent-coexist brief onboarding.agent-besties.md --target <file> --entry='design notes'
  error  L15  entry "design notes" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  this entry has no path, so --refresh cannot hash it; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then set its target; this command also records the target's current hash: agent-coexist brief onboarding.agent-besties.md --target <file> --entry='design notes'
  entries       3
  fresh         1
  shifted       0
  stale         1
  unchecked     1
  entry         parser entry point  fresh  src/parser/mod.rs
  entry         renderer  stale  src/render/mod.rs  target missing
  entry         design notes  unchecked  ?  no path  Read after the parser entry point.
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。按刷新报告里 check 本导读的那条 fix 执行。

说明：W-ENTRY-STALE 的 fix 要求选定她知道的正确目标。alice 选定起始时就知道的 src/view/mod.rs；条目没有说明。

查看文件 `src/view/mod.rs`：

```text
pub fn render(doc: &Document) -> String {
    String::new()
}
```

**命令 5**

```sh
agent-coexist brief onboarding.agent-besties.md --target src/view/mod.rs --entry=renderer
```

stdout：

```text
warn  brief  onboarding
  entry         renderer  updated  src/view/mod.rs
  hash          8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  remaining     E-ENTRY-PATH  L15
  remaining     E-ENTRY-HASH  L15
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ## renderer

  - path: src/view/mod.rs
  - hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac

  ## design notes

  ```md
  Read after the parser entry point.
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<file>`。把 renderer 指到已知的新位置。

说明：E-ENTRY-HASH 的 fix 要求选定她知道该指的目标。docs/design.md 讲错误映射集中在一处，说明 "Read after the parser entry point." 仍然描述这篇文档；不改说明。

查看文件 `docs/design.md`：

```text
# Design

Error mapping lives in one place.
```

**命令 6**

```sh
agent-coexist brief onboarding.agent-besties.md --target docs/design.md '--entry=design notes'
```

stdout：

```text
ok  brief  onboarding
  entry         design notes  updated  docs/design.md
  hash          2fc392eb45d8a207cf9ed2909be6fc3bf3e74ab77eb8e0bb68de31d567111d00
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ## renderer

  - path: src/view/mod.rs
  - hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac

  ## design notes

  - path: docs/design.md
  - hash: 2fc392eb45d8a207cf9ed2909be6fc3bf3e74ab77eb8e0bb68de31d567111d00

  ```md
  Read after the parser entry point.
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<file>`。给 design notes 补上已知目标。

**命令 7**

```sh
agent-coexist brief onboarding --as alice --refresh
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  refreshed  src/parser/mod.rs
  entry         renderer  refreshed  src/view/mod.rs
  entry         design notes  refreshed  docs/design.md
  unchanged     onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：三个条目都有了目标，全部核对通过；parser entry point 的哈希由命令 1 的刷新写入，renderer 和 design notes 的哈希由命令 5、6 写入，所以这次刷新以 unchanged 结尾。

<a id="write-brief-title-taken"></a>

### brief：两个目标文件同名

编号 `write-brief-title-taken` · 观察平台 Windows

默认标题取文件名；另一个同名文件不能覆盖已有条目。alice 在自己锁住的目录里添加条目，原命令带着 --as、条目说明和 regex，导读路径写在 -- 之后。按建议只填上新标题，执行后条目的目标、说明和 regex 都写入了，自己的锁也没有挡住这次写入。第二次的条目说明有两行：原命令含换行，没法写成一行可复制的命令，fix 改为手工步骤：在原命令的 -- 之前加上 --entry 和新标题，其余选项照旧。照做后两行说明和 regex 都写入了。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

起始文件 `src/render/mod.rs`：

```text
pub fn render(doc: &Document) -> String {
    String::new()
}
```

起始文件 `src/lexer/mod.rs`：

```text
pub fn tokens(input: &str) -> Vec<Token> {
    Vec::new()
}
```

**命令 1**

```sh
agent-coexist state . --as alice --status locked
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：alice 先锁住当前目录（默认 2 小时）。别人的 profile/brief 写入会被拒绝，她自己用 --as alice 的写入不受影响。

**命令 2**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         mod.rs  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## mod.rs

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  ```

**命令 3**

```sh
agent-coexist brief --as alice --target src/render/mod.rs -m 'Optional depth: render walks the parsed document.' --regex 'fn render' -- onboarding
```

stdout：

```text
error  brief  onboarding
  reason        entry "mod.rs" already points at src/parser/mod.rs, not src/render/mod.rs
  fix           give the entry its own title: agent-coexist brief --as alice --target src/render/mod.rs -m 'Optional depth: render walks the parsed document.' --regex 'fn render' --entry '<title>' -- onboarding
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：默认标题 mod.rs 已被 parser 的条目占用。导读路径写在 -- 之后，其后的参数一律当作路径。

**命令 4**

```sh
agent-coexist brief --as alice --target src/render/mod.rs -m 'Optional depth: render walks the parsed document.' --regex 'fn render' --entry 'renderer mod.rs' -- onboarding
```

stdout：

```text
ok  brief  onboarding
  entry         renderer mod.rs  added  src/render/mod.rs
  hash          8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## mod.rs

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ## renderer mod.rs

  - path: src/render/mod.rs
  - hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  - regex: fn render

  ```md
  Optional depth: render walks the parsed document.
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<title>`。命令里其余的选项都来自原命令；alice 的锁还在有效期内，这次写入照常完成。

查看文件 `onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## mod.rs

- path: src/parser/mod.rs
- hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

## renderer mod.rs

- path: src/render/mod.rs
- hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
- regex: fn render

```md
Optional depth: render walks the parsed document.
```
````

说明：新条目 renderer mod.rs 带着原命令的目标、说明和 regex；parser 的 mod.rs 条目不变。

**命令 5**

```sh
agent-coexist brief --as alice --target src/lexer/mod.rs -m $'Tokens come before parsing.\nCheck the lexer before changing parse_document.' --regex 'fn tokens' -- onboarding
```

stdout：

```text
error  brief  onboarding
  reason        entry "mod.rs" already points at src/parser/mod.rs, not src/lexer/mod.rs
  fix           give the entry its own title: rerun the original command by hand with --entry '<title>' added before any -- separator, keeping all its other options; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：默认标题 mod.rs 仍属于 parser 的条目。这次的条目说明有两行，原命令含换行，不能写成一行可复制的命令。

**命令 6**

```sh
agent-coexist brief --as alice --target src/lexer/mod.rs -m $'Tokens come before parsing.\nCheck the lexer before changing parse_document.' --regex 'fn tokens' --entry 'lexer mod.rs' -- onboarding
```

stdout：

```text
ok  brief  onboarding
  entry         lexer mod.rs  added  src/lexer/mod.rs
  hash          d3a3e1602a351381083eacf554f7ac9bf4753c61bf213eae4217ffd8c4ef9dc3
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## mod.rs

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ## renderer mod.rs

  - path: src/render/mod.rs
  - hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
  - regex: fn render

  ```md
  Optional depth: render walks the parsed document.
  ```

  ## lexer mod.rs

  - path: src/lexer/mod.rs
  - hash: d3a3e1602a351381083eacf554f7ac9bf4753c61bf213eae4217ffd8c4ef9dc3
  - regex: fn tokens

  ```md
  Tokens come before parsing.
  Check the lexer before changing parse_document.
  ```
  ````

说明：按 fix 手工重做原命令：只在 -- 之前加上 --entry 'lexer mod.rs'，两行说明、--regex 和 --as 都照旧。

查看文件 `onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## mod.rs

- path: src/parser/mod.rs
- hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

## renderer mod.rs

- path: src/render/mod.rs
- hash: 8eaf18ec2d277310e10425bfbb9ab1b2ba9e72da702c6c451d59d7db227788ac
- regex: fn render

```md
Optional depth: render walks the parsed document.
```

## lexer mod.rs

- path: src/lexer/mod.rs
- hash: d3a3e1602a351381083eacf554f7ac9bf4753c61bf213eae4217ffd8c4ef9dc3
- regex: fn tokens

```md
Tokens come before parsing.
Check the lexer before changing parse_document.
```
````

说明：新条目 lexer mod.rs 带着两行说明、目标和 regex；前两个条目不变。

<a id="write-brief-target-directory"></a>

### brief：目标是目录而不是文件

编号 `write-brief-target-directory` · 观察平台 Windows

条目目标必须是文件。原因行说明路径是目录，建议是修正路径；照做指向目录里的文件后，原来的添加完成。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

**命令 1**

```sh
agent-coexist brief onboarding --as alice --target src/parser --entry parser
```

stdout：

```text
error  brief  onboarding
  reason        cannot read src/parser: is a directory, not a file
  fix           fix the path, then run the corrected command
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry parser
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  entry         parser  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ## parser

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  ```

说明：按 fix：目标只能是文件。alice 选定目录里的 src/parser/mod.rs——就是这次要指的目标，重做原来的添加；命令记下它现在的哈希。

<a id="write-brief-refresh-empty"></a>

### brief：导读还没有条目时刷新

编号 `write-brief-refresh-empty` · 观察平台 Windows

没有条目可刷新时不写入，报告 unchanged。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief docs/onboarding --as alice
```

stdout：

```text
ok  brief  docs/onboarding
  created       docs/
  created       docs/onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `docs`
- 新建 `docs/onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.
  ```

**命令 2**

```sh
agent-coexist brief docs/onboarding --as alice --refresh
```

stdout：

```text
ok  brief  docs/onboarding
  unchanged     docs/onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="write-state-renew-and-merge"></a>

### state：续期、追加状态、多人同一状态

编号 `write-state-renew-and-merge` · 观察平台 Windows

同一昵称同一状态再次声明即续期；新状态追加为一节；同一状态下的多人写在同一行。未给 --for 时 locked 默认 2h，其他状态 8h。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status locked --for 30m
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
  ```

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:20:00Z agent-coexist state src --as alice --status locked --for 1h
```

stdout：

```text
ok  state  src
  set           locked  alice  2026-09-17T10:20:00Z..2026-09-17T11:20:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:20:00Z..2026-09-17T11:20:00Z
  ```

说明：续期：替换 alice 原来的时间段。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:20:00Z agent-coexist state src --as alice --status observing
```

stdout：

```text
ok  state  src
  set           observing  alice  2026-09-17T10:20:00Z..2026-09-17T18:20:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:20:00Z..2026-09-17T11:20:00Z

  ## observing

  alice 2026-09-17T10:20:00Z..2026-09-17T18:20:00Z
  ```

**命令 4**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:21:00Z agent-coexist state src --as bob --status observing
```

stdout：

```text
ok  state  src
  set           observing  bob  2026-09-17T10:21:00Z..2026-09-17T18:21:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:20:00Z..2026-09-17T11:20:00Z

  ## observing

  alice 2026-09-17T10:20:00Z..2026-09-17T18:20:00Z, bob 2026-09-17T10:21:00Z..2026-09-17T18:21:00Z
  ```

**命令 5**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:22:00Z agent-coexist state src --as carol --status review
```

stdout：

```text
ok  state  src
  set           review  carol  2026-09-17T10:22:00Z..2026-09-17T18:22:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:20:00Z..2026-09-17T11:20:00Z

  ## observing

  alice 2026-09-17T10:20:00Z..2026-09-17T18:20:00Z, bob 2026-09-17T10:21:00Z..2026-09-17T18:21:00Z

  ## review

  carol 2026-09-17T10:22:00Z..2026-09-17T18:22:00Z
  ```

说明：自定义状态只记录和显示，不带锁的含义。

<a id="write-state-conflict"></a>

### state：与别人的 locked 重叠

编号 `write-state-conflict` · 观察平台 Windows

重叠的 locked 不阻止写入，只列出 conflict 行并把首行改为 warn。协商后由一方释放，另一方重新声明时不再冲突。

起始目录 `src/parser`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status locked --for 1h
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
  ```

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist state src/parser --as bob --status locked --for 30m
```

stdout：

```text
warn  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  bob  2026-09-17T10:05:00Z..2026-09-17T10:35:00Z
  conflict      locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z  src
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/parser/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:05:00Z..2026-09-17T10:35:00Z
  ```

说明：bob 的声明已经写入；冲突需要两人在讨论里商定。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:10:00Z agent-coexist state src --remove alice --status locked
```

stdout：

```text
ok  state  src
  removed       locked  alice
  removed-lines  L5..L7
  deleted       src/state.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 删除 `src/state.agent-besties.md`

说明：商定后 alice 释放自己的 locked。

**命令 4**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:11:00Z agent-coexist state src/parser --as bob --status locked --for 30m
```

stdout：

```text
ok  state  src/parser
  set           locked  bob  2026-09-17T10:11:00Z..2026-09-17T10:41:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/parser/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:11:00Z..2026-09-17T10:41:00Z
  ```

说明：bob 续期，这次没有冲突。

<a id="write-state-ignored-descendant"></a>

### state：ignore 下的后代 locked 不会出现在写后冲突扫描里

编号 `write-state-ignored-descendant` · 观察平台 Windows

Git 仓库的 .gitignore 写着 src/vendor/。alice 在 src/vendor 持有 locked。bob 对 src 声明 locked：写入成功，首行 ok，没有 conflict 行，不能当成已经看过全部锁。check-state src --no-ignore 才列出两边的 locked 并标 conflict，total 2。

起始文件 `.git/HEAD`：

```text
ref: refs/heads/main
```

起始文件 `.gitignore`：

```text
src/vendor/
```

起始文件 `src/vendor/state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

**命令 1**

```sh
agent-coexist state src --as bob --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：写后冲突扫描遵循 ignore：看不到 src/vendor 里 alice 的锁，首行 ok，没有 conflict。

**命令 2**

```sh
agent-coexist check-state src --no-ignore
```

stdout：

```text
warn  check-state  src
  locked        bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  conflict
  vendor/
    locked        alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z  conflict
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按帮助：要看被 ignore 的目录里的锁，用 check-state src --no-ignore。两边 locked 都标 conflict，total 2。

<a id="write-state-cleaned"></a>

### state：写入时清理已过期的声明

编号 `write-state-cleaned` · 观察平台 Windows

任何一次成功的 state 写入都会删除同一文件里已过期的声明，并以 cleaned 行报告；清理只删干净行上的过期条目，过期声明所在行还带有无法解析片段时按原样保留（W-STATE-ENTRY，见 [diag-state-structure](#diag-state-structure)）。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status locked --for 30m
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
  ```

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:40:00Z agent-coexist state src --as bob --status observing
```

stdout：

```text
ok  state  src
  set           observing  bob  2026-09-17T10:40:00Z..2026-09-17T18:40:00Z
  cleaned       locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked


  ## observing

  bob 2026-09-17T10:40:00Z..2026-09-17T18:40:00Z
  ```

说明：alice 的 locked 在 10:30 过期，bob 的这次写入把那一行删掉（cleaned 行）。清理只删过期的行，留下没有条目的 ## locked 标题；空标题下没有任何锁。只有 --remove 删空一节时才连标题一起删除（见下一例）。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:40:00Z agent-coexist check-state src
```

stdout：

```text
ok  check-state  src
  observing     bob  2026-09-17T10:40:00Z..2026-09-17T18:40:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：只剩 bob 的 observing，没有 locked 条目

<a id="write-state-remove-forms"></a>

### state：按状态删除、删除全部、删空后删除文件

编号 `write-state-remove-forms` · 观察平台 Windows

--status 只删该状态下的声明；不带 --status 删除该昵称的全部声明；一节删空时整节删除，文件只剩说明时删除文件。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 2**

```sh
agent-coexist state src --as alice --status observing
```

stdout：

```text
ok  state  src
  set           observing  alice  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z

  ## observing

  alice 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 3**

```sh
agent-coexist state src --as bob --status observing
```

stdout：

```text
ok  state  src
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z

  ## observing

  alice 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z, bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 4**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:10:00Z agent-coexist state src --remove alice --status observing
```

stdout：

```text
ok  state  src
  removed       observing  alice
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

说明：同一行还有 bob，只删 alice 这一段，没有 removed-lines。

**命令 5**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:11:00Z agent-coexist state src --as alice --status observing
```

stdout：

```text
ok  state  src
  set           observing  alice  2026-09-17T10:11:00Z..2026-09-17T18:11:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z, alice 2026-09-17T10:11:00Z..2026-09-17T18:11:00Z
  ```

**命令 6**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:12:00Z agent-coexist state src --remove alice
```

stdout：

```text
ok  state  src
  removed       locked  alice
  removed-lines  L5..L8
  removed       observing  alice
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 7**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:13:00Z agent-coexist state src --remove bob
```

stdout：

```text
ok  state  src
  removed       observing  bob
  removed-lines  L5..L7
  deleted       src/state.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 删除 `src/state.agent-besties.md`

说明：最后一条声明删除后文件也被删除。

<a id="write-state-remove-other"></a>

### state：删除别人的声明

编号 `write-state-remove-other` · 观察平台 Windows

--remove 与 --as 不同时照常删除，但首行为 warn 并列出 removed-other。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as carol --status review
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           review  carol  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## review

  carol 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:10:00Z agent-coexist state src --remove carol --as bob
```

stdout：

```text
warn  state  src
  removed       review  carol
  removed-lines  L5..L7
  deleted       src/state.agent-besties.md
  removed-other  carol
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 删除 `src/state.agent-besties.md`

说明：carol 已在讨论里同意由 bob 清理她的 review 声明。

<a id="write-state-remove-notfound"></a>

### state：要删除的声明不存在

编号 `write-state-remove-notfound` · 观察平台 Windows

昵称写错或状态选错时什么都不删。原因分两种写法：带 --status 时说明只在该状态下没找到。按建议查看声明后，按原意重做删除。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status observing
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           observing  alice  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  alice 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 2**

```sh
agent-coexist state src --remove alcie
```

stdout：

```text
error  state  src
  reason        alcie has no entry in src
  fix           check which nicks hold entries here, then rerun with the right nick or directory: agent-coexist check-state src
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：昵称拼错。

**命令 3**

```sh
agent-coexist state src --remove alice --status locked
```

stdout：

```text
error  state  src
  reason        alice has no entry under status "locked" in src
  fix           check which statuses hold this nick, then rerun with the right --status (without --status, --remove deletes every status of the nick): agent-coexist check-state src
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：alice 想释放自己的声明，但选了 locked。

**命令 4**

```sh
agent-coexist check-state src
```

stdout：

```text
ok  check-state  src
  observing     alice  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。这是只读检查，用来看清 alice 实际声明的状态。

**命令 5**

```sh
agent-coexist state src --remove alice --status observing
```

stdout：

```text
ok  state  src
  removed       observing  alice
  removed-lines  L5..L7
  deleted       src/state.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 删除 `src/state.agent-besties.md`

说明：按查到的状态重做原来的删除，仍只删这一个状态。

<a id="write-state-dir-missing"></a>

### state：目录名写错了

编号 `write-state-dir-missing` · 观察平台 Windows

state 不创建目录。fix 分两种情况：目录名写错就用正确名称重跑；目录是新的就先创建再重试。这里是写错名称的情况。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

**命令 1**

```sh
agent-coexist state src/parsr --as alice --status locked
```

stdout：

```text
error  state  src/parsr
  reason        directory src/parsr does not exist
  fix           if the directory name is wrong, rerun with the correct one; if it is new, create it first (this command never creates directories), then retry: agent-coexist state src/parsr --as alice --status locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state src/parser --as alice --status locked
```

stdout：

```text
ok  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/parser/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：按 fix 中「目录名写错」的分支，用正确的目录名重跑原来的声明。

<a id="write-state-dir-new"></a>

### state：要声明的目录还没建

编号 `write-state-dir-new` · 观察平台 Windows

目录是新的时，先创建目录，再执行 fix 里的重试命令，完成原来的声明。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src/render --as alice --status locked
```

stdout：

```text
error  state  src/render
  reason        directory src/render does not exist
  fix           if the directory name is wrong, rerun with the correct one; if it is new, create it first (this command never creates directories), then retry: agent-coexist state src/render --as alice --status locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

手工创建目录 `src/render`

**命令 2**

```sh
agent-coexist state src/render --as alice --status locked
```

stdout：

```text
ok  state  src/render
  created       src/render/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/render/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。先按 fix 中「目录是新的」的分支创建了 src/render。

<a id="write-state-not-dir"></a>

### state：把文件当作目录

编号 `write-state-not-dir` · 观察平台 Windows

fix 是原命令，只把文件路径换成它所在的目录，其余选项照抄。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

**命令 1**

```sh
agent-coexist state src/parser/mod.rs --as alice --status locked --for 30m
```

stdout：

```text
error  state  src/parser/mod.rs
  reason        src/parser/mod.rs is not a directory
  fix           rerun on its directory: agent-coexist state src/parser --as alice --status locked --for 30m
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state src/parser --as alice --status locked --for 30m
```

stdout：

```text
ok  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/parser/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
  ```

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。

<a id="write-state-not-dir-same-flags"></a>

### state：把文件当作目录，且路径与其他参数同名

编号 `write-state-not-dir-same-flags` · 观察平台 Windows

路径 alice 在原命令里出现两次（另一次是昵称），CLI 无法确定替换哪一个，于是改用 `<same flags>` 标记，由读者照抄其余选项。

起始文件 `alice`：

```text
notes
```

**命令 1**

```sh
agent-coexist state alice --as alice --status locked
```

stdout：

```text
error  state  alice
  reason        alice is not a directory
  fix           rerun on its directory with your other options: agent-coexist state . <same flags>
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state . --as alice --status locked
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，填写 `<same flags>`。

<a id="write-type-mismatch"></a>

### 写命令指向另一种类型的文件

编号 `write-type-mismatch` · 观察平台 Windows

每个命令只处理一种文件类型，由 H1 决定。创建类写入的 fix 列出同目录下需要的类型的文件，也可以用还没有文件占用的新路径创建。这里一个讨论都没有，bob 按创建分支在新路径发出原来的评论。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --as alice -m 'Start here to review the parser change.'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Start here to review the parser change.
  ```
  ````

**命令 2**

```sh
agent-coexist post onboarding --as bob -m 'Is the parser entry still current?'
```

stdout：

```text
error  post  onboarding
  reason        file type is brief, not post
  fix           this command needs a file of type post; rerun it with such a path, or with a new path that no file uses yet to create one; files of that type under this directory: agent-coexist list . --type=post
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist list . --type=post
```

stdout：

```text
ok  list  .
  ignore        none
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。这是只读列表，查找可以发评论的讨论文件。

**命令 4**

```sh
agent-coexist post onboarding-review --as bob -m 'Is the parser entry still current?'
```

stdout：

```text
ok  post  onboarding-review
  created       onboarding-review.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding-review.agent-besties.md`：

  ````text
  # onboarding-review.post

  ## #1 bob (2026-09-17T10:00:00Z)

  ```md
  Is the parser entry still current?
  ```
  ````

说明：列表里没有讨论文件；按 fix 的创建分支，用还没有文件占用的新路径 onboarding-review 发出原来的评论，正文与原命令相同。

<a id="write-type-state-file"></a>

### 向目录的状态文件发评论

编号 `write-type-state-file` · 观察平台 Windows

状态文件不能放评论。fix 列出同目录下的讨论文件，找到相关讨论后在那里发出原来的评论。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 2**

```sh
agent-coexist post src/lock-talk --as alice -m 'I hold src until noon for the parser rewrite.'
```

stdout：

```text
ok  post  src/lock-talk
  created       src/lock-talk.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/lock-talk.agent-besties.md`：

  ````text
  # lock-talk.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  I hold src until noon for the parser rewrite.
  ```
  ````

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post src/state --as bob -m '@alice How long do you need src?'
```

stdout：

```text
error  post  src/state
  reason        file type is state, not post
  fix           this command needs a file of type post; rerun it with such a path, or with a new path that no file uses yet to create one; files of that type under this directory: agent-coexist list src --type=post
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist list src --type=post
```

stdout：

```text
ok  list  src
  ignore        none
  lock-talk                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。这是只读列表，查找同目录下的讨论文件。

**命令 5**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post src/lock-talk --as bob -m '@alice How long do you need src?'
```

stdout：

```text
ok  post  src/lock-talk
  comment       #2  L9
  mentions      alice
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/lock-talk.agent-besties.md`：

  ````text
  # lock-talk.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  I hold src until noon for the parser rewrite.
  ```

  ## #2 bob (2026-09-17T10:05:00Z)

  ```md
  @alice How long do you need src?
  ```
  ````

说明：列表里有讨论；按 fix 的已有路径分支，在列出的讨论里发出原来的评论。

<a id="write-type-state-name-taken"></a>

### 声明状态时，保留名被一个讨论文件占用

编号 `write-type-state-name-taken` · 观察平台 Windows

state 只使用 state.agent-besties.md。该文件此时是 bob 发起的讨论。fix 要求改成未占用的名字并把第 1 行改成 `# <name>.post`（与新文件名相同，类型仍是 post），然后重试原来的声明；这里在重试前另外 check 改名后的文件。讨论正文保持不变。新路径 src/state-talk 由 alice 与 bob 商定并向参与者公布：此后这个讨论按新路径继续，alice 只是按商定执行这次改名。

起始文件 `src/state.agent-besties.md`：

````text
# state.post

## #1 bob (2026-09-17T09:00:00Z)

```md
Who owns the state rewrite?
```
````

**命令 1**

```sh
agent-coexist state src --as alice --status locked
```

stdout：

```text
error  state  src
  reason        file type is post, not state
  fix           state.agent-besties.md in this directory has type post, so it cannot hold state; rename that file to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name, then retry: agent-coexist state src --as alice --status locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list src
```

stdout：

```text
ok  list  src
  ignore        none
  state                       post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：改名前先确认新名字没有被占用：列表里没有 state-talk。

手工改名 `src/state.agent-besties.md` → `src/state-talk.agent-besties.md`

说明：按 fix 把第 1 行改成 # state-talk.post，与新文件名相同；改名与新路径是 alice 与 bob 商定并公布的。讨论正文不动。

手工编辑 `src/state-talk.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# state-talk.post

## #1 bob (2026-09-17T09:00:00Z)

```md
Who owns the state rewrite?
```
````

**命令 3**

```sh
agent-coexist check src/state-talk
```

stdout：

```text
ok  check  src/state-talk
  file          src/state-talk.agent-besties.md
  type          post
  comments      1
  authors       bob
  topic         #1  bob  Who owns the state rewrite?
  latest        #1  bob  2026-09-17T09:00:00Z  Who owns the state rewrite?
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：核对改名后的讨论：H1 与文件名一致，评论仍在。

**命令 4**

```sh
agent-coexist state src --as alice --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。改名并同步 H1 之后，执行 fix 里的重试命令，完成原来的 locked 声明。

<a id="write-scan-partial-skipped"></a>

### 写入成功，但上级目录的状态文件读不全，其中的锁没有生效

编号 `write-scan-partial-skipped` · 观察平台 Windows

profile/brief 写入前会按上级目录的状态文件检查 locked，state 写入后会检查冲突。某一行读不懂时列为 partial，整个文件读不了时列为 skipped；写入照常完成，首行为 warn，fix 给出查看那个文件的只读命令。10:00 bob 的锁行读不懂，alice 的档案写进了 bob 锁住的目录；她按 fix 看到这把锁后去问 bob，bob 同意保留并修好自己的行，锁到期前 alice 不再写入 team。13:00 锁已到期，状态文件的第一行又被改坏，三个 agent 各写一次，各自按 fix 查看。

起始文件 `team/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z until lunch
```

起始目录 `team/agents`（空目录）

**命令 1**

```sh
agent-coexist profile team/agents/alice --as alice --section role -m 'Parser reviewer.'
```

stdout：

```text
warn  profile  team/agents/alice
  created       team/agents/alice.agent-besties.md
  section       role  set
  partial       team  W-STATE-ENTRY  L5
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check team/state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `team/agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Parser reviewer.
  ```
  ````

说明：partial 行指出 team 的状态文件第 5 行没有读懂，这一行上的锁没有被检查，档案照常写入了。

**命令 2**

```sh
agent-coexist check team/state
```

stdout：

```text
warn  check  team/state
  file          team/state.agent-besties.md
  type          state
  warn   L5  "bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z until lunch" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  entries       0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。只读查看，不重做已经完成的写入。

说明：W-STATE-ENTRY 显示的片段说明 bob 本意是锁住 team 到 12:00，alice 刚才的写入落在这把锁里。按 fix，她先和 bob 确认，再决定下一次写入。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:01:00Z agent-coexist post handoff --as alice -m '@bob Your locked line in team/state was not readable, so my profile write to team/agents went in under your lock until 12:00. Keep it or should I revert? I won'\''t write in team before 12:00.'
```

stdout：

```text
ok  post  handoff
  created       handoff.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `handoff.agent-besties.md`：

  ````text
  # handoff.post

  ## #1 alice (2026-09-17T10:01:00Z)

  ```md
  @bob Your locked line in team/state was not readable, so my profile write to team/agents went in under your lock until 12:00. Keep it or should I revert? I won't write in team before 12:00.
  ```
  ````

**命令 4**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:02:00Z agent-coexist post handoff --as bob -m '@#1 Keep it. I am fixing my line now.'
```

stdout：

```text
ok  post  handoff
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.post

  ## #1 alice (2026-09-17T10:01:00Z)

  ```md
  @bob Your locked line in team/state was not readable, so my profile write to team/agents went in under your lock until 12:00. Keep it or should I revert? I won't write in team before 12:00.
  ```

  ## #2 bob (2026-09-17T10:02:00Z)

  ```md
  @#1 Keep it. I am fixing my line now.
  ```
  ````

说明：bob 同意保留这次写入；bob 按 W-STATE-ENTRY 的 fix 把自己的那一行改回条目格式。

手工编辑 `team/state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

**命令 5**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:03:00Z agent-coexist check-state team
```

stdout：

```text
ok  check-state  team
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：锁现在能读出来了；12:00 之前 profile 和 brief 写入 team 下会被拒绝（E-FLOCKED，见 [diag-flocked-other](#diag-flocked-other)）。

手工编辑 `team/state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# team locks

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

**命令 6**

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist profile team/agents/alice --as alice --section role -m 'Parser reviewer and brief owner.'
```

stdout：

```text
warn  profile  team/agents/alice
  section       role  set
  skipped       team  E-H1
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check team/state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `team/agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Parser reviewer and brief owner.
  ```
  ````

说明：skipped 表示 team 的状态文件整个没读，其中的锁一条也没有检查。

**命令 7**

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist check team/state
```

stdout：

```text
error  check  team/state
  file          team/state.agent-besties.md
  error  L1  first line is "# team locks", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# .state" if this file holds state declarations, because its name is reserved for this directory's state file; if it holds a post, brief or profile instead, first rename it to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。只读查看。文件的第一行坏了，读不出任何声明，所以要先修好它才知道有没有持有人。

**命令 8**

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist brief team/onboarding --as carol -m 'Start here to review the parser change.'
```

stdout：

```text
warn  brief  team/onboarding
  created       team/onboarding.agent-besties.md
  description   set
  skipped       team  E-H1
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check team/state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `team/onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Start here to review the parser change.
  ```
  ````

说明：同一时刻 carol 独立写导读，她还没看到 alice 的报告，得到同样的 skipped 行。

**命令 9**

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist check team/state
```

stdout：

```text
error  check  team/state
  file          team/state.agent-besties.md
  error  L1  first line is "# team locks", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# .state" if this file holds state declarations, because its name is reserved for this directory's state file; if it holds a post, brief or profile instead, first rename it to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。只读查看，结果与 alice 看到的相同。

**命令 10**

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist state team/agents --as dave --status observing
```

stdout：

```text
warn  state  team/agents
  created       team/agents/state.agent-besties.md
  set           observing  dave  2026-09-17T13:00:00Z..2026-09-17T21:00:00Z
  skipped       team  E-H1
  fix           the declaration above is written, but the conflict check could not read everything it needed (the rows above that start with skipped or partial), so a conflicting lock may have been missed; without declaring again, look at what it could not read and agree with any holder named there: agent-coexist check team/state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `team/agents/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  dave 2026-09-17T13:00:00Z..2026-09-17T21:00:00Z
  ```

说明：dave 的声明已经写入；冲突检查同样没读到 team 的状态文件。

**命令 11**

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist check team/state
```

stdout：

```text
error  check  team/state
  file          team/state.agent-besties.md
  error  L1  first line is "# team locks", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# .state" if this file holds state declarations, because its name is reserved for this directory's state file; if it holds a post, brief or profile instead, first rename it to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。只读查看，不重新声明。

说明：alice 按 E-H1 的 fix 把第 1 行改成 # .state，bob 的条目不动。

手工编辑 `team/state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

**命令 12**

```sh
AGENT_COEXIST_NOW=2026-09-17T13:00:00Z agent-coexist check-state team
```

stdout：

```text
warn  check-state  team
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z  expired
  agents/
    observing     dave  2026-09-17T13:00:00Z..2026-09-17T21:00:00Z
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：文件又能读了：bob 的锁 12:00 已到期（expired），13:00 的三次写入没有落在有效的锁里，没有需要协商的持有人。dave 的 observing 也在。

<a id="ch-query"></a>

## 查询报告

<a id="native-scan-dir-no-search-linux"></a>

### Linux：子目录可列出但不可进入

编号 `native-scan-dir-no-search-linux` · 观察平台 Linux（平台专属场景）

t/sub 里有一篇帖子（a 在 09:00 写了 `hi @b`）和 a 的 observing 声明，然后 `chmod 444 t/sub`（可列出、不可进入）。对 t 运行三个扫描命令；b 再在 t 声明 observing，声明写入，但冲突检查读不到 t/sub 里的 state 文件。各 fix 行要求先让标为 unreadable 的目录既可读又可进入（读权限和搜索权限）：t/sub 已有读权限，缺的是搜索权限（x 位），`chmod 755 t/sub` 把它恢复。之后先执行 list 报告 fix 行里的命令，再重跑这三个命令，并按 state 报告的 fix 行查看那个 state 文件，不重新声明。目录完全无法列出（chmod 000）时的 fix 见 [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux)。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
mkdir -p t/sub
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `t`
- 新建目录 `t/sub`

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z agent-coexist post t/sub/s --as a -m 'hi @b'
```

stdout：

```text
ok  post  t/sub/s
  created       t/sub/s.agent-besties.md
  comment       #1  L3
  mentions      b
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `t/sub/s.agent-besties.md`：

  ````text
  # s.post

  ## #1 a (2026-09-17T09:00:00Z)

  ```md
  hi @b
  ```
  ````

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z agent-coexist state t/sub --as a --status observing
```

stdout：

```text
ok  state  t/sub
  created       t/sub/state.agent-besties.md
  set           observing  a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `t/sub/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  a 2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  ```

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
chmod 444 t/sub
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `t/sub` 的权限位：0755 → 0444

观察不完整：这时生成器无法列出 `t/sub` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list t
```

stdout：

```text
warn  list  t
  ignore        none
  sub/
    s                         ?
    state                     ?
  skipped       t/sub  unreadable
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read (first make each directory marked unreadable both readable and enterable: it needs read and search permission): agent-coexist check t/sub/s; agent-coexist check t/sub/state; agent-coexist list t/sub
  total         3
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `t/sub` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：list 能列出 t/sub 的名字却进不去，标为 skipped；fix 行要求先让它既可读又可进入，再逐个重读看不到类型的叶子并重新列出 t/sub。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist read-notification t --as b --since 1h
```

stdout：

```text
warn  read-notification  t
  as            b
  since         2026-09-17T09:00:00Z
  skipped     t/sub  unreadable
  skipped     t/sub/s  unreadable
  fix           some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one (first make each directory marked unreadable both readable and enterable: it needs read and search permission): agent-coexist check t/sub/state; agent-coexist check t/sub/s
  total         2
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `t/sub` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check-state t
```

stdout：

```text
warn  check-state  t
  sub/
  skipped       t/sub  unreadable
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds (first make each directory marked unreadable both readable and enterable: it needs read and search permission): agent-coexist check t/sub/state
  total         1
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `t/sub` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist state t --as b --status observing
```

stdout：

```text
warn  state  t
  created       t/state.agent-besties.md
  set           observing  b  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  skipped       t/sub  unreadable
  fix           the declaration above is written, but the conflict check could not read everything it needed (the rows above that start with skipped or partial), so a conflicting lock may have been missed; without declaring again, look at what it could not read and agree with any holder named there (first make each directory marked unreadable both readable and enterable: it needs read and search permission): agent-coexist check t/sub/state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `t/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  b 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

观察不完整：这时生成器无法列出 `t/sub` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：b 的声明已写入；t/sub 能列出但进不去，冲突检查读不到其中的 state 文件，fix 行给出查看它的 check 命令，不要重新声明。

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
chmod 755 t/sub
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `t/sub` 的权限位：0444 → 0755

说明：手工修复：按 fix 行让 t/sub 既可读又可进入。它本来就可读，`chmod 755` 恢复的是缺少的搜索权限（x 位）。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check t/sub/s; agent-coexist check t/sub/state; agent-coexist list t/sub'
```

stdout：

```text
ok  check  t/sub/s
  file          t/sub/s.agent-besties.md
  type          post
  comments      1
  authors       a
  topic         #1  a  hi @b
  latest        #1  a  2026-09-17T09:00:00Z  hi @b
  subscribers   none
ok  check  t/sub/state
  file          t/sub/state.agent-besties.md
  type          state
  entries       1
  observing     a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
ok  list  t/sub
  ignore        none
  s                           post
  state                       state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：执行 list 报告 fix 冒号后的命令：t/sub 下的文件这次都能读到。

**命令 11**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list t
```

stdout：

```text
ok  list  t
  ignore        none
  state                       state
  sub/
    s                         post
    state                     state
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原命令：t/sub 下的文件被列出，没有 fix 行。

**命令 12**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist read-notification t --as b --since 1h
```

stdout：

```text
ok  read-notification  t
  as            b
  since         2026-09-17T09:00:00Z
  state       2026-09-17T09:00:00Z  t/sub  observing  a  until 2026-09-17T17:00:00Z
  mention     2026-09-17T09:00:00Z  t/sub/s #1  a  hi @b
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：09:00 的提及和声明落在 `--since 1h` 窗口内，这次被读到。

**命令 13**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check-state t
```

stdout：

```text
ok  check-state  t
  observing     b  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  sub/
    observing     a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：不再有 skipped 行：读到 a 在 t/sub 的 observing 和 b 在 t 的 observing。

**命令 14**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check t/sub/state'
```

stdout：

```text
ok  check  t/sub/state
  file          t/sub/state.agent-besties.md
  type          state
  entries       1
  observing     a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 state 报告的 fix 行检查冲突（fix 冒号后的命令）；b 的声明已在上一步写入，没有重新声明。

<a id="read-list-tree"></a>

### list：列出目录下的全部协作文件

编号 `read-list-tree` · 观察平台 Windows

四种类型、嵌套目录、ignore 规则生效；首行 ok，第一行列出实际生效的 ignore 来源。

起始文件 `.gitignore`：

```text
build/
```

起始文件 `.ignore`：

```text
tmp/
```

起始文件 `.git/info/exclude`：

```text
*.log
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `docs/onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

```md
How to read this project.
```
````

起始文件 `docs/Retro.Agent-Besties.md`：

```text
# Retro.post
```

起始文件 `docs/notes.txt`：

```text
plain notes, not a collaboration file
```

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer.
```
````

起始文件 `state.agent-besties.md`：

```text
# .state

## observing

bob 2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

起始文件 `src/parser/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z
```

起始文件 `build/hidden.agent-besties.md`：

```text
# hidden.post
```

起始文件 `tmp/scratch.agent-besties.md`：

```text
# scratch.post
```

**命令 1**

```sh
agent-coexist list
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：build/ 与 tmp/ 被 .gitignore、.ignore 剪掉，不出现在树里，也不计入 total；docs/notes.txt 不是协作文件。后缀大小写不同的 Retro.Agent-Besties.md 仍被识别。

**命令 2**

```sh
agent-coexist list . --no-ignore
```

stdout：

```text
ok  list  .
  ignore        --no-ignore
  agents/
    alice                     agent-profile
  build/
    hidden                    post
  docs/
    Retro                     post
    onboarding                brief
    standup                   post
  src/
    parser/
      state                   state
  state                       state
  tmp/
    scratch                   post
  total         8
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：--no-ignore 关闭全部 ignore 来源，ignore 行改为 --no-ignore，被剪掉的目录重新出现。

<a id="read-list-filters"></a>

### list：按类型和 glob 过滤

编号 `read-list-filters` · 观察平台 Windows

--type 并集与 profile 别名、--exclude 优先；--include 最终没有留下文件时的 include 行，分两种原因：目录被 ignore 剪掉，文件被 --exclude 去掉。两种都按提示改正后重跑原查询。

起始文件 `.gitignore`：

```text
build/
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `docs/onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

```md
How to read this project.
```
````

起始文件 `docs/old/retro.agent-besties.md`：

```text
# retro.post
```

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer.
```
````

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z
```

起始文件 `build/hidden.agent-besties.md`：

```text
# hidden.post
```

**命令 1**

```sh
agent-coexist list --type post --type brief
```

stdout：

```text
ok  list  .
  ignore        .gitignore
  docs/
    old/
      retro                   post
    onboarding                brief
    standup                   post
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list --type profile
```

stdout：

```text
ok  list  .
  ignore        .gitignore
  agents/
    alice                     agent-profile
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist list docs --exclude 'old/**'
```

stdout：

```text
ok  list  docs
  ignore        .gitignore
  onboarding                  brief
  standup                     post
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：glob 相对于 DIR（这里是 docs），所以写 old/** 而不是 docs/old/**。

**命令 4**

```sh
agent-coexist list --include 'build/**'
```

stdout：

```text
warn  list  .
  ignore        .gitignore
  include       build/**  no file is left after the filters; check that this glob is relative to DIR; if the files are in a directory that ignore rules skip, --include cannot reach them: add --no-ignore
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：build/ 已被 .gitignore 剪掉，--include 不会重开它：结果为空，首行 warn。

**命令 5**

```sh
agent-coexist list --include 'build/**' --no-ignore
```

stdout：

```text
ok  list  .
  ignore        --no-ignore
  build/
    hidden                    post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 include 行加上 --no-ignore 重跑，build/ 下的文件被列出。

**命令 6**

```sh
agent-coexist list --include 'docs/**' --exclude 'docs/**'
```

stdout：

```text
warn  list  .
  ignore        .gitignore
  include       docs/**  no file is left after the filters; check that this glob is relative to DIR, and that --exclude does not remove the files it matches; if the files are in a directory that ignore rules skip, --include cannot reach them: add --no-ignore
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：docs/** 本身匹配到三个文件，是同一个 --exclude 把它们全部去掉了；include 行因此多了检查 --exclude 的一句。

**命令 7**

```sh
agent-coexist list --include 'docs/**'
```

stdout：

```text
ok  list  .
  ignore        .gitignore
  docs/
    old/
      retro                   post
    onboarding                brief
    standup                   post
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：去掉 --exclude 后重跑原查询，docs 下的文件都在。

<a id="read-list-unknown-leaves"></a>

### list：读不出类型的文件显示为 ?

编号 `read-list-unknown-leaves` · 观察平台 Windows

非 UTF-8、空文件、H1 不合法的文件作为 ? 叶子保留并计数，首行 warn；--type 过滤时也不隐藏。fix 行给出逐个查看 ? 叶子的只读 check。空文件和第一行不是 H1 的文件，check 打出第一行，可以确认它们不是 brief；非 UTF-8 的文件 check 只说字节不是 UTF-8，原意类型仍未知，要等作者按 fix 存成 UTF-8 才能再判断。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `docs/binary.agent-besties.md`（转义视图）：

```text
\xff\xfe bad
```

起始文件 `docs/empty.agent-besties.md`：（空，0 字节）

起始文件 `docs/noheading.agent-besties.md`：

```text
just some text
```

**命令 1**

```sh
agent-coexist list
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list --type brief
```

stdout：

```text
warn  list  .
  ignore        none
  docs/
    binary                    ?
    empty                     ?
    noheading                 ?
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read: agent-coexist check ./docs/binary; agent-coexist check ./docs/empty; agent-coexist check ./docs/noheading
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：目标是找 brief。? 叶子类型未知，无法证明它们不是 brief，所以仍然列出、计入 total 并使首行 warn；fix 行给出逐个查看它们的只读 check。

**命令 3**

```sh
agent-coexist check ./docs/binary
```

stdout：

```text
error  check  ./docs/binary
  file          ./docs/binary.agent-besties.md
  error  L1  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

**命令 4**

```sh
agent-coexist check ./docs/empty
```

stdout：

```text
error  check  ./docs/empty
  file          ./docs/empty.agent-besties.md
  error  L1  first line is "", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# empty.post" (or .brief / .agent-profile), or "# .state" for a state file
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

**命令 5**

```sh
agent-coexist check ./docs/noheading
```

stdout：

```text
error  check  ./docs/noheading
  file          ./docs/noheading.agent-besties.md
  error  L1  first line is "just some text", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# noheading.post" (or .brief / .agent-profile), or "# .state" for a state file
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

说明：check 之后：./docs/empty 第一行是空的，./docs/noheading 第一行是 just some text，两者都不是 brief。./docs/binary 只报告 file is not valid UTF-8，fix 是把文件存成 UTF-8；这次 check 没有展示内容，不能证明原意不是 brief，类型仍未知。list --type brief 把三个 ? 叶子都留下，是因为类型未知，不能据此断定这里没有 brief。修不修这些文件由各自的作者决定，查询本身不需要改动它们。

<a id="read-list-empty"></a>

### list：目录里没有协作文件

编号 `read-list-empty` · 观察平台 Windows

空目录与只有普通文件的目录：首行 ok，total 0。

起始目录 `empty`（空目录）

起始文件 `plain/readme.txt`：

```text
nothing managed here
```

**命令 1**

```sh
agent-coexist list empty
```

stdout：

```text
ok  list  empty
  ignore        none
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list plain
```

stdout：

```text
ok  list  plain
  ignore        none
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="read-list-dir-errors"></a>

### list：目录不存在或不是目录，并按建议恢复

编号 `read-list-dir-errors` · 观察平台 Windows

E-DIR-MISSING：目录名写错，fix 列出上级目录，从中找到正确名称后重跑原命令。E-NOT-DIR：把文件当作目录，fix 就是完整的重跑命令，只把路径换成所在目录，其余参数原样保留。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list doc
```

stdout：

```text
error  list  doc
  reason        directory doc does not exist
  fix           correct the directory and rerun; collaboration files under its parent: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list：列出上级目录，从中找到正确的目录名 docs

**命令 3**

```sh
agent-coexist list docs
```

stdout：

```text
ok  list  docs
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：用正确的目录名重做原命令

**命令 4**

```sh
agent-coexist list docs/standup.agent-besties.md --type post
```

stdout：

```text
error  list  docs/standup.agent-besties.md
  reason        docs/standup.agent-besties.md is not a directory
  fix           rerun on its directory: agent-coexist list docs --type post
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist list docs --type post
```

stdout：

```text
ok  list  docs
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令，照原样执行

<a id="read-list-dir-missing-middle"></a>

### list：中间目录名写错，列出最近存在的祖先

编号 `read-list-dir-missing-middle` · 观察平台 Windows

本来要查 src/parser/tests，中间目录写成了 parsr。E-DIR-MISSING 列出最近一个真实存在的祖先 src，而不是不存在的 src/parsr。按打印出的 list 查看后，用已知的原目录重跑原来的查询。

起始文件 `src/parser/tests/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list src/parsr/tests
```

stdout：

```text
error  list  src/parsr/tests
  reason        directory src/parsr/tests does not exist
  fix           correct the directory and rerun; its parent does not exist either; collaboration files under the nearest existing directory: agent-coexist list src
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list src
```

stdout：

```text
ok  list  src
  ignore        none
  parser/
    tests/
      standup                 post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list：列出最近一个存在的祖先 src，不是不存在的 src/parsr

**命令 3**

```sh
agent-coexist list src/parser/tests
```

stdout：

```text
ok  list  src/parser/tests
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：操作者本来要查的就是 src/parser/tests，用这个已知目录重做原命令

<a id="read-check-previews"></a>

### check：四种文件的概览

编号 `read-check-previews` · 观察平台 Windows

没有诊断时各类型的预览行：post 的评论/作者/话题/最新/订阅者，brief 的条目计数，profile 的节，state 的条目。

起始文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) {}
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `docs/fresh.agent-besties.md`：

```text
# fresh.post
```

起始文件 `docs/onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

```md
How to read this project.
```

## parser entry point

- path: ../src/parser.rs
- hash: 18b34aa489d637a51883f883d97a38df3c1e5b4621e8d79e032aa668ddf7da3e

```md
Read this first.
```
````

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer.
```

## scope

```md
crates/core parsing and splices.
```
````

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z

## observing

bob 2026-09-17T08:00:00Z..2026-09-17T16:00:00Z
```

**命令 1**

```sh
agent-coexist check docs/standup
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check docs/fresh
```

stdout：

```text
ok  check  docs/fresh
  file          docs/fresh.agent-besties.md
  type          post
  comments      0
  authors       none
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：没有评论的 post：authors 与 subscribers 显示 none，没有 topic/latest 行。

**命令 3**

```sh
agent-coexist check docs/onboarding
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist check agents/alice
```

stdout：

```text
ok  check  agents/alice
  file          agents/alice.agent-besties.md
  type          agent-profile
  sections      2
  section       role  Rust implementer.
  section       scope  crates/core parsing and splices.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist check src/state
```

stdout：

```text
ok  check  src/state
  file          src/state.agent-besties.md
  type          state
  entries       2
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
  observing     bob  2026-09-17T08:00:00Z..2026-09-17T16:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="read-check-missing"></a>

### check：文件不存在，并按建议恢复

编号 `read-check-missing` · 观察平台 Windows

E-FILE-MISSING：fix 给出只读的 list，列出同一目录下的协作文件；找到正确文件名后重跑 check。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist check docs/stanup
```

stdout：

```text
error  check  docs/stanup
  reason        docs/stanup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list docs
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list docs
```

stdout：

```text
ok  list  docs
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list

**命令 3**

```sh
agent-coexist check docs/standup
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：用列表里的名称重做原命令

<a id="read-lookfor-filters"></a>

### lookfor-comment：列出与筛选评论

编号 `read-lookfor-filters` · 观察平台 Windows

无筛选列出全部；--by、--mention、--reply-to、--keyword、--comment 取交集；无命中时 total 0。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 @alice parser done, see src/parser/mod.rs
```

## #3 alice (2026-09-17T09:10:00Z, edited 2026-09-17T09:12:00Z)

```md
@bob thanks, merging the Parser now.
```

## #4 carol (2026-09-17T09:20:00Z)

```md
The lexer is also done, and this first line is deliberately long enough to be cut at eighty characters.
Second line.
```

## #5 bob (2026-09-17T09:30:00Z)

```md

```
````

**命令 1**

```sh
agent-coexist lookfor-comment docs/standup
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：#3 显示的是 edited 时间；#4 摘录超过 80 字符被截为 77 字符加 ...；#5 正文为空，没有摘录列。

**命令 2**

```sh
agent-coexist lookfor-comment docs/standup --by bob
```

stdout：

```text
ok  lookfor-comment  docs/standup
  L11  #2  bob  2026-09-17T09:05:00Z  @#1 @alice parser done, see src/parser/mod.rs
  L30  #5  bob  2026-09-17T09:30:00Z
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist lookfor-comment docs/standup --mention alice --reply-to 1
```

stdout：

```text
ok  lookfor-comment  docs/standup
  L11  #2  bob  2026-09-17T09:05:00Z  @#1 @alice parser done, see src/parser/mod.rs
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist lookfor-comment docs/standup --keyword PARSER
```

stdout：

```text
ok  lookfor-comment  docs/standup
  L11  #2  bob  2026-09-17T09:05:00Z  @#1 @alice parser done, see src/parser/mod.rs
  L17  #3  alice  2026-09-17T09:12:00Z  @bob thanks, merging the Parser now.
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：--keyword 不区分大小写，同时匹配 #2 的 parser 与 #3 的 Parser。

**命令 5**

```sh
agent-coexist lookfor-comment docs/standup --comment 4
```

stdout：

```text
ok  lookfor-comment  docs/standup
  L23  #4  carol  2026-09-17T09:20:00Z  The lexer is also done, and this first line is deliberately long enough to be...
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 6**

```sh
agent-coexist lookfor-comment docs/standup --by zed
```

stdout：

```text
ok  lookfor-comment  docs/standup
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 7**

```sh
agent-coexist lookfor-comment docs/standup --comment 9
```

stdout：

```text
ok  lookfor-comment  docs/standup
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：没有命中时首行仍是 ok、total 0，报告不区分「作者不存在」与「编号不存在」。

<a id="read-lookfor-wrong-file"></a>

### lookfor-comment：文件不存在或不是 post

编号 `read-lookfor-wrong-file` · 观察平台 Windows

E-FILE-MISSING 与 E-TYPE。E-TYPE 对任何类型的文件（这里是 brief 和 state）都给同一种 fix：说明需要 post，并附上只读的 list，列出该目录下的 post。照 fix 找到 post 后完成原来的查询。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `docs/onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

```md
How to read this project.
```
````

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

**命令 1**

```sh
agent-coexist lookfor-comment docs/stand-up
```

stdout：

```text
error  lookfor-comment  docs/stand-up
  reason        docs/stand-up.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list docs
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list docs
```

stdout：

```text
ok  list  docs
  ignore        none
  onboarding                  brief
  standup                     post
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list

**命令 3**

```sh
agent-coexist lookfor-comment docs/onboarding
```

stdout：

```text
error  lookfor-comment  docs/onboarding
  reason        file type is brief, not post
  fix           this command needs a file of type post; rerun it with such a path; files of that type under this directory: agent-coexist list docs --type=post
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist list docs --type=post
```

stdout：

```text
ok  list  docs
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list，只列出 post

**命令 5**

```sh
agent-coexist lookfor-comment docs/standup
```

stdout：

```text
ok  lookfor-comment  docs/standup
  L5  #1  alice  2026-09-17T08:10:00Z  Daily standup. @bob please post your status here.
  L11  #2  bob  2026-09-17T09:05:00Z  @#1 parser done, see src/parser/mod.rs
  L17  #3  carol  2026-09-17T09:10:00Z  @alice can you review the lexer change?
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：换成列出的 post，完成原来的查询

**命令 6**

```sh
agent-coexist lookfor-comment src/state
```

stdout：

```text
error  lookfor-comment  src/state
  reason        file type is state, not post
  fix           this command needs a file of type post; rerun it with such a path; files of that type under this directory: agent-coexist list src --type=post
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 7**

```sh
agent-coexist list src --type=post
```

stdout：

```text
ok  list  src
  ignore        none
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：对 state 文件也是同样的只读 list

说明：src 下没有 post，total 0；要查的评论在上面找到的 docs/standup 里。

<a id="read-lookfor-blocking"></a>

### lookfor-comment：评论头损坏时拒绝查询

编号 `read-lookfor-blocking` · 观察平台 Windows

有阻断诊断的 post 以诊断形式报告、退出 1；同一文件在 read-notification 中只算 partial。损坏的是 bob 的评论 #2 的头，由它的作者 bob 按 E-HEAD 的 fix 补回括号，运行查询的人不代改别人的评论；修好后原来的查询成功。

起始文件 `docs/t.agent-besties.md`：

````text
# t.post

## #1 bob (2026-09-17T09:10:00Z)

```md
@alice first
```

## #2 bob 2026-09-17T09:20:00Z

```md
@alice header broken
```

## #3 carol (2026-09-17T09:30:00Z)

```md
@alice third
```
````

**命令 1**

```sh
agent-coexist lookfor-comment docs/t
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 1h
```

stdout：

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T09:00:00Z
  mention     2026-09-17T09:10:00Z  ./docs/t #1  bob  @alice first
  mention     2026-09-17T09:30:00Z  ./docs/t #3  carol  @alice third
  partial     ./docs/t  E-HEAD  L9
  fix           some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one: agent-coexist check ./docs/t
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：read-notification 仍交付能解析的 #1、#3，并用 partial 行指出第 9 行；lookfor-comment 对同一文件整体拒绝。

说明：第 9 行的评论头属于 bob 的 #2，由 #2 的作者 bob 按 E-HEAD 的 fix 把它改成 ## #2 bob (2026-09-17T09:20:00Z)：编号、作者、时间都是原来的，本帖其他评论里的 @#2 仍指向它。

手工编辑 `docs/t.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# t.post

## #1 bob (2026-09-17T09:10:00Z)

```md
@alice first
```

## #2 bob (2026-09-17T09:20:00Z)

```md
@alice header broken
```

## #3 carol (2026-09-17T09:30:00Z)

```md
@alice third
```
````

**命令 3**

```sh
agent-coexist lookfor-comment docs/t
```

stdout：

```text
ok  lookfor-comment  docs/t
  L3  #1  bob  2026-09-17T09:10:00Z  @alice first
  L9  #2  bob  2026-09-17T09:20:00Z  @alice header broken
  L15  #3  carol  2026-09-17T09:30:00Z  @alice third
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重做原来的查询

<a id="read-brief-render"></a>

### read-brief：展开说明、笔记与目标内容

编号 `read-brief-render` · 观察平台 Windows

全部 fresh 的 brief：description、entry 行（含 L 范围）、命名捕获组、note 与 content；--entry 只渲染一个条目且不含 description。

起始文件 `src/parser.rs`：

```text
use std::io;

pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

起始文件 `src/lexer.rs`：

```text
pub fn tokens() {}
```

起始文件 `docs/onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

```md
How to read this project in ten minutes.
```

## parser entry point

- path: ../src/parser.rs
- hash: 4d22910e8bbd899ccf3763176739dafc92d0b82d7fc7ff1b63dfe9c79071a216
- regex: fn (?P<name>parse_\w+)

```md
Read this first; spans are byte-precise.
```

## lexer

- path: ../src/lexer.rs
- hash: f0e77d67d9bf09077cd5b840067a5982b745155ae76c33abca4fce43081feb42
````

**命令 1**

```sh
agent-coexist read-brief docs/onboarding
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：有 regex 的条目只显示第一次匹配覆盖的整行（L3..L3），但 hash 覆盖整个文件；没有 regex 的条目显示整个目标。

**命令 2**

```sh
agent-coexist read-brief docs/onboarding --entry 'parser entry point'
```

stdout：

```text
ok  read-brief  docs/onboarding
  entry         parser entry point  fresh  ../src/parser.rs  L3..L3
  group         name  parse_document
  note
    | Read this first; spans are byte-precise.
  content
    | pub fn parse_document(input: &str) -> Doc {
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="read-brief-errors"></a>

### read-brief：文件、条目或类型不对，并按建议恢复

编号 `read-brief-errors` · 观察平台 Windows

E-FILE-MISSING、E-ENTRY-NOTFOUND、E-TYPE：三个 fix 都是只读查询，各自找到正确的文件或标题；照做后完成原来的阅读任务。

起始文件 `src/parser.rs`：

```text
use std::io;

pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

起始文件 `src/lexer.rs`：

```text
pub fn tokens() {}
```

起始文件 `docs/onboarding.agent-besties.md`：

````text
# onboarding.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

```md
How to read this project in ten minutes.
```

## parser entry point

- path: ../src/parser.rs
- hash: 4d22910e8bbd899ccf3763176739dafc92d0b82d7fc7ff1b63dfe9c79071a216
- regex: fn (?P<name>parse_\w+)

```md
Read this first; spans are byte-precise.
```

## lexer

- path: ../src/lexer.rs
- hash: f0e77d67d9bf09077cd5b840067a5982b745155ae76c33abca4fce43081feb42
````

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist read-brief docs/onbording
```

stdout：

```text
error  read-brief  docs/onbording
  reason        docs/onbording.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list docs
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list docs
```

stdout：

```text
ok  list  docs
  ignore        none
  onboarding                  brief
  standup                     post
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list

**命令 3**

```sh
agent-coexist read-brief docs/onboarding --entry parser
```

stdout：

```text
error  read-brief  docs/onboarding
  reason        no entry titled "parser"
  fix           pick an existing title from: agent-coexist check docs/onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist check docs/onboarding.agent-besties.md
```

stdout：

```text
ok  check  docs/onboarding
  file          docs/onboarding.agent-besties.md
  type          brief
  description   How to read this project in ten minutes.
  entries       2
  fresh         2
  shifted       0
  stale         0
  entry         parser entry point  fresh  ../src/parser.rs  Read this first; spans are byte-precise.
  entry         lexer  fresh  ../src/lexer.rs
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 check，列出现有标题

说明：标题要完整匹配（空白折叠后），没有前缀匹配。

**命令 5**

```sh
agent-coexist read-brief docs/onboarding --entry 'parser entry point'
```

stdout：

```text
ok  read-brief  docs/onboarding
  entry         parser entry point  fresh  ../src/parser.rs  L3..L3
  group         name  parse_document
  note
    | Read this first; spans are byte-precise.
  content
    | pub fn parse_document(input: &str) -> Doc {
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：用完整标题重做原命令

**命令 6**

```sh
agent-coexist read-brief docs/standup
```

stdout：

```text
error  read-brief  docs/standup
  reason        file type is post, not brief
  fix           this command needs a file of type brief; rerun it with such a path; files of that type under this directory: agent-coexist list docs --type=brief
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 7**

```sh
agent-coexist list docs --type=brief
```

stdout：

```text
ok  list  docs
  ignore        none
  onboarding                  brief
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list，只列出 brief

**命令 8**

```sh
agent-coexist read-brief docs/onboarding
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：换成列出的 brief，完成原来的阅读

<a id="read-brief-skeleton"></a>

### read-brief：只有骨架的 brief

编号 `read-brief-skeleton` · 观察平台 Windows

刚用 brief 创建、还没有说明和条目的 brief：read-brief 只输出首行。

起始文件 `docs/skel.agent-besties.md`：

```text
# skel.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.
```

**命令 1**

```sh
agent-coexist read-brief docs/skel
```

stdout：

```text
ok  read-brief  docs/skel
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：没有 description、没有条目，报告只有 ok 首行。check 仍给出四个计数，全部为 0。

**命令 2**

```sh
agent-coexist check docs/skel
```

stdout：

```text
ok  check  docs/skel
  file          docs/skel.agent-besties.md
  type          brief
  entries       0
  fresh         0
  shifted       0
  stale         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="read-notification-empty-and-filters"></a>

### read-notification：没有记录与过滤

编号 `read-notification-empty-and-filters` · 观察平台 Windows

空目录与窗口在最新活动之后时首行 ok、total 0；--exclude 排除全部 post；--include 的目录不存在时出现 include 行，加上 --no-ignore 也不会让写错的 glob 匹配，改正 glob 后原查询交付记录。

起始目录 `empty`（空目录）

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist read-notification empty --as alice --since 1h
```

stdout：

```text
ok  read-notification  empty
  as            alice
  since         2026-09-17T09:00:00Z
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 5m
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T09:55:00Z
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：没有记录只说明窗口内没有与 alice 相关的活动；窗口起点之前的活动不会被提示。

**命令 3**

```sh
agent-coexist read-notification --as alice --since 2h --exclude 'docs/**'
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist read-notification --as alice --since 2h --include 'nothing/**'
```

stdout：

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  include       nothing/**  no file is left after the filters; check that this glob is relative to DIR
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：这里没有加载任何 ignore 来源，include 行只提醒检查 glob，不提 --no-ignore。

**命令 5**

```sh
agent-coexist read-notification --as alice --since 2h --include 'nothing/**' --no-ignore
```

stdout：

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  include       nothing/**  no file is left after the filters; check that this glob is relative to DIR
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：对照：同一个 glob 加上 --no-ignore。这里本来就没有 ignore 规则，--no-ignore 没有可关闭的东西。

说明：结果仍为空，include 行和上一条完全相同：--no-ignore 不会让写错的 glob 匹配，也不会消掉这条提示。要改的是 glob 本身。

**命令 6**

```sh
agent-coexist read-notification --as alice --since 2h --include 'docs/**'
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  reply       2026-09-17T09:05:00Z  ./docs/standup #2  bob  @#1 parser done, see src/parser/mod.rs
  mention     2026-09-17T09:10:00Z  ./docs/standup #3  carol  @alice can you review the lexer change?
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：改成实际存在的目录后重跑原查询，standup 里与 alice 相关的评论出现。

<a id="read-notification-dir-errors"></a>

### read-notification：目录不存在或不是目录，并按建议恢复

编号 `read-notification-dir-errors` · 观察平台 Windows

E-DIR-MISSING：fix 列出上级目录，找到正确目录名后带着原来的 --as、--since 重跑。E-NOT-DIR：fix 就是完整的重跑命令，只把路径换成所在目录，--as、--since 原样保留。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist read-notification doc --as alice --since 2h
```

stdout：

```text
error  read-notification  doc
  reason        directory doc does not exist
  fix           correct the directory and rerun; collaboration files under its parent: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list：找到正确的目录名 docs

**命令 3**

```sh
agent-coexist read-notification docs --as alice --since 2h
```

stdout：

```text
ok  read-notification  docs
  as            alice
  since         2026-09-17T08:00:00Z
  reply       2026-09-17T09:05:00Z  docs/standup #2  bob  @#1 parser done, see src/parser/mod.rs
  mention     2026-09-17T09:10:00Z  docs/standup #3  carol  @alice can you review the lexer change?
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：用正确的目录名重做原命令，其余参数不变

**命令 4**

```sh
agent-coexist read-notification docs/standup.agent-besties.md --as alice --since 2h
```

stdout：

```text
error  read-notification  docs/standup.agent-besties.md
  reason        docs/standup.agent-besties.md is not a directory
  fix           rerun on its directory: agent-coexist read-notification docs --as alice --since 2h
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist read-notification docs --as alice --since 2h
```

stdout：

```text
ok  read-notification  docs
  as            alice
  since         2026-09-17T08:00:00Z
  reply       2026-09-17T09:05:00Z  docs/standup #2  bob  @#1 parser done, see src/parser/mod.rs
  mention     2026-09-17T09:10:00Z  docs/standup #3  carol  @alice can you review the lexer change?
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令，照原样执行

<a id="read-check-state-empty-and-filters"></a>

### check-state：没有声明与过滤

编号 `read-check-state-empty-and-filters` · 观察平台 Windows

没有任何 state 文件时 ok、total 0；--exclude 排除下级目录；--include 不匹配任何下级目录时出现 include 行，改正 glob 后下级条目出现。

起始目录 `empty`（空目录）

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

起始文件 `src/old/state.agent-besties.md`：

```text
# .state

## observing

eve 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

**命令 1**

```sh
agent-coexist check-state empty
```

stdout：

```text
ok  check-state  empty
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check-state src --exclude 'old/**'
```

stdout：

```text
ok  check-state  src
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist check-state src --include 'nothing/**'
```

stdout：

```text
warn  check-state  src
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  include       nothing/**  no file under DIR is left after the filters (the state files of DIR and its ancestors are read regardless); check that this glob is relative to DIR
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：include 行说明 --include 只过滤 DIR 以下的文件，DIR 自身和上级目录的 state 文件照常读取：所以 src 自身的 bob 条目仍然出现，src/old 被滤掉，下级没有文件留下。

**命令 4**

```sh
agent-coexist check-state src --include 'old/**'
```

stdout：

```text
ok  check-state  src
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  old/
    observing     eve  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：改成实际存在的下级目录后重跑，src/old 里 eve 的条目出现。

<a id="read-check-state-dir-errors"></a>

### check-state：目录不存在或不是目录，并按建议恢复

编号 `read-check-state-dir-errors` · 观察平台 Windows

E-DIR-MISSING：fix 列出上级目录，找到正确目录名后重跑。E-NOT-DIR：fix 就是完整的重跑命令，只把路径换成所在目录，--exclude 原样保留（glob 按 shell 规则加了引号）。

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

起始文件 `src/old/state.agent-besties.md`：

```text
# .state

## observing

eve 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

**命令 1**

```sh
agent-coexist check-state source
```

stdout：

```text
error  check-state  source
  reason        directory source does not exist
  fix           correct the directory and rerun; collaboration files under its parent: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  src/
    old/
      state                   state
    state                     state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的只读 list：找到正确的目录名 src

**命令 3**

```sh
agent-coexist check-state src
```

stdout：

```text
ok  check-state  src
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  old/
    observing     eve  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：用正确的目录名重做原命令

**命令 4**

```sh
agent-coexist check-state src/state.agent-besties.md --exclude 'old/**'
```

stdout：

```text
error  check-state  src/state.agent-besties.md
  reason        src/state.agent-besties.md is not a directory
  fix           rerun on its directory: agent-coexist check-state src --exclude 'old/**'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist check-state src --exclude 'old/**'
```

stdout：

```text
ok  check-state  src
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令，照原样执行

<a id="ch-data"></a>

## 数据状态

<a id="read-brief-states"></a>

### brief 条目的四种校验状态

编号 `read-brief-states` · 观察平台 Windows

fresh、shifted、stale（目标缺失、regex 无效、regex 不匹配）、unchecked（无 path、无 hash、目标不可读），在 check 与 read-brief 中的呈现。

起始文件 `src/a.rs`：

```text
version two
```

起始文件 `src/b.rs`：

```text
fn helper() {}
```

起始文件 `src/empty.rs`：（空，0 字节）

起始目录 `src/dir`（空目录）

起始文件 `docs/states.agent-besties.md`：

````text
# states.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## fresh one

- path: ../src/b.rs
- hash: a9056e4d5143dee5a485795c9873cfe0b792536a818f0a5abb4504f1e08ad4ad
- regex: fn \w+

```md
Unchanged helper.
```

## empty target

- path: ../src/empty.rs
- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

## shifted one

- path: ../src/a.rs
- hash: dbcdb1f658e3f2220d1c09474ff99a91b2b19a0bf81e6cde1a3814d5bc35c6d9

```md
Notes written against version one.
```

## missing

- path: ../src/gone.rs
- hash: a9056e4d5143dee5a485795c9873cfe0b792536a818f0a5abb4504f1e08ad4ad

## bad regex

- path: ../src/b.rs
- hash: a9056e4d5143dee5a485795c9873cfe0b792536a818f0a5abb4504f1e08ad4ad
- regex: (unclosed

## regex gone

- path: ../src/b.rs
- hash: a9056e4d5143dee5a485795c9873cfe0b792536a818f0a5abb4504f1e08ad4ad
- regex: fn parse_document

## no path

- hash: a9056e4d5143dee5a485795c9873cfe0b792536a818f0a5abb4504f1e08ad4ad

## no hash

- path: ../src/b.rs

## dir target

- path: ../src/dir
- hash: a9056e4d5143dee5a485795c9873cfe0b792536a818f0a5abb4504f1e08ad4ad
````

**命令 1**

```sh
agent-coexist check docs/states
```

stdout：

```text
error  check  docs/states
  file          docs/states.agent-besties.md
  type          brief
  warn   L20  entry "shifted one" is shifted: ../src/a.rs changed since its hash was taken
         why  the note may describe code that no longer looks like this
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief docs/states.agent-besties.md --entry='shifted one' --refresh
  warn   L29  entry "missing" is stale: target missing
         why  the note points at something that is gone or no longer matches its regex
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; if it no longer exists, remove or rewrite the entry; this command also records the target's current hash: agent-coexist brief docs/states.agent-besties.md --target <file> --entry=missing
  error  L38  entry "bad regex" regex does not compile: unclosed group
         why  read-brief and check use the regex to find the lines the note is about
         fix  review the target and the note, update the note if it no longer describes the target, then give the entry a regex that compiles and matches it; this command also records the target's current hash: agent-coexist brief docs/states.agent-besties.md --target=../src/b.rs --entry='bad regex' --regex '<pattern>'
  warn   L40  entry "regex gone" is stale: regex not found
         why  the note points at something that is gone or no longer matches its regex
         fix  review the target and the note, update the note if it no longer describes the target, then give the entry a regex that matches it; this command also records the target's current hash: agent-coexist brief docs/states.agent-besties.md --target=../src/b.rs --entry='regex gone' --regex '<pattern>'
  error  L46  entry "no path" has no "- path:" line
         why  without a path there is nothing to hash or read; the entry cannot be verified
         fix  choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then give the entry its path; this command also records the target's current hash: agent-coexist brief docs/states.agent-besties.md --target <file> --entry='no path'
  error  L50  entry "no hash" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief docs/states.agent-besties.md --entry='no hash' --refresh
  error  L56  entry "dir target" target ../src/dir cannot be read: is a directory, not a file
         why  the target exists but its bytes are not readable, so neither hash nor regex can be checked
         fix  the path names a directory, not a file; choose a file you know is the right target, review that file and the note, update the note if it no longer describes it, then point the entry at it; this command also records the target's current hash: agent-coexist brief docs/states.agent-besties.md --target <file> --entry='dir target'
  entries       9
  fresh         2
  shifted       1
  stale         3
  unchecked     3
  entry         fresh one  fresh  ../src/b.rs  Unchanged helper.
  entry         empty target  fresh  ../src/empty.rs
  entry         shifted one  shifted  ../src/a.rs  Notes written against version one.
  entry         missing  stale  ../src/gone.rs  target missing
  entry         bad regex  stale  ../src/b.rs  regex invalid
  entry         regex gone  stale  ../src/b.rs  regex not found
  entry         no path  unchecked  ?  no path
  entry         no hash  unchecked  ../src/b.rs  no hash
  entry         dir target  unchecked  ../src/dir  target unreadable
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：状态由本次运行计算，不存储在文件里。fresh 只说明目标字节没变，不说明笔记正确。no path、no hash、regex 无效、目标不可读是 error 级诊断，所以 check 首行 error、退出 1。这些条目的 fix 都把选定、审阅目标和说明放在命令之前，并说明命令会记下目标现在的哈希：shifted one 的说明写着 "Notes written against version one."，目标已是 version two，正要先改说明再刷新。

**命令 2**

```sh
agent-coexist read-brief docs/states
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：read-brief 首行 warn、退出 0。stale 条目与 no path、目标不可读的条目不输出 content；no hash 的条目仍输出内容与范围；空目标只有 content 键行，没有 L 范围。各诊断的修复见诊断章节。

<a id="read-brief-large-target"></a>

### brief 目标超过 64 MiB

编号 `read-brief-large-target` · 观察平台 Windows

超过内存上限的目标：有 regex 的条目为 unchecked target too large；无 regex 的条目按 hash 判定 fresh，但 read-brief 省略 content 并使首行 warn。

起始文件 `data/big.txt`：67108865 字节，SHA-256 `c36e8e594b7a2ac53e5f4a3f6039a9e8b86a8a199d5d7ee0a32da7b98045fb71`；64 MiB 的 x 加一个换行（67108865 字节）（超过内联上限，原字节未保存，私有记录里只有这里显示的大小与 SHA-256，内容按上面的说明由生成器确定地生成）

起始文件 `docs/big.agent-besties.md`：

```text
# big.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## with regex

- path: ../data/big.txt
- hash: c36e8e594b7a2ac53e5f4a3f6039a9e8b86a8a199d5d7ee0a32da7b98045fb71
- regex: x+

## hash only

- path: ../data/big.txt
- hash: c36e8e594b7a2ac53e5f4a3f6039a9e8b86a8a199d5d7ee0a32da7b98045fb71
```

**命令 1**

```sh
agent-coexist check docs/big
```

stdout：

```text
warn  check  docs/big
  file          docs/big.agent-besties.md
  type          brief
  entries       2
  fresh         1
  shifted       0
  stale         0
  unchecked     1
  entry         with regex  unchecked  ../data/big.txt  target too large
  entry         hash only  fresh  ../data/big.txt
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-brief docs/big
```

stdout：

```text
warn  read-brief  docs/big
  entry         with regex  unchecked  ../data/big.txt  target too large
  entry         hash only  fresh  ../data/big.txt  target too large
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="read-notification-comments"></a>

### read-notification：回复、提及与订阅

编号 `read-notification-comments` · 观察平台 Windows

窗口 [since, now) 内他人评论产生 reply、mention、subscribed 记录；每条评论至多一条记录；自己的评论、窗口外与当前这一秒的评论不出现。

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 @alice parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```

## #4 dave (2026-09-17T07:30:00Z, edited 2026-09-17T09:40:00Z)

```md
Lexer benchmarks attached; no action needed.
```

## #5 eve (2026-09-17T07:00:00Z)

```md
@alice old message before the window.
```

## #6 frank (2026-09-17T09:45:00Z)

```md
Release checklist: tag, changelog, binaries, checksums, announcement, and the docs site refresh.
```

## #7 gus (2026-09-17T09:50:00Z)

```md

```

## #8 alice (2026-09-17T09:55:00Z)

```md
@alice note to self.
```

## #9 heidi (2026-09-17T10:00:00Z)

```md
@alice written in the current second.
```
````

**命令 1**

```sh
agent-coexist read-notification --as alice --since 2026-09-17T08:00:00Z
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：#2 同时回复 alice 的 #1 并提及 @alice，只记一条 reply。#4 创建于窗口前、编辑于窗口内，按编辑时间计入。#8 是 alice 自己的评论；#9 与当前时间同一秒，被排除，下一次查询的 --since 应与本次窗口重叠。

**命令 2**

```sh
agent-coexist read-notification --as alice --since 2026-09-17T11:40:00+02:00
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T09:40:00Z
  subscribed  2026-09-17T09:40:00Z  ./docs/standup #4  dave  Lexer benchmarks attached; no action needed.
  subscribed  2026-09-17T09:45:00Z  ./docs/standup #6  frank  Release checklist: tag, changelog, binaries, checksums, announcement, and the...
  subscribed  2026-09-17T09:50:00Z  ./docs/standup #7  gus
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：带时区偏移的时间换算为 UTC 显示在 since 行（09:40:00Z）。

**命令 3**

```sh
agent-coexist read-notification --as alice --since 30m
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T09:30:00Z
  subscribed  2026-09-17T09:40:00Z  ./docs/standup #4  dave  Lexer benchmarks attached; no action needed.
  subscribed  2026-09-17T09:45:00Z  ./docs/standup #6  frank  Release checklist: tag, changelog, binaries, checksums, announcement, and the...
  subscribed  2026-09-17T09:50:00Z  ./docs/standup #7  gus
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：时长按当前时间往回推算，since 行显示已换算好的时间。

<a id="read-notification-state"></a>

### read-notification：状态、冲突、即将到期与已到期

编号 `read-notification-state` · 观察平台 Windows

他人的新声明（已结束的带 expired 列）、与自己 locked 重叠的 conflict、自己 30 分钟内到期的 expiring、窗口内已到期的 expired；祖先 state 也被读取。

起始文件 `state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T08:00:00Z..2026-09-17T10:20:00Z

## observing

alice 2026-09-17T07:00:00Z..2026-09-17T09:30:00Z
```

起始文件 `src/parser/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z

## observing

eve 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z
```

起始文件 `src/parser/lexer/state.agent-besties.md`：

```text
# .state

## locked

carol 2026-09-17T09:45:00Z..2026-09-17T12:00:00Z
```

**命令 1**

```sh
agent-coexist read-notification --as alice --since 2026-09-17T08:00:00Z
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：conflict 行的首行效果：只要有冲突，首行就是 warn（退出码仍为 0）。expiring 的「ends in 20m」按整分钟向下取整。

**命令 2**

```sh
agent-coexist read-notification src/parser --as bob --since 2026-09-17T08:00:00Z
```

stdout：

```text
warn  read-notification  src/parser
  as            bob
  since         2026-09-17T08:00:00Z
  state       2026-09-17T08:00:00Z  .  locked  alice  until 2026-09-17T10:20:00Z
  conflict    2026-09-17T08:00:00Z  .  locked  alice  overlaps your lock on src/parser
  state       2026-09-17T08:00:00Z  src/parser  observing  eve  until 2026-09-17T09:00:00Z  expired
  state       2026-09-17T09:45:00Z  src/parser/lexer  locked  carol  until 2026-09-17T12:00:00Z
  conflict    2026-09-17T09:45:00Z  src/parser/lexer  locked  carol  overlaps your lock on src/parser
  total         5
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：从 src/parser 查询时，根目录 . 的 state 作为祖先被读取：bob 看到 alice 在 . 的 locked 与自己在 src/parser 的锁重叠。

<a id="read-notification-expiry-boundaries"></a>

### read-notification：到期窗口的边界

编号 `read-notification-expiry-boundaries` · 观察平台 Windows

自己的条目恰好 30 分钟后到期计入 expiring，多 1 秒不计；不足 1 分钟显示 0m；until 正好等于现在的那一秒，既不是 expiring 也不是 expired，一秒后才出现 expired。

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T10:30:00Z

## observing

alice 2026-09-17T09:00:00Z..2026-09-17T10:30:01Z

## reviewing

alice 2026-09-17T09:00:00Z..2026-09-17T10:00:59Z

## testing

alice 2026-09-17T09:00:00Z..2026-09-17T10:00:00Z
```

**命令 1**

```sh
agent-coexist read-notification src --as alice --since 2026-09-17T09:00:00Z
```

stdout：

```text
ok  read-notification  src
  as            alice
  since         2026-09-17T09:00:00Z
  expiring    2026-09-17T10:00:59Z  src  reviewing  alice  ends in 0m
  expiring    2026-09-17T10:30:00Z  src  locked  alice  ends in 30m
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：testing 条目的 until 正是现在（10:00:00Z）：它已不再有效，所以不算 expiring；expired 只统计 until 早于现在的条目，所以这一秒两种记录都没有。observing 还剩 30 分 1 秒，超出 30 分钟窗口，也不出现。

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:00:01Z agent-coexist read-notification src --as alice --since 2026-09-17T09:00:00Z
```

stdout：

```text
ok  read-notification  src
  as            alice
  since         2026-09-17T09:00:00Z
  expired     2026-09-17T10:00:00Z  src  testing  alice  expired at 2026-09-17T10:00:00Z
  expiring    2026-09-17T10:00:59Z  src  reviewing  alice  ends in 0m
  expiring    2026-09-17T10:30:00Z  src  locked  alice  ends in 29m
  expiring    2026-09-17T10:30:01Z  src  observing  alice  ends in 30m
  total         4
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：同一文件，时钟拨后一秒（10:00:01Z）。testing 的 until 已早于现在，出现 expired 记录；observing 剩下恰好 30 分钟，进入窗口，显示 ends in 30m；locked 剩 29 分 59 秒，按整分钟向下取整为 29m

**命令 3**

```sh
agent-coexist check-state src
```

stdout：

```text
warn  check-state  src
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T10:30:00Z
  observing     alice  2026-09-17T09:00:00Z..2026-09-17T10:30:01Z
  reviewing     alice  2026-09-17T09:00:00Z..2026-09-17T10:00:59Z
  testing       alice  2026-09-17T09:00:00Z..2026-09-17T10:00:00Z  expired
  total         4
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：check-state 不输出 expiring/expired 记录，只把已到期的条目标为 expired；until 等于现在的条目在这里已标 expired。

<a id="read-notification-incomplete"></a>

### read-notification：无法完整读取的文件

编号 `read-notification-incomplete` · 观察平台 Windows

未闭合代码块与空文件整份 skipped，含杂散文本的 state（W-STRAY）与带坏片段的 state 行（W-STATE-ENTRY）为 partial，保留名 state 文件实为 post 时 skipped E-TYPE 但其中的评论仍交付。照 fix 逐个运行 check 看清每个文件的类型与问题，修好唯一可能藏着记录的未闭合代码块，再重跑原查询。

起始文件 `notes/broken.agent-besties.md`：

````text
# broken.post

## #1 bob (2026-09-17T09:00:00Z)

```md
@alice unclosed
````

起始文件 `notes/empty.agent-besties.md`：（空，0 字节）

起始文件 `w/state.agent-besties.md`：

````text
# state.post

## #1 bob (2026-09-17T09:50:00Z)

```md
@alice hi from a reserved post
```
````

起始文件 `d3/state.agent-besties.md`：

```text
# .state

stray words here

## observing

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

起始文件 `d4/state.agent-besties.md`：

```text
# .state

## locked

carol 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, broken fragment
```

**命令 1**

```sh
agent-coexist read-notification --as alice --since 2h
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：./notes/broken 与 ./notes/empty 的 skipped 行表示这两个文件整份没有读。./w 的 skipped 行列的是目录：w 的状态文件没有作为 state 读取（它其实是 post，E-TYPE），但其中的评论仍按 post 读取，所以上面有 ./w/state #1 的 mention。partial 行表示已交付可解析部分，并指出第一处没能使用的行。

**命令 2**

```sh
agent-coexist check ./d3/state
```

stdout：

```text
warn  check  ./d3/state
  file          ./d3/state.agent-besties.md
  type          state
  warn   L3  text outside any fence at L3 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  entries       1
  observing     bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

**命令 3**

```sh
agent-coexist check ./d4/state
```

stdout：

```text
warn  check  ./d4/state
  file          ./d4/state.agent-besties.md
  type          state
  warn   L5  "broken fragment" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  entries       1
  locked        carol  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

**命令 4**

```sh
agent-coexist check ./notes/broken
```

stdout：

```text
error  check  ./notes/broken
  file          ./notes/broken.agent-besties.md
  type          post
  error  L5  code fence opened at L5 is never closed
         why  everything after it is treated as fence content, so no later comment, entry or section can be found
         fix  add a closing line of 3 backticks where the block should end
  comments      1
  authors       bob
  topic         #1  bob
  latest        #1  bob  2026-09-17T09:00:00Z
  subscribers   none
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

**命令 5**

```sh
agent-coexist check ./notes/empty
```

stdout：

```text
error  check  ./notes/empty
  file          ./notes/empty.agent-besties.md
  error  L1  first line is "", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# empty.post" (or .brief / .agent-profile), or "# .state" for a state file
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

**命令 6**

```sh
agent-coexist check ./w/state
```

stdout：

```text
ok  check  ./w/state
  file          ./w/state.agent-besties.md
  type          post
  comments      1
  authors       bob
  topic         #1  bob  @alice hi from a reserved post
  latest        #1  bob  2026-09-17T09:50:00Z  @alice hi from a reserved post
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的一条只读 check

说明：check 给出每个文件能读出的类型和格式问题。./d3/state 与 ./d4/state 是 state：杂散文本和 carol 行里的坏片段都不是条目，按它们的 fix 要由写下的人改写或删除，其他人先问作者；同一行里合法的 carol 条目已经作为 state 记录交付。./notes/empty 是空文件，里面没有任何记录。./w/state 的类型是 post，不是 state，所以 ./w 没有 state 声明可漏；它的评论已按 post 交付，就是上面那条 mention。只有 ./notes/broken 可能藏着没交付的评论：代码块没有闭合，之后的内容都读不出来。

手工编辑 `notes/broken.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# broken.post

## #1 bob (2026-09-17T09:00:00Z)

```md
@alice unclosed
```
````

说明：这里由 #1 的作者 bob 按 check 的 fix 在评论末尾补上 3 个反引号的闭合行，评论内容不变。

**命令 7**

```sh
agent-coexist read-notification --as alice --since 2h
```

stdout：

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  state       2026-09-17T09:00:00Z  ./d3  observing  bob  until 2026-09-17T11:00:00Z
  state       2026-09-17T09:00:00Z  ./d4  locked  carol  until 2026-09-17T11:00:00Z
  mention     2026-09-17T09:00:00Z  ./notes/broken #1  bob  @alice unclosed
  mention     2026-09-17T09:50:00Z  ./w/state #1  bob  @alice hi from a reserved post
  partial     ./d3  W-STRAY  L3
  partial     ./d4  W-STATE-ENTRY  L5
  skipped     ./notes/empty  E-H1
  skipped     ./w  E-TYPE
  fix           some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one: agent-coexist check ./d3/state; agent-coexist check ./d4/state; agent-coexist check ./notes/empty; agent-coexist check ./w/state
  total         8
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：alice 重跑原查询

说明：./notes/broken #1 的 mention 出现了，它的 skipped 行消失：这条记录之前确实被漏掉。其余 partial 与 skipped 行如实保留，fix 行仍列出它们；这几个文件上面已经逐个看过，没有漏掉的记录。./w 的 skipped 行会一直保留，因为那个文件确实不是 state 文件。

<a id="read-check-state-scope"></a>

### check-state：自身、继承与下级目录的声明

编号 `read-check-state-scope` · 观察平台 Windows

祖先有效条目作为 inherited 行（已到期的不继承），自身条目，下级目录分组；重叠的 locked 两端都标 conflict，兄弟目录不重叠，自定义状态没有锁含义。

起始文件 `state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
zoe 2026-09-17T06:00:00Z..2026-09-17T08:00:00Z

## reviewing

frank 2026-09-17T09:15:00Z..2026-09-17T17:00:00Z
```

起始文件 `src/parser/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z

## observing

eve 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z
```

起始文件 `src/parser/lexer/state.agent-besties.md`：

```text
# .state

## locked

carol 2026-09-17T09:45:00Z..2026-09-17T12:00:00Z
```

起始文件 `src/docs/state.agent-besties.md`：

```text
# .state

## observing

dave 2026-09-17T09:00:00Z..2026-09-17T15:00:00Z
```

**命令 1**

```sh
agent-coexist check-state src/parser
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：zoe 在根目录的 locked 已到期，不作为 inherited 行出现；frank 的 reviewing 是自定义状态，继承显示但不参与冲突。

**命令 2**

```sh
agent-coexist check-state .
```

stdout：

```text
warn  check-state  .
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z  conflict
  locked        zoe  2026-09-17T06:00:00Z..2026-09-17T08:00:00Z  expired
  reviewing     frank  2026-09-17T09:15:00Z..2026-09-17T17:00:00Z
  src/docs/
    observing     dave  2026-09-17T09:00:00Z..2026-09-17T15:00:00Z
  src/parser/
    locked        bob  2026-09-17T09:30:00Z..2026-09-17T11:30:00Z  conflict
    observing     eve  2026-09-17T08:00:00Z..2026-09-17T09:00:00Z  expired
  src/parser/lexer/
    locked        carol  2026-09-17T09:45:00Z..2026-09-17T12:00:00Z  conflict
  total         7
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist check-state src/docs
```

stdout：

```text
ok  check-state  src/docs
  inherited     locked  alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z  .
  inherited     reviewing  frank  2026-09-17T09:15:00Z..2026-09-17T17:00:00Z  .
  observing     dave  2026-09-17T09:00:00Z..2026-09-17T15:00:00Z
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：src/docs 与 src/parser 是兄弟目录，dave 的条目不与 bob、carol 冲突；它继承的 alice locked 与本目录没有他人的 locked 重叠，所以没有 conflict 标记。

<a id="read-state-duplicates-and-ranges"></a>

### state 条目：重复声明、倒置区间与到期

编号 `read-state-duplicates-and-ranges` · 观察平台 Windows

同一昵称在同一状态下重复时 check-state 只显示 until 最晚的一条，read-notification 的历史记录两条都列；结束早于开始、until 等于现在都算到期。

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T10:10:00Z
alice 2026-09-17T09:20:00Z..2026-09-17T13:00:00Z

## observing

bob 2026-09-17T09:30:00Z..2026-09-17T09:10:00Z
carol 2026-09-17T08:00:00Z..2026-09-17T10:00:00Z
dave 2026-09-17T09:59:00Z..2026-09-17T10:29:59Z
```

**命令 1**

```sh
agent-coexist check-state src
```

stdout：

```text
warn  check-state  src
  locked        alice  2026-09-17T09:20:00Z..2026-09-17T13:00:00Z
  observing     bob  2026-09-17T09:30:00Z..2026-09-17T09:10:00Z  expired
  observing     carol  2026-09-17T08:00:00Z..2026-09-17T10:00:00Z  expired
  observing     dave  2026-09-17T09:59:00Z..2026-09-17T10:29:59Z
  total         4
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist read-notification src --as dave --since 2h
```

stdout：

```text
ok  read-notification  src
  as            dave
  since         2026-09-17T08:00:00Z
  state       2026-09-17T08:00:00Z  src  observing  carol  until 2026-09-17T10:00:00Z  expired
  state       2026-09-17T09:00:00Z  src  locked  alice  until 2026-09-17T10:10:00Z
  state       2026-09-17T09:20:00Z  src  locked  alice  until 2026-09-17T13:00:00Z
  state       2026-09-17T09:30:00Z  src  observing  bob  until 2026-09-17T09:10:00Z  expired
  expiring    2026-09-17T10:29:59Z  src  observing  dave  ends in 29m
  total         5
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：read-notification 按全部可解析声明列出历史 state 记录，所以 alice 的两条 locked 都出现；expiring 只看每个昵称的选定条目。

**命令 3**

```sh
agent-coexist check src/state
```

stdout：

```text
warn  check  src/state
  file          src/state.agent-besties.md
  type          state
  warn   L6  alice appears twice under locked
         why  the latest end time wins; the next state write that rewrites the locked section (a declaration or removal under it, or expiry cleanup of an entry in it) merges them into one unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)
         fix  optional; to merge them now, that agent declares again, which replaces both with one entry from now (add --for to choose its length), but only when both entries are on clean lines; when either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is and the duplicate stays: inspect the file, delete alice's entries from every such line by hand, keeping each line's other content, then look at the remaining entries and declare again to merge any duplicate that is left or to start one entry from now: agent-coexist state src --as=alice --status=locked
  warn   L10  bob under observing ends before it starts
         why  treated as expired: it never counts as a lock; the next state write removes it unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent declares it again: agent-coexist state src --as=bob --status=observing
  warn   L11  carol under observing expired at 2026-09-17T10:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state src --as=carol --status=observing
  entries       4
  locked        alice  2026-09-17T09:20:00Z..2026-09-17T13:00:00Z
  observing     bob  2026-09-17T09:30:00Z..2026-09-17T09:10:00Z  expired
  observing     carol  2026-09-17T08:00:00Z..2026-09-17T10:00:00Z  expired
  observing     dave  2026-09-17T09:59:00Z..2026-09-17T10:29:59Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：check 对同一文件给出重复、倒置与到期诊断及其修复建议；这些修复在诊断章节执行。

<a id="read-state-conflict-views"></a>

### 同一组锁冲突在各读命令中的呈现

编号 `read-state-conflict-views` · 观察平台 Windows

根目录 alice locked，src 同一文件内 bob 与 carol locked：check 只报同文件的成对冲突，check-state 标出三者，read-notification 按查询者列出与其锁重叠的他人。list 不报冲突。

起始文件 `state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T08:00:00Z..2026-09-17T12:00:00Z
```

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
carol 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z
```

**命令 1**

```sh
agent-coexist check src/state
```

stdout：

```text
warn  check  src/state
  file          src/state.agent-besties.md
  type          state
  warn   L6  locked is held by both bob and carol
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state src --remove <nick> --status=locked
  entries       2
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        carol  2026-09-17T09:30:00Z..2026-09-17T11:30:00Z  conflict
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check-state src
```

stdout：

```text
warn  check-state  src
  inherited     locked  alice  2026-09-17T08:00:00Z..2026-09-17T12:00:00Z  .  conflict
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        carol  2026-09-17T09:30:00Z..2026-09-17T11:30:00Z  conflict
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist read-notification --as alice --since 4h
```

stdout：

```text
warn  read-notification  .
  as            alice
  since         2026-09-17T06:00:00Z
  state       2026-09-17T09:00:00Z  ./src  locked  bob  until 2026-09-17T11:00:00Z
  conflict    2026-09-17T09:00:00Z  ./src  locked  bob  overlaps your lock on .
  state       2026-09-17T09:30:00Z  ./src  locked  carol  until 2026-09-17T11:30:00Z
  conflict    2026-09-17T09:30:00Z  ./src  locked  carol  overlaps your lock on .
  total         4
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist read-notification --as bob --since 4h
```

stdout：

```text
warn  read-notification  .
  as            bob
  since         2026-09-17T06:00:00Z
  state       2026-09-17T08:00:00Z  .  locked  alice  until 2026-09-17T12:00:00Z
  conflict    2026-09-17T08:00:00Z  .  locked  alice  overlaps your lock on ./src
  state       2026-09-17T09:30:00Z  ./src  locked  carol  until 2026-09-17T11:30:00Z
  conflict    2026-09-17T09:30:00Z  ./src  locked  carol  overlaps your lock on ./src
  total         4
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist list
```

stdout：

```text
ok  list  .
  ignore        none
  src/
    state                     state
  state                       state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="read-check-state-incomplete"></a>

### check-state：无法完整读取的 state 文件

编号 `read-check-state-incomplete` · 观察平台 Windows

含坏片段的行（W-STATE-ENTRY）与重复状态节（E-STATUS-DUP）为 partial，仍计入可解析条目；未闭合代码块（E-FENCE）与保留名文件实为 post（E-TYPE）为 skipped；非保留名的 state 文件被忽略。照 fix 逐个运行 check，按其 fix 合并重复状态节、补上闭合行（不改动任何条目），再重跑原查询。

起始文件 `src/a/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:10:00Z..2026-09-17T11:00:00Z
carol 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, broken fragment
```

起始文件 `src/b/state.agent-besties.md`：

```text
# .state

## locked

dave 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z

## locked

erin 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

起始文件 `src/c/state.agent-besties.md`：

````text
# .state

## locked

```
unclosed
````

起始文件 `src/d/state.agent-besties.md`：

```text
# .state

stray words here

## observing

gina 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

起始文件 `src/e/state.agent-besties.md`：

````text
# state.post

## #1 bob (2026-09-17T09:50:00Z)

```md
not a state file
```
````

起始文件 `src/f/extra.agent-besties.md`：

```text
# .state

## locked

hank 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

**命令 1**

```sh
agent-coexist check-state src
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：carol 所在行有坏片段，该行不会被 state 写入改写，但合法片段仍参与冲突判断。src/f 的 extra.agent-besties.md 不是保留名，check-state 不读它，hank 的锁因此不出现；用 check 可以看到它的内容与 W-NAME-STATE 提示。

**命令 2**

```sh
agent-coexist check src/f/extra
```

stdout：

```text
warn  check  src/f/extra
  file          src/f/extra.agent-besties.md
  type          state
  warn   L1  state file is named "extra.agent-besties.md", not "state.agent-besties.md"
         why  check-state and state look only for state.agent-besties.md (case-insensitive on Windows/macOS); other names are ignored
         fix  inspect this file and any state.agent-besties.md already in its directory; resolve their intended declarations before choosing which file should use that name, and never overwrite a separate existing state file (keep line 1 as "# .state")
  entries       1
  locked        hank  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist check src/a/state
```

stdout：

```text
warn  check  src/a/state
  file          src/a/state.agent-besties.md
  type          state
  warn   L6  "broken fragment" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  warn   L6  locked is held by both bob and carol
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state src/a --remove <nick> --status=locked
  entries       2
  locked        bob  2026-09-17T09:10:00Z..2026-09-17T11:00:00Z  conflict
  locked        carol  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：check-state 的 fix 冒号后的一条只读 check

**命令 4**

```sh
agent-coexist check src/b/state
```

stdout：

```text
error  check  src/b/state
  file          src/b/state.agent-besties.md
  type          state
  error  L7  status "locked" appears twice
         why  entries under a status must live in one place, or upsert and cleanup would touch the wrong one
         fix  merge the two "## locked" sections into one
  warn   L9  locked is held by both dave and erin
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state src/b --remove <nick> --status=locked
  entries       2
  locked        dave  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        erin  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：check-state 的 fix 冒号后的一条只读 check

**命令 5**

```sh
agent-coexist check src/c/state
```

stdout：

```text
error  check  src/c/state
  file          src/c/state.agent-besties.md
  type          state
  error  L5  code fence opened at L5 is never closed
         why  everything after it is treated as fence content, so no later comment, entry or section can be found
         fix  add a closing line of 3 backticks where the block should end
  entries       0
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：check-state 的 fix 冒号后的一条只读 check

**命令 6**

```sh
agent-coexist check src/d/state
```

stdout：

```text
warn  check  src/d/state
  file          src/d/state.agent-besties.md
  type          state
  warn   L3  text outside any fence at L3 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  entries       1
  observing     gina  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：check-state 的 fix 冒号后的一条只读 check

**命令 7**

```sh
agent-coexist check src/e/state
```

stdout：

```text
ok  check  src/e/state
  file          src/e/state.agent-besties.md
  type          post
  comments      1
  authors       bob
  topic         #1  bob  not a state file
  latest        #1  bob  2026-09-17T09:50:00Z  not a state file
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：check-state 的 fix 冒号后的一条只读 check

说明：check 给出每个文件的内容与问题。src/a 的坏片段和 src/d 的杂散文本都不是条目，按它们的 fix 要由写下的人改写或删除，其他人先问作者。src/b 有两个 ## locked 节，fix 是合并成一节；src/c 的代码块没有闭合。src/e/state 的类型是 post，不是 state，所以 src/e 没有 state 声明。

手工编辑 `src/b/state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

dave 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
erin 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

手工编辑 `src/c/state.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# .state

## locked

```
unclosed
```
````

说明：按两条 fix 修改结构，不改动任何条目：src/b 的两个 ## locked 合并为一节，dave 与 erin 的条目原样保留；src/c 的代码块之后没有其他内容，在文件末尾补上闭合行。

**命令 8**

```sh
agent-coexist check-state src
```

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
  partial       src/d  W-STRAY  L3
  skipped       src/e  E-TYPE
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds: agent-coexist check src/a/state; agent-coexist check src/d/state; agent-coexist check src/e/state
  total         8
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原查询

说明：src/b 与 src/c 已完整读取：dave 与 erin 的 locked 仍然重叠，conflict 是真实冲突；代码块里的内容不是条目，src/c 没有声明。src/a、src/d 的 partial 和 src/e 的 skipped 如实保留：前两处要由作者处理，可解析的条目都已列出；src/e 的文件确实不是 state 文件。

<a id="read-state-conflict-dirty"></a>

### 锁冲突的让出方声明写在脏行上

编号 `read-state-conflict-dirty` · 观察平台 Windows

src 下 bob 与 carol 同时 locked。carol 的声明和无法解析的 "broken fragment" 在同一行（W-STATE-ENTRY）。文件不记录片段是谁写的；这个场景没有交代它的来历。check 给出 W-STATE-CONFLICT 的 fix：商量后由让出的一方执行 --remove，并说明若该声明所在行还有无法解析的片段，这条命令会被 E-STATE-DIRTY 拒绝、改走该错误的手工删除。两人同意 carol 让出。carol 执行打印出的命令，得到 E-STATE-DIRTY 拒绝：删除会重写整行。她改走 E-STATE-DIRTY 的第一条路，只手工删掉自己的声明，这就是删除本身；bob 的 locked 不动，来历不明的 fragment 原样留下。

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:10:00Z..2026-09-17T11:00:00Z
carol 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, broken fragment
```

**命令 1**

```sh
agent-coexist check src/state
```

stdout：

```text
warn  check  src/state
  file          src/state.agent-besties.md
  type          state
  warn   L6  "broken fragment" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  warn   L6  locked is held by both bob and carol
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state src --remove <nick> --status=locked
  entries       2
  locked        bob  2026-09-17T09:10:00Z..2026-09-17T11:00:00Z  conflict
  locked        carol  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist state src --remove carol --status=locked
```

stdout：

```text
error  state  src
  reason        carol is still declared on line 6 of the state file in src, mixed with unparseable fragments
  fix           either delete carol's declaration from line 6 by hand, which is the removal itself (re-run then only if carol has more entries this command should remove), or have whoever wrote that line's unparseable fragments make them parseable, then re-run: agent-coexist state src --remove carol '--status=locked'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：冲突 fix 已说明脏行会变成 E-STATE-DIRTY。carol 按冲突 fix 让出，仍执行打印出的 --remove

说明：carol 按 E-STATE-DIRTY 的第一条路，只手工删掉自己在那一行的声明；这就是删除本身，不再执行原命令。broken fragment 没有交代作者，原样留下；bob 的 locked 不动。

手工编辑 `src/state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

bob 2026-09-17T09:10:00Z..2026-09-17T11:00:00Z
broken fragment
```

**命令 3**

```sh
agent-coexist check-state src
```

stdout：

```text
warn  check-state  src
  locked        bob  2026-09-17T09:10:00Z..2026-09-17T11:00:00Z
  partial       src  W-STATE-ENTRY  L6
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds: agent-coexist check src/state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：locked 只剩 bob；carol 已不在

**命令 4**

```sh
agent-coexist check src/state
```

stdout：

```text
warn  check  src/state
  file          src/state.agent-besties.md
  type          state
  warn   L6  "broken fragment" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  entries       1
  locked        bob  2026-09-17T09:10:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：剩下的 "broken fragment" 是 W-STATE-ENTRY。它的 fix 说明文件没有记录这个片段是谁写的；carol 不擅自改它，由写它的人改正或删掉

<a id="ch-diagnostics"></a>

## 诊断

<a id="diag-utf8"></a>

### 文件不是合法的 UTF-8

编号 `diag-utf8` · 观察平台 Windows

standup.agent-besties.md 的第 6 行含一个 Latin-1 编码的 é（字节 0xE9）。check 和写入都报告 E-UTF8 并拒绝；按 fix 另存为 UTF-8 后，原来的 post 成功。

起始文件 `standup.agent-besties.md`（转义视图）：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Lunch at the caf\xe9 at noon.
```
````

**命令 1**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  file          standup.agent-besties.md
  error  L6  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as bob -m 'Count me in.'
```

stdout：

```text
error  post  standup
  file          standup.agent-besties.md
  error  L6  file is not valid UTF-8
         why  the CLI cannot locate lines or headings in it
         fix  re-save the file as UTF-8
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-UTF8 at L6 first, as the fix rows above say, then run this command again
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：写入命令给出同样的诊断，文件保持原样

说明：按 fix 把文件另存为 UTF-8：é 变成两个字节 0xC3 0xA9。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Lunch at the café at noon.
```
````

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  Lunch at the café at noon.
  latest        #1  alice  2026-09-17T09:00:00Z  Lunch at the café at noon.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist post standup --as bob -m 'Count me in.'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Lunch at the café at noon.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  Count me in.
  ```
  ````

说明：重做原来的 post

<a id="diag-too-large"></a>

### 文件超过 16 MiB

编号 `diag-too-large` · 观察平台 Windows

build-log 帖子里贴了约 16.8 MB 的构建日志；release 帖子比上限少 100 字节。build-log 本身已超限，读写都报告 E-TOO-LARGE，按 fix 把日志移到普通文件、帖子只留简短说明后，原来的 post 成功。release 本身合法，只是这条 200 字节的评论会让它越过上限；换一条更短的评论，原来的 post 在同一帖子里成功，已有内容不动。

起始文件 `build-log.agent-besties.md`：16800061 字节，SHA-256 `898cd3216a6973e9fe38dcec6ce4bdfe8b5168b66dbe4d0490cb84fb70b9c09a`；一个帖子，唯一一条评论（ci，09:00）的正文是 1 200 000 行 "build step ok"（超过内联上限，原字节未保存，私有记录里只有这里显示的大小与 SHA-256，内容按上面的说明由生成器确定地生成）

起始文件 `release.agent-besties.md`：16777116 字节，SHA-256 `4fbcdc0ed5fad3f11024949951bc4ef87a46fb97a02fccd6bf0a9f2a15f01956`；一个帖子，文件大小比 16 MiB 上限少 100 字节；唯一一条评论（ci，09:00）的正文是重复的 "build step ok" 行（超过内联上限，原字节未保存，私有记录里只有这里显示的大小与 SHA-256，内容按上面的说明由生成器确定地生成）

**命令 1**

```sh
agent-coexist check build-log
```

stdout：

```text
error  check  build-log
  file          build-log.agent-besties.md
  error  L1  file is larger than 16 MiB
         why  files this size are never collaboration files; parsing it would stall every list and notification scan
         fix  move the large content elsewhere and keep only a short post or brief here
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post build-log --as bob -m 'Build is green.'
```

stdout：

```text
error  post  build-log
  file          build-log.agent-besties.md
  error  L1  file is larger than 16 MiB
         why  files this size are never collaboration files; parsing it would stall every list and notification scan
         fix  move the large content elsewhere and keep only a short post or brief here
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-TOO-LARGE at L1 first, as the fix rows above say, then run this command again
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist post release --as bob -m 'Release notes: .........................................................................................................................................................................................'
```

stdout：

```text
error  post  release
  file          release.agent-besties.md
  type          post
  error  L1  this write would make the file larger than 16 MiB: it is 16777116 bytes now and would become 16777362 bytes
         why  files this size are never collaboration files; parsing it would stall every list and notification scan
         fix  nothing was written; the file itself is still readable; shorten the new text, or put it in a new file (for a comment, a new post)
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist check release
```

stdout：

```text
ok  check  release
  file          release.agent-besties.md
  type          post
  comments      1
  authors       ci
  topic         #1  ci  build step ok
  latest        #1  ci  2026-09-17T09:00:00Z  build step ok
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：release 本身没有超限，check 可以读

说明：build-log：按 fix 把日志移出协作文件。ci 把原文件改名为普通文本文件 build-log.txt，日志和它原来的标题、时间都留在那里。目录里原本没有 build-log.txt：起始状态只有上面两个帖子，前面的命令都没有改动文件。

手工改名 `build-log.agent-besties.md` → `build-log.txt`

**命令 5**

```sh
agent-coexist post build-log --as ci -m 'Full log moved to build-log.txt.'
```

stdout：

```text
ok  post  build-log
  created       build-log.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `build-log.agent-besties.md`：

  ````text
  # build-log.post

  ## #1 ci (2026-09-17T10:00:00Z)

  ```md
  Full log moved to build-log.txt.
  ```
  ````

说明：ci 用自己的名字发一条简短说明，新建 build-log 帖子；时间是现在（10:00），不冒充原来 09:00 的那条

**命令 6**

```sh
agent-coexist post build-log --as bob -m 'Build is green.'
```

stdout：

```text
ok  post  build-log
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `build-log.agent-besties.md`：

  ````text
  # build-log.post

  ## #1 ci (2026-09-17T10:00:00Z)

  ```md
  Full log moved to build-log.txt.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  Build is green.
  ```
  ````

说明：重做 bob 原来的 post

手工新建 `RELEASE-NOTES.md`：

```text
Release notes: .........................................................................................................................................................................................
```

**命令 7**

```sh
agent-coexist post release --as bob -m 'Notes in RELEASE-NOTES.md.'
```

stdout：

```text
ok  post  release
  comment       #2  L1198370
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `release.agent-besties.md`：16777188 字节，SHA-256 `656b214e9b6203c9d45b4e447300bd08901c9da54d7627123f4a48b3daa8266e`（超过内联上限，原字节未保存，私有记录里只有这里显示的大小与 SHA-256）

说明：release：把长说明放进普通文件 RELEASE-NOTES.md，在原帖子里发一条更短的评论

<a id="diag-h1"></a>

### 第一行不是 H1

编号 `diag-h1` · 观察平台 Windows

notes.agent-besties.md 的第一行是手写的标题 "Standup notes"。check 与 post 都报告 E-H1；fix 给出可直接抄写的第一行，改好后原来的 post 成功。

起始文件 `notes.agent-besties.md`：

````text
Standup notes

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 1**

```sh
agent-coexist check notes
```

stdout：

```text
error  check  notes
  file          notes.agent-besties.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# notes.post" (or .brief / .agent-profile), or "# .state" for a state file
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post notes --as bob -m 'Parser done.'
```

stdout：

```text
error  post  notes
  file          notes.agent-besties.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# notes.post" (or .brief / .agent-profile), or "# .state" for a state file
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-H1 at L1 first, as the fix rows above say, then run this command again
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 把第 1 行改成 # notes.post。

手工编辑 `notes.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# notes.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 3**

```sh
agent-coexist check notes
```

stdout：

```text
ok  check  notes
  file          notes.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist post notes --as bob -m 'Parser done.'
```

stdout：

```text
ok  post  notes
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `notes.agent-besties.md`：

  ````text
  # notes.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  Parser done.
  ```
  ````

说明：重做原来的 post

<a id="diag-h1-bad-stem"></a>

### 第一行不是 H1，且文件名本身不是合法名称

编号 `diag-h1-bad-stem` · 观察平台 Windows

文件名 "standup notes" 含空格，不能照抄进 H1。fix 改为说明名称规则并用 `<name>` 标出要自己选的名称，改文件名时不能覆盖已有文件；先用 check 确认选好的名称还没有文件，再同时改文件名和第一行。

起始文件 `standup notes.agent-besties.md`：

````text
Standup notes

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 1**

```sh
agent-coexist check 'standup notes'
```

stdout：

```text
error  check  standup notes
  file          standup notes.agent-besties.md
  error  L1  first line is "Standup notes", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# <name>.post" (or .brief / .agent-profile): use 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-"; use "# .state" for a state file; renaming the file to <name>.agent-besties.md as well keeps the H1 and the file name in agreement, but only if no file has that name yet (never overwrite an existing file)
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：`<name>` 是要自己填写的标记。选 standup-notes：文件改名，第一行写成 `# standup-notes.post`。改名不能覆盖已有文件，先确认这个名称还没有文件。

**命令 2**

```sh
agent-coexist check standup-notes
```

stdout：

```text
error  check  standup-notes
  reason        standup-notes.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：E-FILE-MISSING 说明 standup-notes.agent-besties.md 还不存在，可以改名

手工改名 `standup notes.agent-besties.md` → `standup-notes.agent-besties.md`

手工编辑 `standup-notes.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup-notes.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 3**

```sh
agent-coexist check standup-notes
```

stdout：

```text
ok  check  standup-notes
  file          standup-notes.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-fence"></a>

### 代码块没有闭合

编号 `diag-fence` · 观察平台 Windows

评论 #1 用四个反引号开头的代码块里嵌了一段 ``` 代码，但外层没有闭合。E-FENCE 阻断所有写入；#1 的正文就在这个未闭合的代码块里，所以不另报 W-BODY。先补闭合行，原来的 post 才能成功。

起始文件 `review.agent-besties.md`：

`````text
# review.post

## #1 alice (2026-09-17T09:00:00Z)

````md
Please review:
```rust
fn main() {}
```
`````

**命令 1**

```sh
agent-coexist check review
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post review --as bob -m 'Looks good.'
```

stdout：

```text
error  post  review
  file          review.agent-besties.md
  type          post
  error  L5  code fence opened at L5 is never closed
         why  everything after it is treated as fence content, so no later comment, entry or section can be found
         fix  add a closing line of 4 backticks where the block should end
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-FENCE at L5 first, as the fix rows above say, then run this command again
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：#1 是 alice 的评论，由 alice 按 E-FENCE 的 fix 在代码块结尾补一行 4 个反引号。

手工编辑 `review.agent-besties.md`（覆盖原内容），编辑后完整内容：

`````text
# review.post

## #1 alice (2026-09-17T09:00:00Z)

````md
Please review:
```rust
fn main() {}
```
````
`````

**命令 3**

```sh
agent-coexist check review
```

stdout：

```text
ok  check  review
  file          review.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  Please review:
  latest        #1  alice  2026-09-17T09:00:00Z  Please review:
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：代码块闭合后评论 #1 的正文完整显示

**命令 4**

```sh
agent-coexist post review --as bob -m 'Looks good.'
```

stdout：

```text
ok  post  review
  comment       #2  L12
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `review.agent-besties.md`：

  `````text
  # review.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ````md
  Please review:
  ```rust
  fn main() {}
  ```
  ````

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  Looks good.
  ```
  `````

说明：重做原来的 post

<a id="diag-head"></a>

### 评论标题格式错误

编号 `diag-head` · 观察平台 Windows

bob 的标题漏了括号，carol 的编号有前导零。#02 读作 2，和 bob 的 #2 撞号，所以两条回复都写成了 @#2：alice 的 #3 回复 bob，bob 的 #4 回复 carol。两个 E-HEAD 的 fix 都给出可直接抄写的标题，保留原作者和原时间；bob 的 #2 未被占用，保留原编号；carol 的标题换成新编号，fix 同时要求更新原来指向她的 @#2。两人按 fix 改好标题后，@#2 只指 bob 的评论；bob 把自己回复 carol 的引用改成新编号，alice 回复 bob 的引用不动。之后查询确认引用指向正确，alice 原来的 post 成功。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob 2026-09-17T09:05:00Z

```md
Parser done.
```

## #02 carol (2026-09-17T09:10:00Z)

```md
Lexer done.
```

## #3 alice (2026-09-17T09:15:00Z)

```md
@#2 Parser looks good, bob.
```

## #4 bob (2026-09-17T09:20:00Z)

```md
@#2 carol, does the lexer handle tabs?
```
````

**命令 1**

```sh
agent-coexist check standup
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：两个标题都不合法时文件里没有 #2，所以两条 @#2 也报告 W-REPLY-DANGLING；标题修好后这两条警告随之消失

**命令 2**

```sh
agent-coexist post standup --as alice -m 'Thanks both.'
```

stdout：

```text
error  post  standup
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
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9 and E-HEAD at L15 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：第一处手工编辑由 bob 改他自己那一行标题：按它的 E-HEAD fix 改成 ## #2 bob (2026-09-17T09:05:00Z)，编号、作者和时间都是原来的；carol 那一行暂时不动。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #02 carol (2026-09-17T09:10:00Z)

```md
Lexer done.
```

## #3 alice (2026-09-17T09:15:00Z)

```md
@#2 Parser looks good, bob.
```

## #4 bob (2026-09-17T09:20:00Z)

```md
@#2 carol, does the lexer handle tabs?
```
````

说明：第二处手工编辑由 carol 改她自己那一行标题：按它的 E-HEAD fix 改成 ## #5 carol (2026-09-17T09:10:00Z)，作者和时间都是原来的，编号从 2 换成 fix 给出的未占用编号 5。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #5 carol (2026-09-17T09:10:00Z)

```md
Lexer done.
```

## #3 alice (2026-09-17T09:15:00Z)

```md
@#2 Parser looks good, bob.
```

## #4 bob (2026-09-17T09:20:00Z)

```md
@#2 carol, does the lexer handle tabs?
```
````

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L15  comment numbers jump from #2 to #5
         why  every number between #2 and #5 is used elsewhere in this post, so no comment is missing and @#n still resolves; only the order is unusual
         fix  leave the numbering as is; other files may already reference #5
  warn   L21  comment #3 appears after #5
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      5
  authors       alice, bob, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #4  bob  2026-09-17T09:20:00Z  @#2 carol, does the lexer handle tabs?
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：标题都合法了，两条 @#2 现在都指向 bob 的 #2。carol 的 fix 要求更新原来指向她的 @#2：bob 的 #4 点名 carol，是回复她的；alice 的 #3 点名 bob，本来就指 bob 的 #2，不改。carol 的评论写得比 #3、#4 早，新编号 #5 排在它们前面，剩下的两条 warn 只说明编号顺序，按建议保留

**命令 4**

```sh
agent-coexist post standup --as bob --edit 4 -m '@#5 carol, does the lexer handle tabs?'
```

stdout：

```text
ok  post  standup
  comment       #4  L27  edited
  reply-to      #5
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```

  ## #5 carol (2026-09-17T09:10:00Z)

  ```md
  Lexer done.
  ```

  ## #3 alice (2026-09-17T09:15:00Z)

  ```md
  @#2 Parser looks good, bob.
  ```

  ## #4 bob (2026-09-17T09:20:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#5 carol, does the lexer handle tabs?
  ```
  ````

说明：bob 按 carol 那条 fix 更新自己 #4 里的引用：@#2 改成 @#5

**命令 5**

```sh
agent-coexist lookfor-comment standup --reply-to 2
```

stdout：

```text
ok  lookfor-comment  standup
  L21  #3  alice  2026-09-17T09:15:00Z  @#2 Parser looks good, bob.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：回复 bob 的 #2 的只剩 alice 的 #3

**命令 6**

```sh
agent-coexist lookfor-comment standup --reply-to 5
```

stdout：

```text
ok  lookfor-comment  standup
  L27  #4  bob  2026-09-17T10:00:00Z  @#5 carol, does the lexer handle tabs?
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：bob 的 #4 现在回复 carol 的 #5

**命令 7**

```sh
agent-coexist post standup --as alice -m 'Thanks both.'
```

stdout：

```text
ok  post  standup
  comment       #6  L33
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```

  ## #5 carol (2026-09-17T09:10:00Z)

  ```md
  Lexer done.
  ```

  ## #3 alice (2026-09-17T09:15:00Z)

  ```md
  @#2 Parser looks good, bob.
  ```

  ## #4 bob (2026-09-17T09:20:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#5 carol, does the lexer handle tabs?
  ```

  ## #6 alice (2026-09-17T10:00:00Z)

  ```md
  Thanks both.
  ```
  ````

说明：重做原来的 post

<a id="diag-head-unnumbered"></a>

### 评论标题里的编号被改成了别的字符

编号 `diag-head-unnumbered` · 观察平台 Windows

bob 09:05 发了 #2（他自己那条 post 打印 comment #2），carol 09:10 用 @#2 回复了他。之后有人手工编辑，把 bob 标题里的 2 打成了 x。bob 原本要修改自己的 #2，被 E-HEAD 拒绝。标题里读不出编号，CLI 不猜测原编号：fix 仍给出未占用的新编号作为默认，并要求把原来指向这条评论的回复改成新编号；同时说明，若作者知道原编号、帖子里没有标题占用它、也没有其他修复要给它，就用回原编号，原来的回复不用改。#2 现在没人用（同一报告的 W-SEQ-GAP 写明没有评论是 #2），其他修复也没占用它。bob 从自己那条 post 的输出知道原编号是 #2，按这条路径把标题改回 #2；carol 的 @#2 仍然正确，不用改。bob 原来的修改按 --edit 2 完成。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:05:00Z agent-coexist post standup --as bob -m 'Parser done.'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```
  ````

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:10:00Z agent-coexist post standup --as carol -m '@#2 Nice parser.'
```

stdout：

```text
ok  post  standup
  comment       #3  L15
  reply-to      #2
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```

  ## #3 carol (2026-09-17T09:10:00Z)

  ```md
  @#2 Nice parser.
  ```
  ````

说明：之后有人手工编辑时把 bob 标题里的编号 2 打成了 x。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #x bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@#2 Nice parser.
```
````

**命令 3**

```sh
agent-coexist post standup --as bob --edit 2 -m 'Parser done, tests green.'
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 原本要修改自己的 #2，标题坏了，写入被拒绝

**命令 4**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
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
  comments      2
  authors       alice, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #3  carol  2026-09-17T09:10:00Z  @#2 Nice parser.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：CLI 不猜测原编号，默认给出 ## #4 bob (2026-09-17T09:05:00Z)。bob 从上面自己那条 post 的输出知道原编号是 #2；同一报告写明帖子里没有评论是 #2，其他修复也没把 #2 给别人。按 fix 的恢复路径把标题改成 ## #2 bob (2026-09-17T09:05:00Z)：作者和时间都是原来的，编号用回 #2。carol 的 @#2 仍然指这条评论，不用改。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@#2 Nice parser.
```
````

**命令 5**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #3  carol  2026-09-17T09:10:00Z  @#2 Nice parser.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：标题用回 #2 后，carol 的 @#2 仍然正确；没有结构错误，也没有悬空回复

**命令 6**

```sh
agent-coexist lookfor-comment standup --reply-to 2
```

stdout：

```text
ok  lookfor-comment  standup
  L15  #3  carol  2026-09-17T09:10:00Z  @#2 Nice parser.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：carol 的 #3 仍然回复 bob 的 #2，正文未改

**命令 7**

```sh
agent-coexist post standup --as bob --edit 2 -m 'Parser done, tests green.'
```

stdout：

```text
ok  post  standup
  comment       #2  L9  edited
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Parser done, tests green.
  ```

  ## #3 carol (2026-09-17T09:10:00Z)

  ```md
  @#2 Nice parser.
  ```
  ````

说明：重做 bob 原来的修改：评论仍是 #2

<a id="diag-head-unnumbered-unknown"></a>

### 评论标题读不出编号且没有原编号记录

编号 `diag-head-unnumbered-unknown` · 观察平台 Windows

独立夹具：bob 的标题已是 #x，没有先前成功的 post 输出或其他可靠原编号记录，不编造原编号。carol 的 #3 写着 @#2，那是回复里已有的字，不是原编号证据；即使同一报告写明没有评论是 #2，也不能把 #2 当成原编号。问过 #3 的作者 carol 本人：她写 @#2 时想回复的就是 bob 这条解析器评论，2 是凭印象填的、没有核对过；她能确认的是这条回复指向哪条评论，不是原编号。CLI 不猜测：fix 仍给出未占用的新编号作为默认，并要求把原来指向这条评论的回复改成新编号。bob 按默认改自己的标题，正文不动；只有 carol 改自己的回复，其余正文保留。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #x bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@#2 Nice parser.
```
````

**命令 1**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
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
  comments      2
  authors       alice, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #3  carol  2026-09-17T09:10:00Z  @#2 Nice parser.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：夹具没有可靠原编号，不编造。bob 按默认把标题改成 ## #4 bob (2026-09-17T09:05:00Z)：作者和时间都是原来的，编号是 fix 给出的 #4，正文不动。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #4 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@#2 Nice parser.
```
````

**命令 2**

```sh
agent-coexist check standup
```

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L9  comment numbers jump from #1 to #4
         why  no comment in this post has number #2, so @#2 cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order
         fix  leave the numbering as is; other files may already reference #4
  warn   L15  comment #3 appears after #4
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L15  comment #3 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #3 which comment was meant
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #3  carol  2026-09-17T09:10:00Z  @#2 Nice parser.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：W-REPLY-DANGLING 仍指出 carol 的 #3 回复不存在的 @#2。按它的 fix 问了 #3 的作者 carol 本人：她写 @#2 时想回复的就是 bob 这条解析器评论，2 是凭印象填的、没有核对过。她确认的是这条回复指向哪条评论，不是原编号；按默认把指向这条评论的引用改成 @#4

**命令 3**

```sh
agent-coexist post standup --as carol --edit 3 -m '@#4 Nice parser.'
```

stdout：

```text
ok  post  standup
  comment       #3  L15  edited
  reply-to      #4
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #4 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```

  ## #3 carol (2026-09-17T09:10:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#4 Nice parser.
  ```
  ````

说明：只有 carol 改自己的回复：@#2 改成 @#4，其余正文保留

**命令 4**

```sh
agent-coexist check standup
```

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L9  comment numbers jump from #1 to #4
         why  no comment in this post has number #2, so @#2 cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order
         fix  leave the numbering as is; other files may already reference #4
  warn   L15  comment #3 appears after #4
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #3  carol  2026-09-17T10:00:00Z  @#4 Nice parser.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：错误和悬空回复都已消失。剩下的 warn 只说明编号：#2 没有评论使用，#3 排在 #4 之后；两条都只说明编号，按建议保留

**命令 5**

```sh
agent-coexist lookfor-comment standup --reply-to 4
```

stdout：

```text
ok  lookfor-comment  standup
  L15  #3  carol  2026-09-17T10:00:00Z  @#4 Nice parser.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：carol 的 #3 现在回复 bob 的 #4

<a id="diag-head-fill"></a>

### 评论标题丢了时间

编号 `diag-head-fill` · 观察平台 Windows

bob 09:05 用 post 发了 #2，之后有人手工编辑时把标题里的时间删掉了。E-HEAD 的 fix 保留编号和作者，时间写作 `<time>`，要求填评论最初写下的时间，而不是现在的时间。bob 从编辑前查到的记录里找回 09:05，按 fix 填回后，alice 原来的 post 成功。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:05:00Z agent-coexist post standup --as bob -m 'Parser done.'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```
  ````

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:06:00Z agent-coexist lookfor-comment standup --by bob
```

stdout：

```text
ok  lookfor-comment  standup
  L9  #2  bob  2026-09-17T09:05:00Z  Parser done.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：编辑之前的查询记录：#2 写于 09:05

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob

```md
Parser done.
```
````

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## #2 bob" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (<time>)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist post standup --as alice -m 'Thanks, bob.'
```

stdout：

```text
error  post  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## #2 bob" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (<time>)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9 first, as the fix rows above say, then run this command again
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：`<time>` 要填评论最初写下的时间。bob 从上面编辑前的查询记录里找到 2026-09-17T09:05:00Z，把标题改成 `## #2 bob (2026-09-17T09:05:00Z)`。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```
````

**命令 5**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup. Post your status here.
  latest        #2  bob  2026-09-17T09:05:00Z  Parser done.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 6**

```sh
agent-coexist post standup --as alice -m 'Thanks, bob.'
```

stdout：

```text
ok  post  standup
  comment       #3  L15
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```

  ## #3 alice (2026-09-17T10:00:00Z)

  ```md
  Thanks, bob.
  ```
  ````

说明：重做原来的 post

<a id="diag-head-salvage"></a>

### 评论标题里的编号和昵称挨在一起

编号 `diag-head-salvage` · 观察平台 Windows

bob 09:05 用 post 发了 #2；昵称是纯数字的 agent 7 在 09:10 发了 #3（"7" 是合法昵称）；昵称恰好是 edited 的 agent 在 09:15 发了 #4（"edited" 也是合法昵称）。之后有人手工编辑，把 bob 的标题打成 "## # 2 bob (…)"（# 后多了空格），把 7 的标题丢了括号，把 edited 的编号写成带前导零的 #04。bob 原本要修改自己的 #2，被 E-HEAD 拒绝。"# 2 bob" 里的 2 是编号，不是作者：fix 给出 #2、作者 bob 和原时间；7 的标题里编号紧跟在 # 后，后面的 7 就是作者，fix 保留它；edited 后面紧跟括号里的时间，它是作者而不是修改时间的标记，fix 保留作者 edited 和原创建时间，不把这个时间当成修改时间。三条 fix 都可以直接抄写，没有要填的标记。三位作者各自按 fix 改回自己的标题后，bob 原来的修改成功；7 和 edited 也能修改自己的评论，说明三条评论的作者和创建时间都没变。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:05:00Z agent-coexist post standup --as bob -m 'Parser done.'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```
  ````

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:10:00Z agent-coexist post standup --as 7 -m 'Lexer done.'
```

stdout：

```text
ok  post  standup
  comment       #3  L15
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```

  ## #3 7 (2026-09-17T09:10:00Z)

  ```md
  Lexer done.
  ```
  ````

说明：7 是纯数字的合法昵称

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:15:00Z agent-coexist post standup --as edited -m 'Docs done.'
```

stdout：

```text
ok  post  standup
  comment       #4  L21
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  Parser done.
  ```

  ## #3 7 (2026-09-17T09:10:00Z)

  ```md
  Lexer done.
  ```

  ## #4 edited (2026-09-17T09:15:00Z)

  ```md
  Docs done.
  ```
  ````

说明：edited 是合法昵称，碰巧和标题里的修改标记同形

说明：之后有人手工编辑时弄坏了三行标题：bob 那一行的 # 后多了一个空格，7 那一行丢了括号，edited 那一行的编号多了前导零。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## # 2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 7 (2026-09-17T09:10:00Z)

```md
Lexer done.
```

## #4 edited (2026-09-17T09:15:00Z)

```md
Docs done.
```
````

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## # 2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 7 2026-09-17T09:10:00Z

```md
Lexer done.
```

## #4 edited (2026-09-17T09:15:00Z)

```md
Docs done.
```
````

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## # 2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 7 2026-09-17T09:10:00Z

```md
Lexer done.
```

## #04 edited (2026-09-17T09:15:00Z)

```md
Docs done.
```
````

**命令 4**

```sh
agent-coexist post standup --as bob --edit 2 -m 'Parser done, tests green.'
```

stdout：

```text
error  post  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## # 2 bob (2026-09-17T09:05:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (2026-09-17T09:05:00Z)"
  error  L15  "## #3 7 2026-09-17T09:10:00Z" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #3 7 (2026-09-17T09:10:00Z)"
  error  L21  "## #04 edited (2026-09-17T09:15:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #4 edited (2026-09-17T09:15:00Z)"
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9, E-HEAD at L15 and E-HEAD at L21 first, as the fix rows above say, then run this command again
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 原本要修改自己的 #2，标题坏了，写入被拒绝

**命令 5**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  file          standup.agent-besties.md
  type          post
  error  L9  "## # 2 bob (2026-09-17T09:05:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (2026-09-17T09:05:00Z)"
  error  L15  "## #3 7 2026-09-17T09:10:00Z" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #3 7 (2026-09-17T09:10:00Z)"
  error  L21  "## #04 edited (2026-09-17T09:15:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #4 edited (2026-09-17T09:15:00Z)"
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 按他那条 E-HEAD fix 把自己的标题改成 ## #2 bob (2026-09-17T09:05:00Z)：编号、作者、时间都是原来的。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 7 2026-09-17T09:10:00Z

```md
Lexer done.
```

## #04 edited (2026-09-17T09:15:00Z)

```md
Docs done.
```
````

说明：7 按它那条 E-HEAD fix 把自己的标题改成 ## #3 7 (2026-09-17T09:10:00Z)。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 7 (2026-09-17T09:10:00Z)

```md
Lexer done.
```

## #04 edited (2026-09-17T09:15:00Z)

```md
Docs done.
```
````

说明：edited 按它那条 E-HEAD fix 把自己的标题改成 ## #4 edited (2026-09-17T09:15:00Z)：作者仍是 edited，09:15 仍是创建时间。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
Parser done.
```

## #3 7 (2026-09-17T09:10:00Z)

```md
Lexer done.
```

## #4 edited (2026-09-17T09:15:00Z)

```md
Docs done.
```
````

**命令 6**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      4
  authors       alice, bob, 7, edited
  topic         #1  alice  Daily standup. Post your status here.
  latest        #4  edited  2026-09-17T09:15:00Z  Docs done.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 7**

```sh
agent-coexist post standup --as bob --edit 2 -m 'Parser done, tests green.'
```

stdout：

```text
ok  post  standup
  comment       #2  L9  edited
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Parser done, tests green.
  ```

  ## #3 7 (2026-09-17T09:10:00Z)

  ```md
  Lexer done.
  ```

  ## #4 edited (2026-09-17T09:15:00Z)

  ```md
  Docs done.
  ```
  ````

说明：重做 bob 原来的修改：#2 仍然是 bob 的

**命令 8**

```sh
agent-coexist post standup --as 7 --edit 3 -m 'Lexer done, benchmarks attached.'
```

stdout：

```text
ok  post  standup
  comment       #3  L15  edited
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Parser done, tests green.
  ```

  ## #3 7 (2026-09-17T09:10:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Lexer done, benchmarks attached.
  ```

  ## #4 edited (2026-09-17T09:15:00Z)

  ```md
  Docs done.
  ```
  ````

说明：7 修改自己的 #3：作者 7 也没有被改掉

**命令 9**

```sh
agent-coexist post standup --as edited --edit 4 -m 'Docs done, links checked.'
```

stdout：

```text
ok  post  standup
  comment       #4  L21  edited
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Parser done, tests green.
  ```

  ## #3 7 (2026-09-17T09:10:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Lexer done, benchmarks attached.
  ```

  ## #4 edited (2026-09-17T09:15:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Docs done, links checked.
  ```
  ````

说明：edited 修改自己的 #4：作者 edited 没有被改掉，标题保留 09:15 的创建时间并加上这次的修改时间

<a id="diag-head-fill-edited"></a>

### 评论标题丢了创建时间或作者

编号 `diag-head-fill-edited` · 观察平台 Windows

bob 09:05 发了 #2，09:20 改过一次；carol 09:30 发了 #3。之后有人手工编辑，把 bob 标题里的创建时间删掉，只剩 "edited 09:20"；把 carol 的标题打成 "## # 3 (…)"，作者没了。bob 原本要再改一次自己的 #2，被 E-HEAD 拒绝。修改时间不会被当成创建时间：bob 的 fix 保留 #2、bob 和修改时间，创建时间写作 `<time>`。carol 的标题里 # 后的 3 是编号，后面没有作者，fix 写作 `<nick>`，不猜作者。两条 fix 都只要求替换标记，填评论最初的作者和最初写下的时间，不填现在的时间。bob、carol 各自从前面命令留下的文件内容里找回自己的原值，改好自己的标题后，bob 原来的修改成功，carol 也能修改自己的 #3。

起始文件 `review.agent-besties.md`：

````text
# review.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Please review the parser.
```
````

**命令 1**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:05:00Z agent-coexist post review --as bob -m '@#1 Looks good.'
```

stdout：

```text
ok  post  review
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `review.agent-besties.md`：

  ````text
  # review.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Please review the parser.
  ```

  ## #2 bob (2026-09-17T09:05:00Z)

  ```md
  @#1 Looks good.
  ```
  ````

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:20:00Z agent-coexist post review --as bob --edit 2 -m '@#1 Looks good, one nit.'
```

stdout：

```text
ok  post  review
  comment       #2  L9  edited
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `review.agent-besties.md`：

  ````text
  # review.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Please review the parser.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T09:20:00Z)

  ```md
  @#1 Looks good, one nit.
  ```
  ````

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T09:30:00Z agent-coexist post review --as carol -m '@#2 I will fix the nit.'
```

stdout：

```text
ok  post  review
  comment       #3  L15
  reply-to      #2
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `review.agent-besties.md`：

  ````text
  # review.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Please review the parser.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T09:20:00Z)

  ```md
  @#1 Looks good, one nit.
  ```

  ## #3 carol (2026-09-17T09:30:00Z)

  ```md
  @#2 I will fix the nit.
  ```
  ````

说明：之后有人手工编辑时弄坏了两行标题：bob 那一行的创建时间被删掉，carol 那一行的作者被删掉，# 后还多了一个空格。

手工编辑 `review.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# review.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Please review the parser.
```

## #2 bob (edited 2026-09-17T09:20:00Z)

```md
@#1 Looks good, one nit.
```

## #3 carol (2026-09-17T09:30:00Z)

```md
@#2 I will fix the nit.
```
````

手工编辑 `review.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# review.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Please review the parser.
```

## #2 bob (edited 2026-09-17T09:20:00Z)

```md
@#1 Looks good, one nit.
```

## # 3 (2026-09-17T09:30:00Z)

```md
@#2 I will fix the nit.
```
````

**命令 4**

```sh
agent-coexist post review --as bob --edit 2 -m '@#1 Looks good; carol is fixing the nit.'
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 原本要再改一次自己的 #2，标题坏了，写入被拒绝

**命令 5**

```sh
agent-coexist check review
```

stdout：

```text
error  check  review
  file          review.agent-besties.md
  type          post
  error  L9  "## #2 bob (edited 2026-09-17T09:20:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #2 bob (<time>, edited 2026-09-17T09:20:00Z)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  error  L15  "## # 3 (2026-09-17T09:30:00Z)" looks like a comment header but is malformed
         why  comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous
         fix  rewrite the header as "## #3 <nick> (2026-09-17T09:30:00Z)", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written
  comments      1
  authors       alice
  topic         #1  alice  Please review the parser.
  latest        #1  alice  2026-09-17T09:00:00Z  Please review the parser.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：`<time>` 要填 #2 最初写下的时间，不是修改时间，也不是现在。bob 从第一条 post 造成的文件变化里找回 2026-09-17T09:05:00Z，把自己的标题改成 `## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T09:20:00Z)`。

手工编辑 `review.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# review.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Please review the parser.
```

## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T09:20:00Z)

```md
@#1 Looks good, one nit.
```

## # 3 (2026-09-17T09:30:00Z)

```md
@#2 I will fix the nit.
```
````

说明：`<nick>` 要填评论最初的作者。carol 从她那条 post 造成的文件变化里认出 2026-09-17T09:30:00Z 的 #3 是她写的，把标题改成 `## #3 carol (2026-09-17T09:30:00Z)`。

手工编辑 `review.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# review.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Please review the parser.
```

## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T09:20:00Z)

```md
@#1 Looks good, one nit.
```

## #3 carol (2026-09-17T09:30:00Z)

```md
@#2 I will fix the nit.
```
````

**命令 6**

```sh
agent-coexist check review
```

stdout：

```text
ok  check  review
  file          review.agent-besties.md
  type          post
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Please review the parser.
  latest        #3  carol  2026-09-17T09:30:00Z  @#2 I will fix the nit.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 7**

```sh
agent-coexist post review --as bob --edit 2 -m '@#1 Looks good; carol is fixing the nit.'
```

stdout：

```text
ok  post  review
  comment       #2  L9  edited
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `review.agent-besties.md`：

  ````text
  # review.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Please review the parser.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#1 Looks good; carol is fixing the nit.
  ```

  ## #3 carol (2026-09-17T09:30:00Z)

  ```md
  @#2 I will fix the nit.
  ```
  ````

说明：重做 bob 原来的修改：#2 仍然是 bob 的，创建时间仍是 09:05

**命令 8**

```sh
agent-coexist post review --as carol --edit 3 -m '@#2 Nit fixed.'
```

stdout：

```text
ok  post  review
  comment       #3  L15  edited
  reply-to      #2
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `review.agent-besties.md`：

  ````text
  # review.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Please review the parser.
  ```

  ## #2 bob (2026-09-17T09:05:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#1 Looks good; carol is fixing the nit.
  ```

  ## #3 carol (2026-09-17T09:30:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#2 Nit fixed.
  ```
  ````

说明：carol 修改自己的 #3：作者已经恢复为 carol

<a id="diag-seq-dup"></a>

### 评论编号重复

编号 `diag-seq-dup` · 观察平台 Windows

两条评论都是 #1。后一条是 bob 的，正文里写着 @#2（帖子里没有 #2；本场景设定这是 bob 的本意，不指向某条已有评论）。同一份 check 里 carol 的标题是 #x。E-SEQ-DUP 阻断写入。bob 在改标题之前选定未占用编号，不用 #2，也不用 carol 那条修复给出的编号；carol 改自己的标题。两人各改自己的标题后，原来的 post 成功。bob 确认 @#2 仍是本意，不改正文；编号顺序警告仍在。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #1 bob (2026-09-17T09:05:00Z)

```md
@#2 Parser done.
```

## #x carol (2026-09-17T09:10:00Z)

```md
Lexer done.
```
````

**命令 1**

```sh
agent-coexist check standup
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as alice -m Thanks.
```

stdout：

```text
error  post  standup
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
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-SEQ-DUP at L9 and E-HEAD at L15 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 的正文已经写着 @#2（本意如此），carol 的标题修复占用了 #3，所以选定 #4。没有指向这条评论的 @#1，不改引用。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #4 bob (2026-09-17T09:05:00Z)

```md
@#2 Parser done.
```

## #x carol (2026-09-17T09:10:00Z)

```md
Lexer done.
```
````

说明：carol 改自己的标题为 #3，作者和时间原样。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #4 bob (2026-09-17T09:05:00Z)

```md
@#2 Parser done.
```

## #3 carol (2026-09-17T09:10:00Z)

```md
Lexer done.
```
````

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L9  comment numbers jump from #1 to #4
         why  no comment in this post has number #2, so @#2 cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order
         fix  leave the numbering as is; other files may already reference #4
  warn   L9  comment #4 replies to @#2 which does not exist
         why  the reply cannot be followed to its target
         fix  ask the author of #4 which comment was meant
  warn   L15  comment #3 appears after #4
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #3  carol  2026-09-17T09:10:00Z  Lexer done.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：错误已消失。bob 确认 @#2 是本意，不改正文；编号顺序警告仍在

**命令 4**

```sh
agent-coexist post standup --as alice -m Thanks.
```

stdout：

```text
ok  post  standup
  comment       #5  L21
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #4 bob (2026-09-17T09:05:00Z)

  ```md
  @#2 Parser done.
  ```

  ## #3 carol (2026-09-17T09:10:00Z)

  ```md
  Lexer done.
  ```

  ## #5 alice (2026-09-17T10:00:00Z)

  ```md
  Thanks.
  ```
  ````

说明：重做原来的 post

<a id="diag-numbering-exhausted-check"></a>

### 编号已到上限时的标题修复

编号 `diag-numbering-exhausted-check` · 观察平台 Windows

帖子里 alice 的评论用到了最大编号 #18446744073709551614；bob 09:01 写了 #5。先用 lookfor-comment --by bob 记下这条完好标题。之后有人手工编辑：加上 carol 09:02 与 alice 撞号的 #18446744073709551614，erin 09:03 的 #1 用 @#18446744073709551614 回复 carol，并把 bob 的标题弄坏成 "## #x bob"，编号和时间都丢了。比 #18446744073709551614 更大的编号已经没有了，但更小的编号大多没人用。E-HEAD 和 E-SEQ-DUP 的 fix 说明这一点，并要求手工换成一个未用的较低编号，保留原作者、原时间和正文；bob 的标题丢了时间，E-HEAD 的 fix 还要求把 `<time>` 填成评论最初写下的时间。换号后的评论可能排在更大的编号后面，只会多出顺序警告。标题读不出编号时，同一条 E-HEAD fix 还打印：若知道原编号、本帖没有标题占用它、也没有其他修复要给它，就用回原编号、回复不用改。bob 从编辑前的查询记录里找回自己原来的 #5 和 09:01，按这条打印出的恢复路径用回 #5（不是新选号）。carol 只改自己那条的编号：换成未用的较低编号，且同一报告里没有回复已经写着那个编号；这不是恢复已知原编号。alice 的 #18446744073709551614 不动，原来指 alice 的 @#18446744073709551614 不用改，erin 把指 carol 的 @#18446744073709551614 改成 carol 的新编号。帖子的第一条评论就是 #18446744073709551614，W-SEQ-GAP 说明帖子里第一条评论是 #18446744073709551614，不是 #1，按建议保留。往帖子里追加新评论时没有更大的编号可用，那是另一种情况：dave 原来的追加最后按 E-SEQ-EXHAUSTED 的 fix 改发新帖子，另见 [diag-seq-exhausted](#diag-seq-exhausted)。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #18446744073709551614 alice (2026-09-17T09:00:00Z)

```md
Last comment number.
```

## #5 bob (2026-09-17T09:01:00Z)

```md
Question.
```

````

**命令 1**

```sh
agent-coexist lookfor-comment standup --by bob
```

stdout：

```text
ok  lookfor-comment  standup
  L9  #5  bob  2026-09-17T09:01:00Z  Question.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：编辑之前的查询记录：bob 的 #5 写于 09:01

说明：之后有人手工编辑：加上 carol 与 alice 撞号的评论和 erin 回复 carol 的 #1，并把 bob 的标题弄坏：编号 5 打成了 x，括号里的时间也被删掉。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #18446744073709551614 alice (2026-09-17T09:00:00Z)

```md
Last comment number.
```

## #x bob

```md
Question.
```

## #18446744073709551614 carol (2026-09-17T09:02:00Z)

```md
Moving on to the lexer.
```

## #1 erin (2026-09-17T09:03:00Z)

```md
@#18446744073709551614 carol, which lexer?
```
````

**命令 2**

```sh
agent-coexist post standup --as dave -m 'Count me in.'
```

stdout：

```text
error  post  standup
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
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-HEAD at L9 and E-SEQ-DUP at L15 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：dave 原本要追加一条评论；标题错误和重复编号阻断写入

**命令 3**

```sh
agent-coexist check standup
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 按 E-HEAD 的 fix 改自己的标题：默认是把 `## #<n> bob (<time>)` 的 `<n>` 换成未用的较低编号，且本帖没有回复 @#n 指向它。同一句还写明：若知道原编号、本帖没有标题占用它、也没有其他修复要给它，就用回原编号、回复不用改。bob 从上面编辑前的查询记录里找回自己原来的编号 #5 和最初写下的时间 2026-09-17T09:01:00Z；#5 现在没有别的标题使用，其他 fix 行也没给别人。按打印出的恢复路径把标题改成 `## #5 bob (2026-09-17T09:01:00Z)`，正文不动；用回原来的编号，指向他的引用也都不用改。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #18446744073709551614 alice (2026-09-17T09:00:00Z)

```md
Last comment number.
```

## #5 bob (2026-09-17T09:01:00Z)

```md
Question.
```

## #18446744073709551614 carol (2026-09-17T09:02:00Z)

```md
Moving on to the lexer.
```

## #1 erin (2026-09-17T09:03:00Z)

```md
@#18446744073709551614 carol, which lexer?
```
````

说明：carol 按 E-SEQ-DUP 的 fix 改后一条 #18446744073709551614，也就是她自己那条：只改编号。这不是恢复已知原编号（那是上面 bob 用回 #5）。她选用剩下最小的未用编号 #2：没有标题占用它，其他 fix 也没给别人，同一报告也没有 replies to @#2。标题改成 `## #2 carol (2026-09-17T09:02:00Z)`，作者、时间 09:02 和正文都不变。前面 alice 的 #18446744073709551614 不动。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #18446744073709551614 alice (2026-09-17T09:00:00Z)

```md
Last comment number.
```

## #5 bob (2026-09-17T09:01:00Z)

```md
Question.
```

## #2 carol (2026-09-17T09:02:00Z)

```md
Moving on to the lexer.
```

## #1 erin (2026-09-17T09:03:00Z)

```md
@#18446744073709551614 carol, which lexer?
```
````

**命令 4**

```sh
agent-coexist check standup
```

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L3  the first comment in this post is #18446744073709551614, not #1
         why  no comment in this post has number #3, so @#3 cannot be resolved; that comment may have been deleted or renumbered by hand, or the first comment was written with a higher number
         fix  leave the numbering as is; other files may already reference #18446744073709551614
  warn   L9  comment #5 appears after #18446744073709551614
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L15  comment #2 appears after #5
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L21  comment #1 appears after #2
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      4
  authors       alice, bob, carol, erin
  topic         #18446744073709551614  alice  Last comment number.
  latest        #1  erin  2026-09-17T09:03:00Z  @#18446744073709551614 carol, which lexer?
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：错误都已消失。E-SEQ-DUP 的 fix 还要求更新原来指 carol 那条的 @#18446744073709551614：erin 的 #1 点名 carol，要改成 @#2；@#18446744073709551614 现在只指 alice

**命令 5**

```sh
agent-coexist post standup --as erin --edit 1 -m '@#2 carol, which lexer?'
```

stdout：

```text
ok  post  standup
  comment       #1  L21  edited
  reply-to      #2
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #18446744073709551614 alice (2026-09-17T09:00:00Z)

  ```md
  Last comment number.
  ```

  ## #5 bob (2026-09-17T09:01:00Z)

  ```md
  Question.
  ```

  ## #2 carol (2026-09-17T09:02:00Z)

  ```md
  Moving on to the lexer.
  ```

  ## #1 erin (2026-09-17T09:03:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#2 carol, which lexer?
  ```
  ````

说明：erin 改自己回复里的引用：@#18446744073709551614 改成 @#2

**命令 6**

```sh
agent-coexist lookfor-comment standup --reply-to 2
```

stdout：

```text
ok  lookfor-comment  standup
  L21  #1  erin  2026-09-17T10:00:00Z  @#2 carol, which lexer?
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：erin 的 #1 现在回复 carol 的 #2

**命令 7**

```sh
agent-coexist check standup
```

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L3  the first comment in this post is #18446744073709551614, not #1
         why  no comment in this post has number #3, so @#3 cannot be resolved; that comment may have been deleted or renumbered by hand, or the first comment was written with a higher number
         fix  leave the numbering as is; other files may already reference #18446744073709551614
  warn   L9  comment #5 appears after #18446744073709551614
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L15  comment #2 appears after #5
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L21  comment #1 appears after #2
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      4
  authors       alice, bob, carol, erin
  topic         #18446744073709551614  alice  Last comment number.
  latest        #1  erin  2026-09-17T10:00:00Z  @#2 carol, which lexer?
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：剩下的 warn 只说明编号：第一条评论的编号是 #18446744073709551614，不是 #1；其余评论排在 #18446744073709551614 后面。都按建议保留：其他文件可能已经引用这些编号

**命令 8**

```sh
agent-coexist post standup --as dave -m 'Count me in.'
```

stdout：

```text
error  post  standup
  reason        comment numbering is exhausted: #18446744073709551614 is the highest number a header can carry
  fix           start a new post: agent-coexist post <new path> --as=dave -m '<text>'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：重做 dave 原来的追加。结构错误已经修好，但新评论要取最大编号加一，#18446744073709551614 之后没有编号了；fix 说明的另一种情况出现了：E-SEQ-EXHAUSTED

**命令 9**

```sh
agent-coexist post standup-2 --as=dave -m 'Count me in.'
```

stdout：

```text
ok  post  standup-2
  created       standup-2.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup-2.agent-besties.md`：

  ````text
  # standup-2.post

  ## #1 dave (2026-09-17T10:00:00Z)

  ```md
  Count me in.
  ```
  ````

说明：dave 执行 E-SEQ-EXHAUSTED 的 fix 命令：`<new path>` 填 standup-2，`<text>` 填他原来要发的正文

<a id="diag-seq-exhausted"></a>

### 追加评论时编号已用尽

编号 `diag-seq-exhausted` · 观察平台 Windows

帖子里已有最大编号的评论，post 追加失败（E-SEQ-EXHAUSTED），什么也没写。fix 建议改发新帖子，`<new path>` 和 `<text>` 需要填写。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #18446744073709551614 alice (2026-09-17T09:00:00Z)

```md
Last comment number.
```
````

**命令 1**

```sh
agent-coexist post standup --as bob -m 'Count me in.'
```

stdout：

```text
error  post  standup
  reason        comment numbering is exhausted: #18446744073709551614 is the highest number a header can carry
  fix           start a new post: agent-coexist post <new path> --as=bob -m '<text>'
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup-2 --as=bob -m 'Count me in.'
```

stdout：

```text
ok  post  standup-2
  created       standup-2.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup-2.agent-besties.md`：

  ````text
  # standup-2.post

  ## #1 bob (2026-09-17T10:00:00Z)

  ```md
  Count me in.
  ```
  ````

说明：fix 冒号后的命令，`<new path>` 填 standup-2，`<text>` 填 bob 原来要发的正文

<a id="diag-brief-structure"></a>

### 导读条目重名、缺路径、缺哈希

编号 `diag-brief-structure` · 观察平台 Windows

两个条目都叫 parser，第一个没有 path，两个都没有 hash。这份导读是 alice 写的：第一个 parser 条目没有记下路径，alice 作为条目作者选定 src/parser.rs（她知道这是要指的目标），不猜测。E-ENTRY-PATH 的 fix 是一条 brief 命令，但重名（E-ENTRY-DUP）阻断写入，必须先改名。之后 alice 选定 src/parser.rs，再审阅该文件与说明（仍描述则不改说明），再补路径；lexer 已有路径，按 E-ENTRY-HASH 先审阅目标和说明再刷新哈希。这两条命令都会记下目标现在的哈希。

起始文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

起始文件 `src/lexer.rs`：

```text
pub fn tokenize(input: &str) {}
```

起始文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## parser

```md
Start at parse_document.
```

## parser

- path: src/lexer.rs

```md
Tokens come from here.
```
````

**命令 1**

```sh
agent-coexist check handoff
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist brief handoff.agent-besties.md --target src/parser.rs --entry=parser
```

stdout：

```text
error  brief  handoff
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
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-ENTRY-DUP at L9 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：先执行 E-ENTRY-PATH 的 fix；alice 选定 src/parser.rs（她知道这是要指的目标），`<file>` 填它，不猜测

说明：最后的 refused 行说明什么也没写，并指出要先修 E-ENTRY-DUP（L9）。按它的 fix 给第二个条目改名为 lexer。

手工编辑 `handoff.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# handoff.brief

## parser

```md
Start at parse_document.
```

## lexer

- path: src/lexer.rs

```md
Tokens come from here.
```
````

**命令 3**

```sh
agent-coexist check handoff
```

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
  error  L9  entry "lexer" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief handoff.agent-besties.md --entry=lexer --refresh
  entries       2
  fresh         0
  shifted       0
  stale         0
  unchecked     2
  entry         parser  unchecked  ?  no path  Start at parse_document.
  entry         lexer  unchecked  src/lexer.rs  no hash  Tokens come from here.
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：alice 选定 src/parser.rs（她知道这是要指的目标），再审阅该文件与说明：仍是 parse_document，说明 "Start at parse_document." 仍然成立，不改说明。然后执行 E-ENTRY-PATH 的 fix。这条命令也会记下目标现在的哈希。

查看文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

查看文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## parser

```md
Start at parse_document.
```

## lexer

- path: src/lexer.rs

```md
Tokens come from here.
```
````

**命令 4**

```sh
agent-coexist brief handoff.agent-besties.md --target src/parser.rs --entry=parser
```

stdout：

```text
warn  brief  handoff
  entry         parser  updated  src/parser.rs
  hash          cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check handoff.agent-besties.md
  remaining     E-ENTRY-HASH  L12
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## parser

  - path: src/parser.rs
  - hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de

  ```md
  Start at parse_document.
  ```

  ## lexer

  - path: src/lexer.rs

  ```md
  Tokens come from here.
  ```
  ````

说明：重做 E-ENTRY-PATH 的 fix

**命令 5**

```sh
agent-coexist check handoff
```

stdout：

```text
error  check  handoff
  file          handoff.agent-besties.md
  type          brief
  error  L12  entry "lexer" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief handoff.agent-besties.md --entry=lexer --refresh
  entries       2
  fresh         1
  shifted       0
  stale         0
  unchecked     1
  entry         parser  fresh  src/parser.rs  Start at parse_document.
  entry         lexer  unchecked  src/lexer.rs  no hash  Tokens come from here.
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：lexer：按 E-ENTRY-HASH 的 fix 先审阅目标和说明。src/lexer.rs 仍是 tokenize，说明 "Tokens come from here." 仍然成立，不改说明。这条命令记下目标现在的哈希。

查看文件 `src/lexer.rs`：

```text
pub fn tokenize(input: &str) {}
```

查看文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## parser

- path: src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de

```md
Start at parse_document.
```

## lexer

- path: src/lexer.rs

```md
Tokens come from here.
```
````

**命令 6**

```sh
agent-coexist brief handoff.agent-besties.md --entry=lexer --refresh
```

stdout：

```text
ok  brief  handoff
  entry         lexer  refreshed  src/lexer.rs
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## parser

  - path: src/parser.rs
  - hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de

  ```md
  Start at parse_document.
  ```

  ## lexer

  - path: src/lexer.rs
  - hash: 0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4

  ```md
  Tokens come from here.
  ```
  ````

说明：E-ENTRY-HASH 的 fix

**命令 7**

```sh
agent-coexist check handoff
```

stdout：

```text
ok  check  handoff
  file          handoff.agent-besties.md
  type          brief
  entries       2
  fresh         2
  shifted       0
  stale         0
  entry         parser  fresh  src/parser.rs  Start at parse_document.
  entry         lexer  fresh  src/lexer.rs  Tokens come from here.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-entry-regex"></a>

### 导读条目的正则无法编译

编号 `diag-entry-regex` · 观察平台 Windows

条目 `<file>`（标题恰好像一个标记）有路径，fix 要求先审阅目标和说明（仍描述则不改说明）：这条命令也会记下目标现在的哈希。`--target=src/parser.rs` 和 `--entry='<file>'` 都是文件里的真实值，只有 `--regex '<pattern>'` 需要填写。这份导读是 alice 写的。条目 notes 没有路径：fix 同样要求先选定已知正确文件、审阅该文件与说明（必要时更新说明），再设目标和正则；这条命令也会记下目标现在的哈希。`--target=<file>` 和 `--regex '<pattern>'` 都是要填写的标记：文件没有记下目标或可用正则，不能猜测。alice 作为条目作者知道 notes 是关于 src/parser.rs 的任意函数，由她填写这两个标记。

起始文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

起始文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## <file>

- path: src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
- regex: parse_document(

```md
Entry point.
```

## notes

- regex: (fn

```md
Any function.
```
````

**命令 1**

```sh
agent-coexist check handoff
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：条目 `<file>` 有路径。按 fix 先审阅目标和说明：src/parser.rs 仍是 parse_document，说明 "Entry point." 仍然成立，不改说明。这条命令也会记下目标现在的哈希。

查看文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

查看文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## <file>

- path: src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
- regex: parse_document(

```md
Entry point.
```

## notes

- regex: (fn

```md
Any function.
```
````

**命令 2**

```sh
agent-coexist brief handoff.agent-besties.md --target=src/parser.rs '--entry=<file>' --regex 'parse_document\('
```

stdout：

```text
warn  brief  handoff
  entry         <file>  updated  src/parser.rs
  hash          cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check handoff.agent-besties.md
  remaining     E-ENTRY-PATH  L13
  remaining     E-ENTRY-HASH  L13
  remaining     E-ENTRY-REGEX  L15
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## <file>

  - path: src/parser.rs
  - hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
  - regex: parse_document\(

  ```md
  Entry point.
  ```

  ## notes

  - regex: (fn

  ```md
  Any function.
  ```
  ````

说明：只填写 `<pattern>`；`--entry='<file>'` 是真实标题，保持原样

说明：notes 没有记下目标和可用正则，不能猜测。alice 是这条条目的作者，她知道 notes 是关于 src/parser.rs 的任意函数。按 fix 先选定 src/parser.rs，再审阅该文件与说明：仍是任意函数，说明 "Any function." 仍然成立，不改说明。然后由她填写这两个标记。这条命令也会记下目标现在的哈希。

查看文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

查看文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## <file>

- path: src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
- regex: parse_document\(

```md
Entry point.
```

## notes

- regex: (fn

```md
Any function.
```
````

**命令 3**

```sh
agent-coexist brief handoff.agent-besties.md --target=src/parser.rs --entry=notes --regex 'fn \w+'
```

stdout：

```text
ok  brief  handoff
  entry         notes  updated  src/parser.rs
  hash          cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## <file>

  - path: src/parser.rs
  - hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
  - regex: parse_document\(

  ```md
  Entry point.
  ```

  ## notes

  - path: src/parser.rs
  - hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
  - regex: fn \w+

  ```md
  Any function.
  ```
  ````

说明：alice 填写：`--target=<file>` 为 `--target=src/parser.rs`，`<pattern>` 为 `fn \w+`

**命令 4**

```sh
agent-coexist check handoff
```

stdout：

```text
ok  check  handoff
  file          handoff.agent-besties.md
  type          brief
  entries       2
  fresh         2
  shifted       0
  stale         0
  entry         <file>  fresh  src/parser.rs  Entry point.
  entry         notes  fresh  src/parser.rs  Any function.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-entry-target"></a>

### 导读目标存在但读不出

编号 `diag-entry-target` · 观察平台 Windows（平台专属场景）

条目 config 指向目录 src，条目 private 指向对当前用户拒绝读取的文件。两者都报告 E-ENTRY-TARGET。config 的路径是目录：先选定 src/config.toml（知道这是要指的设置文件），再审阅该文件与说明，说明 "Settings live here." 仍然成立所以不改说明，再改指向它；这条命令也会记下目标现在的哈希。private 先恢复读权限、不改条目，再 check；文件仍不可读时不审阅它。

起始文件 `src/config.toml`：

```text
lexer = "fast"
```

起始文件 `notes/private.txt`：

```text
token rotation happens on Mondays
```

起始文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## config

- path: src
- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```md
Settings live here.
```

## private

- path: notes/private.txt
- hash: 0112d017a9b21959eab73859c2ef085e8f707da8e1f39dde12f6b98001945f40

```md
Rotation schedule.
```
````

手工去掉 `notes/private.txt` 的读权限（icacls 为当前用户添加拒绝读取 (R) 的 ACE；原 ACL 已另存）

**命令 1**

```sh
agent-coexist check handoff
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：config：src 是目录。选定 src/config.toml（知道这是要指的设置文件），再审阅该文件与说明：说明 "Settings live here." 仍然成立，不改说明。然后按命令改指向它。这条命令也会记下目标现在的哈希。

查看文件 `src/config.toml`：

```text
lexer = "fast"
```

查看文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## config

- path: src
- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```md
Settings live here.
```

## private

- path: notes/private.txt
- hash: 0112d017a9b21959eab73859c2ef085e8f707da8e1f39dde12f6b98001945f40

```md
Rotation schedule.
```
````

**命令 2**

```sh
agent-coexist brief handoff.agent-besties.md --target src/config.toml --entry=config
```

stdout：

```text
ok  brief  handoff
  entry         config  updated  src/config.toml
  hash          e37bc05134584ca148ae50ec169cb69957a5c818c4419d96e9363b80ede77251
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## config

  - path: src/config.toml
  - hash: e37bc05134584ca148ae50ec169cb69957a5c818c4419d96e9363b80ede77251

  ```md
  Settings live here.
  ```

  ## private

  - path: notes/private.txt
  - hash: 0112d017a9b21959eab73859c2ef085e8f707da8e1f39dde12f6b98001945f40

  ```md
  Rotation schedule.
  ```
  ````

说明：config：`<file>` 填 src/config.toml

说明：private：路径必须是可读文件。先恢复读权限、不改条目，再 check；文件仍不可读时不审阅它。

恢复 `notes/private.txt` 的读权限（icacls 恢复之前另存的原 ACL）

**命令 3**

```sh
agent-coexist check handoff
```

stdout：

```text
ok  check  handoff
  file          handoff.agent-besties.md
  type          brief
  entries       2
  fresh         2
  shifted       0
  stale         0
  entry         config  fresh  src/config.toml  Settings live here.
  entry         private  fresh  notes/private.txt  Rotation schedule.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-entry-drift"></a>

### 导读目标已变化或已移走

编号 `diag-entry-drift` · 观察平台 Windows

这份导读是 alice 写的。parser 的目标内容变了（shifted）；lexer 的目标被移到 src/lex/mod.rs（stale: target missing）；errors 的正则找不到匹配（stale: regex not found）。shifted 的 fix 先要求审阅目标和说明：说明不再描述目标就先改说明，再刷新哈希。移走的目标：fix 先要求已知正确文件（知道它现在在哪才指向它；若已删除或不确定位置则删除或改写条目，不猜替代目标）。alice 知道 lexer.rs 已移到 src/lex/mod.rs。先审阅该文件与说明：说明仍描述则不改，这条命令会记下新目标现在的哈希，原正则保留。errors 的目标内容其实也变了：记录哈希时类型叫 ParseError，现在叫 ParseFailure。stale 不会再同时报告 shifted，所以只有正则失配透露了这次变化。它的 fix 同样先要求审阅目标和说明：先看 src/errors.rs，确认说明 "Error type." 对新的类型仍然成立所以不改说明，再给出新正则；fix 已填好原目标，这条命令也会把哈希换成目标现在的内容。

起始文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Result<Doc, ParseFailure> {}
```

起始文件 `src/lex/mod.rs`：

```text
pub fn tokenize(input: &str) {}
```

起始文件 `src/errors.rs`：

```text
pub enum ParseFailure { Eof }
```

起始文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## parser

- path: src/parser.rs
- hash: 430788f454908013f22498861b4e8de70156db6fe22091f723b91c0c290763a2

```md
parse_document returns a Doc.
```

## lexer

- path: src/lexer.rs
- hash: 0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4
- regex: fn tokenize

```md
Tokenizer entry.
```

## errors

- path: src/errors.rs
- hash: 734a72f2cd059273ff8a1811a42cb14c6ac91ec741de975fc4bfc3ee83045450
- regex: enum ParseError

```md
Error type.
```
````

**命令 1**

```sh
agent-coexist check handoff
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：parser：按 fix 先审阅目标和说明，发现返回类型变了，说明不再描述目标，于是先改说明，再执行 fix 冒号后的刷新命令。这条命令记下目标现在的哈希。

查看文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Result<Doc, ParseFailure> {}
```

**命令 2**

```sh
agent-coexist brief handoff --entry parser -m 'parse_document returns Result<Doc, ParseFailure>.'
```

stdout：

```text
ok  brief  handoff
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## parser

  - path: src/parser.rs
  - hash: 430788f454908013f22498861b4e8de70156db6fe22091f723b91c0c290763a2

  ```md
  parse_document returns Result<Doc, ParseFailure>.
  ```

  ## lexer

  - path: src/lexer.rs
  - hash: 0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4
  - regex: fn tokenize

  ```md
  Tokenizer entry.
  ```

  ## errors

  - path: src/errors.rs
  - hash: 734a72f2cd059273ff8a1811a42cb14c6ac91ec741de975fc4bfc3ee83045450
  - regex: enum ParseError

  ```md
  Error type.
  ```
  ````

**命令 3**

```sh
agent-coexist brief handoff.agent-besties.md --entry=parser --refresh
```

stdout：

```text
ok  brief  handoff
  entry         parser  refreshed  src/parser.rs
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## parser

  - path: src/parser.rs
  - hash: e1509dbfe1ad447375686b48bfe17b3e856e7af7816681cde1d1bab86a8b60b5

  ```md
  parse_document returns Result<Doc, ParseFailure>.
  ```

  ## lexer

  - path: src/lexer.rs
  - hash: 0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4
  - regex: fn tokenize

  ```md
  Tokenizer entry.
  ```

  ## errors

  - path: src/errors.rs
  - hash: 734a72f2cd059273ff8a1811a42cb14c6ac91ec741de975fc4bfc3ee83045450
  - regex: enum ParseError

  ```md
  Error type.
  ```
  ````

说明：W-ENTRY-SHIFTED 的 fix 命令

说明：lexer：目标缺失时不猜测替代文件。alice 知道 lexer.rs 已移到 src/lex/mod.rs。按 fix 先审阅该文件与说明：仍是 fn tokenize，说明 "Tokenizer entry." 仍然成立，不改说明。这条命令会记下新目标现在的哈希，原正则保留。

查看文件 `src/lex/mod.rs`：

```text
pub fn tokenize(input: &str) {}
```

查看文件 `handoff.agent-besties.md`：

````text
# handoff.brief

## parser

- path: src/parser.rs
- hash: e1509dbfe1ad447375686b48bfe17b3e856e7af7816681cde1d1bab86a8b60b5

```md
parse_document returns Result<Doc, ParseFailure>.
```

## lexer

- path: src/lexer.rs
- hash: 0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4
- regex: fn tokenize

```md
Tokenizer entry.
```

## errors

- path: src/errors.rs
- hash: 734a72f2cd059273ff8a1811a42cb14c6ac91ec741de975fc4bfc3ee83045450
- regex: enum ParseError

```md
Error type.
```
````

**命令 4**

```sh
agent-coexist brief handoff.agent-besties.md --target src/lex/mod.rs --entry=lexer
```

stdout：

```text
ok  brief  handoff
  entry         lexer  updated  src/lex/mod.rs
  hash          0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## parser

  - path: src/parser.rs
  - hash: e1509dbfe1ad447375686b48bfe17b3e856e7af7816681cde1d1bab86a8b60b5

  ```md
  parse_document returns Result<Doc, ParseFailure>.
  ```

  ## lexer

  - path: src/lex/mod.rs
  - hash: 0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4
  - regex: fn tokenize

  ```md
  Tokenizer entry.
  ```

  ## errors

  - path: src/errors.rs
  - hash: 734a72f2cd059273ff8a1811a42cb14c6ac91ec741de975fc4bfc3ee83045450
  - regex: enum ParseError

  ```md
  Error type.
  ```
  ````

说明：lexer：alice 知道 lexer.rs 已移到 src/lex/mod.rs，`<file>` 填它，原正则 fn tokenize 保留

说明：errors：按 fix 先审阅目标和说明。文件里现在是 ParseFailure，不再是正则里的 ParseError；说明 "Error type." 仍描述这个目标（错误类型），所以不改说明，只换正则。

查看文件 `src/errors.rs`：

```text
pub enum ParseFailure { Eof }
```

**命令 5**

```sh
agent-coexist brief handoff.agent-besties.md --target=src/errors.rs --entry=errors --regex 'enum ParseFailure'
```

stdout：

```text
ok  brief  handoff
  entry         errors  updated  src/errors.rs
  hash          63e3454c485d2bbd4090dd2fee80673261fd76e1ecc73c866859fdc3820ed6f1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ## parser

  - path: src/parser.rs
  - hash: e1509dbfe1ad447375686b48bfe17b3e856e7af7816681cde1d1bab86a8b60b5

  ```md
  parse_document returns Result<Doc, ParseFailure>.
  ```

  ## lexer

  - path: src/lex/mod.rs
  - hash: 0a434c613bf88bfa1db35df863d4154ef200915111c4fbcd3b7c014c32eaade4
  - regex: fn tokenize

  ```md
  Tokenizer entry.
  ```

  ## errors

  - path: src/errors.rs
  - hash: 63e3454c485d2bbd4090dd2fee80673261fd76e1ecc73c866859fdc3820ed6f1
  - regex: enum ParseFailure

  ```md
  Error type.
  ```
  ````

说明：errors：fix 已填好原目标，只把 `<pattern>` 填成新正则；命令同时记录目标现在的哈希，和原来记下的哈希不同，因为目标内容已经变了

**命令 6**

```sh
agent-coexist check handoff
```

stdout：

```text
ok  check  handoff
  file          handoff.agent-besties.md
  type          brief
  entries       3
  fresh         3
  shifted       0
  stale         0
  entry         parser  fresh  src/parser.rs  parse_document returns Result<Doc, ParseFailure>.
  entry         lexer  fresh  src/lex/mod.rs  Tokenizer entry.
  entry         errors  fresh  src/errors.rs  Error type.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-entry-local"></a>

### 导读里的绝对路径、未知属性和游离文字

编号 `diag-entry-local` · 观察平台 Windows

这份导读是 alice 写的，下面的条目和说明代码块外那行 TODO 都是她留下的。条目用了本机绝对路径（W-ENTRY-ABS）：alice 知道原来的目标就是 src/parser.rs，不猜测。fix 要求先选定已知正确文件、审阅该文件与说明（仍描述则不改说明），再写成相对路径；这条命令也会记下目标现在的哈希。多了一行 owner 属性（W-ENTRY-KEY），说明代码块外还有一行 TODO（W-STRAY）。三者都只是警告，导读照常可用；W-STRAY 的 fix 说文件不记录这行是谁写的，由写它的人处理，所以 alice 把自己的 TODO 移进说明代码块。按各自的 fix 处理后 check 为 ok。

起始文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

起始文件 `handoff.agent-besties.md`：

````text
# handoff.brief

```md
Parser handoff.
```

TODO: add the lexer entry.

## parser

- path: ⟨root⟩/work/src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
- owner: alice

```md
Start at parse_document.
```
````

**命令 1**

```sh
agent-coexist check handoff
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：W-ENTRY-ABS：alice 知道原来的目标就是 src/parser.rs，不猜测。按 fix 先审阅该文件与说明：仍是 parse_document，说明 "Start at parse_document." 仍然成立，不改说明。然后把 `<relative path>` 填成相对导读目录的路径。这条命令也会记下目标现在的哈希。

查看文件 `src/parser.rs`：

```text
pub fn parse_document(input: &str) -> Doc {
    todo!()
}
```

查看文件 `handoff.agent-besties.md`：

````text
# handoff.brief

```md
Parser handoff.
```

TODO: add the lexer entry.

## parser

- path: ⟨root⟩/work/src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
- owner: alice

```md
Start at parse_document.
```
````

**命令 2**

```sh
agent-coexist brief handoff.agent-besties.md --target src/parser.rs --entry=parser
```

stdout：

```text
ok  brief  handoff
  entry         parser  updated  src/parser.rs
  hash          cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ```md
  Parser handoff.
  ```

  TODO: add the lexer entry.

  ## parser

  - path: src/parser.rs
  - hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de
  - owner: alice

  ```md
  Start at parse_document.
  ```
  ````

说明：W-ENTRY-ABS：`<relative path>` 填 src/parser.rs

说明：alice 处理自己写下的内容。W-ENTRY-KEY：删掉 owner 行，把内容并入说明。

手工编辑 `handoff.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# handoff.brief

```md
Parser handoff.
```

TODO: add the lexer entry.

## parser

- path: src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de

```md
Start at parse_document.
```
````

**命令 3**

```sh
agent-coexist brief handoff --entry parser -m 'Owner: alice. Start at parse_document.'
```

stdout：

```text
ok  brief  handoff
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `handoff.agent-besties.md`：

  ````text
  # handoff.brief

  ```md
  Parser handoff.
  ```

  TODO: add the lexer entry.

  ## parser

  - path: src/parser.rs
  - hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de

  ```md
  Owner: alice. Start at parse_document.
  ```
  ````

说明：W-STRAY：alice 把她写的 TODO 移进描述代码块。

手工编辑 `handoff.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# handoff.brief

```md
Parser handoff.
TODO: add the lexer entry.
```

## parser

- path: src/parser.rs
- hash: cd855870efb07870ef5c3be66c49dd60fc1838a06c6e9f2913637d0e19aa45de

```md
Owner: alice. Start at parse_document.
```
````

**命令 4**

```sh
agent-coexist check handoff
```

stdout：

```text
ok  check  handoff
  file          handoff.agent-besties.md
  type          brief
  description   Parser handoff.
  entries       1
  fresh         1
  shifted       0
  stale         0
  entry         parser  fresh  src/parser.rs  Owner: alice. Start at parse_document.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-profile-sections"></a>

### 档案的节名重复、不合法或没有正文

编号 `diag-profile-sections` · 观察平台 Windows

这是 alice 自己的档案，里面所有内容都是她写的。role 出现两次（E-SECTION-DUP，阻断）；"my scope" 不合法，fix 给出可抄写的 "## my-scope"；"on call" 不是已有 on-call 节的笔误：alice 本来就要另开一节记下待命备份，节名打算用 backup，正文写 bob（备份联系人）。建议名 on-call 已被占用，fix 改为手工改成与已有节都不同的合法名，不要求合并；她按自己原先的打算改成 backup 并补上正文 bob。prefer 没有正文，fix 是一条命令，但要等重复节合并后才能成功；最后一行游离文字也是 alice 写的，由她按 W-STRAY 的 fix 移进合适的代码块。

起始文件 `alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer.
```

## role

```md
Reviews parser changes.
```

## my scope

```md
crates/core parsing.
```

## prefer

## on call

## on-call

```md
Weekdays.
```

Ping me in the standup post.
````

**命令 1**

```sh
agent-coexist check alice
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist profile alice.agent-besties.md --section=prefer -m 'Small, reviewable diffs.'
```

stdout：

```text
error  profile  alice
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
  refused       nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix E-SECTION-DUP at L9 first, as the fix rows above say, then run this command again; the other rows above do not block this write
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：先执行 prefer 的 W-BODY fix

说明：refused 行说明什么也没写，要先修 E-SECTION-DUP（L9），其余各行不阻断写入。alice 按各条 fix 手工修改自己的档案：合并两个 role；"my scope" 改成 my-scope；"on call" 不是 on-call 的笔误，她本来就要另开待命备份节，按 fix 改成未占用的 backup，正文写事先准备好的 bob；最后一行游离文字是她自己写的，移进 on-call 的代码块。

手工编辑 `alice.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# alice.agent-profile

## role

```md
Rust implementer. Reviews parser changes.
```

## my-scope

```md
crates/core parsing.
```

## prefer

## backup

```md
bob
```

## on-call

```md
Weekdays. Ping me in the standup post.
```
````

**命令 3**

```sh
agent-coexist profile alice.agent-besties.md --section=prefer -m 'Small, reviewable diffs.'
```

stdout：

```text
ok  profile  alice
  section       prefer  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer. Reviews parser changes.
  ```

  ## my-scope

  ```md
  crates/core parsing.
  ```

  ## prefer

  ```md
  Small, reviewable diffs.
  ```

  ## backup

  ```md
  bob
  ```

  ## on-call

  ```md
  Weekdays. Ping me in the standup post.
  ```
  ````

说明：重做 prefer 的 W-BODY fix

**命令 4**

```sh
agent-coexist check alice
```

stdout：

```text
ok  check  alice
  file          alice.agent-besties.md
  type          agent-profile
  sections      5
  section       role  Rust implementer. Reviews parser changes.
  section       my-scope  crates/core parsing.
  section       prefer  Small, reviewable diffs.
  section       backup  bob
  section       on-call  Weekdays. Ping me in the standup post.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-profile-empty"></a>

### 档案没有任何节

编号 `diag-profile-empty` · 观察平台 Windows

只有 H1 的档案得到 W-PROFILE-EMPTY；fix 是一条写 role 节的命令，`<what you do>` 需要填写。

起始文件 `bob.agent-besties.md`：

```text
# bob.agent-profile
```

**命令 1**

```sh
agent-coexist check bob
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist profile bob.agent-besties.md --section role -m 'Reviewer for crates/core.'
```

stdout：

```text
ok  profile  bob
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `bob.agent-besties.md`：

  ````text
  # bob.agent-profile

  ## role

  ```md
  Reviewer for crates/core.
  ```
  ````

说明：fix 命令，`<what you do>` 已填写

**命令 3**

```sh
agent-coexist check bob
```

stdout：

```text
ok  check  bob
  file          bob.agent-besties.md
  type          agent-profile
  sections      1
  section       role  Reviewer for crates/core.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-heading-form"></a>

### 节名、状态名的首字符不合法

编号 `diag-heading-form` · 观察平台 Windows

档案节 "-draft" 以连字符开头，状态名 "2fa" 以数字开头。节名须以字母、数字或 _ 开头、最长 64；状态名须以 a-z 开头、最长 32。本场景只覆盖这两种首字符不合法的形式，不枚举长度与字符集的组合。alice 的档案和她在 2fa 下的声明都是她写的。按各自 fix 改成可抄写的合法名，正文和声明原样保留。

起始文件 `alice.agent-besties.md`：

````text
# alice.agent-profile

## -draft

```md
On-call drafts.
```
````

起始文件 `state.agent-besties.md`：

```text
# .state

## 2fa

alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

**命令 1**

```sh
agent-coexist check alice
```

stdout：

```text
warn  check  alice
  file          alice.agent-besties.md
  type          agent-profile
  warn   L3  section "-draft" at L3 is not a valid section name
         why  --section cannot address it; allowed: starts with a letter, digit or _, then letters, digits, _ and -, at most 64 characters
         fix  rename the heading, e.g. "## draft"
  sections      1
  section       -draft  On-call drafts.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check state
```

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L3  status "2fa" at L3 is not a valid status name
         why  entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters
         fix  rename the heading by hand to a valid, distinct status name, e.g. "## fa" only if that name is unused, then run check again
  entries       1
  2fa           alice  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：alice 按各条 fix 改自己写下的标题："-draft" 改成 draft，正文不动；"2fa" 改成 fa，她的声明原样保留。

手工编辑 `alice.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# alice.agent-profile

## draft

```md
On-call drafts.
```
````

手工编辑 `state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## fa

alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

**命令 3**

```sh
agent-coexist check alice
```

stdout：

```text
ok  check  alice
  file          alice.agent-besties.md
  type          agent-profile
  sections      1
  section       draft  On-call drafts.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist check state
```

stdout：

```text
ok  check  state
  file          state.agent-besties.md
  type          state
  entries       1
  fa            alice  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-control-heading"></a>

### 标题里有控制字符

编号 `diag-control-heading` · 观察平台 Windows

档案节标题含 ESC（0x1B），state 的两个状态标题含 BEL（0x07）。显示时控制字符换成 U+FFFD；凡是需要用这个标题作操作数的 fix 都改为手工改名，不给指向清洗后名称的命令。两个状态标题是同一个标题，E-STATUS-DUP 和两条 W-STATUS-NAME 的 fix 一致：去掉控制字符后合并成一个状态，不分别改成不同的名称，否则一个状态会被拆成两个。alice 和 bob 确认他们本来要写的状态名是 locked，于是按 fix 举例合并成 `## locked`。改成有效的 locked 名后，声明本身（nick 与时间）没有改动，是合法的 locked 标题让它们产生了锁语义，随后出现 W-STATE-CONFLICT。

起始文件 `alice.agent-besties.md`（转义视图）：

````text
# alice.agent-profile

## ro\x1ble

## scope

```md
crates/core parsing.
```
````

起始文件 `state.agent-besties.md`（转义视图）：

```text
# .state

## lo\x07cked

alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z

## lo\x07cked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

**命令 1**

```sh
agent-coexist check alice
```

stdout：

```text
warn  check  alice
  file          alice.agent-besties.md
  type          agent-profile
  warn   L3  section "ro�le" at L3 has no fenced body
         why  there is nothing to show as its content
         fix  edit the heading by hand to remove control characters and give it a valid, distinct title or name, then run check again
  warn   L3  section "ro�le" at L3 is not a valid section name
         why  --section cannot address it; allowed: starts with a letter, digit or _, then letters, digits, _ and -, at most 64 characters
         fix  rename the heading, e.g. "## role"
  sections      2
  section       ro�le
  section       scope  crates/core parsing.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check state
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：alice 按 W-BODY 的 fix 把自己档案里节标题的控制字符去掉、把节名改成 role；这条 fix 只处理标题，缺失的正文是 W-BODY 的消息本身报告的，由她另行补上。alice 和 bob 确认两个状态标题本来要写 locked，由他们按 E-STATUS-DUP 和 W-STATUS-NAME 的 fix 去掉控制字符，合并成同一个 locked 节；nick 和时间原样保留。有效的 locked 名使这两条声明产生锁语义，所以随后会出现 W-STATE-CONFLICT。

手工编辑 `alice.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# alice.agent-profile

## role

```md
Rust implementer.
```

## scope

```md
crates/core parsing.
```
````

手工编辑 `state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

**命令 3**

```sh
agent-coexist check alice
```

stdout：

```text
ok  check  alice
  file          alice.agent-besties.md
  type          agent-profile
  sections      2
  section       role  Rust implementer.
  section       scope  crates/core parsing.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist check state
```

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L5  locked is held by both alice and bob
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state . --remove <nick> --status=locked
  entries       2
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：改成有效 locked 后，alice 和 bob 的声明产生锁语义，于是出现 W-STATE-CONFLICT，见 [diag-state-advisory](#diag-state-advisory)

<a id="diag-post-encoding"></a>

### 帖子带 BOM、CRLF 和重复的订阅行

编号 `diag-post-encoding` · 观察平台 Windows

三个警告都不阻断：post 照常追加（新行用 LF）。check 显示实际生效的订阅者只有第一行的 alice、bob；第二行的 carol 从未生效。W-SUB-DUP 的 fix 要求只留第一行、每人一次，删掉后面的订阅行而不是把它并进来：并进来就等于替 carol 订阅。按 fix 另存为无 BOM、全 LF、只留第一行后，check 为 ok，订阅者仍是 alice、bob；carol 要不要订阅由她自己决定。

起始文件 `standup.agent-besties.md`（转义视图）：

````text
\u{feff}# standup.post\r
\r
- subscribers: alice, bob, alice\r
- subscribers: carol\r
\r
## #1 alice (2026-09-17T09:00:00Z)\r
\r
```md\r
Daily standup.\r
```\r
````

**命令 1**

```sh
agent-coexist check standup
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup --as bob -m 'Parser done.'
```

stdout：

```text
ok  post  standup
  comment       #2  L12
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`（转义视图）：

  ````text
  \u{feff}# standup.post\r
  \r
  - subscribers: alice, bob, alice\r
  - subscribers: carol\r
  \r
  ## #1 alice (2026-09-17T09:00:00Z)\r
  \r
  ```md\r
  Daily standup.\r
  ```\r

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  Parser done.
  ```
  ````

说明：警告不阻断写入

说明：按 fix 手工整理：去掉 BOM、换成 LF；只留第一行订阅者，去掉重复的 alice；第二行整行删掉。carol 只出现在第二行，现在没有订阅，删掉它不改变谁收到通知。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup.
```

## #2 bob (2026-09-17T10:00:00Z)

```md
Parser done.
```
````

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  Daily standup.
  latest        #2  bob  2026-09-17T10:00:00Z  Parser done.
  subscribers   alice, bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：订阅者仍是 alice、bob

<a id="diag-post-sequence"></a>

### 编号跳跃、顺序颠倒、回复目标不存在和游离文字

编号 `diag-post-sequence` · 观察平台 Windows

手工编辑过的帖子：#1、#4、#2 依次出现，#4 回复不存在的 @#3，开头和末尾有代码块外的文字。#1 到 #4 之间 #2 在后面出现，真正缺的是 #3，W-SEQ-GAP 的 why 说的就是 #3。另一个帖子 order 的 #1、#3、#2 只是顺序颠倒，中间的编号都在，W-SEQ-GAP 的 why 改为说明没有缺评论。第三个帖子 first 的第一条评论是 #2，#1 排在它后面：W-SEQ-GAP 说明帖子里第一条评论是 #2，不是 #1，why 同样说明没有缺评论。编号相关的警告按建议保留；悬空回复按建议去问 #4 的作者，作者改正自己的评论。游离文字的 fix 说文件不记录它是谁写的，由写它的人移走或删掉：这个场景设定末尾的 notes 节是 alice 自己留下的笔记，她删掉它；开头的 Agenda 行没有交代是谁写的，alice 不擅自删，按 fix 在帖子里问是谁写的，这一行暂时保留。

起始文件 `seq.agent-besties.md`：

````text
# seq.post

Agenda: parser, lexer

## #1 alice (2026-09-17T09:00:00Z)

```md
Parser plan.
```

## #4 bob (2026-09-17T09:01:00Z)

```md
@#3 what about this?
```

## #2 carol (2026-09-17T09:02:00Z)

```md
Lexer plan.
```

## notes

plain text
````

起始文件 `order.agent-besties.md`：

````text
# order.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Parser plan.
```

## #3 bob (2026-09-17T09:05:00Z)

```md
Tests are green.
```

## #2 carol (2026-09-17T09:02:00Z)

```md
Lexer plan.
```
````

起始文件 `first.agent-besties.md`：

````text
# first.post

## #2 bob (2026-09-17T09:05:00Z)

```md
Tests are green.
```

## #1 alice (2026-09-17T09:00:00Z)

```md
Parser plan.
```
````

**命令 1**

```sh
agent-coexist check seq
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check order
```

stdout：

```text
warn  check  order
  file          order.agent-besties.md
  type          post
  warn   L9  comment numbers jump from #1 to #3
         why  every number between #1 and #3 is used elsewhere in this post, so no comment is missing and @#n still resolves; only the order is unusual
         fix  leave the numbering as is; other files may already reference #3
  warn   L15  comment #2 appears after #3
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  Parser plan.
  latest        #2  carol  2026-09-17T09:02:00Z  Lexer plan.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：order：#2 在 #3 之后出现，中间没有缺编号，两个警告都按建议保留。

**命令 3**

```sh
agent-coexist check first
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：first：第一条评论是 #2，#1 在它后面出现，没有缺编号。两个警告都按建议保留。

**命令 4**

```sh
agent-coexist post seq --as alice -m '@#4 which comment did you mean? There is no #3.'
```

stdout：

```text
ok  post  seq
  comment       #5  L27
  reply-to      #4
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `seq.agent-besties.md`：

  ````text
  # seq.post

  Agenda: parser, lexer

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Parser plan.
  ```

  ## #4 bob (2026-09-17T09:01:00Z)

  ```md
  @#3 what about this?
  ```

  ## #2 carol (2026-09-17T09:02:00Z)

  ```md
  Lexer plan.
  ```

  ## notes

  plain text

  ## #5 alice (2026-09-17T10:00:00Z)

  ```md
  @#4 which comment did you mean? There is no #3.
  ```
  ````

说明：按 fix 询问 #4 的作者

**命令 5**

```sh
agent-coexist post seq --as bob --edit 4 -m '@#1 what about this?'
```

stdout：

```text
ok  post  seq
  comment       #4  L11  edited
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `seq.agent-besties.md`：

  ````text
  # seq.post

  Agenda: parser, lexer

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Parser plan.
  ```

  ## #4 bob (2026-09-17T09:01:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#1 what about this?
  ```

  ## #2 carol (2026-09-17T09:02:00Z)

  ```md
  Lexer plan.
  ```

  ## notes

  plain text

  ## #5 alice (2026-09-17T10:00:00Z)

  ```md
  @#4 which comment did you mean? There is no #3.
  ```
  ````

说明：bob 改正自己的评论

说明：末尾的 notes 节（标题和下面那行文字）是 alice 自己留下的笔记，她按 W-STRAY 的 fix 删掉；开头的 Agenda 行暂时不动。

手工编辑 `seq.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# seq.post

Agenda: parser, lexer

## #1 alice (2026-09-17T09:00:00Z)

```md
Parser plan.
```

## #4 bob (2026-09-17T09:01:00Z, edited 2026-09-17T10:00:00Z)

```md
@#1 what about this?
```

## #2 carol (2026-09-17T09:02:00Z)

```md
Lexer plan.
```

## #5 alice (2026-09-17T10:00:00Z)

```md
@#4 which comment did you mean? There is no #3.
```
````

**命令 6**

```sh
agent-coexist post seq --as alice -m 'Who wrote the "Agenda: parser, lexer" line above #1? Please move it into a comment or delete it.'
```

stdout：

```text
ok  post  seq
  comment       #6  L29
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `seq.agent-besties.md`：

  ````text
  # seq.post

  Agenda: parser, lexer

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Parser plan.
  ```

  ## #4 bob (2026-09-17T09:01:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  @#1 what about this?
  ```

  ## #2 carol (2026-09-17T09:02:00Z)

  ```md
  Lexer plan.
  ```

  ## #5 alice (2026-09-17T10:00:00Z)

  ```md
  @#4 which comment did you mean? There is no #3.
  ```

  ## #6 alice (2026-09-17T10:00:00Z)

  ```md
  Who wrote the "Agenda: parser, lexer" line above #1? Please move it into a comment or delete it.
  ```
  ````

说明：开头的 Agenda 行没有记录是谁写的。alice 不是它的作者，按 W-STRAY 的 fix 先问是谁写的，不擅自移动或删除

**命令 7**

```sh
agent-coexist check seq
```

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
  warn   L17  comment #2 appears after #4
         why  readers assume comments are in order; @#n still resolves
         fix  move the section so numbers increase, or leave it if the order is intentional
  warn   L23  comment numbers jump from #2 to #5
         why  no comment in this post has number #3, so @#3 cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order
         fix  leave the numbering as is; other files may already reference #5
  comments      5
  authors       alice, bob, carol
  topic         #1  alice  Parser plan.
  latest        #6  alice  2026-09-17T10:00:00Z  Who wrote the "Agenda: parser, lexer" line above #1? Please move it into a co...
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：编号相关的警告按建议保留：其他文件可能已经引用 #4。alice 的第一条新评论取最大编号加一（#5），排在已颠倒的 #2 之后，因此多出一条 #2 到 #5 的跳跃警告。这一条跳跃当前能确定的原因是新评论在编号已无序之后取了最大编号加一；夹具没有记录 #3 是否曾经存在，why 也同时列出删除、手工改号和 max+1。Agenda 行在写它的人回应之前保持原样，仍是 W-STRAY

<a id="diag-post-body"></a>

### 手工追加的评论没有代码块

编号 `diag-post-body` · 观察平台 Windows

carol 手工追加了 #2，正文直接写在标题下。W-BODY 的 fix 是 --edit 该评论的命令，只有作者 carol 能执行；这里由 carol 本人把那句话放进代码块。执行后原来那行裸文字仍是 W-STRAY；它是 carol 写的，由 carol 按其 fix 删除，不移进 alice 的 #1。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 carol (2026-09-17T09:30:00Z)

Sure, I will take it.
````

**命令 1**

```sh
agent-coexist check standup
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post standup.agent-besties.md --as=carol --edit=2 -m 'Sure, I will take it.'
```

stdout：

```text
ok  post  standup
  comment       #2  L9  edited
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  Daily standup. Post your status here.
  ```

  ## #2 carol (2026-09-17T09:30:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Sure, I will take it.
  ```

  Sure, I will take it.
  ````

说明：carol 本人执行 W-BODY 的 fix（她是 #2 的作者），`<text>` 填她原来那句话

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
warn  check  standup
  file          standup.agent-besties.md
  type          post
  warn   L15  text outside any fence at L15 is not part of the record
         why  the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown
         fix  this file does not record who wrote this text, even when it sits under another agent's heading; whoever wrote it moves it where it belongs (a fenced body, or in a state file an entry line under a status) or deletes it, and only a comment's author changes that comment; anyone else asks who wrote it before moving or deleting it
  comments      2
  authors       alice, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #2  carol  2026-09-17T10:00:00Z  Sure, I will take it.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：那句话已经在 carol 的 #2 代码块里，carol 按 W-STRAY 的 fix 删掉自己留在第 15 行的裸文字；代码块里的正文保持不变。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```

## #2 carol (2026-09-17T09:30:00Z, edited 2026-09-17T10:00:00Z)

```md
Sure, I will take it.
```

````

**命令 4**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, carol
  topic         #1  alice  Daily standup. Post your status here.
  latest        #2  carol  2026-09-17T10:00:00Z  Sure, I will take it.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-name-mismatch"></a>

### 文件名与 H1 名称不一致

编号 `diag-name-mismatch` · 观察平台 Windows

bob.agent-besties.md 的 H1 是 alice.post：fix 给出两种改法，这里选改文件名。"my notes" 不是合法名称，fix 只剩改文件名一种。改名的 fix 要求目标名还没有文件、绝不覆盖；先列出目录确认 alice 与 retro 都不存在，再改名。

起始文件 `bob.agent-besties.md`：

````text
# alice.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

起始文件 `my notes.agent-besties.md`：

````text
# retro.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Daily standup. Post your status here.
```
````

**命令 1**

```sh
agent-coexist check bob
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check 'my notes'
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  bob                         post
  my notes                    post
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 先确认目标名没有文件：目录里没有 alice 和 retro

说明：两个目标名都空着，两个文件都按 fix 改名为 H1 的名称。

手工改名 `bob.agent-besties.md` → `alice.agent-besties.md`

手工改名 `my notes.agent-besties.md` → `retro.agent-besties.md`

**命令 4**

```sh
agent-coexist check alice
```

stdout：

```text
ok  check  alice
  file          alice.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist check retro
```

stdout：

```text
ok  check  retro
  file          retro.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  Daily standup. Post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  Daily standup. Post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-state-advisory"></a>

### 状态冲突、重复、倒置和过期

编号 `diag-state-advisory` · 观察平台 Windows

alice 与 bob 同时 locked（W-STATE-CONFLICT），carol 在 observing 下出现两次（W-STATE-DUP），dave 的结束早于开始（W-STATE-RANGE），erin、frank 已过期（W-STATE-EXPIRED，在 review 下）。各 fix 都是给相关成员的建议：冲突要先商量，由让出的一方执行；重复可以不管，只有重写该状态节的写入才会合并，且两条都在干净行上；frank 续的是 review，observing 没有过期条目，所以 carol 的重复仍在；carol 的两条都在干净行上，再声明 observing 才合并。过期只有仍需要时才续。

起始文件 `state.agent-besties.md`：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z

## observing

carol 2026-09-17T08:00:00Z..2026-09-17T12:00:00Z, carol 2026-09-17T08:00:00Z..2026-09-17T13:00:00Z

## review

dave 2026-09-17T11:00:00Z..2026-09-17T10:30:00Z
erin 2026-09-17T07:00:00Z..2026-09-17T08:00:00Z
frank 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z
```

**命令 1**

```sh
agent-coexist check state
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：frank 仍在关注这个目录，按他那条 fix 续期 review；erin 已经不需要，不续。

**命令 2**

```sh
agent-coexist state . --as=frank --status=review
```

stdout：

```text
ok  state  .
  set           review  frank  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  cleaned       review  dave  2026-09-17T11:00:00Z..2026-09-17T10:30:00Z
  cleaned       review  erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z

  ## observing

  carol 2026-09-17T08:00:00Z..2026-09-17T12:00:00Z, carol 2026-09-17T08:00:00Z..2026-09-17T13:00:00Z

  ## review

  frank 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

说明：frank 的 W-STATE-EXPIRED fix 命令。这次写入重写 review 节（声明所在节，且节内有过期条目），所以清理了 dave 和 erin（cleaned 行）；observing 不是目标、也没有过期条目，carol 的重复仍在

**命令 3**

```sh
agent-coexist check state
```

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
  entries       4
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T12:00:00Z  conflict
  observing     carol  2026-09-17T08:00:00Z..2026-09-17T13:00:00Z
  review        frank  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：frank 写的是 review，不是 observing：carol 仍出现两次

说明：carol 的两条都在同一条干净行上。按 fix 里干净行的那一支重新声明 observing：这次写入重写 observing 节，两条合并成一条。脏行上的重复不会这样合并。

**命令 4**

```sh
agent-coexist state . --as=carol --status=observing
```

stdout：

```text
ok  state  .
  set           observing  carol  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z

  ## observing

  carol 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z

  ## review

  frank 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

说明：carol 的 W-STATE-DUP fix 命令（同一状态、干净行）

**命令 5**

```sh
agent-coexist check state
```

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L7  locked is held by both alice and bob
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state . --remove <nick> --status=locked
  entries       4
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  conflict
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T12:00:00Z  conflict
  observing     carol  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  review        frank  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：observing 已合并，locked 冲突还在

说明：alice 与 bob 商量后由 bob 让出，bob 执行冲突 fix，`<nick>` 填 bob。

**命令 6**

```sh
agent-coexist state . --remove bob --status=locked
```

stdout：

```text
ok  state  .
  removed       locked  bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z

  ## observing

  carol 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z

  ## review

  frank 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

说明：W-STATE-CONFLICT 的 fix 命令

**命令 7**

```sh
agent-coexist check state
```

stdout：

```text
ok  check  state
  file          state.agent-besties.md
  type          state
  entries       3
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
  observing     carol  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  review        frank  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-state-structure"></a>

### 状态文件的节重复、状态名不合法、条目写错

编号 `diag-state-structure` · 观察平台 Windows

gina 想声明 observing，但 observing 出现两次（E-STATUS-DUP）阻断写入。同一文件里 "Review Queue" 不是合法状态名，它下面过期条目的 fix 因此改为先改标题；"frank tomorrow" 不是条目格式。文件不记录片段是谁写的；这个场景设定 "frank tomorrow" 是 gina 自己记在她那一行的备忘（frank 明天才来），不是任何人的声明。gina 按各 fix 手工修好结构：合并两个 observing 标题、改 "Review Queue" 标题都不改动任何人的条目；只删她自己的备忘。之后原来的声明成功。erin 的过期条目只有 erin 本人仍需要时才续，gina 不替她续。

起始文件 `state.agent-besties.md`：

```text
# .state

## observing

carol 2026-09-17T08:00:00Z..2026-09-17T13:00:00Z

## Review Queue

erin 2026-09-17T07:00:00Z..2026-09-17T08:00:00Z

## observing

gina 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z, frank tomorrow
```

**命令 1**

```sh
agent-coexist state . --as gina --status observing --for 4h
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check state
```

stdout：

```text
error  check  state
  file          state.agent-besties.md
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
  entries       3
  observing     carol  2026-09-17T08:00:00Z..2026-09-17T13:00:00Z
  Review Queue  erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z  expired
  observing     gina  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：gina 按各 fix 手工修改：两个 observing 合成一节，carol 和 gina 的条目原样保留；删掉她自己留下的备忘 "frank tomorrow"，它不是 frank 的声明；文件里还没有 review-queue 节，把 "Review Queue" 标题改成它，erin 的条目原样保留。

手工编辑 `state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## observing

carol 2026-09-17T08:00:00Z..2026-09-17T13:00:00Z
gina 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z

## review-queue

erin 2026-09-17T07:00:00Z..2026-09-17T08:00:00Z
```

**命令 3**

```sh
agent-coexist check state
```

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L10  erin under review-queue expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state . --as=erin --status=review-queue
  entries       3
  observing     carol  2026-09-17T08:00:00Z..2026-09-17T13:00:00Z
  observing     gina  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
  review-queue  erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z  expired
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：状态名合法后，erin 的过期条目给出由 erin 执行的续期命令；只有 erin 本人仍需要时才续，gina 不替她执行

**命令 4**

```sh
agent-coexist state . --as gina --status observing --for 4h
```

stdout：

```text
ok  state  .
  set           observing  gina  2026-09-17T10:00:00Z..2026-09-17T14:00:00Z
  cleaned       review-queue  erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  ## observing

  carol 2026-09-17T08:00:00Z..2026-09-17T13:00:00Z, gina 2026-09-17T10:00:00Z..2026-09-17T14:00:00Z

  ## review-queue

  ```

说明：重做 gina 原来的声明；这次写入按 W-STATE-EXPIRED 的 why 清理了已过期的 erin 条目

<a id="diag-state-invalid-repeated"></a>

### 同一个不合法的状态标题出现两次

编号 `diag-state-invalid-repeated` · 观察平台 Windows

state 文件里有两节都叫 "Review Queue"：alice 的声明在第一节，bob 的在第二节。carol 要声明 observing，被 E-STATUS-DUP 拒绝。check 对这两个标题各报一条 W-STATUS-NAME；因为它们是同一个标题，两条 fix 都和 E-STATUS-DUP 一致，要求合并成一节并改成合法名称，不能把两份分别改成不同的名字，否则同一个状态会被拆成两个，alice 和 bob 的声明就分开了。按 fix 合并后，两人的条目都在同一个合法状态下，carol 原来的声明成功。

起始文件 `state.agent-besties.md`：

```text
# .state

## Review Queue

alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z

## Review Queue

bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

**命令 1**

```sh
agent-coexist state . --as carol --status observing --for 2h
```

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

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：carol 原本要声明 observing

**命令 2**

```sh
agent-coexist check state
```

stdout：

```text
error  check  state
  file          state.agent-besties.md
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
  entries       2
  Review Queue  alice  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
  Review Queue  bob  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 手工修改：两个 "Review Queue" 合成一节，标题改成 fix 举例的合法名称 review-queue，文件里还没有这个名称的节；alice 和 bob 的 nick 和时间原样搬进这一节。review-queue 不是 locked，不产生锁语义。

手工编辑 `state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## review-queue

alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

**命令 3**

```sh
agent-coexist check state
```

stdout：

```text
ok  check  state
  file          state.agent-besties.md
  type          state
  entries       2
  review-queue  alice  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
  review-queue  bob  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：alice 和 bob 现在都在 review-queue 下

**命令 4**

```sh
agent-coexist state . --as carol --status observing --for 2h
```

stdout：

```text
ok  state  .
  set           observing  carol  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  ## review-queue

  alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
  bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z

  ## observing

  carol 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：重做 carol 原来的声明

<a id="diag-state-unselected"></a>

### 状态内容写在了别的文件名里

编号 `diag-state-unselected` · 观察平台 Windows

locks.agent-besties.md 的 H1 是 # .state，但 state 与 check-state 只认 state.agent-besties.md。文件里 alice 的 locked 重复、dave 倒置的 review、erin 已过期的 review 在改名之前都不给 state 命令：fix 先要求定下哪份文件才是这个目录的 state.agent-besties.md 并重新 check，条目要改时也只有声明它的 agent 本人手工改。先按 W-NAME-STATE 的 fix 确认目录里还没有 state.agent-besties.md（check state 报告 E-FILE-MISSING），再改名；改名后 alice 的重复给出命令。

起始文件 `locks.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z

## review

dave 2026-09-17T11:00:00Z..2026-09-17T10:30:00Z
erin 2026-09-17T07:00:00Z..2026-09-17T08:00:00Z
```

**命令 1**

```sh
agent-coexist check locks
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check state
```

stdout：

```text
error  check  state
  reason        state.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 W-NAME-STATE 的 fix 先查看已有的 state.agent-besties.md：E-FILE-MISSING 说明这个名称还没有文件，改名不会覆盖任何文件

说明：没有需要合并的另一个 state 文件，把 locks.agent-besties.md 改名为 state.agent-besties.md。

手工改名 `locks.agent-besties.md` → `state.agent-besties.md`

**命令 3**

```sh
agent-coexist check state
```

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L5  alice appears twice under locked
         why  the latest end time wins; the next state write that rewrites the locked section (a declaration or removal under it, or expiry cleanup of an entry in it) merges them into one unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)
         fix  optional; to merge them now, that agent declares again, which replaces both with one entry from now (add --for to choose its length), but only when both entries are on clean lines; when either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is and the duplicate stays: inspect the file, delete alice's entries from every such line by hand, keeping each line's other content, then look at the remaining entries and declare again to merge any duplicate that is left or to start one entry from now: agent-coexist state . --as=alice --status=locked
  warn   L9  dave under review ends before it starts
         why  treated as expired: it never counts as a lock; the next state write removes it unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent declares it again: agent-coexist state . --as=dave --status=review
  warn   L10  erin under review expired at 2026-09-17T08:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state . --as=erin --status=review
  entries       3
  locked        alice  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z
  review        dave  2026-09-17T11:00:00Z..2026-09-17T10:30:00Z  expired
  review        erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z  expired
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist state . --as=alice --status=locked
```

stdout：

```text
ok  state  .
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  cleaned       review  dave  2026-09-17T11:00:00Z..2026-09-17T10:30:00Z
  cleaned       review  erin  2026-09-17T07:00:00Z..2026-09-17T08:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z

  ## review

  ```

说明：alice 选择合并，执行 fix 冒号后的命令

**命令 5**

```sh
agent-coexist check state
```

stdout：

```text
ok  check  state
  file          state.agent-besties.md
  type          state
  entries       1
  locked        alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="diag-state-dirty"></a>

### 要删除的声明和无法解析的内容在同一行

编号 `diag-state-dirty` · 观察平台 Windows

alice 要撤掉自己的 locked，但她那一行还有 "rubbish"。文件本身不记录片段是谁写的；这个场景设定 "rubbish" 是 alice 自己写错留下的。bob 也要撤掉 locked，他那一行还有 "junk"，场景没有交代它的来历。删除会重写整行，所以什么也不写（E-STATE-DIRTY）。fix 给两条路：手工删掉自己在那一行的声明，这本身就完成了删除，不需要再执行；或者由写下那些无法解析片段的人把它们改成可解析，再执行 fix 给出的原命令。alice 自己写了 "rubbish"，走第二条，删掉自己留下的片段，原命令保留 --status locked，observing 下的 alice 不受影响；bob 不知道 "junk" 是谁写的，走第一条，只删自己的声明，"junk" 原样留下。

起始文件 `state.agent-besties.md`：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, rubbish
bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, junk

## observing

alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

**命令 1**

```sh
agent-coexist state . --remove alice --status locked
```

stdout：

```text
error  state  .
  reason        alice is still declared on line 5 of the state file in ., mixed with unparseable fragments
  fix           either delete alice's declaration from line 5 by hand, which is the removal itself (re-run then only if alice has more entries this command should remove), or have whoever wrote that line's unparseable fragments make them parseable, then re-run: agent-coexist state . --remove alice --status locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明："rubbish" 是 alice 自己写的，她按 fix 的第二条路删掉自己留下的 ", rubbish"，使这一行可解析。

手工编辑 `state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, junk

## observing

alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

**命令 2**

```sh
agent-coexist state . --remove alice --status locked
```

stdout：

```text
ok  state  .
  removed       locked  alice
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  ## locked

  bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, junk

  ## observing

  alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
  ```

说明：fix 冒号后的原命令

**命令 3**

```sh
agent-coexist state . --remove bob --status locked
```

stdout：

```text
error  state  .
  reason        bob is still declared on line 5 of the state file in ., mixed with unparseable fragments
  fix           either delete bob's declaration from line 5 by hand, which is the removal itself (re-run then only if bob has more entries this command should remove), or have whoever wrote that line's unparseable fragments make them parseable, then re-run: agent-coexist state . --remove bob --status locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 按 fix 的第一条路，手工删掉自己在那一行的声明；这就是删除本身，不再执行原命令。

手工编辑 `state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

junk

## observing

alice 2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

**命令 4**

```sh
agent-coexist check-state .
```

stdout：

```text
warn  check-state  .
  observing     alice  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
  partial       .  W-STATE-ENTRY  L5
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds: agent-coexist check state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：locked 下已经没有 bob，observing 下的 alice 仍在

**命令 5**

```sh
agent-coexist check state
```

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L5  "junk" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  entries       1
  observing     alice  2026-09-17T09:00:00Z..2026-09-17T13:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：剩下的 "junk" 是 W-STATE-ENTRY 警告。它的 fix 说明文件没有记录这个片段是谁写的，写在 bob 那一行也不说明是 bob 写的；bob 不擅自改动它，由写它的人按 fix 改正或删掉，其他人改之前先问是谁写的

<a id="diag-flocked-own"></a>

### 自己的 locked 挡住了自己的写入

编号 `diag-flocked-own` · 观察平台 Windows

alice 先对 agents 声明 locked，再不带 --as 写自己的档案：不带 --as 时任何持有者都会阻断（E-FLOCKED，退出 3）。fix 说明锁是自己的就加上 --as；按 fix 加在任何 -- 之前（这条命令没有 --）后成功。

起始目录 `agents`（空目录）

**命令 1**

```sh
agent-coexist state agents --as alice --status locked --for 1h
```

stdout：

```text
ok  state  agents
  created       agents/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `agents/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
  ```

**命令 2**

```sh
agent-coexist profile agents/alice --section role -m 'Rust implementer.'
```

stdout：

```text
error  profile  agents/alice
  reason        agents is locked by alice until 2026-09-17T11:00:00Z
  fix           if the lock is yours, add --as=alice before any -- separator; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist profile agents/alice --section role -m 'Rust implementer.'
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

说明：锁是 alice 自己的：按 fix 加上 --as=alice（加在任何 -- 之前；这条命令没有 --）。

**命令 3**

```sh
agent-coexist profile agents/alice --section role -m 'Rust implementer.' --as=alice
```

stdout：

```text
ok  profile  agents/alice
  created       agents/alice.agent-besties.md
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer.
  ```
  ````

<a id="diag-flocked-own-separator"></a>

### 自己的 locked 挡住了带 -- 的写入

编号 `diag-flocked-own-separator` · 观察平台 Windows

alice 先对当前目录声明 locked，再不带 --as 写导读。导读路径 -odd-name 以连字符开头，必须写在 -- 之后。E-FLOCKED 的 fix 要求锁是自己的就加上 --as=alice，并且加在任何 -- 之前：加在 -- 之后会被当成路径。按 fix 把 --as=alice 插在 -- 前面后写入成功。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist state . --as alice --status locked --for 1h
```

stdout：

```text
ok  state  .
  created       ./state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
  ```

**命令 2**

```sh
agent-coexist brief -m 'Parser handoff notes.' -- -odd-name
```

stdout：

```text
error  brief  -odd-name
  reason        . is locked by alice until 2026-09-17T11:00:00Z
  fix           if the lock is yours, add --as=alice before any -- separator; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist brief -m 'Parser handoff notes.' -- -odd-name
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

说明：锁是 alice 自己的。按 fix 把 --as=alice 加在 -- 之前，不能加在 -- 之后，否则会变成路径。

**命令 3**

```sh
agent-coexist brief -m 'Parser handoff notes.' --as=alice -- -odd-name
```

stdout：

```text
ok  brief  -odd-name
  created       -odd-name.agent-besties.md
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `-odd-name.agent-besties.md`：

  ````text
  # -odd-name.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Parser handoff notes.
  ```
  ````

<a id="diag-flocked-mistyped-as"></a>

### 持有者自己把 --as 打错了

编号 `diag-flocked-mistyped-as` · 观察平台 Windows

alice 先对 agents 声明 locked，再写自己的档案时把 --as alice 误打成 --as alcie。带 --as 时，只有和它同名的持有者不阻断；alcie 不是持有者，写入被 alice 自己的锁挡住（E-FLOCKED，退出 3）。原命令已经有 --as，fix 不说再加一个，而是说锁是自己的、这条命令里的 --as 却是别人的名字时，把那个 --as 换成持有者的名字。锁确实是 alice 的，打错名字的也是 alice 本人：她把 alcie 改回自己的名字 alice，原命令的其余参数不变，写入成功。这条建议只用于改正自己的名字；不是持有者的人不能借它冒用持有者的名字，应等锁过期或请持有者释放，见 [diag-flocked-other](#diag-flocked-other)。

起始目录 `agents`（空目录）

**命令 1**

```sh
agent-coexist state agents --as alice --status locked --for 1h
```

stdout：

```text
ok  state  agents
  created       agents/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `agents/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T11:00:00Z
  ```

说明：alice 声明 locked

**命令 2**

```sh
agent-coexist profile agents/alice --as alcie --section role -m 'Rust implementer.'
```

stdout：

```text
error  profile  agents/alice
  reason        agents is locked by alice until 2026-09-17T11:00:00Z
  fix           if the lock is yours and the --as in this command names someone else, rerun it with --as=alice in place of that --as; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist profile agents/alice --as alcie --section role -m 'Rust implementer.'
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

说明：alice 写自己的档案，把 --as alice 打成了 --as alcie

说明：锁是 alice 自己的，alcie 是她打错的名字。按 fix 把 fix 冒号后原命令里的 `--as alcie` 换成 `--as=alice`，其余参数不变。

**命令 3**

```sh
agent-coexist profile agents/alice --as=alice --section role -m 'Rust implementer.'
```

stdout：

```text
ok  profile  agents/alice
  created       agents/alice.agent-besties.md
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer.
  ```
  ````

说明：改正名字后重做原来的写入

<a id="diag-flocked-other"></a>

### 别人的 locked 挡住了写入

编号 `diag-flocked-other` · 观察平台 Windows

carol 锁着根目录到 12:00，alice 锁着 docs/agents 到 11:00；根目录的状态文件还有一个写错的片段。bob 写导读被拒（退出 3），also-locked 列出另一位持有者，partial 行说明锁扫描读到了不完整的一行。bob 不是持有者，按 fix 等锁过期（这里把时钟设到 12:00）后执行 fix 给出的原命令。写入成功后锁扫描仍不完整，fix 给出打开那个状态文件的只读命令；bob 照做，确认那一行没读懂的片段是什么。

起始文件 `state.agent-besties.md`：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

carol 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, rubbish
```

起始文件 `docs/agents/state.agent-besties.md`：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

alice 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

**命令 1**

```sh
agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'
```

stdout：

```text
error  brief  docs/agents/handoff
  reason        . is locked by carol until 2026-09-17T12:00:00Z
  fix           if the lock is yours and the --as in this command names someone else, rerun it with --as=carol in place of that --as; otherwise wait for it to expire or ask its holder to release it, then retry: agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'
  also-locked   alice  until 2026-09-17T11:00:00Z
  partial       .  W-STATE-ENTRY  L7
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T12:00:00Z agent-coexist brief docs/agents/handoff --as bob -m 'Parser handoff notes.'
```

stdout：

```text
warn  brief  docs/agents/handoff
  created       docs/agents/handoff.agent-besties.md
  description   set
  partial       .  W-STATE-ENTRY  L7
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `docs/agents/handoff.agent-besties.md`：

  ````text
  # handoff.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Parser handoff notes.
  ```
  ````

说明：12:00，carol 和 alice 的锁都已过期，导读已写入

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T12:00:00Z agent-coexist check state
```

stdout：

```text
warn  check  state
  file          state.agent-besties.md
  type          state
  warn   L7  "rubbish" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  warn   L7  carol under locked expired at 2026-09-17T12:00:00Z
         why  expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)
         fix  if still needed, that agent renews it: agent-coexist state . --as=carol --status=locked
  entries       1
  locked        carol  2026-09-17T09:00:00Z..2026-09-17T12:00:00Z  expired
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 打开锁扫描没读全的状态文件，不重做写入

说明：check 指出第 7 行 locked 下没读懂的片段就是 "rubbish"，它不是 `<nick> <start>..<until>` 形式，不构成一条锁；carol 与 alice 的锁到 12:00 都已到期。按 W-STATE-ENTRY 的 fix，文件没有记录这个片段是谁写的，写在 carol 那一行也不说明是 carol 写的；bob 只是确认它不是锁，不去改它，由写它的人改正或删掉。

<a id="native-lock-reads-windows"></a>

### 文件被另一进程加锁：读命令 E-LOCKED，扫描标为 busy

编号 `native-lock-reads-windows` · 观察平台 Windows（平台专属场景）

生成器用 LockFileEx 对 standup、onboarding 和 state 文件持有独占锁（与另一个 agent-coexist 写入时相同的锁）。单文件读命令失败退出 3，目录扫描继续并把文件标为 busy。锁释放后按建议重试。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

起始文件 `onboarding.agent-besties.md`：

````text
# onboarding.brief

```md
Start here.
```
````

起始文件 `state.agent-besties.md`：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## observing

carol 2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
```

**命令 1**

```sh
agent-coexist check-state .
```

stdout：

```text
ok  check-state  .
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：对照：加锁前 state 文件可以正常读取。

说明：生成器现在以 LockFileEx 独占锁定三个文件；以下命令运行期间锁一直持有。

**命令 2**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist check standup
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist lookfor-comment standup --by alice
```

stdout：

```text
error  lookfor-comment  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist lookfor-comment standup --by alice
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist read-brief onboarding
```

stdout：

```text
error  read-brief  onboarding
  reason        file is locked by another process
  fix           retry shortly: agent-coexist read-brief onboarding
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist list .
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 6**

```sh
agent-coexist read-notification . --as bob --since 1h
```

stdout：

```text
warn  read-notification  .
  as            bob
  since         2026-09-17T09:00:00Z
  skipped     .  busy
  skipped     ./onboarding  busy
  skipped     ./standup  busy
  fix           some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one: agent-coexist check state; agent-coexist check ./onboarding; agent-coexist check ./standup
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 7**

```sh
agent-coexist check-state .
```

stdout：

```text
warn  check-state  .
  skipped       .  busy
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds: agent-coexist check state
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：生成器关闭锁句柄，然后按三个单文件读命令和 list 报告的 fix 行重试，并重跑三个标出 busy 的扫描。

**命令 8**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check standup'
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 check 报告的 fix 行重试（Git Bash）。

**命令 9**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist lookfor-comment standup --by alice'
```

stdout：

```text
ok  lookfor-comment  standup
  L3  #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 lookfor-comment 报告的 fix 行重试。

**命令 10**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist read-brief onboarding'
```

stdout：

```text
ok  read-brief  onboarding
  description
    | Start here.
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 read-brief 报告的 fix 行重试。

**命令 11**

```sh
agent-coexist read-notification . --as bob --since 1h
```

stdout：

```text
ok  read-notification  .
  as            bob
  since         2026-09-17T09:00:00Z
  state       2026-09-17T09:00:00Z  .  observing  carol  until 2026-09-17T18:00:00Z
  mention     2026-09-17T09:00:00Z  ./standup #1  alice  @bob please post your status here.
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑扫描：之前 busy 的文件这次都被读到。

**命令 12**

```sh
agent-coexist check-state .
```

stdout：

```text
ok  check-state  .
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑扫描：这次读到 carol 的 observing 声明。

**命令 13**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check ./onboarding; agent-coexist check ./standup; agent-coexist check ./state'
```

stdout：

```text
ok  check  ./onboarding
  file          ./onboarding.agent-besties.md
  type          brief
  description   Start here.
  entries       0
  fresh         0
  shifted       0
  stale         0
ok  check  ./standup
  file          ./standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
ok  check  ./state
  file          ./state.agent-besties.md
  type          state
  entries       1
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 list 报告的 fix 行逐个重读之前 busy 的叶子。

**命令 14**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  onboarding                  brief
  standup                     post
  state                       state
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原来的 list：之前 busy 的三个叶子都显示出类型，没有 fix 行。

<a id="native-lock-writes-windows"></a>

### 文件被另一进程加锁：写命令 E-LOCKED，其他写入遇到 busy 状态文件

编号 `native-lock-writes-windows` · 观察平台 Windows（平台专属场景）

standup、onboarding 和 state 文件被 LockFileEx 独占锁定。post/subscribe 最多排队 AGENT_COEXIST_LOCK_WAIT_MS（此处 300 ms），brief/state 快速失败；profile 写入自身文件成功，但无法读取目录 state 文件，没有核对有没有他人的 locked 声明。锁释放后先按 profile 报告的建议核对 locked 声明，再重试失败的写入。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

起始文件 `onboarding.agent-besties.md`：

````text
# onboarding.brief

```md
Start here.
```
````

起始文件 `state.agent-besties.md`：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## observing

carol 2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
```

说明：生成器以 LockFileEx 独占锁定三个文件；以下命令运行期间锁一直持有。

**命令 1**

```sh
AGENT_COEXIST_LOCK_WAIT_MS=300 agent-coexist post standup --as bob -m 'status: parser done'
```

stdout：

```text
error  post  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist post standup --as bob -m 'status: parser done'
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_LOCK_WAIT_MS=300 agent-coexist subscribe standup --as bob
```

stdout：

```text
error  subscribe  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist subscribe standup --as bob
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist brief onboarding -m 'Start with the README.'
```

stdout：

```text
error  brief  onboarding
  reason        file is locked by another process
  fix           retry shortly: agent-coexist brief onboarding -m 'Start with the README.'
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 4**

```sh
agent-coexist state --as bob --status observing
```

stdout：

```text
error  state  .
  reason        file is locked by another process
  fix           retry shortly: agent-coexist state --as bob --status observing
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 5**

```sh
agent-coexist profile bob --as bob --section role -m 'parser owner'
```

stdout：

```text
warn  profile  bob
  created       bob.agent-besties.md
  section       role  set
  skipped       .  busy
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there: agent-coexist check state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `bob.agent-besties.md`：

  ````text
  # bob.agent-profile

  ## role

  ```md
  parser owner
  ```
  ````

说明：profile 自己的文件已写入；目录的 state 文件被锁，报告标为 busy，没有核对 locked 声明。

说明：锁释放后，先按 profile 报告的 fix 行核对 locked 声明，再按各报告 fix 行重试写入；原评论正文保持不变。

**命令 6**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check state'
```

stdout：

```text
ok  check  state
  file          state.agent-besties.md
  type          state
  entries       1
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 profile 报告的 fix 行，在再次写入前核对 locked 声明。

**命令 7**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist post standup --as bob -m '\''status: parser done'\'''
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  status: parser done
  ```
  ````

说明：按 post 报告的 fix 行重试（Git Bash）。

**命令 8**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist subscribe standup --as bob'
```

stdout：

```text
ok  subscribe  standup
  subscribers   bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  - subscribers: bob

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  status: parser done
  ```
  ````

说明：按 subscribe 报告的 fix 行重试。

**命令 9**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist brief onboarding -m '\''Start with the README.'\'''
```

stdout：

```text
ok  brief  onboarding
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  ```md
  Start with the README.
  ```
  ````

说明：按 brief 报告的 fix 行重试。

**命令 10**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist state --as bob --status observing'
```

stdout：

```text
ok  state  .
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  carol 2026-09-17T09:00:00Z..2026-09-17T18:00:00Z, bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

说明：按 state 报告的 fix 行重试。

查看文件 `standup.agent-besties.md`：

````text
# standup.post

- subscribers: bob

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```

## #2 bob (2026-09-17T10:00:00Z)

```md
status: parser done
```
````

<a id="native-inuse-windows"></a>

### Windows：文件被其他程序独占打开（E-INUSE）

编号 `native-inuse-windows` · 观察平台 Windows（平台专属场景）

生成器以共享模式 0 打开 standup.agent-besties.md，模拟编辑器或其他工具独占文件。关闭后按建议重试 check 和 post。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

说明：生成器以共享模式 0 打开 standup.agent-besties.md；以下命令运行期间句柄一直打开。

**命令 1**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  reason        file is in use by another program
  fix           close it, then retry: agent-coexist check standup
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 2**

```sh
AGENT_COEXIST_LOCK_WAIT_MS=300 agent-coexist post standup --as bob -m 'status: parser done'
```

stdout：

```text
error  post  standup
  reason        file is in use by another program
  fix           close it, then retry: agent-coexist post standup --as bob -m 'status: parser done'
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 3**

```sh
agent-coexist list .
```

stdout：

```text
warn  list  .
  ignore        none
  standup                     busy
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read: agent-coexist check ./standup
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：生成器关闭独占句柄（相当于关闭编辑器），然后执行 fix 行中的重试命令。

**命令 4**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check standup'
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 check 报告的 fix 行重试（Git Bash）。

**命令 5**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist post standup --as bob -m '\''status: parser done'\'''
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  status: parser done
  ```
  ````

说明：按 post 报告的 fix 行重试。

**命令 6**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check ./standup'
```

stdout：

```text
ok  check  ./standup
  file          ./standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  @bob please post your status here.
  latest        #2  bob  2026-09-17T10:00:00Z  status: parser done
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 list 报告的 fix 行重读之前 busy 的叶子 ./standup。

**命令 7**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原来的 list：standup 不再是 busy，没有 fix 行。

<a id="native-delete-pending-windows"></a>

### Windows：文件处于待删除状态，报告为普通 I/O 失败

编号 `native-delete-pending-windows` · 观察平台 Windows（平台专属场景）

另一程序已对 standup.agent-besties.md 设置删除标记但句柄未关闭：文件名仍可见，任何新打开都被拒绝。这不是锁，不会给出 E-LOCKED。句柄关闭后删除完成，文件不再存在。按 fix 行列出该目录，确认没有 standup 后，alice 从自己已知的备份写回原位置（这是文件所有者自己的操作，不是 CLI 的建议），再完成原来的 check。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

说明：另一程序对 standup.agent-besties.md 设置了删除标记，句柄仍打开。

**命令 1**

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  reason        cannot open standup.agent-besties.md: 拒绝访问。 (os error 5)
  fix           check the permissions of standup.agent-besties.md and of the directories on its path, or wait until another program stops using, deleting or replacing it, then retry: agent-coexist check standup
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：持有句柄的程序关闭句柄，删除标记生效，文件被删除。

这一操作造成的文件变化：

- 删除 `standup.agent-besties.md`

**命令 2**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check standup'
```

stdout：

```text
error  check  standup
  reason        standup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 行重试：文件已不存在，check 报告找不到它（check 只读，不改变文件）。

**命令 3**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist list .'
```

stdout：

```text
ok  list  .
  ignore        none
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行列出该目录下的协作文件：standup 已不在。

说明：list 显示该目录没有 standup。alice 把这份帖子从自己已知的备份写回原位置（文件所有者自己的操作，由 list 的结果触发，不是 CLI 的 fix 建议）。

手工新建 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 4**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check standup'
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：再次执行原来的 check，原来的任务完成。

<a id="native-lock-linux"></a>

### Linux：flock 独占锁下的读、写和扫描

编号 `native-lock-linux` · 观察平台 Linux（平台专属场景）

在 WSL 中，一个 `flock -x` 进程持有 standup.agent-besties.md 的独占锁（与 agent-coexist 写入时的锁相同）。释放后按建议重试 check 和 post。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post standup --as alice -m '@bob please post your status here.'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

**命令 2**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
flock -x standup.agent-besties.md -c 'echo held; read _'
```

说明：另一进程取得独占锁；输出 held 后一直持有，直到它的 stdin 收到换行。

命令 2 仍在运行，它的 stdout 此时已出现就绪文字 `held`；下面的步骤都在这之后执行。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist check standup
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_LOCK_WAIT_MS=300 agent-coexist post standup --as bob -m status
```

stdout：

```text
error  post  standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist post standup --as bob -m status
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list .
```

stdout：

```text
warn  list  .
  ignore        none
  standup                     busy
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read: agent-coexist check ./standup
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
held
```

stderr：（空，0 字节）

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：锁持有进程收到换行后退出，锁释放。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check standup'
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T10:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 check 报告的 fix 行重试。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist post standup --as bob -m status'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  status
  ```
  ````

说明：按 post 报告的 fix 行重试。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check ./standup'
```

stdout：

```text
ok  check  ./standup
  file          ./standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  @bob please post your status here.
  latest        #2  bob  2026-09-17T10:00:00Z  status
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 list 报告的 fix 行重读之前 busy 的叶子 ./standup。

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原来的 list：standup 不再是 busy，没有 fix 行。

<a id="native-permission-file-windows"></a>

### Windows：没有读取权限的协作文件

编号 `native-permission-file-windows` · 观察平台 Windows（平台专属场景）

room/standup.agent-besties.md 对当前用户设置了拒绝读取。读和写都以普通 I/O 失败结束（退出 1），建议检查权限后重试。恢复权限后执行两条 fix 行中的命令。

起始文件 `room/standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

手工去掉 `room/standup.agent-besties.md` 的读权限（icacls 为当前用户添加拒绝读取 (R) 的 ACE；原 ACL 已另存）

**命令 1**

```sh
agent-coexist check room/standup
```

stdout：

```text
error  check  room/standup
  reason        cannot open room/standup.agent-besties.md: 拒绝访问。 (os error 5)
  fix           check the permissions of room/standup.agent-besties.md and of the directories on its path, or wait until another program stops using, deleting or replacing it, then retry: agent-coexist check room/standup
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist post room/standup --as bob -m status
```

stdout：

```text
error  post  room/standup
  reason        cannot open room/standup.agent-besties.md: 拒绝访问。 (os error 5)
  fix           check the permissions of room/standup.agent-besties.md and of the directories on its path, or wait until another program stops using, deleting or replacing it, then retry: agent-coexist post room/standup --as bob -m status
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

恢复 `room/standup.agent-besties.md` 的读权限（icacls 恢复之前另存的原 ACL）

**命令 3**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist check room/standup'
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：恢复读取权限后按 check 报告的 fix 行重试（Git Bash）。

**命令 4**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist post room/standup --as bob -m status'
```

stdout：

```text
ok  post  room/standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  status
  ```
  ````

说明：按 post 报告的 fix 行重试，原来的评论写入。

<a id="native-permission-dir-windows"></a>

### Windows：没有读取权限的目录

编号 `native-permission-dir-windows` · 观察平台 Windows（平台专属场景）

room 目录对当前用户拒绝读取。扫描跳过它并告警；把它直接作为 check-state 的目录时，还会报告规范化失败。恢复权限后重跑这些扫描。

起始文件 `room/standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

起始文件 `room/state.agent-besties.md`：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## observing

carol 2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
```

起始文件 `notes.agent-besties.md`：

````text
# notes.post

## #1 carol (2026-09-17T09:00:00Z)

```md
@bob see room/standup
```
````

手工去掉 `room` 的读权限（icacls 为当前用户添加拒绝读取 (R) 的 ACE；原 ACL 已另存）

**命令 1**

```sh
agent-coexist list .
```

stdout：

```text
warn  list  .
  ignore        none
  notes                       post
  skipped       ./room  unreadable
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read (first make each directory marked unreadable both readable and enterable: it needs read and search permission): agent-coexist list ./room
  total         2
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 2**

```sh
agent-coexist read-notification . --as bob --since 1h
```

stdout：

```text
warn  read-notification  .
  as            bob
  since         2026-09-17T09:00:00Z
  mention     2026-09-17T09:00:00Z  ./notes #1  carol  @bob see room/standup
  skipped     ./room  unreadable
  total         2
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 3**

```sh
agent-coexist check-state .
```

stdout：

```text
warn  check-state  .
  skipped       ./room  unreadable
  total         1
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 4**

```sh
agent-coexist check-state room
```

stdout：

```text
warn  check-state  room
  skipped       room  unreadable
  canonicalize  room  failed  拒绝访问。 (os error 5)
  fix           this command could not resolve the real path of each directory marked canonicalize; first restore that each of those paths exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission), then run this command again
  total         1
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

恢复 `room` 的读权限（icacls 恢复之前另存的原 ACL）

**命令 5**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  notes                       post
  room/
    standup                   post
    state                     state
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：恢复权限后重跑扫描：room 下的帖子被列出。

**命令 6**

```sh
agent-coexist read-notification . --as bob --since 1h
```

stdout：

```text
ok  read-notification  .
  as            bob
  since         2026-09-17T09:00:00Z
  mention     2026-09-17T09:00:00Z  ./notes #1  carol  @bob see room/standup
  state       2026-09-17T09:00:00Z  ./room  observing  carol  until 2026-09-17T18:00:00Z
  mention     2026-09-17T09:00:00Z  ./room/standup #1  alice  @bob please post your status here.
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 7**

```sh
agent-coexist check-state .
```

stdout：

```text
ok  check-state  .
  room/
    observing     carol  2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：恢复权限后再扫描当前目录：room 不再被跳过。

**命令 8**

```sh
agent-coexist check-state room
```

stdout：

```text
ok  check-state  room
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：恢复权限后再扫描同一目录：carol 的 observing 声明被读到。

<a id="native-permission-linux"></a>

### Linux：权限位造成的读取失败与跳过

编号 `native-permission-linux` · 观察平台 Linux（平台专属场景）

WSL 中以普通用户运行（权限位生效）。先让文件 chmod 000；再让它所在的 room 目录 chmod 000，此时文件本身权限正常，挡住访问的是路径上的目录。帖子和 state 声明写于 09:00，在 `--since 1h` 的窗口内。每一步恢复权限后都重跑原来的命令。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z agent-coexist post room/standup --as alice -m 'hi @bob'
```

stdout：

```text
ok  post  room/standup
  created       room/
  created       room/standup.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `room`
- 新建 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  hi @bob
  ```
  ````

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z agent-coexist state room --as carol --status observing
```

stdout：

```text
ok  state  room
  created       room/state.agent-besties.md
  set           observing  carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `room/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  carol 2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  ```

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
chmod 000 room/standup.agent-besties.md
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `room/standup.agent-besties.md` 的权限位：0644 → 0000

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check room/standup
```

stdout：

```text
error  check  room/standup
  reason        cannot open room/standup.agent-besties.md: Permission denied (os error 13)
  fix           check the permissions of room/standup.agent-besties.md and of the directories on its path, then retry: agent-coexist check room/standup
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
chmod 644 room/standup.agent-besties.md
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `room/standup.agent-besties.md` 的权限位：0000 → 0644

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check room/standup'
```

stdout：

```text
ok  check  room/standup
  file          room/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi @bob
  latest        #1  alice  2026-09-17T09:00:00Z  hi @bob
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：恢复权限位后按 fix 行重试。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
chmod 000 room
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `room` 的权限位：0755 → 0000

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：文件已恢复为 644；现在去掉它所在目录的全部权限。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check room/standup
```

stdout：

```text
error  check  room/standup
  reason        cannot read room/standup.agent-besties.md: Permission denied (os error 13)
  fix           check the permissions of room/standup.agent-besties.md and of the directories on its path, then retry: agent-coexist check room/standup
```

stderr：（空，0 字节）

退出码：`1`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：目录不可进入时，读取文件元数据失败（不是 NotFound）。

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list .
```

stdout：

```text
warn  list  .
  ignore        none
  skipped       ./room  unreadable
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read (first make each directory marked unreadable both readable and enterable: it needs read and search permission): agent-coexist list ./room
  total         1
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist read-notification . --as bob --since 1h
```

stdout：

```text
warn  read-notification  .
  as            bob
  since         2026-09-17T09:00:00Z
  skipped     ./room  unreadable
  total         1
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

**命令 11**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check-state room
```

stdout：

```text
warn  check-state  room
  skipped       room  unreadable
  total         1
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `room` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：自身不可进入的 room 仍能被规范化（只需要它的上级目录可搜索），所以这里只有 skipped 行。规范化失败见 [native-canonicalize-linux](#native-canonicalize-linux)。

**命令 12**（在 `⟨cwd⟩` 中执行）

```sh
chmod 755 room
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `room` 的权限位：0000 → 0755

说明：恢复目录权限。

**命令 13**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check room/standup'
```

stdout：

```text
ok  check  room/standup
  file          room/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi @bob
  latest        #1  alice  2026-09-17T09:00:00Z  hi @bob
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 check 报告的 fix 行重试原来的 check。

**命令 14**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  room/
    standup                   post
    state                     state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑扫描：room 不再被跳过。

**命令 15**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist read-notification . --as bob --since 1h
```

stdout：

```text
ok  read-notification  .
  as            bob
  since         2026-09-17T09:00:00Z
  state       2026-09-17T09:00:00Z  ./room  observing  carol  until 2026-09-17T17:00:00Z
  mention     2026-09-17T09:00:00Z  ./room/standup #1  alice  hi @bob
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：恢复后重跑：之前被跳过的 alice 的提及和 carol 的声明都在 09:00，落在窗口内，这次都被读到。

**命令 16**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check-state room
```

stdout：

```text
ok  check-state  room
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：恢复权限后再扫描：读到 carol 的 observing 声明。

<a id="native-canonicalize-linux"></a>

### Linux：当前目录的上级失去搜索权限，目录无法规范化

编号 `native-canonicalize-linux` · 观察平台 Linux（平台专属场景）

Linux 上规范化（转成不含 `..` 和符号链接的绝对路径）要求路径上每一级上级目录都可搜索（x 位）；目标目录自身不可进入时仍能规范化（见 [native-permission-linux](#native-permission-linux)）。场景在 a/b 中运行 check-state room：先进入 a/b，再去掉 a 的搜索权限（chmod 600）。相对路径 room 仍能从当前目录打开，绝对路径却经过 a，无法解析。canonicalize 行带上系统错误原因；fix 要求先让标为 canonicalize 的路径存在、并且路径上每一级目录（含祖先）都可打开（读权限和遍历/搜索权限），再重跑原命令。这里缺的是 a 的搜索权限；按这条 fix 恢复它后，在同一目录重跑原来的 check-state。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir -p a/b/room'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `a`
- 新建目录 `a/b`
- 新建目录 `a/b/room`

说明：准备：工作目录下的 a/b/room。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z agent-coexist state a/b/room --as carol --status observing
```

stdout：

```text
ok  state  a/b/room
  created       a/b/room/state.agent-besties.md
  set           observing  carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `a/b/room/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  carol 2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  ```

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'w=$PWD; cd a/b && chmod 600 "$w/a" && agent-coexist check-state room'
```

stdout：

```text
warn  check-state  room
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  skipped       room  unreadable
  canonicalize  room  failed  Permission denied (os error 13)
  fix           this command could not resolve the real path of each directory marked canonicalize; first restore that each of those paths exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission), then run this command again
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `a` 的权限位：0755 → 0600

观察不完整：这时生成器无法列出 `a` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：先进入 a/b，再去掉 a 的搜索权限，然后在 a/b 中运行 check-state room。shell 退出后 a 保持 600。carol 的 observing 行来自 room 自己的 state 文件，它沿相对路径 room 读取；扫描 room 下面的子目录时从 room 的绝对路径出发、要经过 a，无法列出 room，所以 skipped 行表示 room 下面没有被扫描，不是 room 自己的 state 文件读不到。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
chmod 755 a
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `a` 的权限位：0600 → 0755

说明：按 fix 行先让标为 canonicalize 的路径存在、并且路径上每一级（含祖先）都可打开（读权限和遍历/搜索权限）：这里缺的是 a 的搜索权限，恢复它。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'cd a/b && agent-coexist check-state room'
```

stdout：

```text
ok  check-state  room
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 在同一目录重跑原来的 check-state：不再有规范化失败，读到 carol 的声明。

<a id="native-lock-scan-manual-linux"></a>

### Linux：目录名无法写成单行命令时，写入后核对与扫描的手工后续

编号 `native-lock-scan-manual-linux` · 观察平台 Linux（平台专属场景）

目录名含换行（`$'a\nb'`）。carol 在其中声明 observing，alice 在其中的帖子 s 里提到 @bob（都在 09:00）。另一进程以 `flock -x` 同时锁住这个目录的 state 文件和帖子 s。bob 写自己的 profile：写入成功，但 locked 核对读不到 state 文件；随后 check-state 和 read-notification 也都把被锁的文件标为 busy，list 把这两个叶子标为 busy。目录无法无损写成单行，四个 fix 行都改为手工说明要对哪些文件运行 check。锁释放后按说明手工运行这些 check，再重跑原来的 check-state、read-notification 和 list。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z bash -c 'mkdir $'\''a\nb'\'' && agent-coexist state $'\''a\nb'\'' --as carol --status observing'
```

stdout：

```text
ok  state  a b
  created       a b/state.agent-besties.md
  set           observing  carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'a\nb'`
- 新建 `$'a\nb/state.agent-besties.md'`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  carol 2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  ```

说明：准备（09:00）：目录名含换行，carol 在其中声明 observing。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z bash -c 'agent-coexist post $'\''a\nb/s'\'' --as alice -m '\''@bob please review'\'''
```

stdout：

```text
ok  post  a b/s
  created       a b/s.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `$'a\nb/s.agent-besties.md'`：

  ````text
  # s.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please review
  ```
  ````

说明：准备（09:00）：alice 在同一目录的帖子 s 里提到 bob。

**命令 3**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
bash -c 'exec 3<$'\''a\nb/state.agent-besties.md'\'' 4<$'\''a\nb/s.agent-besties.md'\'' && flock -x 3 && flock -x 4 && echo held && read _'
```

说明：另一进程以 flock -x 锁住这个目录的 state 文件和帖子 s；输出 held 后一直持有，直到它的 stdin 收到换行。

命令 3 仍在运行，它的 stdout 此时已出现就绪文字 `held`；下面的步骤都在这之后执行。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist profile $'\''a\nb/bob'\'' --as bob --section role -m '\''parser owner'\'''
```

stdout：

```text
warn  profile  a b/bob
  created       a b/bob.agent-besties.md
  section       role  set
  skipped       a b  busy
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there; run by hand agent-coexist check <dir>/state for each <dir> in a skipped or partial row; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `$'a\nb/bob.agent-besties.md'`：

  ````text
  # bob.agent-profile

  ## role

  ```md
  parser owner
  ```
  ````

说明：profile 已写入；state 文件被锁，locked 核对不完整。目录名含换行，fix 行只能说明手工要运行的 check。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check-state $'\''a\nb'\'''
```

stdout：

```text
warn  check-state  a b
  skipped       a b  busy
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds by hand with agent-coexist check <dir>/state for each <dir> in a skipped or partial row; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：check-state 读不到被锁的 state 文件，标为 busy；fix 行说明手工对它运行 check。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist read-notification $'\''a\nb'\'' --as bob --since 1h'
```

stdout：

```text
warn  read-notification  a b
  as            bob
  since         2026-09-17T09:00:00Z
  skipped     a b  busy
  skipped     a b/s  busy
  fix           some files were not read in full (skipped or partial rows above), so records in them may be missing here; check shows each file's type, when its first line gives one, and any problem left in it; an E-TYPE row, if any, means that directory's state file has another type, so no state declarations were read from it even when check reports no problem; before concluding nothing is new, check each one, by hand with agent-coexist check <path> for each <path> in a skipped or partial row (<path>/state when the row names a directory); no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：read-notification 读不到被锁的 state 文件和帖子，两者都标为 busy；fix 行说明手工对每个被跳过的文件运行 check。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb'\'''
```

stdout：

```text
warn  list  a b
  ignore        none
  bob                         agent-profile
  s                           busy
  state                       busy
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read, by hand with agent-coexist check <path> for each ? or busy leaf, <path> being DIR, the directory rows above the leaf and its name, joined by /; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：list 读不到被锁的 state 文件和帖子 s 的类型，两个叶子都标为 busy；fix 行说明手工对每个 busy 叶子运行 check（路径由 DIR、上方目录行和叶子名用 / 连接）。

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
held
```

stderr：（空，0 字节）

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：锁持有进程收到换行后退出，两把锁都释放。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/state'\'''
```

stdout：

```text
ok  check  a b/state
  file          a b/state.agent-besties.md
  type          state
  entries       1
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行的说明，手工对 busy 的 state 文件运行 check（profile 与 check-state 的说明写作 `check <dir>/state`，这里的 `<dir>` 是 `a b`，即 `$'a\nb'`）：读到 carol 的 observing 声明，没有 locked。

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/s'\'''
```

stdout：

```text
ok  check  a b/s
  file          a b/s.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please review
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please review
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 read-notification 和 list 的 fix 行，手工对被跳过的帖子 s 运行 check。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check-state $'\''a\nb'\'''
```

stdout：

```text
ok  check-state  a b
  observing     carol  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原来的 check-state：不再有 busy 行。

**命令 11**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist read-notification $'\''a\nb'\'' --as bob --since 1h'
```

stdout：

```text
ok  read-notification  a b
  as            bob
  since         2026-09-17T09:00:00Z
  state       2026-09-17T09:00:00Z  a b  observing  carol  until 2026-09-17T17:00:00Z
  mention     2026-09-17T09:00:00Z  a b/s #1  alice  @bob please review
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原来的 read-notification：09:00 的提及和声明落在窗口内，这次被读到。

**命令 12**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb'\'''
```

stdout：

```text
ok  list  a b
  ignore        none
  bob                         agent-profile
  s                           post
  state                       state
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原来的 list：两个叶子都显示出类型，没有 fix 行。

<a id="native-state-scan-unlistable-linux"></a>

### Linux：state 写入后的冲突检查遇到无法列出的子目录

编号 `native-state-scan-unlistable-linux` · 观察平台 Linux（平台专属场景）

t/sub 和 `$'a\nb/sub'` 里各有 a 在 09:00 的 observing 声明，然后两个 sub 都 `chmod 000`（无法列出）。b 分别在 t 和 `$'a\nb'` 声明 observing：声明写入，但冲突检查读不到 sub，fix 行要求先让标为 unreadable 的目录既可读又可进入（读权限和搜索权限），再对它运行 check-state，不要重新声明。t/sub 可以写成可复制的命令；`$'a\nb/sub'` 只能手工执行。对 `$'a\nb'` 运行 list 同样跳过 sub，fix 行也只能手工说明要列出哪个目录。恢复权限后按各 fix 行执行，再重跑原来的 list。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir -p t/sub $'\''a\nb/sub'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'a\nb'`
- 新建目录 `$'a\nb/sub'`
- 新建目录 `t`
- 新建目录 `t/sub`

说明：准备：t/sub 和含换行的 `$'a\nb/sub'`。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z agent-coexist state t/sub --as a --status observing
```

stdout：

```text
ok  state  t/sub
  created       t/sub/state.agent-besties.md
  set           observing  a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `t/sub/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  a 2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  ```

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
AGENT_COEXIST_NOW=2026-09-17T09:00:00Z bash -c 'agent-coexist state $'\''a\nb/sub'\'' --as a --status observing'
```

stdout：

```text
ok  state  a b/sub
  created       a b/sub/state.agent-besties.md
  set           observing  a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `$'a\nb/sub/state.agent-besties.md'`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  a 2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  ```

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'chmod 000 t/sub $'\''a\nb/sub'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `$'a\nb/sub'` 的权限位：0755 → 0000
- 修改 `t/sub` 的权限位：0755 → 0000

观察不完整：这时生成器无法列出 `t/sub`、`$'a\nb/sub'` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：去掉两个 sub 的全部权限：它们无法被列出，也无法进入。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist state t --as b --status observing
```

stdout：

```text
warn  state  t
  created       t/state.agent-besties.md
  set           observing  b  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  skipped       t/sub  unreadable
  fix           the declaration above is written, but the conflict check could not read everything it needed (the rows above that start with skipped or partial), so a conflicting lock may have been missed; without declaring again, look at what it could not read and agree with any holder named there (first make each directory marked unreadable both readable and enterable: it needs read and search permission): agent-coexist check-state t/sub
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `t/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  b 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

观察不完整：这时生成器无法列出 `t/sub`、`$'a\nb/sub'` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：b 的声明已写入；冲突检查无法列出 t/sub。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist state $'\''a\nb'\'' --as b --status observing'
```

stdout：

```text
warn  state  a b
  created       a b/state.agent-besties.md
  set           observing  b  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  skipped       a b/sub  unreadable
  fix           the declaration above is written, but the conflict check could not read everything it needed (the rows above that start with skipped or partial), so a conflicting lock may have been missed; without declaring again, look at what it could not read and agree with any holder named there (first make each directory marked unreadable both readable and enterable: it needs read and search permission); run by hand agent-coexist check-state <dir> for each <dir> marked unreadable; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `$'a\nb/state.agent-besties.md'`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  b 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

观察不完整：这时生成器无法列出 `t/sub`、`$'a\nb/sub'` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：b 的声明已写入；冲突检查无法列出 `$'a\nb/sub'`，这个路径无法写成单行命令，fix 行改为手工说明。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb'\'''
```

stdout：

```text
warn  list  a b
  ignore        none
  state                       state
  skipped       a b/sub  unreadable
  fix           some leaves above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; read each one again to see what it holds or why it cannot be read (first make each directory marked unreadable both readable and enterable: it needs read and search permission), by hand with agent-coexist list <dir> for each <dir> marked unreadable; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
  total         2
```

stderr：（空，0 字节）

退出码：`0`

在能观察的范围内，这条命令没有观察到文件变化。

观察不完整：这时生成器无法列出 `t/sub`、`$'a\nb/sub'` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：list 无法列出 `$'a\nb/sub'`，标为 skipped；路径无法写成单行命令，fix 行手工说明先让它既可读又可进入，再对它运行 list。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'chmod 755 t/sub $'\''a\nb/sub'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `$'a\nb/sub'` 的权限位：0000 → 0755
- 修改 `t/sub` 的权限位：0000 → 0755

说明：按 fix 行先让标为 unreadable 的两个目录恢复读权限和搜索权限。

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check-state t/sub'
```

stdout：

```text
ok  check-state  t/sub
  inherited     observing  b  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z  t
  observing     a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令：读到 a 的 observing 声明；b 的声明已在上一步写入，没有重新声明。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check-state $'\''a\nb/sub'\'''
```

stdout：

```text
ok  check-state  a b/sub
  inherited     observing  b  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z  a b
  observing     a  2026-09-17T09:00:00Z..2026-09-17T17:00:00Z
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按手工说明，对标为 unreadable 的目录（显示为 `a b/sub`，即 `$'a\nb/sub'`）运行 check-state；没有重新声明。

**命令 11**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb/sub'\'''
```

stdout：

```text
ok  list  a b/sub
  ignore        none
  state                       state
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 list 的手工说明，对标为 unreadable 的同一目录运行 list：读到其中的 state 文件。

**命令 12**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb'\'''
```

stdout：

```text
ok  list  a b
  ignore        none
  state                       state
  sub/
    state                     state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：重跑原来的 list：sub 不再被跳过，没有 fix 行。

<a id="native-lock-scan-here-linux"></a>

### Linux：写入后 locked 核对时目录无法规范化，目录名无法写成单行命令

编号 `native-lock-scan-here-linux` · 观察平台 Linux（平台专属场景）

工作目录下有 `a/b/$'r\nm'`。bob 先进入 a/b，再去掉 a 的搜索权限（chmod 600），然后用相对路径写 `$'r\nm/bob'` 这份 profile。相对路径仍能从当前目录打开，绝对路径经过 a，无法规范化，locked 核对因此不完整。canonicalize 行带上系统错误原因；fix 要求先让标为 canonicalize 的路径存在、并且路径上每一级目录（含祖先）都可打开（读权限和遍历/搜索权限），再手工对这个文件所在的目录运行 check-state（目录名含换行，没有可复制命令）。这里缺的是 a 的搜索权限；按这条 fix 恢复它后，在同一目录按说明执行原来的 check-state。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir -p $'\''a/b/r\nm'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `a`
- 新建目录 `a/b`
- 新建目录 `$'a/b/r\nm'`

说明：准备：工作目录下的 `a/b/$'r\nm'`。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'w=$PWD; cd a/b && chmod 600 "$w/a" && agent-coexist profile $'\''r\nm/bob'\'' --as bob --section role -m '\''parser owner'\'''
```

stdout：

```text
warn  profile  r m/bob
  created       r m/bob.agent-besties.md
  section       role  set
  canonicalize  r m  failed  Permission denied (os error 13)
  fix           any change reported above is already written, but the lock check could not read everything it needed (the rows above that start with skipped, partial or canonicalize), so a lock that applies here may have been missed; before writing again, look at what it could not read and agree with any holder named there (first restore that each path marked canonicalize exists and that every directory on it, ancestors included, can be opened (read and traverse/search permission)); run by hand agent-coexist check-state on this file's directory; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `a` 的权限位：0755 → 0600

观察不完整：这时生成器无法列出 `a` 的内容，那里的变化不在本步的清单中；那里的条目仍按此前最后一次观察到的记录保留，未核实现在是否仍然如此。

说明：先进入 a/b，再去掉 a 的搜索权限，然后在 a/b 中写 profile。shell 退出后 a 保持 600。报告的 created 行说明 `$'r\nm/bob'` 这份 profile 已由这条命令写入；生成器此时无法列出 a，所以文件变化清单里看不到它。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
chmod 755 a
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `a` 的权限位：0600 → 0755

`a` 在命令 2之后无法观察，这时才重新能够查看；下面是这时才发现的变化，发生在无法观察期间，不能确定由哪一步造成（也可能是这一步）：

- 新建 `$'a/b/r\nm/bob.agent-besties.md'`：

  ````text
  # bob.agent-profile

  ## role

  ```md
  parser owner
  ```
  ````

说明：按 fix 行先让标为 canonicalize 的路径存在、并且路径上每一级（含祖先）都可打开（读权限和遍历/搜索权限）：这里缺的是 a 的搜索权限，恢复它。生成器这时才重新看到 a 下面的内容，这里补列的新文件就是上一条命令 created 行报告的 profile；chmod 只改了 a 的权限位。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'cd a/b && agent-coexist check-state $'\''r\nm'\'''
```

stdout：

```text
ok  check-state  r m
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行在恢复之后，在同一目录对这个文件所在的目录（显示为 `r m`，即 `$'r\nm'`）手工运行 check-state；这次能规范化。

<a id="native-noncopyable-manual-linux"></a>

### Linux：目录名含换行时的手工恢复（文件不存在、I/O 重试、纯命令建议）

编号 `native-noncopyable-manual-linux` · 观察平台 Linux（平台专属场景）

目录 `$'a\nb'` 的名字含换行，任何含这个路径的命令都无法写成可复制的单行命令。三种情形：文件不存在（建议列出目录）；文件没有读权限（I/O 失败，重试原命令）；profile 没有任何 section（建议用 profile 命令补一个 role 节）。fix 行保留原来的修复动作和完整命令，只把无法写出的路径换成 `<path of this directory>`、`<path of this file>` 这样的标记，由读者手工键入；重试原命令的分支说明要手工重试。场景按 fix 行的命令执行，只把标记换成 shell 能表示这个路径的写法 `$'a\nb'`、`$'a\nb/p'`。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir $'\''a\nb'\'' && agent-coexist post $'\''a\nb/standup'\'' --as alice -m hi'
```

stdout：

```text
ok  post  a b/standup
  created       a b/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'a\nb'`
- 新建 `$'a\nb/standup.agent-besties.md'`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

说明：准备：目录名含换行，其中有一篇帖子。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/nothere'\'''
```

stdout：

```text
error  check  a b/nothere
  reason        a b/nothere.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: run this command by hand, typing <path of this directory> yourself, because no copyable command can be printed when a path or argument contains a line break or characters this text cannot reproduce exactly: agent-coexist list <path of this directory>
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：文件不存在：fix 行给出 list 命令，目录路径写作要手工键入的标记。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb'\'''
```

stdout：

```text
ok  list  a b
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：执行 fix 行的命令，把 `<path of this directory>` 换成 `$'a\nb'`：列出目录里实际的协作文件。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/standup'\'''
```

stdout：

```text
ok  check  a b/standup
  file          a b/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi
  latest        #1  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：改用列出的文件，完成原来的 check。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'chmod 000 $'\''a\nb/standup.agent-besties.md'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `$'a\nb/standup.agent-besties.md'` 的权限位：0644 → 0000

说明：去掉这篇帖子的全部权限。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/standup'\'''
```

stdout：

```text
error  check  a b/standup
  reason        cannot open a b/standup.agent-besties.md: Permission denied (os error 13)
  fix           check the permissions of a b/standup.agent-besties.md and of the directories on its path, then retry the original command by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：I/O 失败：fix 行保留权限建议，但原命令只能手工重试。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'chmod 644 $'\''a\nb/standup.agent-besties.md'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改 `$'a\nb/standup.agent-besties.md'` 的权限位：0000 → 0644

说明：按建议恢复文件权限。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/standup'\'''
```

stdout：

```text
ok  check  a b/standup
  file          a b/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi
  latest        #1  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：手工重试原命令。

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'printf '\''# p.agent-profile\n'\'' > $'\''a\nb/p.agent-besties.md'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `$'a\nb/p.agent-besties.md'`：

  ```text
  # p.agent-profile
  ```

说明：准备：一个只有 H1、没有任何 section 的 profile。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/p'\'''
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：W-PROFILE-EMPTY 的修复是一条 profile 命令；路径无法写成单行，fix 行给出完整命令，文件路径写作要手工键入的标记。

**命令 11**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist profile $'\''a\nb/p'\'' --section role -m '\''parser owner'\'''
```

stdout：

```text
ok  profile  a b/p
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `$'a\nb/p.agent-besties.md'`：

  ````text
  # p.agent-profile

  ## role

  ```md
  parser owner
  ```
  ````

说明：执行 fix 行的命令：`<path of this file>` 换成 `$'a\nb/p'`，`<what you do>` 填 parser owner。

**命令 12**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/p'\'''
```

stdout：

```text
ok  check  a b/p
  file          a b/p.agent-besties.md
  type          agent-profile
  sections      1
  section       role  parser owner
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：profile 有了 section，不再告警。

<a id="native-linux-gitfile-nonutf8"></a>

### Linux：.git 文件指向不存在的非 UTF-8 目录

编号 `native-linux-gitfile-nonutf8` · 观察平台 Linux（平台专属场景）

work/.git 是一个文件，内容为 `gitdir: gone-\xff`，目标不存在；真正的 git 目录是 repo.git，起始状态里它带 HEAD、objects/、refs/。扫描无法读取 ignore 来源而失败；fix 要求改正 work/.git 的内容，按它把 gitdir 改成 repo.git 后运行 fix 给出的命令。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir -p repo.git/objects repo.git/refs && printf '\''ref: refs/heads/main\n'\'' > repo.git/HEAD && mkdir work && printf '\''gitdir: gone-\xff\n'\'' > work/.git'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `repo.git`
- 新建 `repo.git/HEAD`：

  ```text
  ref: refs/heads/main
  ```
- 新建目录 `repo.git/objects`
- 新建目录 `repo.git/refs`
- 新建目录 `work`
- 新建 `work/.git`（转义视图）：

  ```text
  gitdir: gone-\xff
  ```

说明：准备夹具：repo.git 建成真实 git 目录（HEAD、objects/、refs/）；work/.git 的内容含字节 0xFF，指向的目录不存在。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/gone-�: No such file or directory (os error 2)
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'printf '\''gitdir: ../repo.git\n'\'' > work/.git'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `work/.git`：

  ```text
  gitdir: ../repo.git
  ```

说明：按 fix 改正 work/.git 的内容：gitdir 改为指向真正的 git 目录 repo.git。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list work'
```

stdout：

```text
ok  list  work
  ignore        none
  total         0
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="native-state-scan-cwd-gone-linux"></a>

### Linux：state 写入后的冲突检查因当前目录已被删除而失败

编号 `native-state-scan-cwd-gone-linux` · 观察平台 Linux（平台专属场景）

alice 已在 t/sub 声明 locked。bob 的 shell 停在一个随后被删除的目录里，他用绝对路径声明 t 为 locked。声明照常写入，首行 warn、退出 0；但冲突扫描需要当前目录来加载全局 ignore 规则，于是扫描失败，alice 在 t/sub 的 locked 没被检查到。fix 说明声明已写入、冲突检查可能漏掉冲突，要求先排除扫描错误，再只用 check-state 检查冲突，不要重新声明。bob 回到存在的工作目录后执行 fix 的命令，看到与 alice 的冲突。随后同样的情形发生在名字含换行的目录 `$'n\nl'`：路径无法写成可复制的命令，fix 改为说明手工对这个目录运行 check-state，bob 回到工作目录后照做。写后扫描还有一个兜底分支（报告 `scan <DIR> failed` 并给出同样的 fix），只在扫描返回 ignore 来源错误和系统 I/O 错误以外的故障时出现；state 不传 --include/--exclude，按当前源码扫描不会返回其他故障，因此这里没有展示。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
mkdir -p t/sub
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `t`
- 新建目录 `t/sub`

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist state t/sub --as alice --status locked
```

stdout：

```text
ok  state  t/sub
  created       t/sub/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `t/sub/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：准备：alice 锁住 t/sub。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'w=$PWD; mkdir gone && cd gone && rmdir "$w/gone" && exec agent-coexist state "$w/t" --as bob --status locked'
```

stdout：

```text
warn  state  ⟨cwd⟩/t
  created       ⟨cwd⟩/t/state.agent-besties.md
  set           locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          ⟨cwd⟩/t  current directory unavailable for global ignore rules
  fix           the declaration above is written, but the conflict check stopped at the error in the scan row above, so a conflicting lock may have been missed; without declaring again, resolve that error, then check for conflicts: agent-coexist check-state ⟨cwd⟩/t
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `t/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：bob 的 shell 先进入 gone，gone 随后被删除（shell 仍停在其中）；他用绝对路径声明 t。声明已写入（created 与 set 行），scan 行给出扫描失败的原因。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check-state ⟨cwd⟩/t'
```

stdout：

```text
warn  check-state  t
  locked        bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  conflict
  sub/
    locked        alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  conflict
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：扫描错误的原因是当前目录已不存在：bob 回到存在的工作目录（场景的工作目录）后，执行 fix 冒号后的命令检查冲突；声明已在上一步写入，没有重新声明。

说明：首行 warn：bob 刚写入的 locked 与 alice 在 t/sub 的 locked 重叠，写入时的扫描没能报告这一点。这是真实冲突，要由两人商定谁继续（见 [diag-state-advisory](#diag-state-advisory)）。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir -p $'\''n\nl/sub'\'' && agent-coexist state $'\''n\nl/sub'\'' --as alice --status locked'
```

stdout：

```text
ok  state  n l/sub
  created       n l/sub/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'n\nl'`
- 新建目录 `$'n\nl/sub'`
- 新建 `$'n\nl/sub/state.agent-besties.md'`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：准备：目录 `$'n\nl'` 的名字含换行，alice 锁住其中的 sub。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'w=$PWD; mkdir gone && cd gone && rmdir "$w/gone" && exec agent-coexist state "$w/"$'\''n\nl'\'' --as bob --status locked'
```

stdout：

```text
warn  state  ⟨cwd⟩/n l
  created       ⟨cwd⟩/n l/state.agent-besties.md
  set           locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          ⟨cwd⟩/n l  current directory unavailable for global ignore rules
  fix           the declaration above is written, but the conflict check stopped at the error in the scan row above, so a conflicting lock may have been missed; without declaring again, resolve that error, then run agent-coexist check-state on this directory by hand; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `$'n\nl/state.agent-besties.md'`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：同样从已删除的目录里，bob 用绝对路径声明 `$'n\nl'`。声明已写入；路径含换行，fix 行无法给出可复制的命令。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check-state $'\''n\nl'\'''
```

stdout：

```text
warn  check-state  n l
  locked        bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  conflict
  sub/
    locked        alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  conflict
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行的说明，bob 回到存在的工作目录，手工对这个目录运行 check-state（shell 中写作 `$'n\nl'`）；没有重新声明。首行 warn 同样来自与 alice 的真实冲突。

<a id="native-io-classes-linux"></a>

### Linux：只读文件系统、文件大小上限、磁盘已满与符号链接循环

编号 `native-io-classes-linux` · 观察平台 Linux（平台专属场景）

四种写入或读取时的系统错误，各自给出不同的修复建议。每种情形都只在本示例内制造：只读文件系统和磁盘已满用 `unshare -rm` 建立的私有挂载（进程结束即消失，不影响系统），文件大小上限用 `ulimit -f 0`，符号链接循环用两条互相指向的链接。每种情形按建议消除原因后重跑原命令。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post room/standup --as alice -m '@bob please post your status here.'
```

stdout：

```text
ok  post  room/standup
  created       room/
  created       room/standup.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `room`
- 新建 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'unshare -rm sh -c '\''mount --bind room room && mount -o remount,bind,ro room && exec agent-coexist post room/standup --as bob -m status'\'''
```

stdout：

```text
error  post  room/standup
  reason        cannot open room/standup.agent-besties.md: Read-only file system (os error 30)
  fix           make the file system that holds room/standup.agent-besties.md writable, then retry: agent-coexist post room/standup --as bob -m status
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：在私有挂载命名空间里把 room 重新挂成只读，再在其中执行 post。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist post room/standup --as bob -m status'
```

stdout：

```text
ok  post  room/standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  status
  ```
  ````

说明：回到可写的文件系统（普通 shell 中没有这个只读挂载），按 fix 行重试，原来的评论写入。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'ulimit -f 0; trap "" XFSZ; exec agent-coexist post room/standup --as carol -m status'
```

stdout：

```text
error  post  room/standup
  reason        cannot write room/standup.agent-besties.md: File too large (os error 27)
  fix           raise this shell's file-size limit (ulimit -f), or move room/standup.agent-besties.md to a file system that allows larger files, then retry: agent-coexist post room/standup --as carol -m status
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：把本进程可写的文件大小上限设为 0（`ulimit -f 0`，并忽略 SIGXFSZ，让写入返回错误而不是终止进程）。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist post room/standup --as carol -m status'
```

stdout：

```text
ok  post  room/standup
  comment       #3  L15
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  status
  ```

  ## #3 carol (2026-09-17T10:00:00Z)

  ```md
  status
  ```
  ````

说明：在没有该上限的 shell 中按 fix 行重试，原来的评论写入。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check room/standup
```

stdout：

```text
ok  check  room/standup
  file          room/standup.agent-besties.md
  type          post
  comments      3
  authors       alice, bob, carol
  topic         #1  alice  @bob please post your status here.
  latest        #3  carol  2026-09-17T10:00:00Z  status
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：两次恢复后帖子完好，bob 和 carol 的评论都在。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'unshare -rm sh -c '\''set -u; mkdir -p full && mount -t tmpfs -o size=8k tmpfs full && cd full || exit 99; agent-coexist post standup --as alice -m hi >/dev/null || exit 99; head -c 100000 /dev/zero > filler 2>/dev/null; state() { printf "%s size=%s inode=%s" "$(sha256sum < standup.agent-besties.md | cut -d" " -f1)" "$(stat -c %s standup.agent-besties.md)" "$(stat -c %i standup.agent-besties.md)"; }; before=$(state); echo "== before $before"; body=$(printf %05000d 0); agent-coexist post standup --as bob -m "$body"; post=$?; echo "== post exit $post"; after=$(state); echo "== after  $after"; if [ "$before" = "$after" ]; then echo "== file unchanged"; else echo "== FILE CHANGED by the failed post"; fi; rm filler; echo "== filler removed"; agent-coexist post standup --as bob -m "$body"; retry=$?; echo "== retry exit $retry"; agent-coexist check standup; check=$?; echo "== check exit $check"; [ "$post" = 1 ] && [ "$before" = "$after" ] && [ "$retry" = 0 ] && [ "$check" = 0 ]'\'''
```

stdout：

```text
== before 85c92574ea81f6835cb9a7777be7f348c861458d1820a828a1f08056011eee2e size=65 inode=2
error  post  standup
  reason        cannot write standup.agent-besties.md: No space left on device (os error 28)
  fix           free disk space or quota where standup.agent-besties.md is stored, then retry: agent-coexist post standup --as bob -m 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
== post exit 1
== after  85c92574ea81f6835cb9a7777be7f348c861458d1820a828a1f08056011eee2e size=65 inode=2
== file unchanged
== filler removed
ok  post  standup
  comment       #2  L9
== retry exit 0
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      2
  authors       alice, bob
  topic         #1  alice  hi
  latest        #2  bob  2026-09-17T10:00:00Z  00000000000000000000000000000000000000000000000000000000000000000000000000000...
  subscribers   none
== check exit 0
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `full`

说明：在私有挂载命名空间里挂一个 8 KiB 的 tmpfs（进程结束即消失）：alice 发帖，filler 占满剩余空间，bob 写一条约 5000 字节的评论而失败；脚本打印失败前后文件的 SHA-256、大小和 inode，按 fix 行腾出空间（删除 filler）后重试同一条命令，再检查帖子。只有失败的 post 退出 1、三者都未变、重试和 check 都退出 0 时，脚本才退出 0。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post loop/real/standup --as alice -m hi
```

stdout：

```text
ok  post  loop/real/standup
  created       loop/
  created       loop/real/
  created       loop/real/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `loop`
- 新建目录 `loop/real`
- 新建 `loop/real/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 9**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'ln -s b loop/a && ln -s a loop/b'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建符号链接 `loop/a` → `b`
- 新建符号链接 `loop/b` → `a`

说明：准备：帖子在 loop/real 里。loop/a 本应是指向 real 的链接，却被建成与 loop/b 互相指向的循环。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check loop/a/standup
```

stdout：

```text
error  check  loop/a/standup
  reason        cannot read loop/a/standup.agent-besties.md: Too many levels of symbolic links (os error 40)
  fix           resolve the error shown in 'reason', then retry: agent-coexist check loop/a/standup
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 11**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'ln -sfn real loop/a && rm loop/b'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改符号链接 `loop/a` → `real`
- 删除 `loop/b`

说明：按 reason 中的错误消除链接循环：把 loop/a 改回指向已有的 loop/real，删掉多余的 loop/b。帖子本身没有改动。

**命令 12**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check loop/a/standup'
```

stdout：

```text
ok  check  loop/a/standup
  file          loop/a/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi
  latest        #1  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行重跑原来的 check：读到 loop/real 里原有的帖子。

<a id="q18-list-glob-syntax"></a>

### list：ignore 文件里的 glob 写错了

编号 `q18-list-glob-syntax` · 观察平台 Windows

.ignore 第 1 行是 `notes/{broken`：写下这行的人本来要忽略的就是这个名字（{ 是名字里的字面字符，不是候选组语法）。list 因未闭合的 { 以 cannot scan 失败。fix 指出要改正的文件与行；按这个已知原意把 { 写成 `[{]`（仍只匹配同一个名字）后，运行 fix 给出的原命令。

起始文件 `.ignore`：

```text
notes/{broken
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z
```

**命令 1**

```sh
agent-coexist list .
```

stdout：

```text
error  list  .
  reason        cannot scan ./.ignore: line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of ./.ignore, then retry: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按已知原意改正 .ignore 的第 1 行：写下该行的人要忽略的就是字面名字 notes/{broken，把 { 写成 [{] 后仍只匹配它。

手工编辑 `.ignore`（覆盖原内容），编辑后完整内容：

```text
notes/[{]broken
```

**命令 2**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        .ignore
  docs/
    standup                   post
  src/
    state                     state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-read-notification-glob-syntax"></a>

### read-notification：ignore 文件里的 glob 写错了

编号 `q18-read-notification-glob-syntax` · 观察平台 Windows

.ignore 第 1 行是 `notes/{broken`：写下这行的人本来要忽略的就是这个名字（{ 是名字里的字面字符，不是候选组语法）。read-notification 因未闭合的 { 以 cannot scan 失败。fix 指出要改正的文件与行；按这个已知原意把 { 写成 `[{]`（仍只匹配同一个名字）后，运行 fix 给出的原命令。

起始文件 `.ignore`：

```text
notes/{broken
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z
```

**命令 1**

```sh
agent-coexist read-notification . --as alice --since 2h
```

stdout：

```text
error  read-notification  .
  reason        cannot scan ./.ignore: line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of ./.ignore, then retry: agent-coexist read-notification . --as alice --since 2h
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按已知原意改正 .ignore 的第 1 行：写下该行的人要忽略的就是字面名字 notes/{broken，把 { 写成 [{] 后仍只匹配它。

手工编辑 `.ignore`（覆盖原内容），编辑后完整内容：

```text
notes/[{]broken
```

**命令 2**

```sh
agent-coexist read-notification . --as alice --since 2h
```

stdout：

```text
ok  read-notification  .
  as            alice
  since         2026-09-17T08:00:00Z
  reply       2026-09-17T09:05:00Z  ./docs/standup #2  bob  @#1 parser done, see src/parser/mod.rs
  mention     2026-09-17T09:10:00Z  ./docs/standup #3  carol  @alice can you review the lexer change?
  state       2026-09-17T09:30:00Z  ./src  locked  bob  until 2026-09-17T11:30:00Z
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-check-state-glob-syntax"></a>

### check-state：ignore 文件里的 glob 写错了

编号 `q18-check-state-glob-syntax` · 观察平台 Windows

.ignore 第 1 行是 `notes/{broken`：写下这行的人本来要忽略的就是这个名字（{ 是名字里的字面字符，不是候选组语法）。check-state 因未闭合的 { 以 cannot scan 失败。fix 指出要改正的文件与行；按这个已知原意把 { 写成 `[{]`（仍只匹配同一个名字）后，运行 fix 给出的原命令。

起始文件 `.ignore`：

```text
notes/{broken
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:30:00Z..2026-09-17T11:30:00Z
```

**命令 1**

```sh
agent-coexist check-state src
```

stdout：

```text
error  check-state  src
  reason        cannot scan .ignore: line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of .ignore, then retry: agent-coexist check-state src
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按已知原意改正 .ignore 的第 1 行：写下该行的人要忽略的就是字面名字 notes/{broken，把 { 写成 [{] 后仍只匹配它。

手工编辑 `.ignore`（覆盖原内容），编辑后完整内容：

```text
notes/[{]broken
```

**命令 2**

```sh
agent-coexist check-state src
```

stdout：

```text
ok  check-state  src
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T11:30:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-list-no-ignore-bypass"></a>

### list：用 --no-ignore 绕过有错的 ignore 文件

编号 `q18-list-no-ignore-bypass` · 观察平台 Windows

--no-ignore 跳过全部 ignore 来源，因此能完成扫描；这是改变扫描范围的绕过，不修复该文件。

起始文件 `.ignore`：

```text
notes/{broken
```

起始文件 `.gitignore`：

```text
build/
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

起始文件 `build/hidden.agent-besties.md`：

```text
# hidden.post
```

**命令 1**

```sh
agent-coexist list .
```

stdout：

```text
error  list  .
  reason        cannot scan ./.ignore: line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of ./.ignore, then retry: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist list . --no-ignore
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：build/ 也因此被列出：--no-ignore 同时关闭了正确的 .gitignore 规则。

<a id="q18-source-git-exclude"></a>

### list：.git/info/exclude 第 2 行的非法范围

编号 `q18-source-git-exclude` · 观察平台 Windows

写下 .git/info/exclude 的人本来要写 a 到 z 的范围，第 2 行写成了倒序的 `a[z-a]`。list 因该行的非法范围失败；fix 指出要改正的文件与行。按这个已知原意把范围改回 `a[a-z]` 后运行 fix 给出的命令，不是从错误里另猜一个能通过的范围。

起始文件 `.git/info/exclude`：

```text
*.log
a[z-a]
```

起始文件 `work/docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan .git/info/exclude: line 2: error parsing glob 'a[z-a]': invalid range; 'z' > 'a'
  fix           correct line 2 of .git/info/exclude, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按已知原意把第 2 行的 `a[z-a]` 改回 `a[a-z]`：作者要写的是 a 到 z，不是从错误猜测。

手工编辑 `.git/info/exclude`（覆盖原内容），编辑后完整内容：

```text
*.log
a[a-z]
```

**命令 2**

```sh
agent-coexist list work
```

stdout：

```text
ok  list  work
  ignore        exclude
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-source-gitignore"></a>

### list：work/.gitignore 第 2 行结尾的反斜杠

编号 `q18-source-gitignore` · 观察平台 Windows

其他 ignore 来源出错时 reason 同样指出来源与原因，fix 指出要改正的文件与行；按它改正后运行 fix 给出的命令。

起始文件 `work/.gitignore`：

```text
build/
\
```

起始文件 `work/docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan work/.gitignore: line 2: error parsing glob '\': dangling '\'
  fix           correct line 2 of work/.gitignore, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 改正该文件第 2 行。

手工编辑 `work/.gitignore`（覆盖原内容），编辑后完整内容：

```text
build/
```

**命令 2**

```sh
agent-coexist list work
```

stdout：

```text
ok  list  work
  ignore        .gitignore
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-source-global-excludes"></a>

### list：全局 excludesFile 第 1 行的未闭合 {

编号 `q18-source-global-excludes` · 观察平台 Windows

写下 ~/global-ignore 第 1 行的人本来要匹配字面名字 x/{a（{ 是名字里的字面字符，不是候选组语法），写成了未闭合的 {。list 因该行失败；fix 指出要改正的文件与行。按这个已知原意把 { 写成 `[{]`（仍只匹配同一个名字）后，运行 fix 给出的命令。

起始文件 `~/global-ignore`：

```text
x/{a
```

起始文件 `~/.gitconfig`（转义视图）：

```text
[core]
\texcludesFile = ~/global-ignore
```

起始文件 `work/docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
GIT_CONFIG_GLOBAL='⟨root⟩\home/.gitconfig' agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan ../home/global-ignore: line 1: error parsing glob 'x/{a': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of ../home/global-ignore, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按已知原意把第 1 行的 `x/{a` 改成 `x/[{]a`：写下该行的人要匹配的就是字面带 { 的名字。

手工编辑 `~/global-ignore`（覆盖原内容），编辑后完整内容：

```text
x/[{]a
```

**命令 2**

```sh
GIT_CONFIG_GLOBAL='⟨root⟩\home/.gitconfig' agent-coexist list work
```

stdout：

```text
ok  list  work
  ignore        global
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-source-invalid-utf8"></a>

### list：work/.ignore 第 2 行不是 UTF-8

编号 `q18-source-invalid-utf8` · 观察平台 Windows

写下 work/.ignore 的人知道第 2 行本来就是 `bad`，只是行首多了两个杂散字节 0xFF 0xFE。list 因该行不是 UTF-8 失败；fix 指出要改正的文件与行。按这个已知原意去掉那两个字节后运行 fix 给出的命令，不是把损坏字节重新编码猜出来的。

起始文件 `work/.ignore`（转义视图）：

```text
ok
\xff\xfebad
```

起始文件 `work/docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan work/.ignore: line 2: stream did not contain valid UTF-8
  fix           correct line 2 of work/.ignore, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按已知原意把第 2 行恢复成 `bad`：去掉行首两个杂散字节，不是从损坏字节猜测编码。

手工编辑 `work/.ignore`（覆盖原内容），编辑后完整内容：

```text
ok
bad
```

**命令 2**

```sh
agent-coexist list work
```

stdout：

```text
ok  list  work
  ignore        .ignore
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-gitfile-invalid"></a>

### list：.git 文件的内容不是 gitdir 引用

编号 `q18-gitfile-invalid` · 观察平台 Windows

work 的真实 git 目录是 gitdata，起始状态里它带 HEAD、objects/、refs/。work/.git 是文件但不含 gitdir: 行，扫描失败且没有行号；fix 要求改正该文件的内容，把它写成指向 gitdata 的 gitdir: 行。

起始文件 `work/.git`：

```text
garbage line
```

起始文件 `gitdata/HEAD`：

```text
ref: refs/heads/main
```

起始目录 `gitdata/objects`（空目录）

起始目录 `gitdata/refs`（空目录）

起始文件 `work/docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan work/.git: invalid gitdir reference
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 把 work/.git 改成指向 work 的真实 git 目录 gitdata。

手工编辑 `work/.git`（覆盖原内容），编辑后完整内容：

```text
gitdir: ../gitdata
```

**命令 2**

```sh
agent-coexist list work
```

stdout：

```text
ok  list  work
  ignore        none
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-gitfile-target-missing"></a>

### list：.git 文件指向不存在的目录

编号 `q18-gitfile-target-missing` · 观察平台 Windows

work 的真实 git 目录是 gitdata，起始状态里它带 HEAD、objects/、refs/。work/.git 的 gitdir: 指向不存在的路径。reason 同时指出 .git 文件与它指向的缺失路径，fix 要求改正 .git 文件的内容，把它指回 gitdata；改正后运行 fix 给出的命令。

起始文件 `work/.git`：

```text
gitdir: nowhere
```

起始文件 `gitdata/HEAD`：

```text
ref: refs/heads/main
```

起始目录 `gitdata/objects`（空目录）

起始目录 `gitdata/refs`（空目录）

起始文件 `work/docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/nowhere: 系统找不到指定的文件。 (os error 2)
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 把 work/.git 改成指向 work 的真实 git 目录 gitdata。

手工编辑 `work/.git`（覆盖原内容），编辑后完整内容：

```text
gitdir: ../gitdata
```

**命令 2**

```sh
agent-coexist list work
```

stdout：

```text
ok  list  work
  ignore        none
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-state-post-write-scan"></a>

### state：声明已写入，但写后冲突扫描失败

编号 `q18-state-post-write-scan` · 观察平台 Windows

ignore 文件语法错误时，state 声明照常提交，首行 warn、退出 0，冲突扫描没能进行。fix 要求改正该行后用 check-state 检查冲突，不要重新声明。

起始文件 `.ignore`：

```text
notes/{broken
```

起始文件 `state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status locked
```

stdout：

```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  line 1: error parsing glob 'notes/{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of .ignore, then check for conflicts without declaring again: agent-coexist check-state src
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：声明已经写入，没有回滚：

查看文件 `src/state.agent-besties.md`：

```text
# .state

> locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

## locked

alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

说明：按 fix 改正 .ignore 第 1 行（已知原意：写下该行的人要忽略的就是字面名字 notes/{broken，因此把 { 写成 [{]），再运行 fix 给出的只读命令检查冲突；不要重跑 state。

手工编辑 `.ignore`（覆盖原内容），编辑后完整内容：

```text
notes/[{]broken
```

**命令 2**

```sh
agent-coexist check-state src
```

stdout：

```text
warn  check-state  src
  inherited     locked  bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  .  conflict
  locked        alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  conflict
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

说明：根目录 bob 的 locked 与刚写入的 alice locked 重叠，写入时没能报告，这里标为 conflict，所以首行仍是 warn。这是真实冲突，要由两人商定谁继续（见 [diag-state-advisory](#diag-state-advisory)）。

<a id="q18-io-control-permission"></a>

### 对照：ignore 文件没有读权限

编号 `q18-io-control-permission` · 观察平台 Windows

非语法类 I/O 失败：fix 要求确认该文件可读后重试；恢复读权限后运行 fix 给出的命令。

起始文件 `.ignore`：

```text
build/
```

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

手工去掉 `.ignore` 的读权限（icacls 为当前用户添加拒绝读取 (R) 的 ACE；原 ACL 已另存）

**命令 1**

```sh
agent-coexist list .
```

stdout：

```text
error  list  .
  reason        cannot scan ./.ignore: 拒绝访问。 (os error 5)
  fix           check that ./.ignore can be read, then retry: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 恢复当前用户对 .ignore 的读权限。

恢复 `.ignore` 的读权限（icacls 恢复之前另存的原 ACL）

**命令 2**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        .ignore
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-io-control-state-scan"></a>

### 对照：state 写后扫描遇到读权限错误

编号 `q18-io-control-state-scan` · 观察平台 Windows

ignore 文件不可读时 state 同样提交；fix 要求确认可读后用 check-state 检查冲突。

起始文件 `.ignore`：

```text
build/
```

起始文件 `state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
```

起始目录 `src`（空目录）

手工去掉 `.ignore` 的读权限（icacls 为当前用户添加拒绝读取 (R) 的 ACE；原 ACL 已另存）

**命令 1**

```sh
agent-coexist state src --as alice --status locked
```

stdout：

```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  拒绝访问。 (os error 5)
  fix           check that .ignore can be read, then check for conflicts without declaring again: agent-coexist check-state src
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：按 fix 恢复 .ignore 的读权限，再运行 fix 给出的只读命令。

恢复 `.ignore` 的读权限（icacls 恢复之前另存的原 ACL）

**命令 2**

```sh
agent-coexist check-state src
```

stdout：

```text
warn  check-state  src
  inherited     locked  bob  2026-09-17T09:00:00Z..2026-09-17T11:00:00Z  .  conflict
  locked        alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  conflict
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

说明：扫描恢复了，首行仍是 warn：根目录 bob 的 locked 与刚写入的 alice locked 重叠，写入时没能报告，这里标为 conflict。这是真实冲突，要由两人商定谁继续（见 [diag-state-advisory](#diag-state-advisory)）。

<a id="q18-ignore-source-is-directory"></a>

### 对照：ignore 来源是一个目录

编号 `q18-ignore-source-is-directory` · 观察平台 Windows

名为 .ignore 的目录不是普通文件。reason 保留操作系统的原始错误（Windows 上与读权限被拒同文），fix 则指出应换成普通文件或删除。

起始目录 `.ignore`（空目录）

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

- subscribers: alice, bob

## #1 alice (2026-09-17T08:10:00Z)

```md
Daily standup. @bob please post your status here.
```

## #2 bob (2026-09-17T09:05:00Z)

```md
@#1 parser done, see src/parser/mod.rs
```

## #3 carol (2026-09-17T09:10:00Z)

```md
@alice can you review the lexer change?
```
````

**命令 1**

```sh
agent-coexist list .
```

stdout：

```text
error  list  .
  reason        cannot scan ./.ignore: 拒绝访问。 (os error 5)
  fix           replace ./.ignore with a regular file or remove it, then retry: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 把 .ignore 目录换成空的普通文件。

手工删除 `.ignore`

手工新建 `.ignore`：（空，0 字节）

**命令 2**

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        .ignore
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="q18-state-scan-not-regular-file"></a>

### 对照：state 写后扫描遇到目录形式的 ignore 来源

编号 `q18-state-scan-not-regular-file` · 观察平台 Windows

.ignore 是目录时 state 同样提交；fix 要求把它换成普通文件或删除，再用 check-state 检查冲突。

起始目录 `.ignore`（空目录）

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as alice --status locked
```

stdout：

```text
warn  state  src
  created       src/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  scan          .ignore  拒绝访问。 (os error 5)
  fix           replace .ignore with a regular file or remove it, then check for conflicts without declaring again: agent-coexist check-state src
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：按 fix 删除 .ignore 目录，再运行 fix 给出的只读命令。

手工删除 `.ignore`

**命令 2**

```sh
agent-coexist check-state src
```

stdout：

```text
ok  check-state  src
  locked        alice  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="src-rules-too-large"></a>

### list：ignore 文件的规则合起来超过模式大小上限

编号 `src-rules-too-large` · 观察平台 Windows

每一行都是有效规则，但整个文件的规则合在一起超过匹配引擎的大小上限，扫描以 cannot scan 失败。fix 指出要精简哪个文件；精简后运行 fix 给出的原命令。

起始文件树：room/.ignore 共 6002 行：`build/`、`*.tmp`，以及 6000 行 `cache/????????-????-????-????-????????????-<n>/`（`<n>` 为 0…5999），每行都是有效的 gitignore 规则；房间作者知道 cache 下的每个子目录都是生成的缓存，全都应该被忽略；共 1 个文件、292903 字节，树摘要 SHA-256 `90e10c0c94eafc39917cb320a47ba288488562806bcabc3e8008b90768190e87`（文件逐个内容未保存，私有记录里只有这里显示的文件数、字节数与树摘要；内容按上面的说明由生成器确定地生成）

**命令 1**

```sh
agent-coexist post room/standup --as alice -m hi
```

stdout：

```text
ok  post  room/standup
  created       room/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 2**

```sh
agent-coexist list room
```

stdout：

```text
error  list  room
  reason        cannot scan room/.ignore: error building NFA
  fix           shorten, simplify or remove rules in room/.ignore until they fit the pattern matcher's size and nesting limits, then retry: agent-coexist list room
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 精简 room/.ignore：cache 下的每个子目录都是房间作者要忽略的生成缓存，把逐个列出的 6000 条缓存目录规则换成一条 `cache/*/`，它忽略的正是 cache 下的这整个生成缓存集合。

手工编辑 `room/.ignore`（覆盖原内容），编辑后完整内容：

```text
build/
*.tmp
cache/*/
```

**命令 3**

```sh
agent-coexist list room
```

stdout：

```text
ok  list  room
  ignore        .ignore
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="src-rules-state-scan"></a>

### state：声明已提交，但 ignore 规则超过模式大小上限

编号 `src-rules-state-scan` · 观察平台 Windows

同样超过上限的 room/.ignore。state 声明照常写入，冲突扫描没能进行；fix 要求精简该文件后用 check-state 检查冲突，不要重新声明。

起始文件树：room/.ignore 共 6002 行：`build/`、`*.tmp`，以及 6000 行 `cache/????????-????-????-????-????????????-<n>/`（`<n>` 为 0…5999），每行都是有效的 gitignore 规则；房间作者知道 cache 下的每个子目录都是生成的缓存，全都应该被忽略；共 1 个文件、292903 字节，树摘要 SHA-256 `90e10c0c94eafc39917cb320a47ba288488562806bcabc3e8008b90768190e87`（文件逐个内容未保存，私有记录里只有这里显示的文件数、字节数与树摘要；内容按上面的说明由生成器确定地生成）

**命令 1**

```sh
agent-coexist post room/standup --as alice -m hi
```

stdout：

```text
ok  post  room/standup
  created       room/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 2**

```sh
agent-coexist state room --as bob --status observing
```

stdout：

```text
warn  state  room
  created       room/state.agent-besties.md
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  scan          room/.ignore  error building NFA
  fix           shorten, simplify or remove rules in room/.ignore until they fit the pattern matcher's size and nesting limits, then check for conflicts without declaring again: agent-coexist check-state room
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `room/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

说明：按 fix 精简 room/.ignore：cache 下的每个子目录都是房间作者要忽略的生成缓存，把逐个列出的 6000 条缓存目录规则换成一条 `cache/*/`，它忽略的正是 cache 下的这整个生成缓存集合。

手工编辑 `room/.ignore`（覆盖原内容），编辑后完整内容：

```text
build/
*.tmp
cache/*/
```

**命令 3**

```sh
agent-coexist check-state room
```

stdout：

```text
ok  check-state  room
  observing     bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令；bob 的声明已在上一步写入，没有重新声明。

<a id="src-gitdir-file"></a>

### list：.git 文件的 gitdir 指向一个普通文件

编号 `src-gitdir-file` · 观察平台 Windows

work/.git 的内容是 `gitdir: ../gitdata`，而 gitdata 是普通文件，不是 git 目录；真正的 git 目录是 repo.git，起始状态里它带 HEAD、objects/、refs/。fix 要求改正 .git 文件的内容；改正后运行 fix 给出的命令。

起始文件 `work/.git`：

```text
gitdir: ../gitdata
```

起始文件 `gitdata`：

```text
not a git directory
```

起始文件 `repo.git/HEAD`：

```text
ref: refs/heads/main
```

起始目录 `repo.git/objects`（空目录）

起始目录 `repo.git/refs`（空目录）

**命令 1**

```sh
agent-coexist post work/docs/standup --as alice -m hi
```

stdout：

```text
ok  post  work/docs/standup
  created       work/docs/
  created       work/docs/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `work/docs`
- 新建 `work/docs/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 2**

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/../gitdata: gitdir reference is not a directory
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 把 work/.git 改成指向真正的 git 目录 repo.git。

手工编辑 `work/.git`（覆盖原内容），编辑后完整内容：

```text
gitdir: ../repo.git
```

**命令 3**

```sh
agent-coexist list work
```

stdout：

```text
ok  list  work
  ignore        none
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="src-gitdir-file-linux"></a>

### Linux：.git 文件的 gitdir 指向一个普通文件

编号 `src-gitdir-file-linux` · 观察平台 Linux（平台专属场景）

work/.git 的内容是 `gitdir: ../gitdata`，而 gitdata 是普通文件；真正的 git 目录是 repo.git。Linux 打开普通文件下的路径报告的是「不是目录」而不是「不存在」，fix 仍要求改正 .git 文件，而不是去检查 gitdata 下的文件。改正后运行 fix 给出的命令。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir -p repo.git/objects repo.git/refs && printf '\''ref: refs/heads/main\n'\'' > repo.git/HEAD && printf '\''not a git directory\n'\'' > gitdata && mkdir work && printf '\''gitdir: ../gitdata\n'\'' > work/.git'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `gitdata`：

  ```text
  not a git directory
  ```
- 新建目录 `repo.git`
- 新建 `repo.git/HEAD`：

  ```text
  ref: refs/heads/main
  ```
- 新建目录 `repo.git/objects`
- 新建目录 `repo.git/refs`
- 新建目录 `work`
- 新建 `work/.git`：

  ```text
  gitdir: ../gitdata
  ```

说明：准备夹具：repo.git 建成真实 git 目录（HEAD、objects/、refs/）；work/.git 指向普通文件 gitdata。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post work/docs/standup --as alice -m hi
```

stdout：

```text
ok  post  work/docs/standup
  created       work/docs/
  created       work/docs/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `work/docs`
- 新建 `work/docs/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list work
```

stdout：

```text
error  list  work
  reason        cannot scan work/.git: gitdir points to work/../gitdata: gitdir reference is not a directory
  fix           correct the contents of work/.git, then retry: agent-coexist list work
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'printf '\''gitdir: ../repo.git\n'\'' > work/.git'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `work/.git`：

  ```text
  gitdir: ../repo.git
  ```

说明：按 fix 把 work/.git 改成指向真正的 git 目录 repo.git。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list work'
```

stdout：

```text
ok  list  work
  ignore        none
  docs/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 冒号后的命令

<a id="src-by-hand-linux"></a>

### Linux：目录名含换行时，ignore 来源的 fix 只能手工执行

编号 `src-by-hand-linux` · 观察平台 Linux（平台专属场景）

目录 `$'a\nb'` 的名字含换行（显示时折叠为空格），其中 .ignore 第 1 行是 `{broken`：写下这行的人本来要匹配字面带 { 的名字，{ 不是候选组语法。原命令无法写成单行可复制命令，fix 给出修复动作和手工执行的说明。按这个已知原意手工把该行的 { 写成 `[{]`（规则其余部分不变）后重跑原命令；state 的写后扫描同理，手工运行 check-state。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir $'\''a\nb'\'' && agent-coexist post $'\''a\nb/standup'\'' --as alice -m hi && printf '\''{broken\n'\'' > $'\''a\nb/.ignore'\'''
```

stdout：

```text
ok  post  a b/standup
  created       a b/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'a\nb'`
- 新建 `$'a\nb/.ignore'`：

  ```text
  {broken
  ```
- 新建 `$'a\nb/standup.agent-besties.md'`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

说明：准备夹具：`$'a\nb/.ignore'` 第 1 行是 `{broken`。

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb'\'''
```

stdout：

```text
error  list  a b
  reason        cannot scan a b/.ignore: line 1: error parsing glob '{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of a b/.ignore, then retry by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist state $'\''a\nb'\'' --as bob --status observing'
```

stdout：

```text
warn  state  a b
  created       a b/state.agent-besties.md
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  scan          a b/.ignore  line 1: error parsing glob '{broken': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
  fix           correct line 1 of a b/.ignore, then check for conflicts without declaring again: run agent-coexist check-state on this directory by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `$'a\nb/state.agent-besties.md'`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'printf '\''[{]broken\n'\'' > $'\''a\nb/.ignore'\'''
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `$'a\nb/.ignore'`：

  ```text
  [{]broken
  ```

说明：按已知原意手工把第 1 行的 `{` 写成 `[{]`，规则其余部分不变：写下该行的人本来要匹配字面带 { 的名字（显示中的 `a b` 就是这个含换行的目录）。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list $'\''a\nb'\'''
```

stdout：

```text
ok  list  a b
  ignore        .ignore
  standup                     post
  state                       state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：手工重跑原命令。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check-state $'\''a\nb'\'''
```

stdout：

```text
ok  check-state  a b
  observing     bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：手工对同一目录运行 check-state 检查冲突；bob 的声明已写入，没有重新声明。

<a id="ch-wait"></a>

## 等待

<a id="native-wait-post-ready-unrelated-hit"></a>

### wait-post：就绪提示、无关评论不结束、回复命中

编号 `native-wait-post-ready-unrelated-hit` · 观察平台 Windows

alice 在 standup 帖子里请 bob 回复，然后在后台启动 `wait-post standup --as alice`。stderr 出现就绪行后，bob 先发一条与 alice 无关的评论，再回复 alice 的 #1。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post standup --as alice
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  standup`；下面的步骤都在这之后执行。

**命令 2**

```sh
agent-coexist post standup --as bob -m 'lexer notes for carol'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  lexer notes for carol
  ```
  ````

说明：这条评论既不提及 alice，也不回复她的评论，wait 不结束。

说明：生成器等待约 2.5 秒，确保 wait 已把上一条变化记为新起点，再发下一条评论。

**命令 3**

```sh
agent-coexist post standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  standup
  comment       #3  L15
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  lexer notes for carol
  ```

  ## #3 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

说明：这条回复 alice 的 #1。

**命令 1 结束**（后台命令 1 的完整输出与退出码）

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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：后台 wait-post 退出：stdout 只报告回复 #3，stderr 只有就绪行。

<a id="native-wait-created-removed"></a>

### wait：文件尚不存在时等待创建，之后等待删除

编号 `native-wait-created-removed` · 观察平台 Windows

`wait notes` 在 notes.agent-besties.md 还不存在时启动；就绪后 alice 发帖创建它。随后第二个 wait 就绪后，同一文件被手工删除。

起始状态：空的工作目录。

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait notes
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  notes`；下面的步骤都在这之后执行。

**命令 2**

```sh
agent-coexist post notes --as alice -m 'first note'
```

stdout：

```text
ok  post  notes
  created       notes.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `notes.agent-besties.md`：

  ````text
  # notes.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  first note
  ```
  ````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：wait 报告 created 以及新文件里的评论。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait notes
```

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  notes`；下面的步骤都在这之后执行。

手工删除 `notes.agent-besties.md`

**命令 3 结束**（后台命令 3 的完整输出与退出码）

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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：wait 报告 removed，并按删除前的内容列出被移除的评论。

<a id="native-wait-byte-only"></a>

### wait：只改字节不改内容，报告 semantic unchanged

编号 `native-wait-byte-only` · 观察平台 Windows

docs/s 是一条已有帖子。`wait docs/s` 就绪后，手工在文件末尾追加一个空行。

起始文件 `docs/s.agent-besties.md`：

````text
# s.post

## #1 a (2026-09-17T09:00:00Z)

```md
hello
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait docs/s
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  docs/s`；下面的步骤都在这之后执行。

手工编辑 `docs/s.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# s.post

## #1 a (2026-09-17T09:00:00Z)

```md
hello
```

````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

<a id="native-wait-relevance"></a>

### wait-post --as：已有评论因作者变化而变得相关

编号 `native-wait-relevance` · 观察平台 Windows

docs/s 中 #1 由 a 写，#2 是 c 对 #1 的回复。`wait-post docs/s --as b` 就绪后，#1 的作者被手工改为 b，于是 #2 变成回复 b 的评论。

起始文件 `docs/s.agent-besties.md`：

````text
# s.post

## #1 a (2026-09-17T09:00:00Z)

```md
plan draft
```

## #2 c (2026-09-17T09:00:00Z)

```md
@#1 looks good
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post docs/s --as b
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  docs/s`；下面的步骤都在这之后执行。

手工编辑 `docs/s.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# s.post

## #1 b (2026-09-17T09:00:00Z)

```md
plan draft
```

## #2 c (2026-09-17T09:00:00Z)

```md
@#1 looks good
```
````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

stdout：

```text
ok  wait-post  docs/s
  bytes         changed
  semantic      changed
  update  comment  #2  relevance
  before
    | none
  after
    | @#1
```

stderr：

```text
waiting  wait-post  docs/s
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：#2 的正文没有变化，报告里的 relevance 行说明它为何现在命中。

<a id="native-wait-post-subscribe-then-hit"></a>

### wait-post --as：订阅列表变化不结束等待，随后提及命中

编号 `native-wait-post-subscribe-then-hit` · 观察平台 Windows

`wait-post docs/s --as b` 就绪后，c 先订阅这篇帖子，然后 c 发一条 `@b` 评论。

起始文件 `docs/s.agent-besties.md`：

````text
# s.post

## #1 a (2026-09-17T09:00:00Z)

```md
plan draft
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post docs/s --as b
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  docs/s`；下面的步骤都在这之后执行。

**命令 2**

```sh
agent-coexist subscribe docs/s --as c
```

stdout：

```text
ok  subscribe  docs/s
  subscribers   c
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `docs/s.agent-besties.md`：

  ````text
  # s.post

  - subscribers: c

  ## #1 a (2026-09-17T09:00:00Z)

  ```md
  plan draft
  ```
  ````

说明：订阅列表变化不会结束 `--as` 过滤的 wait。

说明：生成器等待约 2.5 秒，让 wait 先采样订阅变化，再发提及 b 的评论。

**命令 3**

```sh
agent-coexist post docs/s --as c -m '@b can you take this?'
```

stdout：

```text
ok  post  docs/s
  comment       #2  L11
  mentions      b
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `docs/s.agent-besties.md`：

  ````text
  # s.post

  - subscribers: c

  ## #1 a (2026-09-17T09:00:00Z)

  ```md
  plan draft
  ```

  ## #2 c (2026-09-17T10:00:00Z)

  ```md
  @b can you take this?
  ```
  ````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

stdout：

```text
ok  wait-post  docs/s
  bytes         changed
  semantic      changed
  add     comment  #2
  author
    | c
  created
    | 2026-09-17T10:00:00Z
  edited
    | none
  body
    | @b can you take this?
```

stderr：

```text
waiting  wait-post  docs/s
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

<a id="native-wait-state-locked"></a>

### wait-state --status locked：observing 变化不结束，locked 解除命中

编号 `native-wait-state-locked` · 观察平台 Windows

src 目录里 bob 声明了 locked。`wait-state src --status locked` 就绪后，carol 先声明 observing，随后 bob 移除自己的声明。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as bob --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src --status locked
```

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist state src --as carol --status observing
```

stdout：

```text
ok  state  src
  set           observing  carol  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z

  ## observing

  carol 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

说明：生成器等待约 2.5 秒，让 wait 先采样 carol 的声明，再由 bob 解除 locked。

**命令 4**

```sh
agent-coexist state src --remove bob
```

stdout：

```text
ok  state  src
  removed       locked  bob
  removed-lines  L5..L8
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  carol 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 2 结束**（后台命令 2 的完整输出与退出码）

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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

<a id="native-wait-state-removed-conflict"></a>

### wait-state：冲突中的 state 文件被整体删除

编号 `native-wait-state-removed-conflict` · 观察平台 Windows

src 里 bob 和 carol 同时声明 locked（冲突）。`wait-state src` 就绪后，state 文件被手工删除，两个声明同时消失。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as bob --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 2**

```sh
agent-coexist state src --as carol --status locked
```

stdout：

```text
warn  state  src
  set           locked  carol  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  conflict      locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  src
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z, carol 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src
```

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

手工删除 `src/state.agent-besties.md`

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
ok  wait-state  src
  bytes         changed
  semantic      changed
  removed       src/state.agent-besties.md
  remove  status  locked
  remove  locked  bob
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
  remove  locked  carol
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr：

```text
waiting  wait-state  src
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：wait-state 报告 removed 和被移除的两个 locked 声明；文件已不存在，报告不列出删除前的冲突告警，删除本身已经解除了这两个声明。

<a id="native-wait-invalid-after-ready"></a>

### wait：就绪后文件被改坏，等待以错误结束

编号 `native-wait-invalid-after-ready` · 观察平台 Windows

`wait standup` 就绪后，standup.agent-besties.md 的第 1 行被手工改成 `garbage`，评论仍在。wait 以错误结束。按 fix 行改回第 1 行（文件仍是原来的 alice #1，没有编造等待期间的新评论），重新启动 wait，就绪后重读文件，随后 bob 的回复被新的 wait 报告。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait standup
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
garbage

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

stdout：

```text
error  wait  standup
  file          standup.agent-besties.md
  error  L1  first line is "garbage", not "# <name>.<type>"
         why  the H1 is the only place the file type is recorded; without it nothing else can be parsed
         fix  set line 1 to "# standup.post" (or .brief / .agent-profile), or "# .state" for a state file
```

stderr：

```text
waiting  wait  standup
```

退出码：`1`

从上一步到这条命令结束没有观察到文件变化。

说明：按 fix 行手工把第 1 行改回 `# standup.post`（手工修复，没有可执行命令）；评论 #1 保持原样，没有写入等待期间并不存在的回复。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait standup
```

说明：重新启动原来的 wait。

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：新 wait 的起点是修复后的文件（仍是原来的 alice #1）；就绪后重读一次，确认还没有新回复。

**命令 4**

```sh
agent-coexist post standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait  standup
  bytes         changed
  semantic      changed
  add     comment  #2
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
waiting  wait  standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重新启动的 wait 报告 bob 的回复，原来的等待任务完成。

<a id="native-wait-stop-fence"></a>

### wait-post：文件在启动时就有阻断诊断，不进入等待

编号 `native-wait-stop-fence` · 观察平台 Windows

standup 帖子的 #1 代码块没有闭合。`wait-post standup --as alice` 立即以检查形错误结束（退出 1），stderr 没有就绪行。按诊断手工补上闭合行后重新启动 wait，就绪后 bob 回复命中。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
please reply @bob
````

**命令 1**

```sh
agent-coexist wait-post standup --as alice
```

stdout：

```text
error  wait-post  standup
  file          standup.agent-besties.md
  type          post
  error  L5  code fence opened at L5 is never closed
         why  everything after it is treated as fence content, so no later comment, entry or section can be found
         fix  add a closing line of 3 backticks where the block should end
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：文件无法解析时 wait 不会建立起点：没有就绪行，直接报告诊断。

说明：按 fix 行手工补上三个反引号的闭合行（这是手工修复，没有可执行命令）。

手工编辑 `standup.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
please reply @bob
```
````

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post standup --as alice
```

说明：重新执行原来的 wait-post。

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  standup`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist post standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  please reply @bob
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait-post  standup
  bytes         changed
  semantic      changed
  add     comment  #2
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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：修复后的 wait 正常就绪，并报告 bob 的回复。

<a id="native-wait-stop-state-dup"></a>

### wait-state、wait：state 文件里同一 nick 重复声明，不进入等待

编号 `native-wait-stop-state-dup` · 观察平台 Windows

src/state.agent-besties.md 在 locked 下有两行 bob，两行都能解析（没有无法解析的片段）。bob 想知道 src 里何时出现新的 locked 声明，也想知道这个 state 文件的任何变化。check-state 能读这个文件（取较晚的结束时间），但 bob 运行 `wait-state src --status locked` 和通用的 `wait src/state` 时，两者都拒绝这个起点：首行 error，退出 1，没有就绪行。两个 fix 行相同。这一夹具没有脏行，bob 只执行行末给本人的命令 `the command for bob: agent-coexist state src --as=bob --status=locked`（重新声明，把两行换成一条从现在开始的声明），然后自己重新启动原来的两个 wait（fix 行没有给出重启的命令）；就绪后 carol 声明 locked，两个 wait 都报告这次变化。先出现的声明写在脏行上时，见 [native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty)。

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z
bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z
```

**命令 1**

```sh
agent-coexist check-state src
```

stdout：

```text
ok  check-state  src
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T12:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：对照：普通读取命令接受这个文件。

**命令 2**

```sh
agent-coexist wait-state src --status locked
```

stdout：

```text
error  wait-state  src
  file          src/state.agent-besties.md
  type          state
  warn   L6  bob appears twice under locked
         why  the latest end time wins; the next state write that rewrites the locked section (a declaration or removal under it, or expiry cleanup of an entry in it) merges them into one unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)
         fix  the wait cannot start while a nick is listed twice under one status, so start it again only after the duplicate is gone; to merge, bob itself declares again, which replaces both entries with one entry from now (add --for to choose its length), but only when both entries are on clean lines; when either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is and the duplicate stays: inspect the file, bob deletes its own entries from every such line by hand, keeping each line's other content, then looks at the remaining entries and declares again to merge any duplicate that is left or to start one entry from now; the command for bob: agent-coexist state src --as=bob --status=locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 启动 wait-state（wait-state 没有 --as，等待者不在命令里出现）。

**命令 3**

```sh
agent-coexist wait src/state
```

stdout：

```text
error  wait  src/state
  file          src/state.agent-besties.md
  type          state
  warn   L6  bob appears twice under locked
         why  the latest end time wins; the next state write that rewrites the locked section (a declaration or removal under it, or expiry cleanup of an entry in it) merges them into one unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)
         fix  the wait cannot start while a nick is listed twice under one status, so start it again only after the duplicate is gone; to merge, bob itself declares again, which replaces both entries with one entry from now (add --for to choose its length), but only when both entries are on clean lines; when either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is and the duplicate stays: inspect the file, bob deletes its own entries from every such line by hand, keeping each line's other content, then looks at the remaining entries and declares again to merge any duplicate that is left or to start one entry from now; the command for bob: agent-coexist state src --as=bob --status=locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 改用通用的 wait 直接等待这个 state 文件的任何变化，同样不进入等待。

说明：这一夹具两条都在干净行上：bob 本人执行行末给他的重新声明后，重复没有了，所以再启动原来的两个 wait。脏行上重新声明仍留重复、不能启动，见 [native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty)。

**命令 4**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist state src --as=bob --status=locked'
```

stdout：

```text
ok  state  src
  set           locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

说明：bob 本人只执行 fix 行中的 state 命令（Git Bash）。

查看文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

**命令 5**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src --status locked
```

说明：bob 重新执行原来的 wait-state。

命令 5 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

**命令 6**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait src/state
```

说明：bob 重新执行原来的通用 wait。

命令 6 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  src/state`；下面的步骤都在这之后执行。

**命令 7**

```sh
agent-coexist state src --as carol --status locked
```

stdout：

```text
warn  state  src
  set           locked  carol  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  conflict      locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z  src
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z, carol 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 5 结束**（后台命令 5 的完整输出与退出码）

stdout：

```text
warn  wait-state  src
  bytes         changed
  semantic      changed
  warn   L5  locked is held by both bob and carol
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state src --remove <nick> --status=locked
  add     locked  carol
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr：

```text
waiting  wait-state  src
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：等待已完成；报告仍列出 bob/carol 的 locked 冲突

**命令 6 结束**（后台命令 6 的完整输出与退出码）

stdout：

```text
warn  wait  src/state
  bytes         changed
  semantic      changed
  warn   L5  locked is held by both bob and carol
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state src --remove <nick> --status=locked
  add     locked  carol
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr：

```text
waiting  wait  src/state
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：等待已完成；报告仍列出 bob/carol 的 locked 冲突

<a id="native-wait-stop-state-dup-dirty"></a>

### wait-state：重复声明里先出现的一条与无法解析片段同一行，不进入等待

编号 `native-wait-stop-state-dup-dirty` · 观察平台 Windows

src/state.agent-besties.md 在 locked 下先有一行 bob 与无法解析的 "rubbish" 同在一行，随后另有一行可解析的 bob。bob 运行 `wait-state src --status locked` 时拒绝这个起点：首行 error，退出 1，没有就绪行。check-state 能读这个文件。本场景按打印出的手工前提从脏行删掉 bob 的声明、留下 "rubbish"（文件不记录它是谁写的，不改）。删完后只剩一条可解析的 bob，没有剩下的重复，也不要一条从现在开始的声明，所以不执行行末命令，然后重新启动原来的 wait-state；就绪后 carol 声明 locked，wait 报告这次变化。对照两行都能解析的 [native-wait-stop-state-dup](#native-wait-stop-state-dup)。

起始文件 `src/state.agent-besties.md`：

```text
# .state

## locked

bob 2026-09-17T09:00:00Z..2026-09-17T11:00:00Z, rubbish
bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z
```

**命令 1**

```sh
agent-coexist check-state src
```

stdout：

```text
warn  check-state  src
  locked        bob  2026-09-17T09:30:00Z..2026-09-17T12:00:00Z
  partial       src  W-STATE-ENTRY  L5
  fix           some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; see what each one holds: agent-coexist check src/state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：对照：普通读取命令接受这个文件，并能看到脏行上的 W-STATE-ENTRY。

**命令 2**

```sh
agent-coexist wait-state src --status locked
```

stdout：

```text
error  wait-state  src
  file          src/state.agent-besties.md
  type          state
  warn   L6  bob appears twice under locked
         why  the latest end time wins; the next state write that rewrites the locked section (a declaration or removal under it, or expiry cleanup of an entry in it) merges them into one unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)
         fix  the wait cannot start while a nick is listed twice under one status, so start it again only after the duplicate is gone; to merge, bob itself declares again, which replaces both entries with one entry from now (add --for to choose its length), but only when both entries are on clean lines; when either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is and the duplicate stays: inspect the file, bob deletes its own entries from every such line by hand, keeping each line's other content, then looks at the remaining entries and declares again to merge any duplicate that is left or to start one entry from now; the command for bob: agent-coexist state src --as=bob --status=locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 启动 wait-state。先出现的 bob 与无法解析片段同一行，后一条可解析；wait 仍因重复声明拒绝，没有就绪行。

说明：按打印出的手工前提：查看文件，从每一条带无法解析片段的行删掉 bob 的声明，保留该行其余内容。"rubbish" 没有交代作者，原样留下；后一条可解析的 bob 不动。这一步没有可执行命令。

手工编辑 `src/state.agent-besties.md`（覆盖原内容），编辑后完整内容：

```text
# .state

## locked

rubbish
bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z
```

查看文件 `src/state.agent-besties.md`：

```text
# .state

## locked

rubbish
bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z
```

说明：这一夹具删完后只剩一条可解析的 bob：没有剩下的重复，也不要一条从现在开始的声明，所以不执行行末的 state 命令，直接重新启动原来的 wait-state。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src --status locked
```

说明：bob 重新执行原来的 wait-state。

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

**命令 4**

```sh
agent-coexist state src --as carol --status locked
```

stdout：

```text
warn  state  src
  set           locked  carol  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  conflict      locked  bob  2026-09-17T09:30:00Z..2026-09-17T12:00:00Z  src
  partial       src  W-STATE-ENTRY  L5
  fix           the declaration above is written, but the conflict check could not read everything it needed (the rows above that start with skipped or partial), so a conflicting lock may have been missed; without declaring again, look at what it could not read and agree with any holder named there: agent-coexist check src/state
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  ## locked

  rubbish
  bob 2026-09-17T09:30:00Z..2026-09-17T12:00:00Z, carol 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
warn  wait-state  src
  bytes         changed
  semantic      changed
  warn   L5  "rubbish" under locked is not "<nick> <start>..<until>"
         why  the fragment is kept but ignored by conflict checks and cleanup
         fix  this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it
  warn   L6  locked is held by both bob and carol
         why  two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues
         fix  agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state src --remove <nick> --status=locked
  add     locked  carol
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr：

```text
waiting  wait-state  src
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：等待已完成；报告仍列出 bob/carol 的 locked 冲突

<a id="native-wait-stop-wrong-type"></a>

### wait-post 指向 brief、wait-state 的 state 文件实为 post

编号 `native-wait-stop-wrong-type` · 观察平台 Windows

bob 本来要等 docs 里的 standup 帖子，却把同目录的导读名 notes 当作帖子名，运行 `wait-post docs/notes`。src/state.agent-besties.md 的 H1 是 `# x.post`。两个 wait 都立即以 E-TYPE 结束（退出 1，无就绪行）。按建议列出 docs 下的帖子，只把路径换成原来要等的 standup、不加过滤地等待它；把错误的 state 文件改名为 x.agent-besties.md（名字取自现有 H1 `# x.post`，第 1 行已经符合，不必再改；src 里当时没有占用该名的文件）后重试 wait-state。

起始文件 `docs/notes.agent-besties.md`：

````text
# notes.brief

```md
Start here.
```
````

起始文件 `docs/standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

起始文件 `src/state.agent-besties.md`：

```text
# x.post
```

**命令 1**

```sh
agent-coexist wait-post docs/notes
```

stdout：

```text
error  wait-post  docs/notes
  reason        file type is brief, not post
  fix           this command needs a file of type post; rerun it with such a path; files of that type under this directory: agent-coexist list docs --type=post
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 2**

```sh
MSYS_NO_PATHCONV=1 bash --noprofile --norc -c 'agent-coexist list docs --type=post'
```

stdout：

```text
ok  list  docs
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行列出 docs 下类型为 post 的文件（Git Bash）。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post docs/standup
```

说明：列出的 standup 就是 bob 原来要等的帖子；只把路径换成它，其余与原命令相同（没有 --as 过滤）。

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  docs/standup`；下面的步骤都在这之后执行。

**命令 4**

```sh
agent-coexist post docs/standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  docs/standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `docs/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
ok  wait-post  docs/standup
  bytes         changed
  semantic      changed
  add     comment  #2
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
waiting  wait-post  docs/standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：原命令没有 --as，帖子的任何变化都会结束它；这里由 bob 的回复结束原来要等的 standup，原来的等待任务完成。

**命令 5**

```sh
agent-coexist wait-state src
```

stdout：

```text
error  wait-state  src
  reason        file type is post, not state
  fix           state.agent-besties.md in this directory has type post, so it cannot hold state; rename that file to <name>.agent-besties.md with a valid name that no file has yet (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name, then retry: agent-coexist wait-state src
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 6**

```sh
agent-coexist list src
```

stdout：

```text
ok  list  src
  ignore        none
  state                       post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：改名前确认新名字没有被占用：列表里没有 x。

说明：新名字 x 取自该文件现有 H1 `# x.post`，第 1 行已经符合，不必再改。按 fix 行先把它改名（手工操作），再执行其中的重试命令。

手工改名 `src/state.agent-besties.md` → `src/x.agent-besties.md`

**命令 7**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src
```

说明：fix 行中的重试命令。

命令 7 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

**命令 8**

```sh
agent-coexist state src --as carol --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  carol  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  carol 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 7 结束**（后台命令 7 的完整输出与退出码）

stdout：

```text
ok  wait-state  src
  bytes         changed
  semantic      changed
  created       src/state.agent-besties.md
  add     status  locked
  add     locked  carol
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr：

```text
waiting  wait-state  src
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重试的 wait-state 以缺失的 state 文件为起点就绪，并报告新建的声明。

<a id="native-wait-start-parent-not-dir"></a>

### wait-post、wait-state：路径的上级是普通文件，不进入等待

编号 `native-wait-start-parent-not-dir` · 观察平台 Windows

notes 是当前目录里的普通文件，alice 的帖子是当前目录的 standup。alice 误把 notes 当作目录，运行 `wait-post notes/standup --as alice`：首行 error，reason 说明 notes 不是目录，fix 只要求改正路径（没有可执行的命令），退出 1，没有就绪行。alice 改用正确路径 standup 重新启动原来的 wait-post，bob 的回复命中。wait-state 的同类错误报告不同：bob 把文件 src/parser/mod.rs 当作目录运行 `wait-state`，得到 E-NOT-DIR，fix 是改为所在目录的完整命令；bob 执行它，就绪后 carol 声明 locked 命中。

起始文件 `notes`：

```text
meeting notes
```

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

起始文件 `src/parser/mod.rs`：

```text
pub fn parse() {}
```

**命令 1**

```sh
agent-coexist wait-post notes/standup --as alice
```

stdout：

```text
error  wait-post  notes/standup
  reason        cannot read notes: not a directory
  fix           fix the path, then run the corrected command
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：wait-post 在建立起点之前检查上级目录：notes 是普通文件，没有就绪行。

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post standup --as alice
```

说明：alice 按 fix 改正路径：帖子在当前目录，路径是 standup；--as alice 与原命令相同。

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  standup`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist post standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait-post  standup
  bytes         changed
  semantic      changed
  add     comment  #2
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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：改正路径后的 wait-post 报告 bob 的回复，原来的等待任务完成。

**命令 4**

```sh
agent-coexist wait-state src/parser/mod.rs --status locked
```

stdout：

```text
error  wait-state  src/parser/mod.rs
  reason        src/parser/mod.rs is not a directory
  fix           rerun on its directory: agent-coexist wait-state src/parser --status locked
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：bob 想等 src/parser 里出现 locked 声明，却把其中的文件当作目录传给 wait-state。

**命令 5**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src/parser --status locked
```

说明：bob 执行 fix 行中的命令：路径换成所在目录，--status locked 保留。

命令 5 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src/parser`；下面的步骤都在这之后执行。

**命令 6**

```sh
agent-coexist state src/parser --as carol --status locked
```

stdout：

```text
ok  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  carol  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/parser/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  carol 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 5 结束**（后台命令 5 的完整输出与退出码）

stdout：

```text
ok  wait-state  src/parser
  bytes         changed
  semantic      changed
  created       src/parser/state.agent-besties.md
  add     locked  carol
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr：

```text
waiting  wait-state  src/parser
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重试的 wait-state 报告 carol 新建的 locked 声明，原来的等待任务完成。

<a id="native-wait-start-dir-missing"></a>

### wait：上级目录不存在，不进入等待

编号 `native-wait-start-dir-missing` · 观察平台 Windows

alice 要等 bob 在新目录 review 里创建 parser 帖子，但 review 还不存在。`wait review/parser` 以 E-DIR-MISSING 结束（退出 1，没有就绪行）：wait 不创建目录。fix 分两种情况：目录名写错就用正确名称重跑；目录是新的就先创建再重试。这里是新目录：alice 创建 review，再执行 fix 里的重试命令；就绪后 bob 发帖创建 review/parser，wait 报告 created。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist wait review/parser
```

stdout：

```text
error  wait  review/parser
  reason        directory review does not exist
  fix           if the directory name is wrong, rerun with the correct one; if it is new, create it first (this command never creates directories), then retry: agent-coexist wait review/parser
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：上级目录不存在时 wait 不建立起点（同一路径的文件不存在本身是合法起点，见 [native-wait-created-removed](#native-wait-created-removed)）。

说明：review 是新目录，按 fix 的第二种情况，alice 先手工创建它（这一步没有 CLI 命令）。

手工创建目录 `review`

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait review/parser
```

说明：fix 行中的重试命令，与原来的 wait 相同。

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  review/parser`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist post review/parser --as bob -m 'parser review notes'
```

stdout：

```text
ok  post  review/parser
  created       review/parser.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `review/parser.agent-besties.md`：

  ````text
  # parser.post

  ## #1 bob (2026-09-17T10:00:00Z)

  ```md
  parser review notes
  ```
  ````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait  review/parser
  bytes         changed
  semantic      changed
  created       review/parser.agent-besties.md
  add     comment  #1
  author
    | bob
  created
    | 2026-09-17T10:00:00Z
  edited
    | none
  body
    | parser review notes
```

stderr：

```text
waiting  wait  review/parser
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重试的 wait 报告 created 和 bob 的评论，原来的等待任务完成。

<a id="native-wait-parent-gone-after-ready"></a>

### wait-post：就绪后上级目录被删除，恢复原目录后重试并补读

编号 `native-wait-parent-gone-after-ready` · 观察平台 Windows

alice 在已有的 room 目录里有一篇 standup 帖子，后台启动 `wait-post room/standup --as alice`。stderr 出现就绪行后，room 目录被整个删掉。wait 以错误结束。alice 按 fix 把原来的 room 目录建回；帖子的内容已随目录一起丢失，她再从自己已知的备份把 standup 帖子原样写回（仍是原来的 alice #1，没有新评论；文件所有者自己的操作，不是 fix 的建议），然后执行 fix 里的重试命令；就绪后再读一次帖子，随后 bob 的回复被重启的 wait 报告。启动时上级目录就不存在的情形见 [native-wait-start-dir-missing](#native-wait-start-dir-missing)。

起始文件 `room/standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post room/standup --as alice
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  room/standup`；下面的步骤都在这之后执行。

说明：另一进程把 room 目录整个删掉（其中的 standup 帖子一并消失）。

手工删除 `room`

**命令 1 结束**（后台命令 1 的完整输出与退出码）

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

退出码：`1`

从上一步到这条命令结束没有观察到文件变化。

说明：下一次采样打不开文件：上级目录已不在，wait 以错误结束，不是把帖子当作删除命中。

说明：按 fix 把原来的 room 目录建回；alice 再从自己已知的备份把 standup 帖子原样写回（文件所有者自己的操作，不是 fix 的建议）。评论 #1 保持原样，没有写入等待期间并不存在的回复。

手工新建 `room/standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post room/standup --as alice
```

说明：fix 行中的重试命令。

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  room/standup`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist check room/standup
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：新 wait 的起点是建回后的文件（仍是原来的 alice #1）；就绪后重读一次，确认还没有新回复。

**命令 4**

```sh
agent-coexist post room/standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  room/standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait-post  room/standup
  bytes         changed
  semantic      changed
  add     comment  #2
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
waiting  wait-post  room/standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重新启动的 wait-post 报告 bob 的回复，原来的等待任务完成。

<a id="native-wait-parent-replaced-by-file-after-ready-linux"></a>

### wait-post：就绪后上级目录被普通文件占用，挪开后恢复原目录并补读

编号 `native-wait-parent-replaced-by-file-after-ready-linux` · 观察平台 Linux（平台专属场景）

alice 在已有的 room 目录里有一篇 standup 帖子，后台启动 `wait-post room/standup --as alice`。stderr 出现就绪行后，room 目录被挪到一边（其中的帖子仍在），原路径放上一个普通文件。wait 以错误结束。alice 按 fix 把这个文件挪到一边留下，把原来的 room 目录移回（仍是原来的 alice #1，没有新评论），执行 fix 里的重试命令；就绪后再读一次帖子，随后 bob 的回复被重启的 wait 报告。启动时上级就是普通文件的情形见 [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir)。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post room/standup --as alice -m '@bob please post your status here.'
```

stdout：

```text
ok  post  room/standup
  created       room/
  created       room/standup.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `room`
- 新建 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

**命令 2**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
agent-coexist wait-post room/standup --as alice
```

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  room/standup`；下面的步骤都在这之后执行。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mv -- room room.kept && printf '\''ordinary file\n'\'' > room'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 类型改变为普通文件 `room`：

  ```text
  ordinary file
  ```
- 新建目录 `room.kept`
- 新建 `room.kept/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````
- 删除 `room/standup.agent-besties.md`

说明：把原来的 room 目录挪到一边（帖子仍在其中），在原路径放一个普通文件。

**命令 2 结束**（后台命令 2 的完整输出与退出码）

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

退出码：`1`

从上一步到这条命令结束没有观察到文件变化。

说明：下一次采样时上级已不是目录，wait 以错误结束，不是把帖子当作删除命中。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mv -- room occupied && mv -- room.kept room'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `occupied`：

  ```text
  ordinary file
  ```
- 类型改变为目录 `room`
- 删除目录 `room.kept`
- 删除 `room.kept/standup.agent-besties.md`
- 新建 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

说明：按 fix 把占用路径的普通文件挪到一边留下，把原来的 room 目录移回（仍是原来的 alice #1）。

**命令 5**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
agent-coexist wait-post room/standup --as alice
```

说明：fix 行中的重试命令。

命令 5 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  room/standup`；下面的步骤都在这之后执行。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check room/standup
```

stdout：

```text
ok  check  room/standup
  file          room/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T10:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：新 wait 的起点是移回后的文件（仍是原来的 alice #1）；就绪后重读一次，确认还没有新回复。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post room/standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  room/standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 5 结束**（后台命令 5 的完整输出与退出码）

stdout：

```text
ok  wait-post  room/standup
  bytes         changed
  semantic      changed
  add     comment  #2
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
waiting  wait-post  room/standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重新启动的 wait-post 报告 bob 的回复，原来的等待任务完成。

<a id="native-wait-file-became-dir-after-ready-linux"></a>

### wait：就绪后被监视的文件换成同名目录，挪开后重试并补读

编号 `native-wait-file-became-dir-after-ready-linux` · 观察平台 Linux（平台专属场景）

`wait standup` 就绪后，正在等待的 standup.agent-besties.md 被换成同名目录。wait 以错误结束。alice 按 fix 把这个目录挪到一边留下，不在原路径再放文件，执行 fix 里的重试命令；就绪后再读一次，文件仍不存在。随后 bob 发帖创建它，重启的 wait 报告 created。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post standup --as alice -m '@bob please post your status here.'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

**命令 2**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
agent-coexist wait standup
```

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'rm -f -- standup.agent-besties.md && mkdir -- standup.agent-besties.md'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 类型改变为目录 `standup.agent-besties.md`

说明：把正在等待的文件换成同名目录。

**命令 2 结束**（后台命令 2 的完整输出与退出码）

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

退出码：`1`

从上一步到这条命令结束没有观察到文件变化。

说明：下一次采样打开的不是普通文件，wait 以错误结束，不是把帖子当作删除命中。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mv -- standup.agent-besties.md occupied'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `occupied`
- 删除目录 `standup.agent-besties.md`

说明：按 fix 把占用该路径的目录挪到一边留下，不在原路径再放文件。

**命令 5**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
agent-coexist wait standup
```

说明：fix 行中的重试命令。

命令 5 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check standup
```

stdout：

```text
error  check  standup
  reason        standup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按 fix 就绪后再读一次：原路径仍没有普通文件，check 报告它不存在。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post standup --as bob -m 'parser done'
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 bob (2026-09-17T10:00:00Z)

  ```md
  parser done
  ```
  ````

**命令 5 结束**（后台命令 5 的完整输出与退出码）

stdout：

```text
ok  wait  standup
  bytes         changed
  semantic      changed
  created       standup.agent-besties.md
  add     comment  #1
  author
    | bob
  created
    | 2026-09-17T10:00:00Z
  edited
    | none
  body
    | parser done
```

stderr：

```text
waiting  wait  standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重启的 wait 以缺失文件为起点，报告 bob 发帖创建它，原来的等待任务完成。

<a id="native-ctrlc-wait-windows"></a>

### wait：在 Windows 控制台按 Ctrl-C 取消

编号 `native-ctrlc-wait-windows` · 观察平台 Windows（平台专属场景）

在真实 ConPTY 控制台中运行 `wait standup`，stdout 和 stderr 分别重定向到文件；就绪行写入 stderr 后，向控制台键入 Ctrl-C（0x03）。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
hi
```
````

**命令 1**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist wait standup >"⟨capture⟩\conpty-1.stdout" 2>"⟨capture⟩\conpty-1.stderr"
```

stdout：（空，0 字节）

stderr：

```text
waiting  wait  standup
```

终端帧，真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h
```

退出码：`130`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout、stderr）。

说明：就绪行出现后，生成器向控制台键入 Ctrl-C。退出码 130，stdout 为空：取消不是变化报告。终端帧只含 cmd/ConPTY 的控制序列。

<a id="native-ctrlc-wait-post-windows"></a>

### wait-post：在 Windows 控制台按 Ctrl-C 取消

编号 `native-ctrlc-wait-post-windows` · 观察平台 Windows（平台专属场景）

在真实 ConPTY 控制台中运行 `wait-post standup --as alice`，stdout 和 stderr 分别重定向到文件；就绪行写入 stderr 后，向控制台键入 Ctrl-C（0x03）。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
hi
```
````

**命令 1**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist wait-post standup --as alice >"⟨capture⟩\conpty-1.stdout" 2>"⟨capture⟩\conpty-1.stderr"
```

stdout：（空，0 字节）

stderr：

```text
waiting  wait-post  standup
```

终端帧，真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h
```

退出码：`130`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout、stderr）。

说明：就绪行出现后，生成器向控制台键入 Ctrl-C。退出码 130，stdout 为空：取消不是变化报告。终端帧只含 cmd/ConPTY 的控制序列。

<a id="native-ctrlc-wait-state-windows"></a>

### wait-state：在 Windows 控制台按 Ctrl-C 取消

编号 `native-ctrlc-wait-state-windows` · 观察平台 Windows（平台专属场景）

在真实 ConPTY 控制台中运行 `wait-state src --status locked`，stdout 和 stderr 分别重定向到文件；就绪行写入 stderr 后，向控制台键入 Ctrl-C（0x03）。

起始目录 `src`（空目录）

**命令 1**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist wait-state src --status locked >"⟨capture⟩\conpty-1.stdout" 2>"⟨capture⟩\conpty-1.stderr"
```

stdout：（空，0 字节）

stderr：

```text
waiting  wait-state  src
```

终端帧，真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h
```

退出码：`130`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout、stderr）。

说明：就绪行出现后，生成器向控制台键入 Ctrl-C。退出码 130，stdout 为空：取消不是变化报告。终端帧只含 cmd/ConPTY 的控制序列。

<a id="native-ctrlc-wait-linux"></a>

### wait：在 Linux 终端按 Ctrl-C 取消

编号 `native-ctrlc-wait-linux` · 观察平台 Linux（平台专属场景）

在 WSL 中用 `script` 分配真实 PTY 运行 `wait standup`，stdout 重定向到文件，stderr 留在终端；就绪行出现在终端后键入 Ctrl-C。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post standup --as alice -m hi
```

stdout：

```text
ok  post  standup
  created       standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 2**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
TERM=xterm script -qfec 'agent-coexist wait standup > ../cap/wait.stdout' /dev/null
```

命令 2 仍在运行，它的终端输出此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：（空，0 字节）

stderr：连接到终端，未单独捕获；见终端帧

终端帧，真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
waiting  wait  standup\r
^C
```

退出码：`130`

从上一步到这条命令结束没有观察到文件变化（清单列出 ⟨root⟩ 下除 bin、cap 以外的全部条目，不含重定向目标 ⟨root⟩/cap/wait.stdout（命令里写作 ../cap/wait.stdout），它的内容就是上面的 stdout）。

说明：生成器向终端键入 Ctrl-C。`script -e` 以 wait 自身的退出状态退出。终端帧是 PTY 交给终端的字节，经过了行规程处理，不是 CLI 写出的原始字节：CLI 的就绪行只以 LF 结尾，帧里 LF 前的 `\r` 是 PTY 输出处理把 LF 换成 CRLF 加上的；`^C` 是行规程对键入的 Ctrl-C 的回显，也不是 CLI 输出。重定向的 stdout 为空。

<a id="native-wait-restart-baseline"></a>

### 重新启动的 wait 不报告等待前已写入的回复

编号 `native-wait-restart-baseline` · 观察平台 Windows（平台专属场景）

第一个 `wait-post standup --as alice` 被 Ctrl-C 取消后，bob 回复了 alice；alice 随后重新启动 wait。新 wait 的起点已包含这条回复，所以 alice 在它就绪后再读一次帖子。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist wait-post standup --as alice >"⟨capture⟩\conpty-1.stdout" 2>"⟨capture⟩\conpty-1.stderr"
```

stdout：（空，0 字节）

stderr：

```text
waiting  wait-post  standup
```

终端帧，真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h
```

退出码：`130`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout、stderr）。

说明：就绪行出现后键入 Ctrl-C，第一个 wait 被取消。

**命令 2**

```sh
agent-coexist post standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

说明：没有 wait 在运行时，bob 回复了 alice。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist wait-post standup --as alice >"⟨capture⟩\conpty-2.stdout" 2>"⟨capture⟩\conpty-2.stderr"
```

说明：alice 重新启动 wait；它的起点是此刻的文件，已经包含 bob 的回复。

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  standup`；下面的步骤都在这之后执行。

**命令 4**

```sh
agent-coexist lookfor-comment standup --reply-to 1
```

stdout：

```text
ok  lookfor-comment  standup
  L9  #2  bob  2026-09-17T10:00:00Z  @#1 parser done
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：第二个 wait 就绪后（仍在运行），alice 再读一次：回复在这里读到。

说明：就绪后生成器又等待约 2.5 秒，让 wait 至少再采样两次，然后向控制台键入 Ctrl-C。

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：（空，0 字节）

stderr：

```text
waiting  wait-post  standup
```

终端帧，真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h
```

退出码：`130`

从上一步到这条命令结束没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout、stderr）。

说明：第二个 wait 一直在等，stdout 为空，从未把 bob 的回复当作新变化报告。

<a id="native-lock-wait-windows"></a>

### wait 采样时遇到锁或独占打开，以退出 3 结束

编号 `native-lock-wait-windows` · 观察平台 Windows（平台专属场景）

`wait standup` 就绪后，生成器以 LockFileEx 锁住文件；wait 以 E-LOCKED 结束。锁释放后按 fix 行重新启动 wait，并在就绪后重读文件。第二轮就绪后改为以不共享方式打开文件（E-INUSE），同样恢复；最后 bob 的回复被重启的 wait 报告。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait standup
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

说明：生成器以 LockFileEx 独占锁定 standup.agent-besties.md。

**命令 1 结束**（后台命令 1 的完整输出与退出码）

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

退出码：`3`

从上一步到这条命令结束没有观察到文件变化。

说明：下一次采样的 1 秒锁等待到期，wait 以 E-LOCKED 结束。

说明：生成器释放锁，然后执行 fix 行中的重试命令（后台）。

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait standup
```

说明：fix 行中的重试命令。

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 行要求就绪后再读一次文件：重启的 wait 不报告文件忙期间的变化。这里没有新评论。

说明：另一程序以共享模式 0 打开 standup.agent-besties.md。

**命令 2 结束**（后台命令 2 的完整输出与退出码）

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

退出码：`3`

从上一步到这条命令结束没有观察到文件变化。

说明：下一次采样打不开文件，wait 以 E-INUSE 结束。

说明：该程序关闭文件，然后执行 fix 行中的重试命令（后台）。

**命令 4**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait standup
```

说明：fix 行中的重试命令。

命令 4 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

**命令 5**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行在就绪后重读文件。

**命令 6**

```sh
agent-coexist post standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 4 结束**（后台命令 4 的完整输出与退出码）

stdout：

```text
ok  wait  standup
  bytes         changed
  semantic      changed
  add     comment  #2
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
waiting  wait  standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重启的 wait 报告 bob 的回复，原来的等待任务完成。

<a id="native-wait-io-delete-pending-windows"></a>

### wait：就绪后文件待删除，普通 I/O 失败后重试并补读

编号 `native-wait-io-delete-pending-windows` · 观察平台 Windows（平台专属场景）

`wait standup` 就绪后，另一程序对 standup.agent-besties.md 设置删除标记但句柄未关闭。wait 下一次采样无法打开文件，以普通 I/O 失败结束（退出 1，不是 E-LOCKED）。句柄关闭后文件被删除；alice 从备份写回原来的帖子（仍是 alice 的 #1，没有新评论），按 fix 行重试 wait，就绪后重读文件，随后 bob 的回复被重启的 wait 报告。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait standup
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

说明：另一程序对 standup.agent-besties.md 设置了删除标记，句柄仍打开。

**命令 1 结束**（后台命令 1 的完整输出与退出码）

stdout：

```text
error  wait  standup
  reason        cannot open standup.agent-besties.md: 拒绝访问。 (os error 5)
  fix           a retried wait starts from a new baseline and does not report changes made while the file could not be read, so after its ready line, read the file again; check the permissions of standup.agent-besties.md and of the directories on its path, or wait until another program stops using, deleting or replacing it, then retry: agent-coexist wait standup
```

stderr：

```text
waiting  wait  standup
```

退出码：`1`

从上一步到这条命令结束没有观察到文件变化。

说明：下一次采样打不开文件，wait 以普通 I/O 失败结束（os error 5），不是锁。

说明：持有句柄的程序关闭句柄，删除标记生效，文件被删除。

这一操作造成的文件变化：

- 删除 `standup.agent-besties.md`

说明：alice 把这份帖子从备份写回原位置（手工操作）；内容仍是原来的 alice #1，没有编造等待期间的新评论。

手工新建 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait standup
```

说明：fix 行中的重试命令。

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  standup`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist check standup
```

stdout：

```text
ok  check  standup
  file          standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行在就绪后重读文件：重启的 wait 不报告无法读取期间的变化。这里没有新评论。

**命令 4**

```sh
agent-coexist post standup --as bob -m '@#1 parser done'
```

stdout：

```text
ok  post  standup
  comment       #2  L9
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T10:00:00Z)

  ```md
  @#1 parser done
  ```
  ````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait  standup
  bytes         changed
  semantic      changed
  add     comment  #2
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
waiting  wait  standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重启的 wait 报告 bob 的回复，原来的等待任务完成。

<a id="native-lock-wait-state-windows"></a>

### wait-state 采样时 state 文件被锁，以退出 3 结束

编号 `native-lock-wait-state-windows` · 观察平台 Windows（平台专属场景）

`wait-state src --status locked` 就绪后，生成器以 LockFileEx 锁住 src/state.agent-besties.md；wait-state 以 E-LOCKED 结束。锁释放后按 fix 行重新启动，并按要求在就绪后对该目录重新运行 check-state；最后 carol 声明 locked，被重启的 wait-state 报告。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as bob --status observing
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src --status locked
```

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

说明：生成器以 LockFileEx 独占锁定 src/state.agent-besties.md。

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
error  wait-state  src
  reason        file is locked by another process
  fix           a retried wait starts from a new baseline and does not report changes made while the file was busy, so after its ready line, run check-state on the directory again; retry shortly: agent-coexist wait-state src --status locked
```

stderr：

```text
waiting  wait-state  src
```

退出码：`3`

从上一步到这条命令结束没有观察到文件变化。

说明：下一次采样的 1 秒锁等待到期，wait-state 以 E-LOCKED 结束。

说明：生成器释放锁，然后执行 fix 行中的重试命令（后台）。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src --status locked
```

说明：fix 行中的重试命令。

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

**命令 4**

```sh
agent-coexist check-state src
```

stdout：

```text
ok  check-state  src
  observing     bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fix 行要求就绪后对目录重新运行 check-state：文件被锁期间没有新的 locked 声明。

**命令 5**

```sh
agent-coexist state src --as carol --status locked
```

stdout：

```text
ok  state  src
  set           locked  carol  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z

  ## locked

  carol 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
ok  wait-state  src
  bytes         changed
  semantic      changed
  add     locked  carol
  start
    | 2026-09-17T10:00:00Z
  until
    | 2026-09-17T12:00:00Z
```

stderr：

```text
waiting  wait-state  src
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：重启的 wait-state 报告 carol 的 locked 声明，原来的等待任务完成。

<a id="wait-report-post-edit"></a>

### wait-post --as：已相关的回复被修改，报告正文的修改前后

编号 `wait-report-post-edit` · 观察平台 Windows

standup 帖子里 bob 的 #2 已经回复了 alice 的 #1。alice 启动 `wait-post standup --as alice`，就绪后 bob 用 `post --edit 2` 补充自己的回复。#2 本来就与 alice 相关，修改使它再次命中。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```

## #2 bob (2026-09-17T09:00:00Z)

```md
@#1 parser done
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post standup --as alice
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  standup`；下面的步骤都在这之后执行。

**命令 2**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:05:00Z agent-coexist post standup --as bob --edit 2 -m '@#1 parser done; tests pass'
```

stdout：

```text
ok  post  standup
  comment       #2  L9  edited
  reply-to      #1
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```

  ## #2 bob (2026-09-17T09:00:00Z, edited 2026-09-17T10:05:00Z)

  ```md
  @#1 parser done; tests pass
  ```
  ````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

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

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：报告没有新增评论，而是对 #2 的两条 update：edited 从 none 变为修改时间，body 列出修改前后的正文。

<a id="wait-report-post-subscriber"></a>

### wait-post（不带 --as）：订阅列表增加和取消

编号 `wait-report-post-subscriber` · 观察平台 Windows

不带 `--as` 的 `wait-post standup` 把帖子的任何变化都当作命中，包括订阅列表。第一个 wait 就绪后 carol 订阅这篇帖子；第二个 wait 就绪后 carol 取消订阅。带 `--as` 时订阅变化不结束等待，见 [native-wait-post-subscribe-then-hit](#native-wait-post-subscribe-then-hit)。

起始文件 `standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post standup
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  standup`；下面的步骤都在这之后执行。

**命令 2**

```sh
agent-coexist subscribe standup --as carol
```

stdout：

```text
ok  subscribe  standup
  subscribers   carol
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  - subscribers: carol

  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

stdout：

```text
ok  wait-post  standup
  bytes         changed
  semantic      changed
  add     subscriber  carol
```

stderr：

```text
waiting  wait-post  standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：报告只有 add subscriber carol 一条记录，评论没有变化。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-post standup
```

说明：再启动一次同样的 wait。

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-post  standup`；下面的步骤都在这之后执行。

**命令 4**

```sh
agent-coexist subscribe standup --as carol --cancel
```

stdout：

```text
ok  subscribe  standup
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post


  ## #1 alice (2026-09-17T09:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
ok  wait-post  standup
  bytes         changed
  semantic      changed
  remove  subscriber  carol
```

stderr：

```text
waiting  wait-post  standup
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：取消订阅报告为 remove subscriber carol。

<a id="wait-report-state-renewal"></a>

### wait-state --status locked：续期报告起止时间的修改前后

编号 `wait-report-state-renewal` · 观察平台 Windows

bob 在 10:00 锁住 src（默认 2 小时，到 12:00）。carol 启动 `wait-state src --status locked`，就绪后 bob 在 10:30 再次声明 locked，时长 3 小时。重新声明从当前时间重新开始，所以这是一次续期，wait-state 把它当作命中。

起始目录 `src`（空目录）

**命令 1**

```sh
agent-coexist state src --as bob --status locked
```

stdout：

```text
ok  state  src
  created       src/state.agent-besties.md
  set           locked  bob  2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:00:00Z..2026-09-17T12:00:00Z
  ```

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait-state src --status locked
```

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait-state  src`；下面的步骤都在这之后执行。

**命令 3**

```sh
AGENT_COEXIST_NOW=2026-09-17T10:30:00Z agent-coexist state src --as bob --status locked --for 3h
```

stdout：

```text
ok  state  src
  set           locked  bob  2026-09-17T10:30:00Z..2026-09-17T13:30:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `src/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  bob 2026-09-17T10:30:00Z..2026-09-17T13:30:00Z
  ```

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait-state  src
  bytes         changed
  semantic      changed
  update  locked  bob  start
  before
    | 2026-09-17T10:00:00Z
  after
    | 2026-09-17T10:30:00Z
  update  locked  bob  until
  before
    | 2026-09-17T12:00:00Z
  after
    | 2026-09-17T13:30:00Z
```

stderr：

```text
waiting  wait-state  src
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：续期不是新增声明：报告是 bob 的 locked 声明的两条 update，start 和 until 各列出修改前后的时间。

<a id="wait-report-brief"></a>

### wait：导读新增条目、修改条目说明、修改描述

编号 `wait-report-brief` · 观察平台 Windows

alice 的导读 onboarding 已有描述，还没有条目。bob 用 `wait onboarding` 等这份导读变化，依次启动三个 wait，每个就绪后 alice 改一次导读：先为 src/parser/mod.rs 新增条目 parser entry point，再改这个条目的说明，最后改导读描述。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

**命令 1**

```sh
agent-coexist brief onboarding --as alice -m 'Start here to review the parser change.'
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Start here to review the parser change.
  ```
  ````

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait onboarding
```

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  onboarding`；下面的步骤都在这之后执行。

**命令 3**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' --regex 'fn parse_document' -m 'Start at parse_document.'
```

stdout：

```text
ok  brief  onboarding
  entry         parser entry point  added  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Start here to review the parser change.
  ```

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document.
  ```
  ````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait  onboarding
  bytes         changed
  semantic      changed
  add     entry  parser entry point
  path
    | src/parser/mod.rs
  hash
    | c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  regex
    | fn parse_document
  note
    | Start at parse_document.
```

stderr：

```text
waiting  wait  onboarding
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：新增条目报告为 add entry，后面列出条目的 path、hash、regex 和 note。

**命令 4**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait onboarding
```

说明：再启动一次同样的 wait。

命令 4 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  onboarding`；下面的步骤都在这之后执行。

**命令 5**

```sh
agent-coexist brief onboarding --as alice --entry 'parser entry point' -m 'Start at parse_document; check its error mapping first.'
```

stdout：

```text
ok  brief  onboarding
  note          set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Start here to review the parser change.
  ```

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document; check its error mapping first.
  ```
  ````

**命令 4 结束**（后台命令 4 的完整输出与退出码）

stdout：

```text
ok  wait  onboarding
  bytes         changed
  semantic      changed
  update  entry  parser entry point  note
  before
    | Start at parse_document.
  after
    | Start at parse_document; check its error mapping first.
```

stderr：

```text
waiting  wait  onboarding
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：只改了说明：报告是 update entry … note，列出修改前后的说明；path、hash、regex 没有变，不出现。

**命令 6**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait onboarding
```

说明：第三次启动同样的 wait。

命令 6 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  onboarding`；下面的步骤都在这之后执行。

**命令 7**

```sh
agent-coexist brief onboarding --as alice -m 'Review route for the parser error-mapping change.'
```

stdout：

```text
ok  brief  onboarding
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Review route for the parser error-mapping change.
  ```

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document; check its error mapping first.
  ```
  ````

**命令 6 结束**（后台命令 6 的完整输出与退出码）

stdout：

```text
ok  wait  onboarding
  bytes         changed
  semantic      changed
  update  description
  before
    | Start here to review the parser change.
  after
    | Review route for the parser error-mapping change.
```

stderr：

```text
waiting  wait  onboarding
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：导读描述不属于任何条目，报告是不带标识的 update description，列出修改前后的描述。

<a id="wait-report-document-name"></a>

### wait：H1 名称改正，报告 document name 的修改前后

编号 `wait-report-document-name` · 观察平台 Windows

notes.agent-besties.md 的第 1 行写成了 `# note.post`，check 给出 W-NAME 警告。bob 用 `wait notes` 等这篇帖子变化；就绪后 alice 按 W-NAME 的第二种改法，手工把第 1 行改为 `# notes.post`。

起始文件 `notes.agent-besties.md`：

````text
# note.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Meeting notes go here.
```
````

**命令 1**

```sh
agent-coexist check notes
```

stdout：

```text
warn  check  notes
  file          notes.agent-besties.md
  type          post
  warn   L1  file name "notes" does not match the H1 name "note"
         why  list shows the file name, the file says another name; readers will be confused
         fix  rename the file to note.agent-besties.md if no file has that name yet (never overwrite an existing file), or set line 1 to "# notes.post"
  comments      1
  authors       alice
  topic         #1  alice  Meeting notes go here.
  latest        #1  alice  2026-09-17T09:00:00Z  Meeting notes go here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：W-NAME 是警告，不阻止 wait 以这个文件为起点。

**命令 2**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait notes
```

命令 2 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  notes`；下面的步骤都在这之后执行。

说明：alice 按 fix 中「set line 1」的改法手工改正第 1 行（这是手工修复，没有可执行命令）；评论保持原样。

手工编辑 `notes.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# notes.post

## #1 alice (2026-09-17T09:00:00Z)

```md
Meeting notes go here.
```
````

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  wait  notes
  bytes         changed
  semantic      changed
  update  document  name
  before
    | note
  after
    | notes
```

stderr：

```text
waiting  wait  notes
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：报告是 update document name，列出修改前后的名称；改正后的文件没有警告，评论没有变化，所以没有其他记录。

<a id="wait-report-type-switch"></a>

### wait：导读被改写为档案，报告类型切换

编号 `wait-report-type-switch` · 观察平台 Windows

agents/alice 原是 alice 用 brief 命令建的导读，有描述和一个条目。她决定把这个文件改作自己的档案。bob 用 `wait agents/alice` 等这个文件的任何变化（通用 wait 接受四种类型）。就绪后 alice 手工把文件改写成档案。只有通用 wait 能观察类型切换：wait-post 和 wait-state 遇到错误类型会以 E-TYPE 结束。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

起始目录 `agents`（空目录）

**命令 1**

```sh
agent-coexist brief agents/alice --as alice -m 'Notes for alice.'
```

stdout：

```text
ok  brief  agents/alice
  created       agents/alice.agent-besties.md
  description   set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `agents/alice.agent-besties.md`：

  ````text
  # alice.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Notes for alice.
  ```
  ````

**命令 2**

```sh
agent-coexist brief agents/alice --as alice --target ../src/parser/mod.rs --entry parser
```

stdout：

```text
ok  brief  agents/alice
  entry         parser  added  ../src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

  ```md
  Notes for alice.
  ```

  ## parser

  - path: ../src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  ````

说明：--target 相对于导读所在目录 agents。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait agents/alice
```

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  agents/alice`；下面的步骤都在这之后执行。

说明：alice 手工把整个文件改写为档案：第 1 行改为 `# alice.agent-profile`，只保留一节 role。

手工编辑 `agents/alice.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# alice.agent-profile

## role

```md
Rust implementer. Owns crates/core.
```
````

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
ok  wait  agents/alice
  bytes         changed
  semantic      changed
  update  document  type
  before
    | brief
  after
    | agent-profile
  update  description
  before
    | Notes for alice.
  after
    | none
  add     section  role
  body
    | Rust implementer. Owns crates/core.
  remove  entry  parser
  path
    | ../src/parser/mod.rs
  hash
    | c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  regex
    | none
  note
    | none
```

stderr：

```text
waiting  wait  agents/alice
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：类型切换时，两种类型的内容不做对应：报告先列出 update document type 和 update description（档案没有描述，after 为 none），再按新类型列出 add section，最后按旧类型列出 remove entry。

<a id="wait-report-profile"></a>

### wait：档案的节被修改、新增、删除

编号 `wait-report-profile` · 观察平台 Windows

agents/alice 档案有 role 和 scope 两节。bob 用 `wait agents/alice` 等这份档案变化，依次启动三个 wait，每个就绪后 alice 用 profile 改一次：修改 role，新增 contact，删除 scope。

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer. Owns crates/core.
```

## scope

```md
Parser error mapping. Send review requests to parser-review.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait agents/alice
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  agents/alice`；下面的步骤都在这之后执行。

**命令 2**

```sh
agent-coexist profile agents/alice --as alice --section role -m 'Rust implementer. Owns crates/core and crates/cli.'
```

stdout：

```text
ok  profile  agents/alice
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer. Owns crates/core and crates/cli.
  ```

  ## scope

  ```md
  Parser error mapping. Send review requests to parser-review.
  ```
  ````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

stdout：

```text
ok  wait  agents/alice
  bytes         changed
  semantic      changed
  update  section  role  body
  before
    | Rust implementer. Owns crates/core.
  after
    | Rust implementer. Owns crates/core and crates/cli.
```

stderr：

```text
waiting  wait  agents/alice
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：报告是 update section role body，列出修改前后的正文。

**命令 3**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait agents/alice
```

说明：再启动一次同样的 wait。

命令 3 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  agents/alice`；下面的步骤都在这之后执行。

**命令 4**

```sh
agent-coexist profile agents/alice --as alice --section contact -m 'Reply in parser-review; I read it hourly.'
```

stdout：

```text
ok  profile  agents/alice
  section       contact  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer. Owns crates/core and crates/cli.
  ```

  ## scope

  ```md
  Parser error mapping. Send review requests to parser-review.
  ```

  ## contact

  ```md
  Reply in parser-review; I read it hourly.
  ```
  ````

**命令 3 结束**（后台命令 3 的完整输出与退出码）

stdout：

```text
ok  wait  agents/alice
  bytes         changed
  semantic      changed
  add     section  contact
  body
    | Reply in parser-review; I read it hourly.
```

stderr：

```text
waiting  wait  agents/alice
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：新增的节报告为 add section contact，后面是它的 body。

**命令 5**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait agents/alice
```

说明：第三次启动同样的 wait。

命令 5 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  agents/alice`；下面的步骤都在这之后执行。

**命令 6**

```sh
agent-coexist profile agents/alice --as alice --section scope --remove
```

stdout：

```text
ok  profile  agents/alice
  section       scope  removed
  removed-lines  L9..L14
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer. Owns crates/core and crates/cli.
  ```

  ## contact

  ```md
  Reply in parser-review; I read it hourly.
  ```
  ````

**命令 5 结束**（后台命令 5 的完整输出与退出码）

stdout：

```text
ok  wait  agents/alice
  bytes         changed
  semantic      changed
  remove  section  scope
  body
    | Parser error mapping. Send review requests to parser-review.
```

stderr：

```text
waiting  wait  agents/alice
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：删除的节报告为 remove section scope，后面是删除前的 body。

<a id="wait-report-order"></a>

### wait：只调换顺序，报告 order

编号 `wait-report-order` · 观察平台 Windows

agents/alice 档案有 role、scope 两节。`wait agents/alice` 就绪后，alice 手工把两节对调，每节的正文都不变。

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role

```md
Rust implementer. Owns crates/core.
```

## scope

```md
Parser error mapping. Send review requests to parser-review.
```
````

**命令 1**（在后台启动，继续运行；输出在它结束时列出）

```sh
agent-coexist wait agents/alice
```

命令 1 仍在运行，它的 stderr 此时已出现就绪文字 `waiting  wait  agents/alice`；下面的步骤都在这之后执行。

手工编辑 `agents/alice.agent-besties.md`（覆盖原内容），编辑后完整内容：

````text
# alice.agent-profile

## scope

```md
Parser error mapping. Send review requests to parser-review.
```

## role

```md
Rust implementer. Owns crates/core.
```
````

**命令 1 结束**（后台命令 1 的完整输出与退出码）

stdout：

```text
ok  wait  agents/alice
  bytes         changed
  semantic      changed
  order   sections
  before
    | role, scope
  after
    | scope, role
```

stderr：

```text
waiting  wait  agents/alice
```

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：没有节被新增、删除或修改，报告只有一条 order sections，before 和 after 列出调换前后的节名顺序。

<a id="ch-presentation"></a>

## 输出呈现

<a id="native-progress-windows"></a>

### Windows 控制台中的扫描进度行

编号 `native-progress-windows` · 观察平台 Windows（平台专属场景）

stderr 是真实 ConPTY 控制台、stdout 重定向到文件时，扫描超过 500 ms 后 stderr 每 200 ms 重写一行 `scanning dirs: N, files: M...`，结束时擦除。管道下同一命令的 stderr 为空，作为对照。

起始文件树：big/ 下 10000 个空子目录（a00..a99/b00..b99）和一篇帖子 big/standup.agent-besties.md；共 1 个文件、97 字节，树摘要 SHA-256 `842950f19288d69d22446986fdf79aca9037d3a41c96ebaa2fdbc5da39460d9b`（文件逐个内容未保存，私有记录里只有这里显示的文件数、字节数与树摘要；内容按上面的说明由生成器确定地生成）

说明：终端帧是 ConPTY 交给终端的字节，不是 CLI 写出的原始字节：ConPTY 按屏幕内容重新绘制，CLI 用 `\r` 重写进度行，在帧中表现为光标定位（如 `\x1b[H`）后的新计数。帧里是否出现单独的擦除序列取决于 ConPTY 何时绘制，不能据此判断 CLI 是否擦除；CLI 用 `\r` 重写、结束时用空格擦除进度行的字节，见下一个场景 [native-progress-linux](#native-progress-linux)（Linux PTY 只把 LF 换成 CRLF，进度行不含 LF，原样到达终端）。

**命令 1**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist list big >"⟨capture⟩\conpty-1.stdout"
```

stdout：

```text
ok  list  big
  ignore        none
  standup                     post
  total         1
```

stderr：连接到终端，未单独捕获；见终端帧

<!-- runtime-examples:volatile:start -->
终端帧（随时序变化：进度帧的条数和计数取决于扫描速度），真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h\x1b[?25l\x1b[2J\x1b[m\x1b[H\x1b]0;C:\\WINDOWS\\System32\\cmd.exe\x07\x1b[?25hscanning dirs: 3047, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 4340, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 5623, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 6851, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 8153, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 9394, files: 0...\x1b[?25l\x1b[H\x1b[?25h
```
<!-- runtime-examples:volatile:end -->

退出码：`0`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout）。

**命令 2**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist read-notification big --as bob --since 1h >"⟨capture⟩\conpty-2.stdout"
```

stdout：

```text
ok  read-notification  big
  as            bob
  since         2026-09-17T09:00:00Z
  mention     2026-09-17T09:00:00Z  big/standup #1  alice  @bob please post your status here.
  total         1
```

stderr：连接到终端，未单独捕获；见终端帧

<!-- runtime-examples:volatile:start -->
终端帧（随时序变化：进度帧的条数和计数取决于扫描速度），真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h\x1b[?25l\x1b[2J\x1b[m\x1b[H\x1b]0;C:\\WINDOWS\\System32\\cmd.exe\x07\x1b[?25hscanning dirs: 4193, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 5849, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 7586, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 9328, files: 0...\x1b[?25l\x1b[H\x1b[?25h
```
<!-- runtime-examples:volatile:end -->

退出码：`0`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout）。

**命令 3**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist check-state big >"⟨capture⟩\conpty-3.stdout"
```

stdout：

```text
ok  check-state  big
  total         0
```

stderr：连接到终端，未单独捕获；见终端帧

<!-- runtime-examples:volatile:start -->
终端帧（随时序变化：进度帧的条数和计数取决于扫描速度），真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h\x1b[?25l\x1b[2J\x1b[m\x1b[H\x1b]0;C:\\WINDOWS\\System32\\cmd.exe\x07\x1b[?25hscanning dirs: 4197, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 5820, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 7516, files: 0...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 9221, files: 0...\x1b[?25l\x1b[H\x1b[?25h
```
<!-- runtime-examples:volatile:end -->

退出码：`0`

这条命令没有观察到文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout）。

**命令 4**

cmd.exe 命令行（Windows 进程参数原样，不是 POSIX shell 命令；重定向目标若是 ⟨capture⟩ 只在生成机器上存在）：

```bat
cmd.exe /d /c agent-coexist state big --as bob --status observing >"⟨capture⟩\conpty-4.stdout"
```

stdout：

```text
ok  state  big
  created       big/state.agent-besties.md
  set           observing  bob  2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
```

stderr：连接到终端，未单独捕获；见终端帧

<!-- runtime-examples:volatile:start -->
终端帧（随时序变化：进度帧的条数和计数取决于扫描速度），真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\x1b[?9001h\x1b[?1004h\x1b[?25l\x1b[2J\x1b[m\x1b[H\x1b]0;C:\\WINDOWS\\System32\\cmd.exe\x07\x1b[?25hscanning dirs: 2950, files: 1...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 4191, files: 1...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 5427, files: 1...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 6716, files: 1...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 7961, files: 1...\x1b[?25l\x1b[H\x1b[?25hscanning dirs: 9192, files: 1...\x1b[?25l\x1b[H\x1b[?25h
```
<!-- runtime-examples:volatile:end -->

退出码：`0`

这条命令造成的文件变化（清单只看示例根目录 ⟨root⟩，不含重定向目标 ⟨capture⟩ 下的文件，它们的内容就是上面的 stdout）：

- 新建 `big/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## observing

  bob 2026-09-17T10:00:00Z..2026-09-17T18:00:00Z
  ```

**命令 5**

```sh
agent-coexist list big
```

stdout：

```text
ok  list  big
  ignore        none
  standup                     post
  state                       state
  total         2
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：对照：stderr 是管道时没有进度行。

<a id="native-progress-linux"></a>

### Linux 终端中的扫描进度行

编号 `native-progress-linux` · 观察平台 Linux（平台专属场景）

在 WSL 中用 `script` 分配真实 PTY，stdout 重定向到文件；stderr 的进度行 `scanning dirs: N, files: M...` 以 `\r` 重写，结束时用空格擦除。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir -p big/a{00..99}/b{00..99}/c{0..4}'
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化（共 60101 项，超过 50 项只列汇总，逐项清单只在私有记录中）：新建目录 60101 个。

说明：准备 big 下 100×100 个二级目录，每个含 5 个空的叶目录：共新建 60101 个目录（含 big 本身），其中 50000 个是空叶目录，使扫描超过 500 ms。

**命令 2**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
TERM=xterm script -qfec 'agent-coexist list big > ../cap/list.stdout' /dev/null
```

说明：这条命令在 `script` 分配的 PTY 中运行，生成器只等它结束，期间没有其他步骤，也没有就绪文字；页面按后台命令的格式在启动处和结束处各列一次，输出全部在「命令 2 结束」处。

**命令 2 结束**（后台命令 2 的完整输出与退出码）

stdout：

```text
ok  list  big
  ignore        none
  total         0
```

stderr：连接到终端，未单独捕获；见终端帧

<!-- runtime-examples:volatile:start -->
终端帧（随时序变化：进度帧的条数和计数取决于扫描速度），真实 PTY/ConPTY 字节（转义视图，末尾没有换行符）：

```text
\rscanning dirs: 27986, files: 0...\rscanning dirs: 39536, files: 0...\rscanning dirs: 50775, files: 0...\r                                 \r
```
<!-- runtime-examples:volatile:end -->

退出码：`0`

从上一步到这条命令结束没有观察到文件变化（清单列出 ⟨root⟩ 下除 bin、cap 以外的全部条目，不含重定向目标 ⟨root⟩/cap/list.stdout（命令里写作 ../cap/list.stdout），它的内容就是上面的 stdout）。

说明：终端帧是 PTY 交给终端的字节。PTY 的输出处理只把 LF 换成 CRLF；进度行不含 LF，所以帧里的 `\r` 重写和结束时的空格擦除就是 CLI 写到 stderr 的字节。

<a id="native-operands-windows"></a>

### Windows：反斜杠路径与大小写不同的路径

编号 `native-operands-windows` · 观察平台 Windows（平台专属场景）

Windows 接受 `room\standup` 作为路径，也按不区分大小写的文件系统打开 `ROOM/STANDUP`；文件名按磁盘上的实际拼写 standup 与 H1 比较，因此不告警。Linux 对照见下一场景 [native-operands-linux](#native-operands-linux)。

起始文件 `room/standup.agent-besties.md`：

````text
# standup.post

## #1 alice (2026-09-17T09:00:00Z)

```md
@bob please post your status here.
```
````

**命令 1**

```sh
agent-coexist check 'room\standup'
```

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

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check ROOM/STANDUP
```

stdout：

```text
ok  check  ROOM/STANDUP
  file          ROOM/STANDUP.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T09:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="native-operands-linux"></a>

### Linux：反斜杠和大小写是文件名的一部分

编号 `native-operands-linux` · 观察平台 Linux（平台专属场景）

读者本来要 check 的是 alice 的 room/standup 帖子，却按 Windows 写法输入了 `room\standup` 和 `ROOM/STANDUP`。同样的两条命令在 Linux 上指向不存在的文件。按 fix 行列出实际存在的文件，只确认路径，再用原来要 check 的 room/standup 完成原来的 check。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
mkdir room
```

stdout：（空，0 字节）

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `room`

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist post room/standup --as alice -m '@bob please post your status here.'
```

stdout：

```text
ok  post  room/standup
  created       room/standup.agent-besties.md
  comment       #1  L3
  mentions      bob
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `room/standup.agent-besties.md`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  @bob please post your status here.
  ```
  ````

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check 'room\standup'
```

stdout：

```text
error  check  room\standup
  reason        room\standup.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list .'
```

stdout：

```text
ok  list  .
  ignore        none
  room/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行列出实际存在的协作文件。

**命令 5**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check ROOM/STANDUP
```

stdout：

```text
error  check  ROOM/STANDUP
  reason        ROOM/STANDUP.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); its directory does not exist either; collaboration files under the nearest existing directory: agent-coexist list .
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list .'
```

stdout：

```text
ok  list  .
  ignore        none
  room/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：ROOM 目录不存在，fix 行列出的是最近的现有上级目录。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist check room/standup
```

stdout：

```text
ok  check  room/standup
  file          room/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  @bob please post your status here.
  latest        #1  alice  2026-09-17T10:00:00Z  @bob please post your status here.
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：列出的就是原来要 check 的 alice 的 room/standup；改正路径，完成原来的 check。

<a id="native-nonutf8-linux"></a>

### Linux：非 UTF-8 目录名的显示、恢复命令与手工分支

编号 `native-nonutf8-linux` · 观察平台 Linux（平台专属场景）

目录名含字节 0xFF（显示为 U+FFFD）。可用 POSIX shell 表示时，恢复命令用 `$(printf ...)` 保留原字节并可直接执行；含换行的目录名无法写成单行命令，给出手工恢复说明。为了得到带原操作数的 fix 行，场景让另一进程以 `flock -x` 暂时锁住文件（E-LOCKED，退出 3）。

起始状态：空的工作目录。

**命令 1**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir $'\''room-\xff'\'' && agent-coexist post $'\''room-\xff/standup'\'' --as alice -m hi'
```

stdout：

```text
ok  post  room-�/standup
  created       room-�/standup.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'room-\xff'`
- 新建 `$'room-\xff/standup.agent-besties.md'`：

  ````text
  # standup.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

**命令 2**（在 `⟨cwd⟩` 中执行）

```sh
agent-coexist list .
```

stdout：

```text
ok  list  .
  ignore        none
  room-�/
    standup                   post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 3**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''room-\xff/nothere'\'''
```

stdout：

```text
error  check  room-�/nothere
  reason        room-�/nothere.agent-besties.md does not exist
  fix           correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list room-"$(printf '\377')"
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

**命令 4**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist list room-"$(printf '\''\377'\'')"'
```

stdout：

```text
ok  list  room-�
  ignore        none
  standup                     post
  total         1
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按 fix 行执行：列出的是原字节目录。

**命令 5**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
bash -c 'exec flock -x $'\''room-\xff/standup.agent-besties.md'\'' -c '\''echo held; read _'\'''
```

说明：另一进程以 flock -x 锁住这份帖子；输出 held 后一直持有，直到它的 stdin 收到换行。

命令 5 仍在运行，它的 stdout 此时已出现就绪文字 `held`；下面的步骤都在这之后执行。

**命令 6**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''room-\xff/standup'\'''
```

stdout：

```text
error  check  room-�/standup
  reason        file is locked by another process
  fix           retry shortly: agent-coexist check room-"$(printf '\377')"/standup
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

**命令 5 结束**（后台命令 5 的完整输出与退出码）

stdout：

```text
held
```

stderr：（空，0 字节）

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：锁持有进程收到换行后退出，锁释放。

**命令 7**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check room-"$(printf '\''\377'\'')"/standup'
```

stdout：

```text
ok  check  room-�/standup
  file          room-�/standup.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi
  latest        #1  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：执行 E-LOCKED 的重试命令：`$(printf ...)` 还原出原字节路径。

**命令 8**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'mkdir $'\''a\nb'\'' && agent-coexist post $'\''a\nb/s'\'' --as alice -m hi'
```

stdout：

```text
ok  post  a b/s
  created       a b/s.agent-besties.md
  comment       #1  L3
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建目录 `$'a\nb'`
- 新建 `$'a\nb/s.agent-besties.md'`：

  ````text
  # s.post

  ## #1 alice (2026-09-17T10:00:00Z)

  ```md
  hi
  ```
  ````

说明：目录名含换行：显示中换行折叠为空格。

**命令 9**（在后台启动，继续运行；输出在它结束时列出，在 `⟨cwd⟩` 中执行）

```sh
bash -c 'exec flock -x $'\''a\nb/s.agent-besties.md'\'' -c '\''echo held; read _'\'''
```

说明：另一进程以 flock -x 锁住这份帖子，直到它的 stdin 收到换行。

命令 9 仍在运行，它的 stdout 此时已出现就绪文字 `held`；下面的步骤都在这之后执行。

**命令 10**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/s'\'''
```

stdout：

```text
error  check  a b/s
  reason        file is locked by another process
  fix           retry shortly by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly
```

stderr：（空，0 字节）

退出码：`3`

这条命令没有观察到文件变化。

说明：重试命令无法无损写成单行，fix 行给出手工处理说明。

**命令 9 结束**（后台命令 9 的完整输出与退出码）

stdout：

```text
held
```

stderr：（空，0 字节）

退出码：`0`

从上一步到这条命令结束没有观察到文件变化。

说明：锁持有进程退出，锁释放。

**命令 11**（在 `⟨cwd⟩` 中执行）

```sh
bash -c 'agent-coexist check $'\''a\nb/s'\'''
```

stdout：

```text
ok  check  a b/s
  file          a b/s.agent-besties.md
  type          post
  comments      1
  authors       alice
  topic         #1  alice  hi
  latest        #1  alice  2026-09-17T10:00:00Z  hi
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：按说明手工重试原命令。

<a id="read-brief-lossy-content"></a>

### read-brief：非 UTF-8 目标的内容展示

编号 `read-brief-lossy-content` · 观察平台 Windows

目标含非 UTF-8 字节与 CRLF 时，content 文本块按有损解码原样显示每一行。

起始文件 `data/raw.bin`（转义视图）：

```text
ok\r
\xffbad
```

起始文件 `docs/raw.agent-besties.md`：

```text
# raw.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## raw bytes

- path: ../data/raw.bin
- hash: c76cfe52f6ee847bc91040fe43c318ca090c34ca65d55fc104a8b3c5500d0e45
```

**命令 1**

```sh
agent-coexist read-brief docs/raw
```

stdout（转义视图）：

```text
ok  read-brief  docs/raw
  entry         raw bytes  fresh  ../data/raw.bin  L1..L2
  content
    | ok\r
    | �bad
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="read-brief-content-line-ends"></a>

### read-brief：content 文本块的行尾

编号 `read-brief-content-line-ends` · 观察平台 Windows

目标末尾的换行只结束最后一行，不再多出一行空的 | ；目标里真正的空行照常显示；空目标只有 content 键行。

起始文件 `src/one.rs`：

```text
fn a(){}
```

起始文件 `src/blank.txt`：

```text
a

```

起始文件 `src/empty.txt`：（空，0 字节）

起始文件 `docs/ends.agent-besties.md`：

```text
# ends.brief

> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.

## one line

- path: ../src/one.rs
- hash: 79f5b6e8a62a91e933ee1dd2a1e1d91aea2d90d85ee77b9ace43cfdca7faf9c1

## trailing blank line

- path: ../src/blank.txt
- hash: a7da489976d0047490617adb4f7a1f27f7af8b52a5176fd002ffe471863520ab

## empty file

- path: ../src/empty.txt
- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

**命令 1**

```sh
agent-coexist read-brief docs/ends
```

stdout：

```text
ok  read-brief  docs/ends
  entry         one line  fresh  ../src/one.rs  L1..L1
  content
    | fn a(){}
  entry         trailing blank line  fresh  ../src/blank.txt  L1..L2
  content
    | a
    | 
  entry         empty file  fresh  ../src/empty.txt
  content
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：fn a(){}\n 是一行（L1..L1）；a\n\n 是两行，第二行为空（L1..L2）；空文件没有行，也没有 L 范围。

<a id="read-list-wide-names"></a>

### list：长文件名与 Unicode 文件名的类型列

编号 `read-list-wide-names` · 观察平台 Windows

整棵树共用一个类型列起点，由最长的名称决定；列宽按 Unicode 标量计数，宽字符在终端里可能不对齐。

起始文件 `docs/设计评审.agent-besties.md`：

```text
# 设计评审.post
```

起始文件 `docs/a-very-long-collaboration-thread-name-for-column-width.agent-besties.md`：

```text
# a-very-long-collaboration-thread-name-for-column-width.post
```

起始文件 `docs/short.agent-besties.md`：

```text
# short.post
```

**命令 1**

```sh
agent-coexist list
```

stdout：

```text
ok  list  .
  ignore        none
  docs/
    a-very-long-collaboration-thread-name-for-column-width  post
    short                                                   post
    设计评审                                                    post
  total         3
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

**命令 2**

```sh
agent-coexist check 'docs/设计评审'
```

stdout：

```text
ok  check  docs/设计评审
  file          docs/设计评审.agent-besties.md
  type          post
  comments      0
  authors       none
  subscribers   none
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

<a id="ch-file-guidance"></a>

## 写入文件的指导文字

<a id="write-brief-skeleton"></a>

### brief：新建空导读

编号 `write-brief-skeleton` · 观察平台 Windows

新导读文件带一行给读者的说明：路径相对于文件所在目录，哈希是目标字节的 SHA-256。已存在时报告 unchanged。

起始状态：空的工作目录。

**命令 1**

```sh
agent-coexist brief onboarding --as alice
```

stdout：

```text
ok  brief  onboarding
  created       onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `onboarding.agent-besties.md`：

  ```text
  # onboarding.brief

  > paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.
  ```

**命令 2**

```sh
agent-coexist brief onboarding --as alice
```

stdout：

```text
ok  brief  onboarding
  unchanged     onboarding.agent-besties.md
```

stderr：（空，0 字节）

退出码：`0`

这条命令没有观察到文件变化。

说明：文件已存在，不写入。

<a id="write-brief-insert-layout"></a>

### brief：向手写导读插入说明、哈希和 regex

编号 `write-brief-insert-layout` · 观察平台 Windows

插入的说明代码块和属性行与后面的标题、说明之间保留一个空行；新属性行接在已有属性行之后，与 CLI 新建文件的排版一致。

起始文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

起始文件 `onboarding.agent-besties.md`：

````text
# onboarding.brief

## parser entry point

- path: src/parser/mod.rs
```md
Start at parse_document.
```
## renderer

- path: src/parser/mod.rs
````

**命令 1**

```sh
agent-coexist brief onboarding --as alice -m 'Start here to review the parser change.'
```

stdout：

```text
warn  brief  onboarding
  description   set
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  remaining     E-ENTRY-HASH  L7
  remaining     E-ENTRY-HASH  L13
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  ```md
  Start here to review the parser change.
  ```

  ## parser entry point

  - path: src/parser/mod.rs
  ```md
  Start at parse_document.
  ```
  ## renderer

  - path: src/parser/mod.rs
  ````

说明：说明写入后报告首行仍是 warn：remaining 行列出两个手写条目都没有 hash（E-ENTRY-HASH）——L7 的 parser entry point 和 L13 的 renderer。fix 指向对本导读运行 check；原来的插入还没做完，先继续补哈希和说明。

说明：手写条目没有哈希基线，先把目标和说明审阅一遍再写：src/parser/mod.rs 仍是 parse_document 的定义，parser entry point 的说明 "Start at parse_document." 仍然描述这个目标，不改说明。

查看文件 `src/parser/mod.rs`：

```text
pub fn parse_document(input: &str) -> Result<Document, Error> {
    todo!()
}
```

**命令 2**

```sh
agent-coexist brief onboarding --as alice --target src/parser/mod.rs --entry 'parser entry point' --regex 'fn parse_document'
```

stdout：

```text
warn  brief  onboarding
  entry         parser entry point  updated  src/parser/mod.rs
  hash          c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  remaining     E-ENTRY-HASH  L16
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  ```md
  Start here to review the parser change.
  ```

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document.
  ```
  ## renderer

  - path: src/parser/mod.rs
  ````

说明：--target 给 parser entry point 补上哈希和 regex，记下的是目标现在的哈希，说明原样保留；此后 remaining 行只剩 renderer 一条。check 指针仍在，插入完成后再跟。

**命令 3**

```sh
agent-coexist brief onboarding --as alice --entry renderer -m 'Optional depth.'
```

stdout：

```text
warn  brief  onboarding
  note          set
  fix           skipped entries and remaining errors in this file need check on this file; follow the suggestions check prints: agent-coexist check onboarding.agent-besties.md
  remaining     E-ENTRY-HASH  L16
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  ```md
  Start here to review the parser change.
  ```

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document.
  ```
  ## renderer

  - path: src/parser/mod.rs

  ```md
  Optional depth.
  ```
  ````

说明：三次写入都已完成；remaining 仍列出 renderer 的 E-ENTRY-HASH。下一条按这条报告里的 check 指针执行。

**命令 4**

```sh
agent-coexist check onboarding.agent-besties.md
```

stdout：

```text
error  check  onboarding
  file          onboarding.agent-besties.md
  type          brief
  error  L16  entry "renderer" has no valid "- hash:" line
         why  the hash is how check tells fresh from shifted; without it drift is invisible
         fix  review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief onboarding.agent-besties.md --entry=renderer --refresh
  description   Start here to review the parser change.
  entries       2
  fresh         1
  shifted       0
  stale         0
  unchecked     1
  entry         parser entry point  fresh  src/parser/mod.rs  Start at parse_document.
  entry         renderer  unchecked  src/parser/mod.rs  no hash  Optional depth.
```

stderr：（空，0 字节）

退出码：`1`

这条命令没有观察到文件变化。

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。对本导读运行 check。

说明：E-ENTRY-HASH 的 fix 先要求审阅目标和说明。renderer 的说明 "Optional depth." 是 alice 在命令 3 写下的，目标就是上面看过的 src/parser/mod.rs，期间没有改动，说明仍然成立；刷新只是给 renderer 首次记下目标现在的字节。

**命令 5**

```sh
agent-coexist brief onboarding.agent-besties.md --entry=renderer --refresh
```

stdout：

```text
ok  brief  onboarding
  entry         renderer  refreshed  src/parser/mod.rs
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `onboarding.agent-besties.md`：

  ````text
  # onboarding.brief

  ```md
  Start here to review the parser change.
  ```

  ## parser entry point

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367
  - regex: fn parse_document

  ```md
  Start at parse_document.
  ```
  ## renderer

  - path: src/parser/mod.rs
  - hash: c399aff1248675b25447a94e5e2b618b8cba18985d5e5b2fe52ed174124cd367

  ```md
  Optional depth.
  ```
  ````

说明：按上一条 fix 行执行：只取从 agent-coexist 开始的命令部分，保留其中的操作数，没有需要填写的标记。补上 renderer 缺的哈希。

<a id="write-state-create"></a>

### state：第一次声明，新建状态文件

编号 `write-state-create` · 观察平台 Windows

新状态文件带一行说明 locked 与 observing 的含义。目录必须已存在，state 不会创建目录。

起始目录 `src/parser`（空目录）

**命令 1**

```sh
agent-coexist state src/parser --as alice --status locked --for 30m
```

stdout：

```text
ok  state  src/parser
  created       src/parser/state.agent-besties.md
  set           locked  alice  2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 新建 `src/parser/state.agent-besties.md`：

  ```text
  # .state

  > locked = please do not change this directory or its subdirectories until the end time; observing = I am watching, changes are welcome.

  ## locked

  alice 2026-09-17T10:00:00Z..2026-09-17T10:30:00Z
  ```

<a id="write-insert-layout-post-profile"></a>

### 向手写文件插入订阅行和缺失的正文

编号 `write-insert-layout-post-profile` · 观察平台 Windows

订阅行、评论正文和节正文插入到紧跟着下一个标题的位置时，中间保留一个空行。

起始文件 `standup.agent-besties.md`：

````text
# standup.post
## #1 alice (2026-09-17T09:00:00Z)
## #2 bob (2026-09-17T09:10:00Z)

```md
Ready.
```
````

起始文件 `agents/alice.agent-besties.md`：

````text
# alice.agent-profile

## role
## scope

```md
Parser error mapping.
```
````

**命令 1**

```sh
agent-coexist subscribe standup --as carol
```

stdout：

```text
ok  subscribe  standup
  subscribers   carol
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  - subscribers: carol

  ## #1 alice (2026-09-17T09:00:00Z)
  ## #2 bob (2026-09-17T09:10:00Z)

  ```md
  Ready.
  ```
  ````

**命令 2**

```sh
agent-coexist post standup --as alice --edit 1 -m 'Standup at 10:00.'
```

stdout：

```text
ok  post  standup
  comment       #1  L5  edited
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `standup.agent-besties.md`：

  ````text
  # standup.post

  - subscribers: carol

  ## #1 alice (2026-09-17T09:00:00Z, edited 2026-09-17T10:00:00Z)

  ```md
  Standup at 10:00.
  ```

  ## #2 bob (2026-09-17T09:10:00Z)

  ```md
  Ready.
  ```
  ````

**命令 3**

```sh
agent-coexist profile agents/alice --as alice --section role -m 'Rust implementer.'
```

stdout：

```text
ok  profile  agents/alice
  section       role  set
```

stderr：（空，0 字节）

退出码：`0`

这条命令造成的文件变化：

- 修改后的 `agents/alice.agent-besties.md`：

  ````text
  # alice.agent-profile

  ## role

  ```md
  Rust implementer.
  ```

  ## scope

  ```md
  Parser error mapping.
  ```
  ````

<a id="coverage"></a>

## 覆盖清单

共 844 个覆盖项。

| 覆盖项 | 示例 |
|---|---|
| `brief:fresh` | [read-brief-render](#read-brief-render)<br>[read-brief-states](#read-brief-states) |
| `brief:fresh:empty-target` | [read-brief-states](#read-brief-states)<br>[read-brief-content-line-ends](#read-brief-content-line-ends) |
| `brief:shifted` | [read-brief-states](#read-brief-states) |
| `brief:stale:regex-invalid` | [read-brief-states](#read-brief-states) |
| `brief:stale:regex-not-found` | [read-brief-states](#read-brief-states) |
| `brief:stale:target-missing` | [read-brief-states](#read-brief-states) |
| `brief:unchecked:no-hash` | [read-brief-states](#read-brief-states) |
| `brief:unchecked:no-path` | [read-brief-states](#read-brief-states) |
| `brief:unchecked:target-too-large` | [read-brief-large-target](#read-brief-large-target) |
| `brief:unchecked:target-unreadable` | [read-brief-states](#read-brief-states) |
| `diag-suppressed:W-BODY-in-unclosed-fence` | [diag-fence](#diag-fence) |
| `diag:E-COMMENT-NOTFOUND` | [write-post-edit-missing-comment](#write-post-edit-missing-comment) |
| `diag:E-DIR-MISSING` | [write-state-dir-missing](#write-state-dir-missing)<br>[write-state-dir-new](#write-state-dir-new) |
| `diag:E-DIR-MISSING:nearest-existing-dir` | [read-list-dir-missing-middle](#read-list-dir-missing-middle) |
| `diag:E-DIR-MISSING:read` | [read-list-dir-errors](#read-list-dir-errors)<br>[read-list-dir-missing-middle](#read-list-dir-missing-middle)<br>[read-notification-dir-errors](#read-notification-dir-errors)<br>[read-check-state-dir-errors](#read-check-state-dir-errors) |
| `diag:E-DIR-MISSING:wait` | [native-wait-start-dir-missing](#native-wait-start-dir-missing) |
| `diag:E-EDIT-AUTHOR` | [write-post-edit-other-author](#write-post-edit-other-author)<br>[write-fix-operand-dash](#write-fix-operand-dash)<br>[write-fix-operand-quoted](#write-fix-operand-quoted) |
| `diag:E-ENTRY-DUP` | [diag-brief-structure](#diag-brief-structure) |
| `diag:E-ENTRY-HASH` | [diag-brief-structure](#diag-brief-structure) |
| `diag:E-ENTRY-HASH:read-display` | [read-brief-states](#read-brief-states) |
| `diag:E-ENTRY-NOTFOUND` | [write-brief-note-and-notfound](#write-brief-note-and-notfound) |
| `diag:E-ENTRY-NOTFOUND:read` | [read-brief-errors](#read-brief-errors) |
| `diag:E-ENTRY-PATH` | [diag-brief-structure](#diag-brief-structure) |
| `diag:E-ENTRY-PATH:read-display` | [read-brief-states](#read-brief-states) |
| `diag:E-ENTRY-REGEX` | [diag-entry-regex](#diag-entry-regex) |
| `diag:E-ENTRY-REGEX:no-target` | [diag-entry-regex](#diag-entry-regex) |
| `diag:E-ENTRY-REGEX:read-display` | [read-brief-states](#read-brief-states) |
| `diag:E-ENTRY-REGEX:with-target` | [diag-entry-regex](#diag-entry-regex) |
| `diag:E-ENTRY-TARGET` | [diag-entry-target](#diag-entry-target) |
| `diag:E-ENTRY-TARGET:directory` | [diag-entry-target](#diag-entry-target) |
| `diag:E-ENTRY-TARGET:permission` | [diag-entry-target](#diag-entry-target) |
| `diag:E-ENTRY-TARGET:read-display` | [read-brief-states](#read-brief-states) |
| `diag:E-ENTRY-TITLE-TAKEN` | [write-brief-title-taken](#write-brief-title-taken) |
| `diag:E-FENCE` | [diag-fence](#diag-fence) |
| `diag:E-FENCE:wait-stop` | [native-wait-stop-fence](#native-wait-stop-fence) |
| `diag:E-FENCE:write-refusal` | [diag-fence](#diag-fence) |
| `diag:E-FILE-MISSING` | [write-file-missing-chain](#write-file-missing-chain) |
| `diag:E-FILE-MISSING:directory` | [usage-path-is-directory](#usage-path-is-directory) |
| `diag:E-FILE-MISSING:nearest-existing-dir` | [write-file-missing-chain](#write-file-missing-chain) |
| `diag:E-FILE-MISSING:read` | [usage-path-is-directory](#usage-path-is-directory)<br>[read-check-missing](#read-check-missing)<br>[read-lookfor-wrong-file](#read-lookfor-wrong-file)<br>[read-brief-errors](#read-brief-errors) |
| `diag:E-FLOCKED` | [diag-flocked-own](#diag-flocked-own) |
| `diag:E-FLOCKED:also-locked` | [diag-flocked-other](#diag-flocked-other) |
| `diag:E-FLOCKED:mistyped-as` | [diag-flocked-mistyped-as](#diag-flocked-mistyped-as) |
| `diag:E-FLOCKED:other-holder` | [diag-flocked-other](#diag-flocked-other) |
| `diag:E-FLOCKED:own-lock` | [diag-flocked-own](#diag-flocked-own) |
| `diag:E-FLOCKED:scan-partial` | [diag-flocked-other](#diag-flocked-other) |
| `diag:E-FLOCKED:separator` | [diag-flocked-own-separator](#diag-flocked-own-separator) |
| `diag:E-H1` | [diag-h1](#diag-h1) |
| `diag:E-H1:bad-stem` | [diag-h1-bad-stem](#diag-h1-bad-stem) |
| `diag:E-H1:valid-stem` | [diag-h1](#diag-h1) |
| `diag:E-H1:write-refusal` | [diag-h1](#diag-h1) |
| `diag:E-HEAD` | [diag-head](#diag-head) |
| `diag:E-HEAD:distinct-next` | [diag-head](#diag-head) |
| `diag:E-HEAD:edited-nick` | [diag-head-salvage](#diag-head-salvage) |
| `diag:E-HEAD:exhausted` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `diag:E-HEAD:exhausted:fill` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `diag:E-HEAD:fill` | [diag-head-fill](#diag-head-fill) |
| `diag:E-HEAD:fill:edited-only` | [diag-head-fill-edited](#diag-head-fill-edited) |
| `diag:E-HEAD:fill:numeric-only` | [diag-head-fill-edited](#diag-head-fill-edited) |
| `diag:E-HEAD:numeric-nick` | [diag-head-salvage](#diag-head-salvage) |
| `diag:E-HEAD:read-display` | [read-lookfor-blocking](#read-lookfor-blocking) |
| `diag:E-HEAD:renumbered` | [diag-head](#diag-head) |
| `diag:E-HEAD:separated-number` | [diag-head-salvage](#diag-head-salvage) |
| `diag:E-HEAD:unnumbered` | [diag-head-unnumbered](#diag-head-unnumbered) |
| `diag:E-HEAD:unnumbered:unknown` | [diag-head-unnumbered-unknown](#diag-head-unnumbered-unknown) |
| `diag:E-INUSE` | [native-inuse-windows](#native-inuse-windows) |
| `diag:E-INUSE:check` | [native-inuse-windows](#native-inuse-windows) |
| `diag:E-INUSE:post` | [native-inuse-windows](#native-inuse-windows) |
| `diag:E-LOCKED` | [native-lock-reads-windows](#native-lock-reads-windows) |
| `diag:E-LOCKED:brief` | [native-lock-writes-windows](#native-lock-writes-windows) |
| `diag:E-LOCKED:check` | [native-lock-reads-windows](#native-lock-reads-windows) |
| `diag:E-LOCKED:linux` | [native-lock-linux](#native-lock-linux) |
| `diag:E-LOCKED:lookfor-comment` | [native-lock-reads-windows](#native-lock-reads-windows) |
| `diag:E-LOCKED:post` | [native-lock-writes-windows](#native-lock-writes-windows) |
| `diag:E-LOCKED:read-brief` | [native-lock-reads-windows](#native-lock-reads-windows) |
| `diag:E-LOCKED:state` | [native-lock-writes-windows](#native-lock-writes-windows) |
| `diag:E-LOCKED:subscribe` | [native-lock-writes-windows](#native-lock-writes-windows) |
| `diag:E-NAME` | [write-post-invalid-name](#write-post-invalid-name) |
| `diag:E-NOT-DIR` | [write-state-not-dir](#write-state-not-dir)<br>[write-state-not-dir-same-flags](#write-state-not-dir-same-flags) |
| `diag:E-NOT-DIR:read` | [read-list-dir-errors](#read-list-dir-errors)<br>[read-notification-dir-errors](#read-notification-dir-errors)<br>[read-check-state-dir-errors](#read-check-state-dir-errors) |
| `diag:E-NOT-DIR:wait-state` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir) |
| `diag:E-SECTION-DUP` | [diag-profile-sections](#diag-profile-sections) |
| `diag:E-SECTION-NOTFOUND` | [write-profile-section-notfound](#write-profile-section-notfound) |
| `diag:E-SEQ-DUP` | [diag-seq-dup](#diag-seq-dup) |
| `diag:E-SEQ-DUP:exhausted` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `diag:E-SEQ-DUP:no-reply-collision` | [diag-seq-dup](#diag-seq-dup) |
| `diag:E-SEQ-DUP:other-repair-header` | [diag-seq-dup](#diag-seq-dup) |
| `diag:E-SEQ-EXHAUSTED` | [diag-seq-exhausted](#diag-seq-exhausted) |
| `diag:E-STATE-DIRTY` | [diag-state-dirty](#diag-state-dirty) |
| `diag:E-STATE-NOTFOUND` | [write-state-remove-notfound](#write-state-remove-notfound) |
| `diag:E-STATE-NOTFOUND:status` | [write-state-remove-notfound](#write-state-remove-notfound) |
| `diag:E-STATUS-DUP` | [diag-state-structure](#diag-state-structure) |
| `diag:E-STATUS-DUP:control-heading` | [diag-control-heading](#diag-control-heading) |
| `diag:E-STATUS-DUP:invalid-name` | [diag-state-invalid-repeated](#diag-state-invalid-repeated) |
| `diag:E-TARGET-MISSING` | [write-brief-target-relative](#write-brief-target-relative) |
| `diag:E-TOO-LARGE` | [diag-too-large](#diag-too-large) |
| `diag:E-TOO-LARGE:existing` | [diag-too-large](#diag-too-large) |
| `diag:E-TOO-LARGE:write-limit` | [diag-too-large](#diag-too-large) |
| `diag:E-TYPE` | [write-type-mismatch](#write-type-mismatch)<br>[write-type-state-file](#write-type-state-file) |
| `diag:E-TYPE:read:brief` | [read-lookfor-wrong-file](#read-lookfor-wrong-file) |
| `diag:E-TYPE:read:post` | [read-brief-errors](#read-brief-errors) |
| `diag:E-TYPE:read:state` | [read-lookfor-wrong-file](#read-lookfor-wrong-file) |
| `diag:E-TYPE:reserved` | [write-type-state-name-taken](#write-type-state-name-taken) |
| `diag:E-TYPE:wait-post` | [native-wait-stop-wrong-type](#native-wait-stop-wrong-type) |
| `diag:E-TYPE:wait-state` | [native-wait-stop-wrong-type](#native-wait-stop-wrong-type) |
| `diag:E-UTF8` | [diag-utf8](#diag-utf8) |
| `diag:E-UTF8:check` | [diag-utf8](#diag-utf8) |
| `diag:E-UTF8:write-refusal` | [diag-utf8](#diag-utf8) |
| `diag:FIX_CONTROL_HEADING` | [diag-control-heading](#diag-control-heading) |
| `diag:FIX_UNSELECTED_STATE` | [diag-state-unselected](#diag-state-unselected) |
| `diag:W-BODY` | [diag-post-body](#diag-post-body) |
| `diag:W-BODY:control-heading` | [diag-control-heading](#diag-control-heading) |
| `diag:W-BODY:post` | [diag-post-body](#diag-post-body) |
| `diag:W-BODY:profile` | [diag-profile-sections](#diag-profile-sections) |
| `diag:W-BODY:profile-invalid-name` | [diag-profile-sections](#diag-profile-sections) |
| `diag:W-BOM` | [diag-post-encoding](#diag-post-encoding) |
| `diag:W-CRLF` | [diag-post-encoding](#diag-post-encoding) |
| `diag:W-ENTRY-ABS` | [diag-entry-local](#diag-entry-local) |
| `diag:W-ENTRY-KEY` | [diag-entry-local](#diag-entry-local) |
| `diag:W-ENTRY-SHIFTED` | [diag-entry-drift](#diag-entry-drift) |
| `diag:W-ENTRY-SHIFTED:read-display` | [read-brief-states](#read-brief-states) |
| `diag:W-ENTRY-STALE` | [diag-entry-drift](#diag-entry-drift) |
| `diag:W-ENTRY-STALE:read-display` | [read-brief-states](#read-brief-states) |
| `diag:W-ENTRY-STALE:regex-not-found` | [diag-entry-drift](#diag-entry-drift) |
| `diag:W-ENTRY-STALE:target-missing` | [diag-entry-drift](#diag-entry-drift) |
| `diag:W-NAME` | [diag-name-mismatch](#diag-name-mismatch) |
| `diag:W-NAME-STATE` | [diag-state-unselected](#diag-state-unselected) |
| `diag:W-NAME:bad-stem` | [diag-name-mismatch](#diag-name-mismatch) |
| `diag:W-NAME:valid-stem` | [diag-name-mismatch](#diag-name-mismatch) |
| `diag:W-PROFILE-EMPTY` | [diag-profile-empty](#diag-profile-empty) |
| `diag:W-REPLY-DANGLING` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-SECTION-NAME` | [diag-profile-sections](#diag-profile-sections) |
| `diag:W-SECTION-NAME:first-character` | [diag-heading-form](#diag-heading-form) |
| `diag:W-SECTION-NAME:suggested` | [diag-profile-sections](#diag-profile-sections) |
| `diag:W-SECTION-NAME:unique-manual` | [diag-profile-sections](#diag-profile-sections) |
| `diag:W-SEQ-GAP` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `diag:W-SEQ-GAP:first` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `diag:W-SEQ-GAP:first-later` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-SEQ-GAP:later` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-SEQ-GAP:unknown-cause` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-SEQ-ORDER` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-STATE-CONFLICT` | [diag-state-advisory](#diag-state-advisory) |
| `diag:W-STATE-CONFLICT:dirty-line` | [read-state-conflict-dirty](#read-state-conflict-dirty) |
| `diag:W-STATE-CONFLICT:read-display` | [read-state-conflict-views](#read-state-conflict-views) |
| `diag:W-STATE-DUP` | [diag-state-advisory](#diag-state-advisory) |
| `diag:W-STATE-DUP:other-status-leaves` | [diag-state-advisory](#diag-state-advisory) |
| `diag:W-STATE-DUP:read-display` | [read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `diag:W-STATE-DUP:unselected` | [diag-state-unselected](#diag-state-unselected) |
| `diag:W-STATE-DUP:wait-stop` | [native-wait-stop-state-dup](#native-wait-stop-state-dup) |
| `diag:W-STATE-DUP:wait-stop:dirty` | [native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty) |
| `diag:W-STATE-DUP:wait-stop:wait` | [native-wait-stop-state-dup](#native-wait-stop-state-dup) |
| `diag:W-STATE-ENTRY` | [diag-state-structure](#diag-state-structure) |
| `diag:W-STATE-EXPIRED` | [diag-state-advisory](#diag-state-advisory) |
| `diag:W-STATE-EXPIRED:invalid-status` | [diag-state-structure](#diag-state-structure) |
| `diag:W-STATE-EXPIRED:read-display` | [read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `diag:W-STATE-EXPIRED:unselected` | [diag-state-unselected](#diag-state-unselected) |
| `diag:W-STATE-RANGE` | [diag-state-advisory](#diag-state-advisory) |
| `diag:W-STATE-RANGE:read-display` | [read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `diag:W-STATE-RANGE:unselected` | [diag-state-unselected](#diag-state-unselected) |
| `diag:W-STATUS-NAME` | [diag-state-structure](#diag-state-structure) |
| `diag:W-STATUS-NAME:control-heading` | [diag-control-heading](#diag-control-heading) |
| `diag:W-STATUS-NAME:first-character` | [diag-heading-form](#diag-heading-form) |
| `diag:W-STATUS-NAME:repeated` | [diag-state-invalid-repeated](#diag-state-invalid-repeated) |
| `diag:W-STRAY:brief` | [diag-entry-local](#diag-entry-local) |
| `diag:W-STRAY:post` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-STRAY:post-body` | [diag-post-body](#diag-post-body) |
| `diag:W-STRAY:profile` | [diag-profile-sections](#diag-profile-sections) |
| `diag:W-SUB-DUP` | [diag-post-encoding](#diag-post-encoding) |
| `diag:W-SUB-DUP:duplicate-nick` | [diag-post-encoding](#diag-post-encoding) |
| `diag:W-SUB-DUP:repeated-line` | [diag-post-encoding](#diag-post-encoding) |
| `diag:io-report:open` | [native-delete-pending-windows](#native-delete-pending-windows)<br>[native-permission-file-windows](#native-permission-file-windows) |
| `diag:wait-stop:blocking` | [native-wait-stop-fence](#native-wait-stop-fence) |
| `file-text:body-fence-length` | [write-post-body-fence](#write-post-body-fence) |
| `file-text:brief-skeleton-hint` | [write-brief-skeleton](#write-brief-skeleton) |
| `file-text:insert-separation:brief` | [write-brief-insert-layout](#write-brief-insert-layout) |
| `file-text:insert-separation:post` | [write-insert-layout-post-profile](#write-insert-layout-post-profile) |
| `file-text:insert-separation:profile` | [write-insert-layout-post-profile](#write-insert-layout-post-profile) |
| `file-text:state-comment` | [write-state-create](#write-state-create) |
| `info:alias:type-profile` | [info-type-alias](#info-type-alias) |
| `info:guide:env-independent` | [info-guide-ignores-env](#info-guide-ignores-env) |
| `info:guide:index` | [info-guide-index](#info-guide-index) |
| `info:guide:read` | [info-guide-read](#info-guide-read) |
| `info:guide:start` | [info-guide-start](#info-guide-start) |
| `info:guide:wait` | [info-guide-wait](#info-guide-wait) |
| `info:guide:write` | [info-guide-write](#info-guide-write) |
| `info:guide:write-failed` | [native-guide-write-failed-linux](#native-guide-write-failed-linux) |
| `info:help-entry:brief:--help` | [info-help-brief](#info-help-brief) |
| `info:help-entry:brief:-h` | [info-help-brief](#info-help-brief) |
| `info:help-entry:brief:help` | [info-help-brief](#info-help-brief) |
| `info:help-entry:check-state:--help` | [info-help-check-state](#info-help-check-state) |
| `info:help-entry:check-state:-h` | [info-help-check-state](#info-help-check-state) |
| `info:help-entry:check-state:help` | [info-help-check-state](#info-help-check-state) |
| `info:help-entry:check:--help` | [info-help-check](#info-help-check) |
| `info:help-entry:check:-h` | [info-help-check](#info-help-check) |
| `info:help-entry:check:help` | [info-help-check](#info-help-check) |
| `info:help-entry:guide:--help` | [info-help-guide](#info-help-guide) |
| `info:help-entry:guide:-h` | [info-help-guide](#info-help-guide) |
| `info:help-entry:guide:help` | [info-help-guide](#info-help-guide) |
| `info:help-entry:list:--help` | [info-help-list](#info-help-list) |
| `info:help-entry:list:-h` | [info-help-list](#info-help-list) |
| `info:help-entry:list:help` | [info-help-list](#info-help-list) |
| `info:help-entry:lookfor-comment:--help` | [info-help-lookfor-comment](#info-help-lookfor-comment) |
| `info:help-entry:lookfor-comment:-h` | [info-help-lookfor-comment](#info-help-lookfor-comment) |
| `info:help-entry:lookfor-comment:help` | [info-help-lookfor-comment](#info-help-lookfor-comment) |
| `info:help-entry:post:--help` | [info-help-post](#info-help-post) |
| `info:help-entry:post:-h` | [info-help-post](#info-help-post) |
| `info:help-entry:post:help` | [info-help-post](#info-help-post) |
| `info:help-entry:profile:--help` | [info-help-profile](#info-help-profile) |
| `info:help-entry:profile:-h` | [info-help-profile](#info-help-profile) |
| `info:help-entry:profile:help` | [info-help-profile](#info-help-profile) |
| `info:help-entry:read-brief:--help` | [info-help-read-brief](#info-help-read-brief) |
| `info:help-entry:read-brief:-h` | [info-help-read-brief](#info-help-read-brief) |
| `info:help-entry:read-brief:help` | [info-help-read-brief](#info-help-read-brief) |
| `info:help-entry:read-notification:--help` | [info-help-read-notification](#info-help-read-notification) |
| `info:help-entry:read-notification:-h` | [info-help-read-notification](#info-help-read-notification) |
| `info:help-entry:read-notification:help` | [info-help-read-notification](#info-help-read-notification) |
| `info:help-entry:state:--help` | [info-help-state](#info-help-state) |
| `info:help-entry:state:-h` | [info-help-state](#info-help-state) |
| `info:help-entry:state:help` | [info-help-state](#info-help-state) |
| `info:help-entry:subscribe:--help` | [info-help-subscribe](#info-help-subscribe) |
| `info:help-entry:subscribe:-h` | [info-help-subscribe](#info-help-subscribe) |
| `info:help-entry:subscribe:help` | [info-help-subscribe](#info-help-subscribe) |
| `info:help-entry:top:--help` | [info-help-top](#info-help-top) |
| `info:help-entry:top:-h` | [info-help-top](#info-help-top) |
| `info:help-entry:top:help` | [info-help-top](#info-help-top) |
| `info:help-entry:wait-post:--help` | [info-help-wait-post](#info-help-wait-post) |
| `info:help-entry:wait-post:-h` | [info-help-wait-post](#info-help-wait-post) |
| `info:help-entry:wait-post:help` | [info-help-wait-post](#info-help-wait-post) |
| `info:help-entry:wait-state:--help` | [info-help-wait-state](#info-help-wait-state) |
| `info:help-entry:wait-state:-h` | [info-help-wait-state](#info-help-wait-state) |
| `info:help-entry:wait-state:help` | [info-help-wait-state](#info-help-wait-state) |
| `info:help-entry:wait:--help` | [info-help-wait](#info-help-wait) |
| `info:help-entry:wait:-h` | [info-help-wait](#info-help-wait) |
| `info:help-entry:wait:help` | [info-help-wait](#info-help-wait) |
| `info:help:brief` | [info-help-brief](#info-help-brief) |
| `info:help:check` | [info-help-check](#info-help-check) |
| `info:help:check-state` | [info-help-check-state](#info-help-check-state) |
| `info:help:env-independent` | [info-guide-ignores-env](#info-guide-ignores-env) |
| `info:help:guide` | [info-help-guide](#info-help-guide) |
| `info:help:help` | [info-help-help](#info-help-help) |
| `info:help:list` | [info-help-list](#info-help-list) |
| `info:help:lookfor-comment` | [info-help-lookfor-comment](#info-help-lookfor-comment) |
| `info:help:override` | [info-help-overrides](#info-help-overrides) |
| `info:help:override:before-invalid` | [info-help-overrides](#info-help-overrides) |
| `info:help:override:missing-required` | [info-help-overrides](#info-help-overrides) |
| `info:help:post` | [info-help-post](#info-help-post) |
| `info:help:profile` | [info-help-profile](#info-help-profile) |
| `info:help:read-brief` | [info-help-read-brief](#info-help-read-brief) |
| `info:help:read-notification` | [info-help-read-notification](#info-help-read-notification) |
| `info:help:state` | [info-help-state](#info-help-state) |
| `info:help:subscribe` | [info-help-subscribe](#info-help-subscribe) |
| `info:help:top` | [info-help-top](#info-help-top) |
| `info:help:wait` | [info-help-wait](#info-help-wait) |
| `info:help:wait-post` | [info-help-wait-post](#info-help-wait-post) |
| `info:help:wait-state` | [info-help-wait-state](#info-help-wait-state) |
| `info:post:write-failed` | [native-post-write-failed-linux](#native-post-write-failed-linux) |
| `info:type:union` | [info-type-alias](#info-type-alias) |
| `info:version` | [info-version](#info-version) |
| `info:version-entry:--version` | [info-version](#info-version) |
| `info:version-entry:-V` | [info-version](#info-version) |
| `io:class:file-too-large:linux` | [native-io-classes-linux](#native-io-classes-linux) |
| `io:class:other:linux` | [native-io-classes-linux](#native-io-classes-linux) |
| `io:class:readonly:linux` | [native-io-classes-linux](#native-io-classes-linux) |
| `io:class:storage-full:linux` | [native-io-classes-linux](#native-io-classes-linux) |
| `io:retry` | [native-permission-file-windows](#native-permission-file-windows) |
| `io:retry:linux` | [native-permission-linux](#native-permission-linux) |
| `io:retry:windows` | [native-permission-file-windows](#native-permission-file-windows) |
| `io:retry:windows:delete-pending` | [native-delete-pending-windows](#native-delete-pending-windows) |
| `native:scan:dir-no-search` | [native-scan-dir-no-search-linux](#native-scan-dir-no-search-linux) |
| `native:state-scan:io` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `native:state-scan:io:cwd-gone` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `native:state-scan:io:manual` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `notif:ancestor-state` | [read-notification-state](#read-notification-state) |
| `notif:before-window-excluded` | [read-notification-comments](#read-notification-comments) |
| `notif:blank-body` | [read-notification-comments](#read-notification-comments) |
| `notif:conflict` | [read-notification-state](#read-notification-state) |
| `notif:current-second-excluded` | [read-notification-comments](#read-notification-comments) |
| `notif:edited-time` | [read-notification-comments](#read-notification-comments) |
| `notif:excerpt-truncated` | [read-notification-comments](#read-notification-comments) |
| `notif:expired` | [read-notification-state](#read-notification-state) |
| `notif:expired:after-now` | [read-notification-expiry-boundaries](#read-notification-expiry-boundaries) |
| `notif:expired:at-now-gap` | [read-notification-expiry-boundaries](#read-notification-expiry-boundaries) |
| `notif:expiring` | [read-notification-state](#read-notification-state) |
| `notif:expiring:0m` | [read-notification-expiry-boundaries](#read-notification-expiry-boundaries) |
| `notif:expiring:boundary-30m` | [read-notification-expiry-boundaries](#read-notification-expiry-boundaries) |
| `notif:mention` | [read-notification-comments](#read-notification-comments) |
| `notif:ordering` | [read-notification-state](#read-notification-state) |
| `notif:own-comment-excluded` | [read-notification-comments](#read-notification-comments) |
| `notif:partial:W-STATE-ENTRY` | [read-notification-incomplete](#read-notification-incomplete) |
| `notif:partial:W-STRAY` | [read-notification-incomplete](#read-notification-incomplete) |
| `notif:reply` | [read-notification-comments](#read-notification-comments) |
| `notif:reply-beats-mention` | [read-notification-comments](#read-notification-comments) |
| `notif:reserved-post-still-notifies` | [read-notification-incomplete](#read-notification-incomplete) |
| `notif:since-absolute` | [read-notification-comments](#read-notification-comments) |
| `notif:since-duration` | [read-notification-comments](#read-notification-comments) |
| `notif:since-offset` | [read-notification-comments](#read-notification-comments) |
| `notif:skipped:E-FENCE` | [read-notification-incomplete](#read-notification-incomplete) |
| `notif:skipped:E-H1` | [read-notification-incomplete](#read-notification-incomplete) |
| `notif:skipped:E-TYPE-reserved` | [read-notification-incomplete](#read-notification-incomplete) |
| `notif:state` | [read-notification-state](#read-notification-state) |
| `notif:state-expired-flag` | [read-notification-state](#read-notification-state) |
| `notif:subscribed` | [read-notification-comments](#read-notification-comments) |
| `notif:unread-fix-executed` | [read-notification-incomplete](#read-notification-incomplete) |
| `operand:backslash:linux` | [native-operands-linux](#native-operands-linux) |
| `operand:backslash:windows` | [native-operands-windows](#native-operands-windows) |
| `operand:case:linux` | [native-operands-linux](#native-operands-linux) |
| `operand:case:windows` | [native-operands-windows](#native-operands-windows) |
| `operand:nonutf8:linux` | [native-nonutf8-linux](#native-nonutf8-linux) |
| `operand:real-value-looks-like-marker` | [diag-entry-regex](#diag-entry-regex) |
| `output:check:unicode-path` | [read-list-wide-names](#read-list-wide-names) |
| `output:list:type-column-width` | [read-list-wide-names](#read-list-wide-names) |
| `output:list:unicode-name` | [read-list-wide-names](#read-list-wide-names) |
| `output:read-brief:crlf-content` | [read-brief-lossy-content](#read-brief-lossy-content) |
| `output:read-brief:empty-target-key-only` | [read-brief-content-line-ends](#read-brief-content-line-ends) |
| `output:read-brief:final-lf-no-extra-line` | [read-brief-content-line-ends](#read-brief-content-line-ends) |
| `output:read-brief:lossy-content` | [read-brief-lossy-content](#read-brief-lossy-content) |
| `output:read-brief:real-blank-line` | [read-brief-content-line-ends](#read-brief-content-line-ends) |
| `platform:windows:invalid-filename` | [write-post-path-newline-windows](#write-post-path-newline-windows) |
| `progress:erase:linux` | [native-progress-linux](#native-progress-linux) |
| `progress:pipe-silent` | [native-progress-windows](#native-progress-windows) |
| `progress:tty:check-state` | [native-progress-windows](#native-progress-windows) |
| `progress:tty:linux` | [native-progress-linux](#native-progress-linux) |
| `progress:tty:list` | [native-progress-windows](#native-progress-windows) |
| `progress:tty:read-notification` | [native-progress-windows](#native-progress-windows) |
| `progress:tty:state` | [native-progress-windows](#native-progress-windows) |
| `q18:check-state` | [q18-check-state-glob-syntax](#q18-check-state-glob-syntax) |
| `q18:fix:by-hand:linux` | [src-by-hand-linux](#src-by-hand-linux) |
| `q18:fix:can-be-read` | [q18-io-control-permission](#q18-io-control-permission) |
| `q18:fix:correct-contents` | [q18-gitfile-invalid](#q18-gitfile-invalid)<br>[q18-gitfile-target-missing](#q18-gitfile-target-missing)<br>[src-gitdir-file](#src-gitdir-file)<br>[src-gitdir-file-linux](#src-gitdir-file-linux) |
| `q18:fix:correct-line` | [q18-list-glob-syntax](#q18-list-glob-syntax)<br>[q18-read-notification-glob-syntax](#q18-read-notification-glob-syntax)<br>[q18-check-state-glob-syntax](#q18-check-state-glob-syntax)<br>[q18-source-git-exclude](#q18-source-git-exclude)<br>[q18-source-gitignore](#q18-source-gitignore)<br>[q18-source-global-excludes](#q18-source-global-excludes)<br>[q18-source-invalid-utf8](#q18-source-invalid-utf8) |
| `q18:fix:not-regular-file` | [q18-ignore-source-is-directory](#q18-ignore-source-is-directory) |
| `q18:fix:rules` | [src-rules-too-large](#src-rules-too-large) |
| `q18:gitfile-nonutf8:linux` | [native-linux-gitfile-nonutf8](#native-linux-gitfile-nonutf8) |
| `q18:io-control` | [q18-io-control-permission](#q18-io-control-permission)<br>[q18-io-control-state-scan](#q18-io-control-state-scan) |
| `q18:io-control:linux-dir-no-exec` | [native-scan-dir-no-search-linux](#native-scan-dir-no-search-linux) |
| `q18:io-control:not-regular-file` | [q18-ignore-source-is-directory](#q18-ignore-source-is-directory)<br>[q18-state-scan-not-regular-file](#q18-state-scan-not-regular-file) |
| `q18:io-control:permission` | [q18-io-control-permission](#q18-io-control-permission) |
| `q18:io-control:state-scan` | [q18-io-control-state-scan](#q18-io-control-state-scan)<br>[q18-state-scan-not-regular-file](#q18-state-scan-not-regular-file) |
| `q18:list` | [q18-list-glob-syntax](#q18-list-glob-syntax) |
| `q18:no-ignore-bypass` | [q18-list-no-ignore-bypass](#q18-list-no-ignore-bypass) |
| `q18:read-notification` | [q18-read-notification-glob-syntax](#q18-read-notification-glob-syntax) |
| `q18:reason:dangling-backslash` | [q18-source-gitignore](#q18-source-gitignore) |
| `q18:reason:invalid-range` | [q18-source-git-exclude](#q18-source-git-exclude) |
| `q18:reason:invalid-utf8` | [q18-source-invalid-utf8](#q18-source-invalid-utf8) |
| `q18:reason:unclosed-alternate` | [q18-list-glob-syntax](#q18-list-glob-syntax)<br>[q18-read-notification-glob-syntax](#q18-read-notification-glob-syntax)<br>[q18-check-state-glob-syntax](#q18-check-state-glob-syntax)<br>[q18-source-global-excludes](#q18-source-global-excludes) |
| `q18:recovery-executed` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux)<br>[q18-list-glob-syntax](#q18-list-glob-syntax)<br>[q18-read-notification-glob-syntax](#q18-read-notification-glob-syntax)<br>[q18-check-state-glob-syntax](#q18-check-state-glob-syntax)<br>[q18-source-git-exclude](#q18-source-git-exclude)<br>[q18-source-gitignore](#q18-source-gitignore)<br>[q18-source-global-excludes](#q18-source-global-excludes)<br>[q18-source-invalid-utf8](#q18-source-invalid-utf8)<br>[q18-gitfile-invalid](#q18-gitfile-invalid)<br>[q18-gitfile-target-missing](#q18-gitfile-target-missing)<br>[q18-state-post-write-scan](#q18-state-post-write-scan)<br>[q18-io-control-permission](#q18-io-control-permission)<br>[q18-io-control-state-scan](#q18-io-control-state-scan)<br>[q18-ignore-source-is-directory](#q18-ignore-source-is-directory)<br>[q18-state-scan-not-regular-file](#q18-state-scan-not-regular-file)<br>[src-rules-too-large](#src-rules-too-large)<br>[src-rules-state-scan](#src-rules-state-scan)<br>[src-gitdir-file](#src-gitdir-file)<br>[src-gitdir-file-linux](#src-gitdir-file-linux)<br>[src-by-hand-linux](#src-by-hand-linux) |
| `q18:source:git-exclude` | [q18-source-git-exclude](#q18-source-git-exclude) |
| `q18:source:gitfile` | [q18-gitfile-invalid](#q18-gitfile-invalid) |
| `q18:source:gitfile-target-file` | [src-gitdir-file](#src-gitdir-file) |
| `q18:source:gitfile-target-file:linux` | [src-gitdir-file-linux](#src-gitdir-file-linux) |
| `q18:source:gitfile-target-missing` | [q18-gitfile-target-missing](#q18-gitfile-target-missing) |
| `q18:source:gitignore` | [q18-source-gitignore](#q18-source-gitignore) |
| `q18:source:global-excludes` | [q18-source-global-excludes](#q18-source-global-excludes) |
| `q18:source:invalid-utf8` | [q18-source-invalid-utf8](#q18-source-invalid-utf8) |
| `q18:source:rules` | [src-rules-too-large](#src-rules-too-large) |
| `q18:state-scan` | [q18-state-post-write-scan](#q18-state-post-write-scan) |
| `q18:state-scan:by-hand:linux` | [src-by-hand-linux](#src-by-hand-linux) |
| `q18:state-scan:fix` | [q18-state-post-write-scan](#q18-state-post-write-scan) |
| `q18:state-scan:io` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `q18:state-scan:no-redeclare` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux)<br>[q18-state-post-write-scan](#q18-state-post-write-scan)<br>[q18-io-control-state-scan](#q18-io-control-state-scan)<br>[q18-state-scan-not-regular-file](#q18-state-scan-not-regular-file)<br>[src-rules-state-scan](#src-rules-state-scan)<br>[src-by-hand-linux](#src-by-hand-linux) |
| `q18:state-scan:rules` | [src-rules-state-scan](#src-rules-state-scan) |
| `q18:state-scan:write-committed` | [q18-state-post-write-scan](#q18-state-post-write-scan) |
| `read:canonicalize-failed:linux` | [native-canonicalize-linux](#native-canonicalize-linux) |
| `read:canonicalize-failed:windows` | [native-permission-dir-windows](#native-permission-dir-windows) |
| `read:check-state:empty` | [read-check-state-empty-and-filters](#read-check-state-empty-and-filters) |
| `read:check-state:empty-group-heading` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `read:check-state:exclude` | [read-check-state-empty-and-filters](#read-check-state-empty-and-filters) |
| `read:check-state:include-hint` | [read-check-state-empty-and-filters](#read-check-state-empty-and-filters) |
| `read:check-state:ok` | [read-check-state-scope](#read-check-state-scope) |
| `read:check-state:skipped-busy` | [native-lock-reads-windows](#native-lock-reads-windows) |
| `read:check-state:warn` | [read-check-state-scope](#read-check-state-scope) |
| `read:check:brief-all-fresh` | [read-check-previews](#read-check-previews) |
| `read:check:brief-counts` | [read-brief-states](#read-brief-states) |
| `read:check:brief-empty` | [read-brief-skeleton](#read-brief-skeleton) |
| `read:check:brief-too-large` | [read-brief-large-target](#read-brief-large-target) |
| `read:check:missing` | [read-check-missing](#read-check-missing) |
| `read:check:ok` | [read-check-previews](#read-check-previews) |
| `read:check:post` | [read-check-previews](#read-check-previews) |
| `read:check:post-empty` | [read-check-previews](#read-check-previews) |
| `read:check:profile` | [read-check-previews](#read-check-previews) |
| `read:check:state` | [read-check-previews](#read-check-previews) |
| `read:list:busy-leaf` | [native-lock-reads-windows](#native-lock-reads-windows) |
| `read:list:busy-leaf:inuse` | [native-inuse-windows](#native-inuse-windows) |
| `read:list:busy-leaf:linux` | [native-lock-linux](#native-lock-linux) |
| `read:list:default-dir` | [read-list-tree](#read-list-tree) |
| `read:list:dir-missing` | [read-list-dir-errors](#read-list-dir-errors) |
| `read:list:empty` | [read-list-empty](#read-list-empty) |
| `read:list:exclude` | [read-list-filters](#read-list-filters) |
| `read:list:ignore-row:none` | [read-list-empty](#read-list-empty) |
| `read:list:ignore-row:sources` | [read-list-tree](#read-list-tree) |
| `read:list:ignored-dirs-pruned` | [read-list-tree](#read-list-tree) |
| `read:list:include-hint:exclude-removed-all` | [read-list-filters](#read-list-filters) |
| `read:list:include-hint:ignored` | [read-list-filters](#read-list-filters) |
| `read:list:include-no-ignore` | [read-list-filters](#read-list-filters) |
| `read:list:no-ignore` | [read-list-tree](#read-list-tree) |
| `read:list:non-managed-files-not-listed` | [read-list-tree](#read-list-tree) |
| `read:list:not-dir` | [read-list-dir-errors](#read-list-dir-errors) |
| `read:list:ok` | [read-list-tree](#read-list-tree) |
| `read:list:suffix-case-insensitive` | [read-list-tree](#read-list-tree) |
| `read:list:tree` | [read-list-tree](#read-list-tree) |
| `read:list:type-alias-profile` | [read-list-filters](#read-list-filters) |
| `read:list:type-union` | [read-list-filters](#read-list-filters) |
| `read:list:unknown-leaf` | [read-list-unknown-leaves](#read-list-unknown-leaves) |
| `read:list:unknown-leaf-under-type-filter` | [read-list-unknown-leaves](#read-list-unknown-leaves) |
| `read:list:unread-fix` | [read-list-unknown-leaves](#read-list-unknown-leaves) |
| `read:list:warn` | [read-list-unknown-leaves](#read-list-unknown-leaves) |
| `read:lookfor:all` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:blank-body-no-excerpt` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:blocking-report` | [read-lookfor-blocking](#read-lookfor-blocking) |
| `read:lookfor:by` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:comment` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:edited-time` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:excerpt-truncated` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:keyword-case-insensitive` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:mention-and-reply-intersect` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:lookfor:zero` | [read-lookfor-filters](#read-lookfor-filters) |
| `read:metadata-io-error:linux` | [native-permission-linux](#native-permission-linux) |
| `read:notification:empty` | [read-notification-empty-and-filters](#read-notification-empty-and-filters) |
| `read:notification:exclude` | [read-notification-empty-and-filters](#read-notification-empty-and-filters) |
| `read:notification:include-hint` | [read-notification-empty-and-filters](#read-notification-empty-and-filters) |
| `read:notification:include-no-ignore` | [read-notification-empty-and-filters](#read-notification-empty-and-filters) |
| `read:notification:ok` | [read-notification-comments](#read-notification-comments) |
| `read:notification:partial:E-HEAD` | [read-lookfor-blocking](#read-lookfor-blocking) |
| `read:notification:warn-conflict` | [read-notification-state](#read-notification-state) |
| `read:notification:warn-incomplete` | [read-notification-incomplete](#read-notification-incomplete) |
| `read:notification:window-after-activity` | [read-notification-empty-and-filters](#read-notification-empty-and-filters) |
| `read:read-brief:content-omitted` | [read-brief-states](#read-brief-states) |
| `read:read-brief:content-whole-file` | [read-brief-render](#read-brief-render) |
| `read:read-brief:description` | [read-brief-render](#read-brief-render) |
| `read:read-brief:entry-filter` | [read-brief-render](#read-brief-render) |
| `read:read-brief:fresh-too-large-omitted` | [read-brief-large-target](#read-brief-large-target) |
| `read:read-brief:group` | [read-brief-render](#read-brief-render) |
| `read:read-brief:note` | [read-brief-render](#read-brief-render) |
| `read:read-brief:ok` | [read-brief-render](#read-brief-render) |
| `read:read-brief:regex-range` | [read-brief-render](#read-brief-render) |
| `read:read-brief:skeleton-first-line-only` | [read-brief-skeleton](#read-brief-skeleton) |
| `read:read-brief:warn` | [read-brief-states](#read-brief-states) |
| `read:read-notification:skipped-busy` | [native-lock-reads-windows](#native-lock-reads-windows) |
| `read:scan-row:nonutf8` | [native-nonutf8-linux](#native-nonutf8-linux) |
| `read:skipped-unreadable:linux` | [native-permission-linux](#native-permission-linux) |
| `read:skipped-unreadable:windows` | [native-permission-dir-windows](#native-permission-dir-windows) |
| `recovery:E-DIR-MISSING:wait` | [native-wait-start-dir-missing](#native-wait-start-dir-missing) |
| `recovery:E-ENTRY-HASH` | [diag-brief-structure](#diag-brief-structure) |
| `recovery:E-ENTRY-PATH` | [diag-brief-structure](#diag-brief-structure) |
| `recovery:E-ENTRY-REGEX` | [diag-entry-regex](#diag-entry-regex) |
| `recovery:E-ENTRY-REGEX:no-target:review-first` | [diag-entry-regex](#diag-entry-regex) |
| `recovery:E-ENTRY-REGEX:review-first` | [diag-entry-regex](#diag-entry-regex) |
| `recovery:E-ENTRY-TARGET:permission` | [diag-entry-target](#diag-entry-target) |
| `recovery:E-ENTRY-TARGET:retarget` | [diag-entry-target](#diag-entry-target) |
| `recovery:E-ENTRY-TITLE-TAKEN:keeps-options` | [write-brief-title-taken](#write-brief-title-taken) |
| `recovery:E-ENTRY-TITLE-TAKEN:manual` | [write-brief-title-taken](#write-brief-title-taken) |
| `recovery:E-FLOCKED:mistyped-as` | [diag-flocked-mistyped-as](#diag-flocked-mistyped-as) |
| `recovery:E-FLOCKED:own` | [diag-flocked-own](#diag-flocked-own) |
| `recovery:E-FLOCKED:own:before-separator` | [diag-flocked-own-separator](#diag-flocked-own-separator) |
| `recovery:E-FLOCKED:wait-then-retry` | [diag-flocked-other](#diag-flocked-other) |
| `recovery:E-H1` | [diag-h1](#diag-h1) |
| `recovery:E-H1:bad-stem` | [diag-h1-bad-stem](#diag-h1-bad-stem) |
| `recovery:E-HEAD` | [diag-head](#diag-head) |
| `recovery:E-HEAD:author-edit` | [diag-head-salvage](#diag-head-salvage) |
| `recovery:E-HEAD:exhausted` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `recovery:E-HEAD:exhausted-restore` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `recovery:E-HEAD:exhausted:fill-from-record` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `recovery:E-HEAD:fill-from-record` | [diag-head-fill](#diag-head-fill) |
| `recovery:E-HEAD:fill-from-record:author-edit` | [diag-head-fill-edited](#diag-head-fill-edited) |
| `recovery:E-HEAD:unnumbered-references` | [diag-head-unnumbered-unknown](#diag-head-unnumbered-unknown) |
| `recovery:E-HEAD:unnumbered-restore` | [diag-head-unnumbered](#diag-head-unnumbered) |
| `recovery:E-HEAD:update-references` | [diag-head](#diag-head) |
| `recovery:E-INUSE:retry` | [native-inuse-windows](#native-inuse-windows) |
| `recovery:E-LOCKED:retry` | [native-lock-reads-windows](#native-lock-reads-windows)<br>[native-lock-writes-windows](#native-lock-writes-windows) |
| `recovery:E-LOCKED:retry:linux` | [native-lock-linux](#native-lock-linux) |
| `recovery:E-NOT-DIR:wait-state` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir) |
| `recovery:E-SEQ-DUP` | [diag-seq-dup](#diag-seq-dup) |
| `recovery:E-SEQ-DUP:exhausted` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `recovery:E-SEQ-DUP:exhausted:update-references` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `recovery:E-SEQ-DUP:no-reply-collision` | [diag-seq-dup](#diag-seq-dup) |
| `recovery:E-SEQ-DUP:other-repair-header` | [diag-seq-dup](#diag-seq-dup) |
| `recovery:E-SEQ-EXHAUSTED` | [diag-seq-exhausted](#diag-seq-exhausted) |
| `recovery:E-STATE-DIRTY:conflict-yield` | [read-state-conflict-dirty](#read-state-conflict-dirty) |
| `recovery:E-STATE-DIRTY:delete-completes` | [diag-state-dirty](#diag-state-dirty) |
| `recovery:E-STATE-DIRTY:retry-keeps-status` | [diag-state-dirty](#diag-state-dirty) |
| `recovery:E-TOO-LARGE` | [diag-too-large](#diag-too-large) |
| `recovery:E-TYPE:wait-post` | [native-wait-stop-wrong-type](#native-wait-stop-wrong-type) |
| `recovery:E-UTF8` | [diag-utf8](#diag-utf8) |
| `recovery:FIX_ETYPE_RESERVED:wait-state` | [native-wait-stop-wrong-type](#native-wait-stop-wrong-type) |
| `recovery:W-BODY:post` | [diag-post-body](#diag-post-body) |
| `recovery:W-BODY:profile` | [diag-profile-sections](#diag-profile-sections) |
| `recovery:W-BOM` | [diag-post-encoding](#diag-post-encoding) |
| `recovery:W-CRLF` | [diag-post-encoding](#diag-post-encoding) |
| `recovery:W-ENTRY-ABS` | [diag-entry-local](#diag-entry-local) |
| `recovery:W-ENTRY-ABS:review-first` | [diag-entry-local](#diag-entry-local) |
| `recovery:W-ENTRY-KEY` | [diag-entry-local](#diag-entry-local) |
| `recovery:W-ENTRY-SHIFTED:review-first` | [diag-entry-drift](#diag-entry-drift) |
| `recovery:W-ENTRY-STALE` | [diag-entry-drift](#diag-entry-drift) |
| `recovery:W-ENTRY-STALE:target-missing:review-first` | [diag-entry-drift](#diag-entry-drift) |
| `recovery:W-NAME` | [diag-name-mismatch](#diag-name-mismatch) |
| `recovery:W-NAME-STATE` | [diag-state-unselected](#diag-state-unselected) |
| `recovery:W-PROFILE-EMPTY` | [diag-profile-empty](#diag-profile-empty) |
| `recovery:W-REPLY-DANGLING:ask` | [diag-post-sequence](#diag-post-sequence) |
| `recovery:W-SECTION-NAME:first-character` | [diag-heading-form](#diag-heading-form) |
| `recovery:W-STATE-CONFLICT:agree-first` | [diag-state-advisory](#diag-state-advisory) |
| `recovery:W-STATE-CONFLICT:dirty-remove-refused` | [read-state-conflict-dirty](#read-state-conflict-dirty) |
| `recovery:W-STATE-DUP:same-status-merges` | [diag-state-advisory](#diag-state-advisory) |
| `recovery:W-STATE-DUP:wait-stop` | [native-wait-stop-state-dup](#native-wait-stop-state-dup) |
| `recovery:W-STATE-DUP:wait-stop:dirty` | [native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty) |
| `recovery:W-STATE-DUP:wait-stop:wait` | [native-wait-stop-state-dup](#native-wait-stop-state-dup) |
| `recovery:W-STATE-EXPIRED:renew` | [diag-state-advisory](#diag-state-advisory) |
| `recovery:W-STATUS-NAME:first-character` | [diag-heading-form](#diag-heading-form) |
| `recovery:W-STATUS-NAME:repeated-merge` | [diag-state-invalid-repeated](#diag-state-invalid-repeated) |
| `recovery:W-STRAY` | [diag-entry-local](#diag-entry-local) |
| `recovery:W-SUB-DUP` | [diag-post-encoding](#diag-post-encoding) |
| `recovery:advisory:continue` | [diag-post-encoding](#diag-post-encoding) |
| `recovery:advisory:decline` | [diag-state-advisory](#diag-state-advisory)<br>[diag-state-structure](#diag-state-structure) |
| `recovery:advisory:keep` | [diag-post-sequence](#diag-post-sequence) |
| `recovery:control-heading` | [diag-control-heading](#diag-control-heading) |
| `recovery:dash-value` | [usage-value-looks-like-flag](#usage-value-looks-like-flag) |
| `recovery:dash-value:format-char` | [usage-value-looks-like-flag-invisible](#usage-value-looks-like-flag-invisible) |
| `recovery:dash-value:not-copyable` | [usage-value-looks-like-flag-multiline](#usage-value-looks-like-flag-multiline) |
| `recovery:dash-value:re-encode` | [native-dash-value-invalid-unicode-windows](#native-dash-value-invalid-unicode-windows)<br>[native-dash-value-invalid-utf8-linux](#native-dash-value-invalid-utf8-linux) |
| `recovery:duration:default-window` | [usage-duration-default-out-of-range](#usage-duration-default-out-of-range) |
| `recovery:nonutf8-printf:linux` | [native-nonutf8-linux](#native-nonutf8-linux) |
| `recovery:path-is-directory` | [usage-path-is-directory](#usage-path-is-directory) |
| `recovery:post:reply-to-not-found` | [write-post-reply-not-found](#write-post-reply-not-found) |
| `recovery:prerequisite-order:E-ENTRY-DUP` | [diag-brief-structure](#diag-brief-structure) |
| `recovery:prerequisite-order:E-FENCE` | [diag-fence](#diag-fence) |
| `recovery:prerequisite-order:E-SECTION-DUP` | [diag-profile-sections](#diag-profile-sections) |
| `recovery:prerequisite-order:E-STATUS-DUP` | [diag-state-structure](#diag-state-structure) |
| `recovery:read:check` | [read-check-missing](#read-check-missing) |
| `recovery:read:check-state` | [read-check-state-dir-errors](#read-check-state-dir-errors) |
| `recovery:read:list` | [read-list-dir-errors](#read-list-dir-errors)<br>[read-list-dir-missing-middle](#read-list-dir-missing-middle) |
| `recovery:read:lookfor` | [read-lookfor-wrong-file](#read-lookfor-wrong-file) |
| `recovery:read:lookfor-blocking` | [read-lookfor-blocking](#read-lookfor-blocking) |
| `recovery:read:read-brief` | [read-brief-errors](#read-brief-errors) |
| `recovery:read:read-notification` | [read-notification-dir-errors](#read-notification-dir-errors) |
| `recovery:wait:file-became-dir-after-ready` | [native-wait-file-became-dir-after-ready-linux](#native-wait-file-became-dir-after-ready-linux) |
| `recovery:wait:invalid-snapshot` | [native-wait-invalid-after-ready](#native-wait-invalid-after-ready) |
| `recovery:wait:io-after-ready` | [native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows) |
| `recovery:wait:parent-gone-after-ready` | [native-wait-parent-gone-after-ready](#native-wait-parent-gone-after-ready) |
| `recovery:wait:parent-not-dir` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir) |
| `recovery:wait:parent-replaced-by-file-after-ready` | [native-wait-parent-replaced-by-file-after-ready-linux](#native-wait-parent-replaced-by-file-after-ready-linux) |
| `state:conflict-mark` | [read-check-state-scope](#read-check-state-scope) |
| `state:conflict:ancestor-descendant` | [read-check-state-scope](#read-check-state-scope) |
| `state:conflict:check-pair-same-file` | [read-state-conflict-views](#read-state-conflict-views) |
| `state:conflict:check-state-marks` | [read-state-conflict-views](#read-state-conflict-views) |
| `state:conflict:list-silent` | [read-state-conflict-views](#read-state-conflict-views) |
| `state:conflict:notification-per-own-lock` | [read-state-conflict-views](#read-state-conflict-views) |
| `state:custom-status` | [read-check-state-scope](#read-check-state-scope) |
| `state:descendant-groups` | [read-check-state-scope](#read-check-state-scope) |
| `state:dirty-line-fragment-counts` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `state:dirty-line-fragment-preserved` | [read-state-conflict-dirty](#read-state-conflict-dirty) |
| `state:duplicate:history-lists-both` | [read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `state:duplicate:latest-until-selected` | [read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `state:expired-mark` | [read-check-state-scope](#read-check-state-scope) |
| `state:expiry:until-equals-now` | [read-notification-expiry-boundaries](#read-notification-expiry-boundaries)<br>[read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `state:inherited` | [read-check-state-scope](#read-check-state-scope) |
| `state:inherited-valid-only` | [read-check-state-scope](#read-check-state-scope) |
| `state:misnamed-ignored` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `state:no-conflict:siblings` | [read-check-state-scope](#read-check-state-scope) |
| `state:own` | [read-check-state-scope](#read-check-state-scope) |
| `state:partial:E-STATUS-DUP` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `state:partial:W-STATE-ENTRY` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `state:partial:W-STRAY` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `state:range:start-after-until` | [read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `state:skipped:E-FENCE` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `state:skipped:E-TYPE-reserved` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `state:unread-fix-executed` | [read-check-state-incomplete](#read-check-state-incomplete) |
| `text:const:FIX_CANONICALIZE` | [native-canonicalize-linux](#native-canonicalize-linux) |
| `text:const:FIX_CHECK_STATE_UNREAD_MANUAL` | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:FIX_FILE_CHECK_MANUAL` | [native-brief-refresh-manual-linux](#native-brief-refresh-manual-linux) |
| `text:const:FIX_LIST_UNREAD_MANUAL` | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:FIX_LOCK_SCAN_INCOMPLETE_MANUAL` | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:FIX_MANUAL_COMMAND` | [native-noncopyable-manual-linux](#native-noncopyable-manual-linux) |
| `text:const:FIX_NOTIFY_UNREAD_MANUAL` | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:FIX_STATE_SCAN_INCOMPLETE_MANUAL` | [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:const:FIX_STATE_SCAN_STOPPED` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `text:const:FIX_STATE_SCAN_STOPPED_MANUAL` | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `text:const:FIX_WAIT_RESTORE_DIR_GONE` | [native-wait-parent-gone-after-ready](#native-wait-parent-gone-after-ready) |
| `text:const:FIX_WAIT_RESTORE_DIR_REPLACED` | [native-wait-parent-replaced-by-file-after-ready-linux](#native-wait-parent-replaced-by-file-after-ready-linux) |
| `text:const:FIX_WAIT_RESTORE_NOT_REGULAR` | [native-wait-file-became-dir-after-ready-linux](#native-wait-file-became-dir-after-ready-linux) |
| `text:const:MANUAL_CHECK_LIST_LEAVES` | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:MANUAL_CHECK_SCAN_FILES` | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:MANUAL_CHECK_STATE_FILES` | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:MANUAL_CHECK_STATE_HERE` | [native-lock-scan-here-linux](#native-lock-scan-here-linux) |
| `text:const:MANUAL_CHECK_STATE_UNREADABLE` | [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:const:MANUAL_LIST_UNREADABLE` | [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:const:REREAD_STATE` | [native-lock-wait-state-windows](#native-lock-wait-state-windows) |
| `text:const:SCAN_CANONICALIZE_FIRST` | [native-lock-scan-here-linux](#native-lock-scan-here-linux) |
| `text:const:SCAN_UNREADABLE_FIRST` | [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:io_report_with_retry_lead:then-retry-the-original-command-by-hand` | [native-noncopyable-manual-linux](#native-noncopyable-manual-linux) |
| `usage:brief:entry-needs-action` | [usage-brief-entry-alone](#usage-brief-entry-alone) |
| `usage:brief:refresh-exclusive` | [usage-brief-refresh-combined](#usage-brief-refresh-combined) |
| `usage:brief:regex-compile:size-limit` | [usage-brief-regex-size](#usage-brief-regex-size) |
| `usage:brief:regex-compile:syntax` | [usage-brief-regex-syntax](#usage-brief-regex-syntax) |
| `usage:brief:regex-needs-target` | [usage-brief-regex-without-target](#usage-brief-regex-without-target) |
| `usage:brief:regex-newline` | [usage-brief-regex-newline](#usage-brief-regex-newline) |
| `usage:brief:regex-with-refresh` | [usage-brief-regex-with-refresh](#usage-brief-regex-with-refresh) |
| `usage:brief:regex-with-remove` | [usage-brief-regex-with-remove](#usage-brief-regex-with-remove) |
| `usage:brief:remove-needs-entry` | [usage-brief-remove-without-entry](#usage-brief-remove-without-entry) |
| `usage:brief:remove-with-m` | [usage-brief-remove-with-m](#usage-brief-remove-with-m) |
| `usage:brief:remove-with-target` | [usage-brief-remove-with-target](#usage-brief-remove-with-target) |
| `usage:brief:target-newline` | [usage-brief-target-newline](#usage-brief-target-newline) |
| `usage:clap-invalid-utf8` | [native-invalid-unicode-windows](#native-invalid-unicode-windows)<br>[native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `usage:clap-invalid-utf8:linux` | [native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `usage:clap-invalid-utf8:windows` | [native-invalid-unicode-windows](#native-invalid-unicode-windows) |
| `usage:clap:flag-takes-no-value` | [usage-flag-with-value](#usage-flag-with-value) |
| `usage:clap:help-flag-on-help` | [info-help-help](#info-help-help) |
| `usage:clap:help-nested-unknown` | [usage-help-extra-command](#usage-help-extra-command) |
| `usage:clap:help-unknown-subcommand` | [usage-help-unknown-command](#usage-help-unknown-command) |
| `usage:clap:invalid-before-help` | [info-help-overrides](#info-help-overrides) |
| `usage:clap:missing-required:options` | [usage-missing-options](#usage-missing-options) |
| `usage:clap:missing-required:positional` | [usage-missing-path](#usage-missing-path) |
| `usage:clap:missing-subcommand` | [usage-no-command](#usage-no-command) |
| `usage:clap:missing-value` | [usage-missing-value](#usage-missing-value) |
| `usage:clap:multi-form-usage` | [usage-brief-grammar-multiline-usage](#usage-brief-grammar-multiline-usage) |
| `usage:clap:repeated-flag` | [usage-repeated-flag](#usage-repeated-flag) |
| `usage:clap:repeated-option` | [usage-repeated-option](#usage-repeated-option) |
| `usage:clap:subcommand-after-escape` | [usage-separator-before-command](#usage-separator-before-command) |
| `usage:clap:u64:empty` | [usage-number-empty](#usage-number-empty) |
| `usage:clap:u64:invalid-digit` | [usage-number-not-digits](#usage-number-not-digits) |
| `usage:clap:u64:negative` | [usage-number-negative](#usage-number-negative) |
| `usage:clap:u64:overflow` | [usage-number-too-large](#usage-number-too-large) |
| `usage:clap:unexpected-argument:dash-value` | [usage-value-looks-like-flag](#usage-value-looks-like-flag) |
| `usage:clap:unexpected-argument:dash-value:duration` | [usage-dash-value-duration](#usage-dash-value-duration) |
| `usage:clap:unexpected-argument:dash-value:format-char` | [usage-value-looks-like-flag-invisible](#usage-value-looks-like-flag-invisible) |
| `usage:clap:unexpected-argument:dash-value:nick` | [usage-dash-value-nick](#usage-dash-value-nick) |
| `usage:clap:unexpected-argument:dash-value:not-copyable` | [usage-value-looks-like-flag-multiline](#usage-value-looks-like-flag-multiline) |
| `usage:clap:unexpected-argument:dash-value:status` | [usage-dash-value-status](#usage-dash-value-status) |
| `usage:clap:unexpected-argument:dash-value:time` | [usage-dash-value-since](#usage-dash-value-since) |
| `usage:clap:unexpected-argument:escape-tip` | [usage-unknown-flag](#usage-unknown-flag) |
| `usage:clap:unexpected-argument:foreign-option` | [usage-option-not-on-command](#usage-option-not-on-command) |
| `usage:clap:unexpected-argument:format-char` | [usage-unknown-flag-invisible](#usage-unknown-flag-invisible) |
| `usage:clap:unexpected-argument:guide` | [usage-guide-two-topics](#usage-guide-two-topics) |
| `usage:clap:unexpected-argument:positional` | [usage-extra-positional](#usage-extra-positional) |
| `usage:clap:unexpected-argument:positional-occupied` | [usage-option-not-on-command](#usage-option-not-on-command) |
| `usage:clap:unexpected-argument:similar` | [usage-unknown-flag-similar](#usage-unknown-flag-similar) |
| `usage:clap:unexpected-argument:single-dash` | [usage-single-dash-long](#usage-single-dash-long) |
| `usage:clap:unexpected-argument:top` | [usage-unknown-top-flag](#usage-unknown-top-flag) |
| `usage:clap:unknown-subcommand` | [usage-unknown-command](#usage-unknown-command) |
| `usage:clap:unknown-subcommand:similar` | [usage-unknown-command-similar](#usage-unknown-command-similar) |
| `usage:clap:version-after-command` | [info-version](#info-version) |
| `usage:clap:version-not-subcommand` | [info-version](#info-version) |
| `usage:duration:edge-whitespace` | [usage-duration-edge-space](#usage-duration-edge-space) |
| `usage:duration:invalid` | [usage-duration-order](#usage-duration-order) |
| `usage:duration:out-of-range:default` | [usage-duration-default-out-of-range](#usage-duration-default-out-of-range) |
| `usage:duration:out-of-range:explicit` | [usage-duration-out-of-range](#usage-duration-out-of-range) |
| `usage:duration:overflow` | [usage-duration-overflow](#usage-duration-overflow) |
| `usage:entry:edge-whitespace-folded` | [usage-entry-edge-space](#usage-entry-edge-space) |
| `usage:env-lock-wait-not-unicode` | [native-invalid-unicode-windows](#native-invalid-unicode-windows)<br>[native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `usage:env-lock-wait-not-unicode:linux` | [native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `usage:env-lock-wait-not-unicode:windows` | [native-invalid-unicode-windows](#native-invalid-unicode-windows) |
| `usage:env-now-not-unicode` | [native-invalid-unicode-windows](#native-invalid-unicode-windows)<br>[native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `usage:env:lock-wait:edge-whitespace` | [usage-env-lock-wait-edge-space](#usage-env-lock-wait-edge-space) |
| `usage:env:lock-wait:invalid` | [usage-env-lock-wait](#usage-env-lock-wait) |
| `usage:env:lock-wait:overflow` | [usage-env-lock-wait-overflow](#usage-env-lock-wait-overflow) |
| `usage:env:now:edge-control` | [usage-env-now-trailing-cr](#usage-env-now-trailing-cr) |
| `usage:env:now:empty` | [usage-env-now-empty](#usage-env-now-empty) |
| `usage:env:now:invalid` | [usage-env-now-shape](#usage-env-now-shape) |
| `usage:glob:empty` | [usage-glob-empty](#usage-glob-empty) |
| `usage:glob:empty:after-anchor` | [usage-glob-slash-only](#usage-glob-slash-only) |
| `usage:glob:invalid-range` | [usage-glob-range](#usage-glob-range) |
| `usage:glob:invalid-range:format-char` | [usage-glob-range](#usage-glob-range) |
| `usage:glob:unclosed-alternates` | [usage-glob-unclosed-alternates](#usage-glob-unclosed-alternates) |
| `usage:glob:unclosed-class` | [usage-glob-unclosed-class](#usage-glob-unclosed-class) |
| `usage:glob:unopened-alternates` | [usage-glob-unopened-alternates](#usage-glob-unopened-alternates) |
| `usage:guide-topic:edge-whitespace` | [usage-guide-topic-edge-space](#usage-guide-topic-edge-space) |
| `usage:guide-topic:invalid` | [usage-guide-topic-invalid](#usage-guide-topic-invalid) |
| `usage:guide-topic:raw-echo` | [usage-guide-topic-whitespace](#usage-guide-topic-whitespace) |
| `usage:keyword:empty` | [usage-keyword-empty](#usage-keyword-empty) |
| `usage:message:empty` | [usage-message-empty](#usage-message-empty) |
| `usage:nick:edge-whitespace` | [usage-nick-edge-space](#usage-nick-edge-space) |
| `usage:nick:empty` | [usage-nick-empty](#usage-nick-empty) |
| `usage:nick:flag:--as` | [usage-nick-invalid](#usage-nick-invalid) |
| `usage:nick:flag:--mention` | [usage-nick-at-sign](#usage-nick-at-sign) |
| `usage:nick:flag:--remove` | [usage-remove-nick-invalid](#usage-remove-nick-invalid) |
| `usage:nick:format-char:bom` | [usage-nick-bom](#usage-nick-bom) |
| `usage:nick:format-char:zero-width` | [usage-mention-zero-width](#usage-mention-zero-width) |
| `usage:nick:grammar` | [usage-nick-invalid](#usage-nick-invalid) |
| `usage:nick:length` | [usage-nick-too-long](#usage-nick-too-long) |
| `usage:nick:plain-control:edge-fix` | [usage-nick-trailing-cr](#usage-nick-trailing-cr) |
| `usage:nick:plain-double-space:no-edge-fix` | [usage-nick-inner-double-space](#usage-nick-inner-double-space) |
| `usage:plain:control` | [usage-plain-control](#usage-plain-control) |
| `usage:plain:control:edge-fix` | [usage-entry-trailing-cr](#usage-entry-trailing-cr) |
| `usage:plain:double-space` | [usage-plain-double-space](#usage-plain-double-space) |
| `usage:plain:double-space:edge-fix` | [usage-entry-trailing-double-space](#usage-entry-trailing-double-space) |
| `usage:precedence:env-before-resolve` | [info-guide-ignores-env](#info-guide-ignores-env) |
| `usage:precedence:env-before-value` | [usage-order-env-before-value](#usage-order-env-before-value) |
| `usage:precedence:grammar-before-env` | [usage-order-grammar-before-env](#usage-order-grammar-before-env) |
| `usage:precedence:help-before-value` | [info-help-overrides](#info-help-overrides) |
| `usage:precedence:now-before-lock-wait` | [usage-order-env-now-first](#usage-order-env-now-first) |
| `usage:precedence:value-before-files` | [usage-order-value-before-files](#usage-order-value-before-files) |
| `usage:profile:needs-section` | [usage-profile-m-without-section](#usage-profile-m-without-section) |
| `usage:profile:section-both-actions` | [usage-profile-section-both](#usage-profile-section-both) |
| `usage:profile:section-needs-action` | [usage-profile-section-alone](#usage-profile-section-alone) |
| `usage:section:edge-whitespace` | [usage-section-edge-space](#usage-section-edge-space) |
| `usage:section:grammar` | [usage-section-invalid](#usage-section-invalid) |
| `usage:section:plain-double-space:edge-fix` | [usage-section-trailing-double-space](#usage-section-trailing-double-space) |
| `usage:since:edge-whitespace` | [usage-since-edge-space](#usage-since-edge-space) |
| `usage:since:invalid` | [usage-since-word](#usage-since-word) |
| `usage:since:invalid:space-separator` | [usage-since-space-time](#usage-since-space-time) |
| `usage:since:invalid:zero` | [usage-since-zero](#usage-since-zero) |
| `usage:since:out-of-range` | [usage-since-out-of-range](#usage-since-out-of-range) |
| `usage:since:overflow` | [usage-since-overflow](#usage-since-overflow) |
| `usage:state:for-in-remove` | [usage-state-for-in-remove](#usage-state-for-in-remove) |
| `usage:state:no-form` | [usage-state-no-form](#usage-state-no-form) |
| `usage:state:status-required` | [usage-state-no-status](#usage-state-no-status) |
| `usage:status:edge-whitespace` | [usage-status-edge-space](#usage-status-edge-space) |
| `usage:status:format-char:bidi` | [usage-status-bidi](#usage-status-bidi) |
| `usage:status:grammar` | [usage-status-invalid](#usage-status-invalid) |
| `usage:status:plain-control:edge-fix` | [usage-status-trailing-cr](#usage-status-trailing-cr) |
| `usage:status:plain-control:no-edge-fix` | [usage-status-trailing-cr-still-invalid](#usage-status-trailing-cr-still-invalid) |
| `usage:status:wait-state` | [usage-wait-state-status-invalid](#usage-wait-state-status-invalid) |
| `usage:type:edge-whitespace` | [usage-type-edge-space](#usage-type-edge-space) |
| `usage:type:invalid` | [usage-type-invalid](#usage-type-invalid) |
| `usage:value-tip:non-unicode-dash` | [native-dash-value-invalid-unicode-windows](#native-dash-value-invalid-unicode-windows)<br>[native-dash-value-invalid-utf8-linux](#native-dash-value-invalid-utf8-linux) |
| `usage:value-tip:non-unicode-dash:linux` | [native-dash-value-invalid-utf8-linux](#native-dash-value-invalid-utf8-linux) |
| `usage:value-tip:non-unicode-dash:windows` | [native-dash-value-invalid-unicode-windows](#native-dash-value-invalid-unicode-windows) |
| `wait:cancel:linux` | [native-ctrlc-wait-linux](#native-ctrlc-wait-linux) |
| `wait:cancel:wait` | [native-ctrlc-wait-windows](#native-ctrlc-wait-windows)<br>[native-ctrlc-wait-linux](#native-ctrlc-wait-linux) |
| `wait:cancel:wait-post` | [native-ctrlc-wait-post-windows](#native-ctrlc-wait-post-windows) |
| `wait:cancel:wait-state` | [native-ctrlc-wait-state-windows](#native-ctrlc-wait-state-windows) |
| `wait:cancel:windows` | [native-ctrlc-wait-windows](#native-ctrlc-wait-windows)<br>[native-ctrlc-wait-post-windows](#native-ctrlc-wait-post-windows)<br>[native-ctrlc-wait-state-windows](#native-ctrlc-wait-state-windows) |
| `wait:fail:E-INUSE` | [native-lock-wait-windows](#native-lock-wait-windows) |
| `wait:fail:E-LOCKED` | [native-lock-wait-windows](#native-lock-wait-windows) |
| `wait:fail:E-LOCKED:wait-state` | [native-lock-wait-state-windows](#native-lock-wait-state-windows) |
| `wait:fail:file-became-dir-after-ready` | [native-wait-file-became-dir-after-ready-linux](#native-wait-file-became-dir-after-ready-linux) |
| `wait:fail:invalid-snapshot` | [native-wait-invalid-after-ready](#native-wait-invalid-after-ready) |
| `wait:fail:io-after-ready` | [native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows) |
| `wait:fail:io-after-ready:delete-pending` | [native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows) |
| `wait:fail:parent-gone-after-ready` | [native-wait-parent-gone-after-ready](#native-wait-parent-gone-after-ready) |
| `wait:fail:parent-not-dir:wait-post` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir) |
| `wait:fail:parent-replaced-by-file-after-ready` | [native-wait-parent-replaced-by-file-after-ready-linux](#native-wait-parent-replaced-by-file-after-ready-linux) |
| `wait:hit:wait-post` | [native-wait-post-ready-unrelated-hit](#native-wait-post-ready-unrelated-hit) |
| `wait:hit:wait-post:edit` | [wait-report-post-edit](#wait-report-post-edit) |
| `wait:hit:wait-post:mention` | [native-wait-post-subscribe-then-hit](#native-wait-post-subscribe-then-hit) |
| `wait:hit:wait-post:relevance` | [native-wait-relevance](#native-wait-relevance) |
| `wait:hit:wait-post:subscriber-add` | [wait-report-post-subscriber](#wait-report-post-subscriber) |
| `wait:hit:wait-post:subscriber-remove` | [wait-report-post-subscriber](#wait-report-post-subscriber) |
| `wait:hit:wait-state` | [native-wait-state-locked](#native-wait-state-locked) |
| `wait:hit:wait-state:removed` | [native-wait-state-removed-conflict](#native-wait-state-removed-conflict) |
| `wait:hit:wait-state:renewal` | [wait-report-state-renewal](#wait-report-state-renewal) |
| `wait:hit:wait:brief-description` | [wait-report-brief](#wait-report-brief) |
| `wait:hit:wait:brief-entry-add` | [wait-report-brief](#wait-report-brief) |
| `wait:hit:wait:brief-entry-update` | [wait-report-brief](#wait-report-brief) |
| `wait:hit:wait:byte-only` | [native-wait-byte-only](#native-wait-byte-only) |
| `wait:hit:wait:created` | [native-wait-created-removed](#native-wait-created-removed) |
| `wait:hit:wait:document-name` | [wait-report-document-name](#wait-report-document-name) |
| `wait:hit:wait:order` | [wait-report-order](#wait-report-order) |
| `wait:hit:wait:profile-section-add` | [wait-report-profile](#wait-report-profile) |
| `wait:hit:wait:profile-section-remove` | [wait-report-profile](#wait-report-profile) |
| `wait:hit:wait:profile-section-update` | [wait-report-profile](#wait-report-profile) |
| `wait:hit:wait:removed` | [native-wait-created-removed](#native-wait-created-removed) |
| `wait:hit:wait:type-switch` | [wait-report-type-switch](#wait-report-type-switch) |
| `wait:ready:wait` | [native-wait-created-removed](#native-wait-created-removed) |
| `wait:ready:wait-post` | [native-wait-post-ready-unrelated-hit](#native-wait-post-ready-unrelated-hit) |
| `wait:ready:wait-state` | [native-wait-state-locked](#native-wait-state-locked) |
| `wait:reread-after-ready` | [native-wait-restart-baseline](#native-wait-restart-baseline) |
| `wait:restart-baseline` | [native-wait-restart-baseline](#native-wait-restart-baseline) |
| `wait:stop:before-ready` | [native-wait-stop-fence](#native-wait-stop-fence) |
| `wait:unrelated:wait-post` | [native-wait-post-ready-unrelated-hit](#native-wait-post-ready-unrelated-hit) |
| `wait:unrelated:wait-post:subscribe` | [native-wait-post-subscribe-then-hit](#native-wait-post-subscribe-then-hit) |
| `wait:unrelated:wait-state` | [native-wait-state-locked](#native-wait-state-locked) |
| `write:body:control-bytes` | [write-post-unicode-control](#write-post-unicode-control) |
| `write:body:normalize` | [write-post-body-fence](#write-post-body-fence) |
| `write:brief:create-bare` | [write-brief-skeleton](#write-brief-skeleton) |
| `write:brief:create-description` | [write-brief-description](#write-brief-description) |
| `write:brief:entry-add` | [write-brief-entry-forms](#write-brief-entry-forms) |
| `write:brief:entry-add-default-title` | [write-brief-entry-forms](#write-brief-entry-forms) |
| `write:brief:entry-remove` | [write-brief-remove-entry](#write-brief-remove-entry) |
| `write:brief:entry-update` | [write-brief-entry-forms](#write-brief-entry-forms) |
| `write:brief:note-insert` | [write-brief-note-and-notfound](#write-brief-note-and-notfound) |
| `write:brief:note-with-target` | [write-brief-entry-forms](#write-brief-entry-forms) |
| `write:brief:refresh-all` | [write-brief-refresh](#write-brief-refresh) |
| `write:brief:refresh-empty` | [write-brief-refresh-empty](#write-brief-refresh-empty) |
| `write:brief:refresh-one` | [write-brief-refresh](#write-brief-refresh) |
| `write:brief:refresh-scan-skipped` | [write-brief-refresh-partial](#write-brief-refresh-partial) |
| `write:brief:refresh-skipped-no-path` | [write-brief-refresh-partial](#write-brief-refresh-partial) |
| `write:brief:refresh-skipped-target-missing` | [write-brief-refresh-partial](#write-brief-refresh-partial) |
| `write:brief:refresh-unchanged` | [write-brief-refresh](#write-brief-refresh) |
| `write:brief:refused` | [write-brief-target-relative](#write-brief-target-relative)<br>[write-brief-note-and-notfound](#write-brief-note-and-notfound)<br>[write-brief-title-taken](#write-brief-title-taken) |
| `write:brief:regex-delete` | [write-brief-entry-forms](#write-brief-entry-forms) |
| `write:brief:regex-set` | [write-brief-entry-forms](#write-brief-entry-forms) |
| `write:brief:replace-description` | [write-brief-description](#write-brief-description) |
| `write:brief:scan-fix` | [write-brief-refresh-partial](#write-brief-refresh-partial) |
| `write:brief:scan-skipped` | [write-scan-partial-skipped](#write-scan-partial-skipped) |
| `write:brief:target-directory` | [write-brief-target-directory](#write-brief-target-directory) |
| `write:brief:target-relative` | [write-brief-target-relative](#write-brief-target-relative) |
| `write:brief:unchanged` | [write-brief-skeleton](#write-brief-skeleton) |
| `write:fix:manual` | [write-post-invalid-name](#write-post-invalid-name) |
| `write:fix:operand-dash` | [write-fix-operand-dash](#write-fix-operand-dash) |
| `write:fix:operand-quote` | [write-fix-operand-quoted](#write-fix-operand-quoted) |
| `write:fix:operand-unicode` | [write-fix-operand-quoted](#write-fix-operand-quoted) |
| `write:fix:placeholder-same-flags` | [write-state-not-dir-same-flags](#write-state-not-dir-same-flags) |
| `write:fix:separator` | [write-brief-title-taken](#write-brief-title-taken) |
| `write:io:path-class` | [write-post-path-newline-windows](#write-post-path-newline-windows)<br>[write-brief-target-directory](#write-brief-target-directory) |
| `write:post:append` | [write-post-reply](#write-post-reply) |
| `write:post:body-fence` | [write-post-body-fence](#write-post-body-fence) |
| `write:post:create` | [write-post-create](#write-post-create) |
| `write:post:edit` | [write-post-edit-own](#write-post-edit-own) |
| `write:post:mentions` | [write-post-create](#write-post-create)<br>[write-post-reply](#write-post-reply) |
| `write:post:refused` | [write-post-edit-other-author](#write-post-edit-other-author)<br>[write-post-edit-missing-comment](#write-post-edit-missing-comment)<br>[write-post-invalid-name](#write-post-invalid-name) |
| `write:post:reply-to` | [write-post-reply](#write-post-reply) |
| `write:post:reply-to-not-found` | [write-post-reply-not-found](#write-post-reply-not-found) |
| `write:post:unicode` | [write-post-unicode-control](#write-post-unicode-control) |
| `write:profile:append-section` | [write-profile-replace-and-add](#write-profile-replace-and-add) |
| `write:profile:create-bare` | [write-profile-bare](#write-profile-bare) |
| `write:profile:create-section` | [write-profile-create-section](#write-profile-create-section) |
| `write:profile:refused` | [write-profile-section-notfound](#write-profile-section-notfound)<br>[write-file-missing-chain](#write-file-missing-chain) |
| `write:profile:remove-section` | [write-profile-remove-section](#write-profile-remove-section) |
| `write:profile:replace-section` | [write-profile-replace-and-add](#write-profile-replace-and-add) |
| `write:profile:scan-partial` | [write-scan-partial-skipped](#write-scan-partial-skipped) |
| `write:profile:scan-skipped` | [write-scan-partial-skipped](#write-scan-partial-skipped) |
| `write:profile:skipped-busy` | [native-lock-writes-windows](#native-lock-writes-windows) |
| `write:profile:unchanged` | [write-profile-bare](#write-profile-bare) |
| `write:refused:type` | [write-type-mismatch](#write-type-mismatch)<br>[write-type-state-file](#write-type-state-file)<br>[write-type-state-name-taken](#write-type-state-name-taken) |
| `write:row:created-dir` | [write-profile-create-section](#write-profile-create-section) |
| `write:row:created-file` | [write-profile-create-section](#write-profile-create-section) |
| `write:row:remaining` | [write-brief-refresh-partial](#write-brief-refresh-partial) |
| `write:row:removed-lines` | [write-profile-remove-section](#write-profile-remove-section)<br>[write-brief-remove-entry](#write-brief-remove-entry) |
| `write:state:check-state-no-ignore` | [write-state-ignored-descendant](#write-state-ignored-descendant) |
| `write:state:cleaned` | [write-state-cleaned](#write-state-cleaned) |
| `write:state:conflict` | [write-state-conflict](#write-state-conflict) |
| `write:state:custom-status` | [write-state-renew-and-merge](#write-state-renew-and-merge) |
| `write:state:declare-create` | [write-state-create](#write-state-create) |
| `write:state:default-duration` | [write-state-renew-and-merge](#write-state-renew-and-merge) |
| `write:state:delete-file` | [write-state-remove-forms](#write-state-remove-forms) |
| `write:state:ignored-descendant-lock` | [write-state-ignored-descendant](#write-state-ignored-descendant) |
| `write:state:merge-line` | [write-state-renew-and-merge](#write-state-renew-and-merge) |
| `write:state:new-status` | [write-state-renew-and-merge](#write-state-renew-and-merge) |
| `write:state:refused` | [write-state-remove-notfound](#write-state-remove-notfound)<br>[write-state-dir-missing](#write-state-dir-missing)<br>[write-state-dir-new](#write-state-dir-new)<br>[write-state-not-dir](#write-state-not-dir)<br>[write-state-not-dir-same-flags](#write-state-not-dir-same-flags) |
| `write:state:release-own` | [write-state-conflict](#write-state-conflict) |
| `write:state:remove-all` | [write-state-remove-forms](#write-state-remove-forms) |
| `write:state:remove-section` | [write-state-remove-forms](#write-state-remove-forms) |
| `write:state:remove-status` | [write-state-remove-forms](#write-state-remove-forms) |
| `write:state:removed-other` | [write-state-remove-other](#write-state-remove-other) |
| `write:state:renew` | [write-state-renew-and-merge](#write-state-renew-and-merge) |
| `write:state:scan-fix` | [write-scan-partial-skipped](#write-scan-partial-skipped) |
| `write:state:scan-skipped` | [write-scan-partial-skipped](#write-scan-partial-skipped) |
| `write:status:warn-written` | [write-post-reply-not-found](#write-post-reply-not-found)<br>[write-brief-refresh-partial](#write-brief-refresh-partial)<br>[write-state-conflict](#write-state-conflict)<br>[write-scan-partial-skipped](#write-scan-partial-skipped) |
| `write:subscribe:add` | [write-subscribe-lifecycle](#write-subscribe-lifecycle) |
| `write:subscribe:cancel` | [write-subscribe-lifecycle](#write-subscribe-lifecycle) |
| `write:subscribe:cancel-last` | [write-subscribe-lifecycle](#write-subscribe-lifecycle) |
| `write:subscribe:cancel-unchanged` | [write-subscribe-lifecycle](#write-subscribe-lifecycle) |
| `write:subscribe:create` | [write-subscribe-lifecycle](#write-subscribe-lifecycle) |
| `write:subscribe:insert-line` | [write-subscribe-existing-post](#write-subscribe-existing-post) |
| `write:subscribe:unchanged` | [write-subscribe-lifecycle](#write-subscribe-lifecycle) |

<a id="source-inventory"></a>

### 按文本来源核对

下表不取自示例的自我声明，而是从产品源码与被测程序中独立列出：诊断目录中的每个 ID、每个内置指南页、顶层帮助列出的每个命令（按示例声明的覆盖项核对），以及源码中不在诊断目录里的每段恢复与报错文字：`FIX_*`/`MESSAGE_*`/`WHY_*` 等常量、ignore 来源问题的各种修复说明与 fix 句式、I/O 失败报告的各分支、参数与环境变量错误消息。这些文字按实际捕获的输出核对，而且只在它真正出现的那种行里才算覆盖：修复说明要在报告的 `fix` 行里，原因要在 `why` 行里，`--include` 提示要在 `include` 行里，失败消息要在 `reason`、`refused` 行或诊断的消息行（`error`/`warn` 加 `L<行号>` 之后）开头，参数与环境变量错误要在 stderr 以 `error: ` 开头的行里；指南、帮助里出现的同样文字不算。共 245 项，未覆盖 10 项（其中 10 项说明了原因）。

| 来源项 | 来源 | 示例 |
|---|---|---|
| `diag:E-UTF8` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-utf8](#diag-utf8) |
| `diag:E-TOO-LARGE` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-too-large](#diag-too-large) |
| `diag:E-H1` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-h1](#diag-h1)<br>[diag-h1-bad-stem](#diag-h1-bad-stem) |
| `diag:E-FENCE` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-fence](#diag-fence)<br>[native-wait-stop-fence](#native-wait-stop-fence) |
| `diag:E-HEAD` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-head](#diag-head)<br>[diag-head-unnumbered](#diag-head-unnumbered)<br>[diag-head-unnumbered-unknown](#diag-head-unnumbered-unknown)<br>[diag-head-fill](#diag-head-fill)<br>[diag-head-salvage](#diag-head-salvage)<br>[diag-head-fill-edited](#diag-head-fill-edited)<br>[diag-numbering-exhausted-check](#diag-numbering-exhausted-check)<br>[read-lookfor-blocking](#read-lookfor-blocking) |
| `diag:E-SEQ-DUP` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-seq-dup](#diag-seq-dup)<br>[diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `diag:E-ENTRY-DUP` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-brief-structure](#diag-brief-structure) |
| `diag:E-ENTRY-PATH` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-brief-structure](#diag-brief-structure)<br>[read-brief-states](#read-brief-states) |
| `diag:E-ENTRY-HASH` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-brief-structure](#diag-brief-structure)<br>[read-brief-states](#read-brief-states) |
| `diag:E-ENTRY-REGEX` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-entry-regex](#diag-entry-regex)<br>[read-brief-states](#read-brief-states) |
| `diag:E-ENTRY-TARGET` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-entry-target](#diag-entry-target)<br>[read-brief-states](#read-brief-states) |
| `diag:E-SECTION-DUP` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-profile-sections](#diag-profile-sections) |
| `diag:E-STATUS-DUP` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-control-heading](#diag-control-heading)<br>[diag-state-structure](#diag-state-structure)<br>[diag-state-invalid-repeated](#diag-state-invalid-repeated) |
| `diag:E-TYPE` | 诊断目录 `crates/core/src/diag/catalog.rs` | [native-wait-stop-wrong-type](#native-wait-stop-wrong-type)<br>[read-lookfor-wrong-file](#read-lookfor-wrong-file)<br>[read-brief-errors](#read-brief-errors)<br>[write-type-mismatch](#write-type-mismatch)<br>[write-type-state-file](#write-type-state-file)<br>[write-type-state-name-taken](#write-type-state-name-taken) |
| `diag:E-LOCKED` | 诊断目录 `crates/core/src/diag/catalog.rs` | [native-lock-reads-windows](#native-lock-reads-windows)<br>[native-lock-writes-windows](#native-lock-writes-windows)<br>[native-lock-linux](#native-lock-linux) |
| `diag:E-INUSE` | 诊断目录 `crates/core/src/diag/catalog.rs` | [native-inuse-windows](#native-inuse-windows) |
| `diag:E-FLOCKED` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-flocked-own](#diag-flocked-own)<br>[diag-flocked-own-separator](#diag-flocked-own-separator)<br>[diag-flocked-mistyped-as](#diag-flocked-mistyped-as)<br>[diag-flocked-other](#diag-flocked-other) |
| `diag:E-FILE-MISSING` | 诊断目录 `crates/core/src/diag/catalog.rs` | [usage-path-is-directory](#usage-path-is-directory)<br>[read-check-missing](#read-check-missing)<br>[read-lookfor-wrong-file](#read-lookfor-wrong-file)<br>[read-brief-errors](#read-brief-errors)<br>[write-file-missing-chain](#write-file-missing-chain) |
| `diag:E-DIR-MISSING` | 诊断目录 `crates/core/src/diag/catalog.rs` | [native-wait-start-dir-missing](#native-wait-start-dir-missing)<br>[read-list-dir-errors](#read-list-dir-errors)<br>[read-list-dir-missing-middle](#read-list-dir-missing-middle)<br>[read-notification-dir-errors](#read-notification-dir-errors)<br>[read-check-state-dir-errors](#read-check-state-dir-errors)<br>[write-state-dir-missing](#write-state-dir-missing)<br>[write-state-dir-new](#write-state-dir-new) |
| `diag:E-NOT-DIR` | 诊断目录 `crates/core/src/diag/catalog.rs` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir)<br>[read-list-dir-errors](#read-list-dir-errors)<br>[read-notification-dir-errors](#read-notification-dir-errors)<br>[read-check-state-dir-errors](#read-check-state-dir-errors)<br>[write-state-not-dir](#write-state-not-dir)<br>[write-state-not-dir-same-flags](#write-state-not-dir-same-flags) |
| `diag:E-TARGET-MISSING` | 诊断目录 `crates/core/src/diag/catalog.rs` | [write-brief-target-relative](#write-brief-target-relative) |
| `diag:E-ENTRY-NOTFOUND` | 诊断目录 `crates/core/src/diag/catalog.rs` | [read-brief-errors](#read-brief-errors)<br>[write-brief-note-and-notfound](#write-brief-note-and-notfound) |
| `diag:E-SECTION-NOTFOUND` | 诊断目录 `crates/core/src/diag/catalog.rs` | [write-profile-section-notfound](#write-profile-section-notfound) |
| `diag:E-STATE-NOTFOUND` | 诊断目录 `crates/core/src/diag/catalog.rs` | [write-state-remove-notfound](#write-state-remove-notfound) |
| `diag:E-EDIT-AUTHOR` | 诊断目录 `crates/core/src/diag/catalog.rs` | [write-post-edit-other-author](#write-post-edit-other-author)<br>[write-fix-operand-dash](#write-fix-operand-dash)<br>[write-fix-operand-quoted](#write-fix-operand-quoted) |
| `diag:E-COMMENT-NOTFOUND` | 诊断目录 `crates/core/src/diag/catalog.rs` | [write-post-edit-missing-comment](#write-post-edit-missing-comment) |
| `diag:E-ENTRY-TITLE-TAKEN` | 诊断目录 `crates/core/src/diag/catalog.rs` | [write-brief-title-taken](#write-brief-title-taken) |
| `diag:E-SEQ-EXHAUSTED` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-seq-exhausted](#diag-seq-exhausted) |
| `diag:W-BOM` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-post-encoding](#diag-post-encoding) |
| `diag:W-CRLF` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-post-encoding](#diag-post-encoding) |
| `diag:W-NAME` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-name-mismatch](#diag-name-mismatch) |
| `diag:W-NAME-STATE` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-state-unselected](#diag-state-unselected) |
| `diag:W-STRAY` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-entry-local](#diag-entry-local)<br>[diag-profile-sections](#diag-profile-sections)<br>[diag-post-sequence](#diag-post-sequence)<br>[diag-post-body](#diag-post-body) |
| `diag:W-BODY` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-profile-sections](#diag-profile-sections)<br>[diag-control-heading](#diag-control-heading)<br>[diag-post-body](#diag-post-body) |
| `diag:W-SUB-DUP` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-post-encoding](#diag-post-encoding) |
| `diag:W-SEQ-GAP` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check)<br>[diag-post-sequence](#diag-post-sequence) |
| `diag:W-SEQ-ORDER` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-REPLY-DANGLING` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-post-sequence](#diag-post-sequence) |
| `diag:W-ENTRY-ABS` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-entry-local](#diag-entry-local) |
| `diag:W-ENTRY-KEY` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-entry-local](#diag-entry-local) |
| `diag:W-ENTRY-SHIFTED` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-entry-drift](#diag-entry-drift)<br>[read-brief-states](#read-brief-states) |
| `diag:W-ENTRY-STALE` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-entry-drift](#diag-entry-drift)<br>[read-brief-states](#read-brief-states) |
| `diag:W-PROFILE-EMPTY` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-profile-empty](#diag-profile-empty) |
| `diag:W-SECTION-NAME` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-profile-sections](#diag-profile-sections)<br>[diag-heading-form](#diag-heading-form) |
| `diag:W-STATUS-NAME` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-heading-form](#diag-heading-form)<br>[diag-control-heading](#diag-control-heading)<br>[diag-state-structure](#diag-state-structure)<br>[diag-state-invalid-repeated](#diag-state-invalid-repeated) |
| `diag:W-STATE-ENTRY` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-state-structure](#diag-state-structure) |
| `diag:W-STATE-DUP` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-state-advisory](#diag-state-advisory)<br>[diag-state-unselected](#diag-state-unselected)<br>[native-wait-stop-state-dup](#native-wait-stop-state-dup)<br>[native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty)<br>[read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `diag:W-STATE-RANGE` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-state-advisory](#diag-state-advisory)<br>[diag-state-unselected](#diag-state-unselected)<br>[read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `diag:W-STATE-CONFLICT` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-state-advisory](#diag-state-advisory)<br>[read-state-conflict-views](#read-state-conflict-views)<br>[read-state-conflict-dirty](#read-state-conflict-dirty) |
| `diag:W-STATE-EXPIRED` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-state-advisory](#diag-state-advisory)<br>[diag-state-structure](#diag-state-structure)<br>[diag-state-unselected](#diag-state-unselected)<br>[read-state-duplicates-and-ranges](#read-state-duplicates-and-ranges) |
| `diag:E-NAME` | 诊断目录 `crates/core/src/diag/catalog.rs` | [write-post-invalid-name](#write-post-invalid-name) |
| `diag:E-STATE-DIRTY` | 诊断目录 `crates/core/src/diag/catalog.rs` | [diag-state-dirty](#diag-state-dirty) |
| `info:guide:index` | 内置指南 `crates/cli/guide/index.md` | [info-guide-index](#info-guide-index) |
| `info:guide:read` | 内置指南 `crates/cli/guide/read.md` | [info-guide-read](#info-guide-read) |
| `info:guide:start` | 内置指南 `crates/cli/guide/start.md` | [info-guide-start](#info-guide-start) |
| `info:guide:wait` | 内置指南 `crates/cli/guide/wait.md` | [info-guide-wait](#info-guide-wait) |
| `info:guide:write` | 内置指南 `crates/cli/guide/write.md` | [info-guide-write](#info-guide-write) |
| `info:help:guide` | 顶层 `--help` 的命令列表 | [info-help-guide](#info-help-guide) |
| `info:help:list` | 顶层 `--help` 的命令列表 | [info-help-list](#info-help-list) |
| `info:help:check` | 顶层 `--help` 的命令列表 | [info-help-check](#info-help-check) |
| `info:help:profile` | 顶层 `--help` 的命令列表 | [info-help-profile](#info-help-profile) |
| `info:help:post` | 顶层 `--help` 的命令列表 | [info-help-post](#info-help-post) |
| `info:help:subscribe` | 顶层 `--help` 的命令列表 | [info-help-subscribe](#info-help-subscribe) |
| `info:help:brief` | 顶层 `--help` 的命令列表 | [info-help-brief](#info-help-brief) |
| `info:help:state` | 顶层 `--help` 的命令列表 | [info-help-state](#info-help-state) |
| `info:help:read-notification` | 顶层 `--help` 的命令列表 | [info-help-read-notification](#info-help-read-notification) |
| `info:help:lookfor-comment` | 顶层 `--help` 的命令列表 | [info-help-lookfor-comment](#info-help-lookfor-comment) |
| `info:help:read-brief` | 顶层 `--help` 的命令列表 | [info-help-read-brief](#info-help-read-brief) |
| `info:help:check-state` | 顶层 `--help` 的命令列表 | [info-help-check-state](#info-help-check-state) |
| `info:help:wait` | 顶层 `--help` 的命令列表 | [info-help-wait](#info-help-wait) |
| `info:help:wait-post` | 顶层 `--help` 的命令列表 | [info-help-wait-post](#info-help-wait-post) |
| `info:help:wait-state` | 顶层 `--help` 的命令列表 | [info-help-wait-state](#info-help-wait-state) |
| `info:help:help` | 顶层 `--help` 的命令列表 | [info-help-help](#info-help-help) |
| `text:args:is-not-a-valid-guide-topic-expected-start-read-write-or-wait` | 错误消息（`crates/cli/src/args.rs`） | [usage-guide-topic-invalid](#usage-guide-topic-invalid)<br>[usage-guide-topic-whitespace](#usage-guide-topic-whitespace)<br>[usage-guide-topic-edge-space](#usage-guide-topic-edge-space) |
| `text:args:is-not-a-valid-type-expected` | 错误消息（`crates/cli/src/args.rs`） | [usage-type-invalid](#usage-type-invalid)<br>[usage-type-edge-space](#usage-type-edge-space) |
| `text:args:m-and-remove-require-section-name` | 错误消息（`crates/cli/src/args.rs`） | [usage-profile-m-without-section](#usage-profile-m-without-section) |
| `text:args:section-requires-exactly-one-of-m-content-or-remove` | 错误消息（`crates/cli/src/args.rs`） | [usage-profile-section-alone](#usage-profile-section-alone) |
| `text:args:section-takes-exactly-one-of-m-content-or-remove-not-both` | 错误消息（`crates/cli/src/args.rs`） | [usage-profile-section-both](#usage-profile-section-both) |
| `text:args:refresh-cannot-be-combined-with-target-regex-m-or-remove` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-refresh-combined](#usage-brief-refresh-combined)<br>[usage-brief-regex-with-refresh](#usage-brief-regex-with-refresh) |
| `text:args:m-cannot-be-combined-with-remove` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-remove-with-m](#usage-brief-remove-with-m) |
| `text:args:target-cannot-be-combined-with-remove` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-remove-with-target](#usage-brief-remove-with-target) |
| `text:args:regex-cannot-be-combined-with-remove` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-regex-with-remove](#usage-brief-regex-with-remove) |
| `text:args:regex-requires-target-path` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-regex-without-target](#usage-brief-regex-without-target) |
| `text:args:regex-must-not-contain-raw-cr-or-lf-got-write-r-or-n-in-the-patt` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-regex-newline](#usage-brief-regex-newline) |
| `text:args:regex-does-not-compile` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-regex-syntax](#usage-brief-regex-syntax)<br>[usage-brief-regex-size](#usage-brief-regex-size) |
| `text:args:target-must-not-contain-raw-cr-or-lf` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-target-newline](#usage-brief-target-newline) |
| `text:args:for-is-only-valid-in-the-declare-form-state-as-nick-status-name` | 错误消息（`crates/cli/src/args.rs`） | [usage-state-for-in-remove](#usage-state-for-in-remove) |
| `text:args:state-requires-as-nick-status-name-or-the-remove-form-remove-nic` | 错误消息（`crates/cli/src/args.rs`） | [usage-state-no-form](#usage-state-no-form) |
| `text:args:status-name-is-required-in-the-declare-form` | 错误消息（`crates/cli/src/args.rs`） | [usage-state-no-status](#usage-state-no-status) |
| `text:args:keyword-must-not-be-empty-omit-keyword-to-list-every-comment` | 错误消息（`crates/cli/src/args.rs`） | [usage-keyword-empty](#usage-keyword-empty) |
| `text:args:must-be-a-nick` | 错误消息（`crates/cli/src/args.rs`） | [info-help-overrides](#info-help-overrides)<br>[usage-nick-invalid](#usage-nick-invalid)<br>[usage-nick-at-sign](#usage-nick-at-sign)<br>[usage-nick-empty](#usage-nick-empty)<br>[usage-nick-too-long](#usage-nick-too-long)<br>[usage-remove-nick-invalid](#usage-remove-nick-invalid)<br>[usage-nick-edge-space](#usage-nick-edge-space)<br>[usage-nick-inner-double-space](#usage-nick-inner-double-space)<br>[usage-nick-bom](#usage-nick-bom)<br>[usage-mention-zero-width](#usage-mention-zero-width) |
| `text:args:status-must-be-a-status-name` | 错误消息（`crates/cli/src/args.rs`） | [usage-status-invalid](#usage-status-invalid)<br>[usage-wait-state-status-invalid](#usage-wait-state-status-invalid)<br>[usage-status-edge-space](#usage-status-edge-space)<br>[usage-status-trailing-cr-still-invalid](#usage-status-trailing-cr-still-invalid)<br>[usage-status-bidi](#usage-status-bidi) |
| `text:args:section-must-be-a-section-name` | 错误消息（`crates/cli/src/args.rs`） | [usage-section-invalid](#usage-section-invalid)<br>[usage-section-edge-space](#usage-section-edge-space) |
| `text:args:remove-requires-entry-title` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-remove-without-entry](#usage-brief-remove-without-entry) |
| `text:args:entry-requires-m-note-target-path-remove-or-refresh` | 错误消息（`crates/cli/src/args.rs`） | [usage-brief-entry-alone](#usage-brief-entry-alone) |
| `text:const:MESSAGE_ESTATE_NOTFOUND_STATUS` | 常量 `MESSAGE_ESTATE_NOTFOUND_STATUS`（`crates/core/src/diag/catalog.rs`） | [write-state-remove-notfound](#write-state-remove-notfound) |
| `text:const:MESSAGE_EFILEMISSING_DIR` | 常量 `MESSAGE_EFILEMISSING_DIR`（`crates/core/src/diag/catalog.rs`） | [usage-path-is-directory](#usage-path-is-directory) |
| `text:const:FIX_EFILEMISSING_DIR` | 常量 `FIX_EFILEMISSING_DIR`（`crates/core/src/diag/catalog.rs`） | [usage-path-is-directory](#usage-path-is-directory) |
| `text:const:FIX_ESTATE_NOTFOUND_STATUS` | 常量 `FIX_ESTATE_NOTFOUND_STATUS`（`crates/core/src/diag/catalog.rs`） | [write-state-remove-notfound](#write-state-remove-notfound) |
| `text:const:FIX_ETYPE_RESERVED` | 常量 `FIX_ETYPE_RESERVED`（`crates/core/src/diag/catalog.rs`） | [native-wait-stop-wrong-type](#native-wait-stop-wrong-type)<br>[write-type-state-name-taken](#write-type-state-name-taken) |
| `text:const:FIX_ETYPE_CREATE` | 常量 `FIX_ETYPE_CREATE`（`crates/core/src/diag/catalog.rs`） | [write-type-mismatch](#write-type-mismatch)<br>[write-type-state-file](#write-type-state-file) |
| `text:const:FIX_ENOTDIR_RETRY` | 常量 `FIX_ENOTDIR_RETRY`（`crates/core/src/diag/catalog.rs`） | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir)<br>[read-list-dir-errors](#read-list-dir-errors)<br>[read-notification-dir-errors](#read-notification-dir-errors)<br>[read-check-state-dir-errors](#read-check-state-dir-errors)<br>[write-state-not-dir](#write-state-not-dir) |
| `text:const:FIX_EENTRYTITLETAKEN_RETRY` | 常量 `FIX_EENTRYTITLETAKEN_RETRY`（`crates/core/src/diag/catalog.rs`） | [write-brief-title-taken](#write-brief-title-taken) |
| `text:const:FIX_EDIRMISSING_CREATE` | 常量 `FIX_EDIRMISSING_CREATE`（`crates/core/src/diag/catalog.rs`） | [native-wait-start-dir-missing](#native-wait-start-dir-missing)<br>[write-state-dir-missing](#write-state-dir-missing)<br>[write-state-dir-new](#write-state-dir-new) |
| `text:const:FIX_EDIRMISSING_ANCESTOR` | 常量 `FIX_EDIRMISSING_ANCESTOR`（`crates/core/src/diag/catalog.rs`） | [read-list-dir-missing-middle](#read-list-dir-missing-middle) |
| `text:const:FIX_WSTALE_TARGET_MISSING` | 常量 `FIX_WSTALE_TARGET_MISSING`（`crates/core/src/diag/catalog.rs`） | [diag-entry-drift](#diag-entry-drift)<br>[read-brief-states](#read-brief-states)<br>[write-brief-refresh-partial](#write-brief-refresh-partial) |
| `text:const:FIX_EENTRYREGEX_REVIEW` | 常量 `FIX_EENTRYREGEX_REVIEW`（`crates/core/src/diag/catalog.rs`） | [diag-entry-regex](#diag-entry-regex)<br>[read-brief-states](#read-brief-states) |
| `text:const:FIX_UNSELECTED_STATE` | 常量 `FIX_UNSELECTED_STATE`（`crates/core/src/diag/catalog.rs`） | [diag-state-unselected](#diag-state-unselected) |
| `text:const:WHY_WSTATEDUP_UNSELECTED` | 常量 `WHY_WSTATEDUP_UNSELECTED`（`crates/core/src/diag/catalog.rs`） | [diag-state-unselected](#diag-state-unselected) |
| `text:const:WHY_WSTATERANGE_UNSELECTED` | 常量 `WHY_WSTATERANGE_UNSELECTED`（`crates/core/src/diag/catalog.rs`） | [diag-state-unselected](#diag-state-unselected) |
| `text:const:WHY_WSTATEEXPIRED_UNSELECTED` | 常量 `WHY_WSTATEEXPIRED_UNSELECTED`（`crates/core/src/diag/catalog.rs`） | [diag-state-unselected](#diag-state-unselected) |
| `text:const:FIX_SECTION_NAME_UNIQUE` | 常量 `FIX_SECTION_NAME_UNIQUE`（`crates/core/src/diag/catalog.rs`） | [diag-profile-sections](#diag-profile-sections) |
| `text:const:FIX_NATIVE_OPERAND` | 常量 `FIX_NATIVE_OPERAND`（`crates/core/src/diag/catalog.rs`） | 未覆盖：这句通用文字在当前诊断目录下不会整句出现：带命令的 fix 模板都以 `: agent-coexist …`、`agent-coexist …` 或 `: {argv}` 收尾，命令无法写成可复制的一行时，分别改为保留子命令、只把路径换成 `<path of …>` 标记的手工命令（FIX_MANUAL_COMMAND，见 [native-noncopyable-manual-linux](#native-noncopyable-manual-linux)），或 `<原建议> by hand; <原因>`（diag/mod.rs manual_retry，见 [native-nonutf8-linux](#native-nonutf8-linux)）；I/O 报告、ignore 来源修复和写入后无法确认的报告（main.rs、error.rs）也只借用它分号后的原因。只有标题、名称这类来自已解析文件内容的值无法引用时才会退回整句，而这些值都是单行有效 UTF-8，含控制字符的另给 FIX_CONTROL_HEADING。这一结论来自阅读 diag/mod.rs 的 manual_fix、manual_retry、manual_command、wbody_fix 与 subst_with，不是运行结果。（native） |
| `text:const:FIX_MANUAL_COMMAND` | 常量 `FIX_MANUAL_COMMAND`（`crates/core/src/diag/catalog.rs`） | [native-noncopyable-manual-linux](#native-noncopyable-manual-linux)<br>[native-brief-refresh-manual-linux](#native-brief-refresh-manual-linux) |
| `text:const:MESSAGE_WRITE_REFUSED` | 常量 `MESSAGE_WRITE_REFUSED`（`crates/core/src/diag/catalog.rs`） | [diag-utf8](#diag-utf8)<br>[diag-too-large](#diag-too-large)<br>[diag-h1](#diag-h1)<br>[diag-fence](#diag-fence)<br>[diag-head-fill](#diag-head-fill)<br>[diag-head-salvage](#diag-head-salvage)<br>[diag-head-fill-edited](#diag-head-fill-edited) |
| `text:const:MESSAGE_WRITE_REFUSED_MIXED` | 常量 `MESSAGE_WRITE_REFUSED_MIXED`（`crates/core/src/diag/catalog.rs`） | [diag-head](#diag-head)<br>[diag-head-unnumbered](#diag-head-unnumbered)<br>[diag-seq-dup](#diag-seq-dup)<br>[diag-numbering-exhausted-check](#diag-numbering-exhausted-check)<br>[diag-brief-structure](#diag-brief-structure)<br>[diag-profile-sections](#diag-profile-sections)<br>[diag-state-structure](#diag-state-structure)<br>[diag-state-invalid-repeated](#diag-state-invalid-repeated) |
| `text:const:FIX_CONTROL_HEADING` | 常量 `FIX_CONTROL_HEADING`（`crates/core/src/diag/catalog.rs`） | [diag-control-heading](#diag-control-heading) |
| `text:const:FIX_ESTATUSDUP_CONTROL` | 常量 `FIX_ESTATUSDUP_CONTROL`（`crates/core/src/diag/catalog.rs`） | [diag-control-heading](#diag-control-heading) |
| `text:const:FIX_INVALID_STATUS` | 常量 `FIX_INVALID_STATUS`（`crates/core/src/diag/catalog.rs`） | [diag-state-structure](#diag-state-structure) |
| `text:const:FIX_EFLOCKED_AS` | 常量 `FIX_EFLOCKED_AS`（`crates/core/src/diag/catalog.rs`） | [diag-flocked-mistyped-as](#diag-flocked-mistyped-as)<br>[diag-flocked-other](#diag-flocked-other) |
| `text:const:FIX_WSTATUSNAME_REPEATED` | 常量 `FIX_WSTATUSNAME_REPEATED`（`crates/core/src/diag/catalog.rs`） | [diag-control-heading](#diag-control-heading)<br>[diag-state-invalid-repeated](#diag-state-invalid-repeated) |
| `text:const:FIX_EH1_BAD_STEM` | 常量 `FIX_EH1_BAD_STEM`（`crates/core/src/diag/catalog.rs`） | [diag-h1-bad-stem](#diag-h1-bad-stem) |
| `text:const:FIX_EH1_STATE_NAME` | 常量 `FIX_EH1_STATE_NAME`（`crates/core/src/diag/catalog.rs`） | [write-brief-refresh-partial](#write-brief-refresh-partial)<br>[write-scan-partial-skipped](#write-scan-partial-skipped) |
| `text:const:FIX_WNAME_BAD_STEM` | 常量 `FIX_WNAME_BAD_STEM`（`crates/core/src/diag/catalog.rs`） | [diag-name-mismatch](#diag-name-mismatch) |
| `text:const:FIX_EHEAD_EXHAUSTED` | 常量 `FIX_EHEAD_EXHAUSTED`（`crates/core/src/diag/catalog.rs`） | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `text:const:FIX_EXHAUSTED_FILL` | 常量 `FIX_EXHAUSTED_FILL`（`crates/core/src/diag/catalog.rs`） | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `text:const:FIX_EXHAUSTED_TAIL` | 常量 `FIX_EXHAUSTED_TAIL`（`crates/core/src/diag/catalog.rs`） | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `text:const:FIX_EHEAD_FILL` | 常量 `FIX_EHEAD_FILL`（`crates/core/src/diag/catalog.rs`） | [diag-head-fill](#diag-head-fill)<br>[diag-head-fill-edited](#diag-head-fill-edited) |
| `text:const:FIX_EHEAD_RENUMBERED` | 常量 `FIX_EHEAD_RENUMBERED`（`crates/core/src/diag/catalog.rs`） | [diag-head](#diag-head) |
| `text:const:FIX_EHEAD_UNNUMBERED` | 常量 `FIX_EHEAD_UNNUMBERED`（`crates/core/src/diag/catalog.rs`） | [diag-head-unnumbered](#diag-head-unnumbered)<br>[diag-head-unnumbered-unknown](#diag-head-unnumbered-unknown)<br>[diag-seq-dup](#diag-seq-dup) |
| `text:const:FIX_ESEQDUP_EXHAUSTED` | 常量 `FIX_ESEQDUP_EXHAUSTED`（`crates/core/src/diag/catalog.rs`） | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `text:const:WHY_WSEQGAP_MALFORMED` | 常量 `WHY_WSEQGAP_MALFORMED`（`crates/core/src/diag/catalog.rs`） | [diag-head](#diag-head)<br>[diag-head-unnumbered](#diag-head-unnumbered)<br>[diag-head-unnumbered-unknown](#diag-head-unnumbered-unknown)<br>[read-lookfor-blocking](#read-lookfor-blocking) |
| `text:const:WHY_WSEQGAP_LATER` | 常量 `WHY_WSEQGAP_LATER`（`crates/core/src/diag/catalog.rs`） | [diag-head](#diag-head)<br>[diag-post-sequence](#diag-post-sequence) |
| `text:const:MESSAGE_WSEQGAP_FIRST` | 常量 `MESSAGE_WSEQGAP_FIRST`（`crates/core/src/diag/catalog.rs`） | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check)<br>[diag-post-sequence](#diag-post-sequence) |
| `text:const:WHY_WSEQGAP_FIRST_LATER` | 常量 `WHY_WSEQGAP_FIRST_LATER`（`crates/core/src/diag/catalog.rs`） | [diag-post-sequence](#diag-post-sequence) |
| `text:const:WHY_WSEQGAP_FIRST` | 常量 `WHY_WSEQGAP_FIRST`（`crates/core/src/diag/catalog.rs`） | [diag-numbering-exhausted-check](#diag-numbering-exhausted-check) |
| `text:const:FIX_WSEQGAP_MALFORMED` | 常量 `FIX_WSEQGAP_MALFORMED`（`crates/core/src/diag/catalog.rs`） | [diag-head](#diag-head)<br>[diag-head-unnumbered](#diag-head-unnumbered)<br>[diag-head-unnumbered-unknown](#diag-head-unnumbered-unknown)<br>[read-lookfor-blocking](#read-lookfor-blocking) |
| `text:const:MESSAGE_ETOOLARGE_WRITE` | 常量 `MESSAGE_ETOOLARGE_WRITE`（`crates/core/src/diag/catalog.rs`） | [diag-too-large](#diag-too-large) |
| `text:const:FIX_ETOOLARGE_WRITE` | 常量 `FIX_ETOOLARGE_WRITE`（`crates/core/src/diag/catalog.rs`） | [diag-too-large](#diag-too-large) |
| `text:const:FIX_EENTRYHASH_NO_PATH` | 常量 `FIX_EENTRYHASH_NO_PATH`（`crates/core/src/diag/catalog.rs`） | [diag-brief-structure](#diag-brief-structure)<br>[diag-entry-regex](#diag-entry-regex)<br>[write-brief-refresh-partial](#write-brief-refresh-partial) |
| `text:const:FIX_EENTRYTARGET_DIR` | 常量 `FIX_EENTRYTARGET_DIR`（`crates/core/src/diag/catalog.rs`） | [diag-entry-target](#diag-entry-target)<br>[read-brief-states](#read-brief-states) |
| `text:const:FIX_WBODY_COMMENT` | 常量 `FIX_WBODY_COMMENT`（`crates/core/src/diag/catalog.rs`） | [diag-post-body](#diag-post-body) |
| `text:const:FIX_WBODY_SECTION_NAME` | 常量 `FIX_WBODY_SECTION_NAME`（`crates/core/src/diag/catalog.rs`） | [diag-profile-sections](#diag-profile-sections) |
| `text:const:FIX_WREPLYDANGLING_WRITE` | 常量 `FIX_WREPLYDANGLING_WRITE`（`crates/core/src/diag/catalog.rs`） | [write-post-reply-not-found](#write-post-reply-not-found) |
| `text:const:FIX_EFILEMISSING_ANCESTOR` | 常量 `FIX_EFILEMISSING_ANCESTOR`（`crates/core/src/diag/catalog.rs`） | [native-operands-linux](#native-operands-linux)<br>[write-file-missing-chain](#write-file-missing-chain) |
| `text:const:FIX_WSTATEDUP_WAIT` | 常量 `FIX_WSTATEDUP_WAIT`（`crates/core/src/diag/catalog.rs`） | [native-wait-stop-state-dup](#native-wait-stop-state-dup)<br>[native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty) |
| `text:const:FIX_WAIT_LOCKED` | 常量 `FIX_WAIT_LOCKED`（`crates/core/src/diag/catalog.rs`） | [native-lock-wait-windows](#native-lock-wait-windows)<br>[native-lock-wait-state-windows](#native-lock-wait-state-windows) |
| `text:const:FIX_WAIT_INUSE` | 常量 `FIX_WAIT_INUSE`（`crates/core/src/diag/catalog.rs`） | [native-lock-wait-windows](#native-lock-wait-windows) |
| `text:const:FIX_WAIT_IO_LEAD` | 常量 `FIX_WAIT_IO_LEAD`（`crates/core/src/diag/catalog.rs`） | [native-wait-parent-gone-after-ready](#native-wait-parent-gone-after-ready)<br>[native-wait-parent-replaced-by-file-after-ready-linux](#native-wait-parent-replaced-by-file-after-ready-linux)<br>[native-wait-file-became-dir-after-ready-linux](#native-wait-file-became-dir-after-ready-linux)<br>[native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows) |
| `text:const:REREAD_FILE` | 常量 `REREAD_FILE`（`crates/core/src/diag/catalog.rs`） | [native-wait-parent-gone-after-ready](#native-wait-parent-gone-after-ready)<br>[native-wait-parent-replaced-by-file-after-ready-linux](#native-wait-parent-replaced-by-file-after-ready-linux)<br>[native-wait-file-became-dir-after-ready-linux](#native-wait-file-became-dir-after-ready-linux)<br>[native-lock-wait-windows](#native-lock-wait-windows)<br>[native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows) |
| `text:const:REREAD_STATE` | 常量 `REREAD_STATE`（`crates/core/src/diag/catalog.rs`） | [native-lock-wait-state-windows](#native-lock-wait-state-windows) |
| `text:const:FIX_WAIT_RESTORE_DIR_GONE` | 常量 `FIX_WAIT_RESTORE_DIR_GONE`（`crates/core/src/diag/catalog.rs`） | [native-wait-parent-gone-after-ready](#native-wait-parent-gone-after-ready) |
| `text:const:FIX_WAIT_RESTORE_DIR_REPLACED` | 常量 `FIX_WAIT_RESTORE_DIR_REPLACED`（`crates/core/src/diag/catalog.rs`） | [native-wait-parent-replaced-by-file-after-ready-linux](#native-wait-parent-replaced-by-file-after-ready-linux) |
| `text:const:FIX_WAIT_RESTORE_NOT_REGULAR` | 常量 `FIX_WAIT_RESTORE_NOT_REGULAR`（`crates/core/src/diag/catalog.rs`） | [native-wait-file-became-dir-after-ready-linux](#native-wait-file-became-dir-after-ready-linux) |
| `text:const:FIX_LOCK_SCAN_INCOMPLETE` | 常量 `FIX_LOCK_SCAN_INCOMPLETE`（`crates/core/src/diag/catalog.rs`） | [diag-flocked-other](#diag-flocked-other)<br>[native-lock-writes-windows](#native-lock-writes-windows)<br>[write-brief-refresh-partial](#write-brief-refresh-partial)<br>[write-scan-partial-skipped](#write-scan-partial-skipped) |
| `text:const:FIX_FILE_CHECK` | 常量 `FIX_FILE_CHECK`（`crates/core/src/diag/catalog.rs`） | [diag-brief-structure](#diag-brief-structure)<br>[diag-entry-regex](#diag-entry-regex)<br>[write-brief-refresh-partial](#write-brief-refresh-partial)<br>[write-brief-insert-layout](#write-brief-insert-layout) |
| `text:const:FIX_FILE_CHECK_MANUAL` | 常量 `FIX_FILE_CHECK_MANUAL`（`crates/core/src/diag/catalog.rs`） | [native-brief-refresh-manual-linux](#native-brief-refresh-manual-linux) |
| `text:const:FIX_LOCK_SCAN_INCOMPLETE_MANUAL` | 常量 `FIX_LOCK_SCAN_INCOMPLETE_MANUAL`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux)<br>[native-lock-scan-here-linux](#native-lock-scan-here-linux) |
| `text:const:FIX_STATE_SCAN_INCOMPLETE` | 常量 `FIX_STATE_SCAN_INCOMPLETE`（`crates/core/src/diag/catalog.rs`） | [native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty)<br>[native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux)<br>[native-scan-dir-no-search-linux](#native-scan-dir-no-search-linux)<br>[write-scan-partial-skipped](#write-scan-partial-skipped) |
| `text:const:FIX_STATE_SCAN_INCOMPLETE_MANUAL` | 常量 `FIX_STATE_SCAN_INCOMPLETE_MANUAL`（`crates/core/src/diag/catalog.rs`） | [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:const:FIX_STATE_SCAN_STOPPED` | 常量 `FIX_STATE_SCAN_STOPPED`（`crates/core/src/diag/catalog.rs`） | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `text:const:FIX_STATE_SCAN_STOPPED_MANUAL` | 常量 `FIX_STATE_SCAN_STOPPED_MANUAL`（`crates/core/src/diag/catalog.rs`） | [native-state-scan-cwd-gone-linux](#native-state-scan-cwd-gone-linux) |
| `text:const:FIX_CHECK_STATE_UNREAD` | 常量 `FIX_CHECK_STATE_UNREAD`（`crates/core/src/diag/catalog.rs`） | [diag-state-dirty](#diag-state-dirty)<br>[native-wait-stop-state-dup-dirty](#native-wait-stop-state-dup-dirty)<br>[native-lock-reads-windows](#native-lock-reads-windows)<br>[native-scan-dir-no-search-linux](#native-scan-dir-no-search-linux)<br>[read-check-state-incomplete](#read-check-state-incomplete)<br>[read-state-conflict-dirty](#read-state-conflict-dirty) |
| `text:const:FIX_CHECK_STATE_UNREAD_MANUAL` | 常量 `FIX_CHECK_STATE_UNREAD_MANUAL`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:FIX_NOTIFY_UNREAD` | 常量 `FIX_NOTIFY_UNREAD`（`crates/core/src/diag/catalog.rs`） | [native-lock-reads-windows](#native-lock-reads-windows)<br>[native-scan-dir-no-search-linux](#native-scan-dir-no-search-linux)<br>[read-lookfor-blocking](#read-lookfor-blocking)<br>[read-notification-incomplete](#read-notification-incomplete) |
| `text:const:FIX_NOTIFY_UNREAD_MANUAL` | 常量 `FIX_NOTIFY_UNREAD_MANUAL`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:FIX_LIST_UNREAD` | 常量 `FIX_LIST_UNREAD`（`crates/core/src/diag/catalog.rs`） | [native-lock-reads-windows](#native-lock-reads-windows)<br>[native-inuse-windows](#native-inuse-windows)<br>[native-lock-linux](#native-lock-linux)<br>[native-permission-dir-windows](#native-permission-dir-windows)<br>[native-permission-linux](#native-permission-linux)<br>[native-scan-dir-no-search-linux](#native-scan-dir-no-search-linux)<br>[read-list-unknown-leaves](#read-list-unknown-leaves) |
| `text:const:FIX_LIST_UNREAD_MANUAL` | 常量 `FIX_LIST_UNREAD_MANUAL`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux)<br>[native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:const:SCAN_UNREADABLE_FIRST` | 常量 `SCAN_UNREADABLE_FIRST`（`crates/core/src/diag/catalog.rs`） | [native-permission-dir-windows](#native-permission-dir-windows)<br>[native-permission-linux](#native-permission-linux)<br>[native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux)<br>[native-scan-dir-no-search-linux](#native-scan-dir-no-search-linux) |
| `text:const:SCAN_CANONICALIZE_FIRST` | 常量 `SCAN_CANONICALIZE_FIRST`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-here-linux](#native-lock-scan-here-linux) |
| `text:const:FIX_CANONICALIZE` | 常量 `FIX_CANONICALIZE`（`crates/core/src/diag/catalog.rs`） | [native-permission-dir-windows](#native-permission-dir-windows)<br>[native-canonicalize-linux](#native-canonicalize-linux) |
| `text:const:MANUAL_CHECK_STATE_FILES` | 常量 `MANUAL_CHECK_STATE_FILES`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:MANUAL_CHECK_SCAN_FILES` | 常量 `MANUAL_CHECK_SCAN_FILES`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:MANUAL_CHECK_STATE_UNREADABLE` | 常量 `MANUAL_CHECK_STATE_UNREADABLE`（`crates/core/src/diag/catalog.rs`） | [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:const:MANUAL_CHECK_STATE_HERE` | 常量 `MANUAL_CHECK_STATE_HERE`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-here-linux](#native-lock-scan-here-linux) |
| `text:const:MANUAL_CHECK_LIST_LEAVES` | 常量 `MANUAL_CHECK_LIST_LEAVES`（`crates/core/src/diag/catalog.rs`） | [native-lock-scan-manual-linux](#native-lock-scan-manual-linux) |
| `text:const:MANUAL_LIST_UNREADABLE` | 常量 `MANUAL_LIST_UNREADABLE`（`crates/core/src/diag/catalog.rs`） | [native-state-scan-unlistable-linux](#native-state-scan-unlistable-linux) |
| `text:const:MESSAGE_WRITE_UNCONFIRMED` | 常量 `MESSAGE_WRITE_UNCONFIRMED`（`crates/core/src/diag/catalog.rs`） | 未覆盖：只在 fsx::commit 的最后一步 sync_data 失败（写入已完整）或恢复原内容本身失败（commit.rs Undo::restore）时显示。[native-io-classes-linux](#native-io-classes-linux) 的夹具都没有走到这两步：只读挂载在打开文件时就失败，提交从未开始；文件大小上限和磁盘已满时 post 失败，文件前后的 SHA-256 和大小（磁盘已满时还有 inode）都相同，也就是文件保持原样，夹具不能说明恢复步骤是否执行过、是否成功。产品里没有故障注入开关，也没有单元测试让 sync_data 或恢复失败（native） |
| `text:const:MESSAGE_WRITE_MIXED` | 常量 `MESSAGE_WRITE_MIXED`（`crates/core/src/diag/catalog.rs`） | 未覆盖：只在 fsx::commit 的最后一步 sync_data 失败（写入已完整）或恢复原内容本身失败（commit.rs Undo::restore）时显示。[native-io-classes-linux](#native-io-classes-linux) 的夹具都没有走到这两步：只读挂载在打开文件时就失败，提交从未开始；文件大小上限和磁盘已满时 post 失败，文件前后的 SHA-256 和大小（磁盘已满时还有 inode）都相同，也就是文件保持原样，夹具不能说明恢复步骤是否执行过、是否成功。产品里没有故障注入开关，也没有单元测试让 sync_data 或恢复失败（native） |
| `text:walk:is-not-a-valid-glob` | 错误消息（`crates/core/src/fsx/walk.rs`） | [usage-glob-empty](#usage-glob-empty)<br>[usage-glob-slash-only](#usage-glob-slash-only)<br>[usage-glob-unclosed-class](#usage-glob-unclosed-class)<br>[usage-glob-range](#usage-glob-range)<br>[usage-glob-unopened-alternates](#usage-glob-unopened-alternates)<br>[usage-glob-unclosed-alternates](#usage-glob-unclosed-alternates)<br>[usage-order-value-before-files](#usage-order-value-before-files) |
| `text:walk:the-pattern-is-empty` | 错误消息（`crates/core/src/fsx/walk.rs`） | [usage-glob-empty](#usage-glob-empty) |
| `text:walk:nothing-is-left-after-removing-the-leading-and-trailing` | 错误消息（`crates/core/src/fsx/walk.rs`） | [usage-glob-slash-only](#usage-glob-slash-only) |
| `text:names:m-text-is-empty-after-normalization-crlf-folded-trailing-whitesp` | 错误消息（`crates/core/src/names.rs`） | [usage-message-empty](#usage-message-empty) |
| `text:const:INCLUDE_HINT_EMPTY` | 常量 `INCLUDE_HINT_EMPTY`（`crates/core/src/ops/query/mod.rs`） | [read-list-filters](#read-list-filters)<br>[read-notification-empty-and-filters](#read-notification-empty-and-filters) |
| `text:const:INCLUDE_HINT_EMPTY_STATE` | 常量 `INCLUDE_HINT_EMPTY_STATE`（`crates/core/src/ops/query/mod.rs`） | [read-check-state-empty-and-filters](#read-check-state-empty-and-filters) |
| `text:const:INCLUDE_HINT_EXCLUDE` | 常量 `INCLUDE_HINT_EXCLUDE`（`crates/core/src/ops/query/mod.rs`） | [read-list-filters](#read-list-filters) |
| `text:const:INCLUDE_HINT_IGNORED` | 常量 `INCLUDE_HINT_IGNORED`（`crates/core/src/ops/query/mod.rs`） | [read-list-filters](#read-list-filters) |
| `text:time:agent-coexist-now-is-not-valid-unicode-set-it-to-a-utc-time-such` | 错误消息（`crates/core/src/time.rs`） | [native-invalid-unicode-windows](#native-invalid-unicode-windows)<br>[native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `text:time:agent-coexist-now-must-be-a-utc-time-yyyy-mm-ddthh-mm-ssz-such-a` | 错误消息（`crates/core/src/time.rs`） | [info-guide-ignores-env](#info-guide-ignores-env)<br>[usage-env-now-trailing-cr](#usage-env-now-trailing-cr)<br>[usage-env-now-shape](#usage-env-now-shape)<br>[usage-env-now-empty](#usage-env-now-empty)<br>[usage-order-env-now-first](#usage-order-env-now-first)<br>[usage-order-env-before-value](#usage-order-env-before-value) |
| `text:time:or-unset-it-to-use-the-system-clock` | 错误消息（`crates/core/src/time.rs`） | [usage-env-now-trailing-cr](#usage-env-now-trailing-cr)<br>[usage-env-now-shape](#usage-env-now-shape) |
| `text:time:unset-it-to-use-the-system-clock` | 错误消息（`crates/core/src/time.rs`） | [info-guide-ignores-env](#info-guide-ignores-env)<br>[usage-env-now-empty](#usage-env-now-empty)<br>[usage-order-env-now-first](#usage-order-env-now-first)<br>[usage-order-env-before-value](#usage-order-env-before-value) |
| `text:time:invalid-rfc-3339-time` | 错误消息（`crates/core/src/time.rs`） | 未覆盖：time::parse 的错误文本从不显示：--since 先尝试 RFC 3339 再按时长解析，失败时报告 --since 自己的消息；帖子与 state 解析用 .ok() 丢弃错误（information） |
| `text:time:for-must-be-a-duration-such-as-1h30m` | 错误消息（`crates/core/src/time.rs`） | [usage-duration-order](#usage-duration-order)<br>[usage-duration-edge-space](#usage-duration-edge-space) |
| `text:time:for-is-too-long-the-declaration-would-end-after-the-latest-repre` | 错误消息（`crates/core/src/time.rs`） | [usage-duration-overflow](#usage-duration-overflow) |
| `text:time:since-reaches-back-before-year-0001-use-a-shorter-duration-or-an` | 错误消息（`crates/core/src/time.rs`） | [usage-since-overflow](#usage-since-overflow)<br>[usage-since-out-of-range](#usage-since-out-of-range) |
| `text:time:since-must-be-an-rfc-3339-time-such-as-2026-09-17t09-12-00z-or-a` | 错误消息（`crates/core/src/time.rs`） | [usage-since-word](#usage-since-word)<br>[usage-since-space-time](#usage-since-space-time)<br>[usage-since-zero](#usage-since-zero)<br>[usage-since-edge-space](#usage-since-edge-space) |
| `text:time:the-declaration-would-end-after-the-latest-representable-time-us` | 错误消息（`crates/core/src/time.rs`） | [usage-duration-out-of-range](#usage-duration-out-of-range)<br>[usage-duration-default-out-of-range](#usage-duration-default-out-of-range) |
| `text:time:agent-coexist-lock-wait-ms-is-not-valid-unicode-set-it-to-digits` | 错误消息（`crates/core/src/time.rs`） | [native-invalid-unicode-windows](#native-invalid-unicode-windows)<br>[native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `text:time:agent-coexist-lock-wait-ms-is-larger-than-the-largest-accepted-n` | 错误消息（`crates/core/src/time.rs`） | [usage-env-lock-wait-overflow](#usage-env-lock-wait-overflow) |
| `text:time:agent-coexist-lock-wait-ms-must-be-a-whole-number-of-millisecond` | 错误消息（`crates/core/src/time.rs`） | [usage-env-lock-wait-edge-space](#usage-env-lock-wait-edge-space)<br>[usage-env-lock-wait](#usage-env-lock-wait) |
| `text:time:or-unset-it-to-use-the-default` | 错误消息（`crates/core/src/time.rs`） | [usage-env-lock-wait-edge-space](#usage-env-lock-wait-edge-space) |
| `text:time:unset-it-to-use-the-default` | 错误消息（`crates/core/src/time.rs`） | [usage-env-lock-wait](#usage-env-lock-wait) |
| `text:repair:correct-line-of` | `crates/core/src/error.rs` 的 `fn repair` | [q18-list-glob-syntax](#q18-list-glob-syntax)<br>[q18-read-notification-glob-syntax](#q18-read-notification-glob-syntax)<br>[q18-check-state-glob-syntax](#q18-check-state-glob-syntax)<br>[q18-list-no-ignore-bypass](#q18-list-no-ignore-bypass)<br>[q18-source-git-exclude](#q18-source-git-exclude)<br>[q18-source-gitignore](#q18-source-gitignore)<br>[q18-source-global-excludes](#q18-source-global-excludes)<br>[q18-source-invalid-utf8](#q18-source-invalid-utf8)<br>[q18-state-post-write-scan](#q18-state-post-write-scan)<br>[src-by-hand-linux](#src-by-hand-linux) |
| `text:repair:correct-the-contents-of` | `crates/core/src/error.rs` 的 `fn repair` | [native-linux-gitfile-nonutf8](#native-linux-gitfile-nonutf8)<br>[q18-gitfile-invalid](#q18-gitfile-invalid)<br>[q18-gitfile-target-missing](#q18-gitfile-target-missing)<br>[src-gitdir-file](#src-gitdir-file)<br>[src-gitdir-file-linux](#src-gitdir-file-linux) |
| `text:repair:replace-with-a-regular-file-or-remove-it` | `crates/core/src/error.rs` 的 `fn repair` | [q18-ignore-source-is-directory](#q18-ignore-source-is-directory)<br>[q18-state-scan-not-regular-file](#q18-state-scan-not-regular-file) |
| `text:repair:check-that-can-be-read` | `crates/core/src/error.rs` 的 `fn repair` | [q18-io-control-permission](#q18-io-control-permission)<br>[q18-io-control-state-scan](#q18-io-control-state-scan) |
| `text:repair:shorten-simplify-or-remove-rules-in-until-they-fit-the-pattern-m` | `crates/core/src/error.rs` 的 `fn repair` | [src-rules-too-large](#src-rules-too-large)<br>[src-rules-state-scan](#src-rules-state-scan) |
| `text:fix:then` | `crates/core/src/error.rs` 的 `fn fix` | [native-linux-gitfile-nonutf8](#native-linux-gitfile-nonutf8)<br>[q18-list-glob-syntax](#q18-list-glob-syntax)<br>[q18-read-notification-glob-syntax](#q18-read-notification-glob-syntax)<br>[q18-check-state-glob-syntax](#q18-check-state-glob-syntax)<br>[q18-list-no-ignore-bypass](#q18-list-no-ignore-bypass)<br>[q18-source-git-exclude](#q18-source-git-exclude)<br>[q18-source-gitignore](#q18-source-gitignore)<br>[q18-source-global-excludes](#q18-source-global-excludes)<br>[q18-source-invalid-utf8](#q18-source-invalid-utf8)<br>[q18-gitfile-invalid](#q18-gitfile-invalid)<br>[q18-gitfile-target-missing](#q18-gitfile-target-missing)<br>[q18-state-post-write-scan](#q18-state-post-write-scan)<br>[q18-io-control-permission](#q18-io-control-permission)<br>[q18-io-control-state-scan](#q18-io-control-state-scan)<br>[q18-ignore-source-is-directory](#q18-ignore-source-is-directory)<br>[q18-state-scan-not-regular-file](#q18-state-scan-not-regular-file)<br>[src-rules-too-large](#src-rules-too-large)<br>[src-rules-state-scan](#src-rules-state-scan)<br>[src-gitdir-file](#src-gitdir-file)<br>[src-gitdir-file-linux](#src-gitdir-file-linux) |
| `text:fix:then-by-hand` | `crates/core/src/error.rs` 的 `fn fix` | [src-by-hand-linux](#src-by-hand-linux) |
| `text:manual_retry:by-hand` | `crates/core/src/diag/mod.rs` 的 `fn manual_retry` | [native-nonutf8-linux](#native-nonutf8-linux) |
| `text:io_repair:check-the-permissions-of-and-of-the-directories-on-its-path-or-w` | `crates/cli/src/main.rs` 的 `fn io_repair` | [native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows)<br>[native-delete-pending-windows](#native-delete-pending-windows)<br>[native-permission-file-windows](#native-permission-file-windows) |
| `text:io_repair:check-the-permissions-of-and-of-the-directories-on-its-path` | `crates/cli/src/main.rs` 的 `fn io_repair` | [native-permission-linux](#native-permission-linux)<br>[native-noncopyable-manual-linux](#native-noncopyable-manual-linux) |
| `text:io_repair:free-disk-space-or-quota-where-is-stored` | `crates/cli/src/main.rs` 的 `fn io_repair` | [native-io-classes-linux](#native-io-classes-linux) |
| `text:io_repair:raise-this-shell-s-file-size-limit-ulimit-f-or-move-to-a-file-sy` | `crates/cli/src/main.rs` 的 `fn io_repair` | [native-io-classes-linux](#native-io-classes-linux) |
| `text:io_repair:move-to-a-file-system-that-allows-larger-files` | `crates/cli/src/main.rs` 的 `fn io_repair` | 未覆盖：只在非 Unix（Windows）上遇到 FileTooLarge 时显示。Windows 没有 Unix 那样的单进程文件大小上限（ulimit -f）；协作文件受 16 MiB 上限约束，远小于 NTFS/ReFS 的单文件上限，本机没有需要搬到其他文件系统才能写入的文件系统，无法安全地真实触发。Linux 上的对应分支（ulimit -f）见 [native-io-classes-linux](#native-io-classes-linux)。（native） |
| `text:io_repair:make-the-file-system-that-holds-writable` | `crates/cli/src/main.rs` 的 `fn io_repair` | [native-io-classes-linux](#native-io-classes-linux) |
| `text:io_repair:resolve-the-error-shown-in-reason` | `crates/cli/src/main.rs` 的 `fn io_repair` | [native-io-classes-linux](#native-io-classes-linux) |
| `text:io_report_with_retry_lead:cannot` | `crates/cli/src/main.rs` 的 `fn io_report_with_retry_lead` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir)<br>[native-wait-parent-gone-after-ready](#native-wait-parent-gone-after-ready)<br>[native-wait-parent-replaced-by-file-after-ready-linux](#native-wait-parent-replaced-by-file-after-ready-linux)<br>[native-wait-file-became-dir-after-ready-linux](#native-wait-file-became-dir-after-ready-linux)<br>[native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows)<br>[native-delete-pending-windows](#native-delete-pending-windows)<br>[native-permission-file-windows](#native-permission-file-windows)<br>[native-permission-linux](#native-permission-linux)<br>[native-noncopyable-manual-linux](#native-noncopyable-manual-linux)<br>[native-io-classes-linux](#native-io-classes-linux)<br>[write-post-path-newline-windows](#write-post-path-newline-windows)<br>[write-brief-target-directory](#write-brief-target-directory) |
| `text:io_report_with_retry_lead:then-retry` | `crates/cli/src/main.rs` 的 `fn io_report_with_retry_lead` | [native-wait-io-delete-pending-windows](#native-wait-io-delete-pending-windows)<br>[native-delete-pending-windows](#native-delete-pending-windows)<br>[native-permission-file-windows](#native-permission-file-windows)<br>[native-permission-linux](#native-permission-linux)<br>[native-io-classes-linux](#native-io-classes-linux) |
| `text:io_report_with_retry_lead:then-retry-the-original-command-by-hand` | `crates/cli/src/main.rs` 的 `fn io_report_with_retry_lead` | [native-noncopyable-manual-linux](#native-noncopyable-manual-linux) |
| `text:io_report_with_retry_lead:the-path-contains-a-line-break-or-other-control-character-origin` | `crates/cli/src/main.rs` 的 `fn io_report_with_retry_lead` | [write-post-path-newline-windows](#write-post-path-newline-windows) |
| `text:io_report_with_retry_lead:fix-the-path-then-run-the-corrected-command` | `crates/cli/src/main.rs` 的 `fn io_report_with_retry_lead` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir)<br>[write-brief-target-directory](#write-brief-target-directory) |
| `text:source_report:cannot-scan` | `crates/cli/src/main.rs` 的 `fn source_report` | [native-linux-gitfile-nonutf8](#native-linux-gitfile-nonutf8)<br>[q18-list-glob-syntax](#q18-list-glob-syntax)<br>[q18-read-notification-glob-syntax](#q18-read-notification-glob-syntax)<br>[q18-check-state-glob-syntax](#q18-check-state-glob-syntax)<br>[q18-list-no-ignore-bypass](#q18-list-no-ignore-bypass)<br>[q18-source-git-exclude](#q18-source-git-exclude)<br>[q18-source-gitignore](#q18-source-gitignore)<br>[q18-source-global-excludes](#q18-source-global-excludes)<br>[q18-source-invalid-utf8](#q18-source-invalid-utf8)<br>[q18-gitfile-invalid](#q18-gitfile-invalid)<br>[q18-gitfile-target-missing](#q18-gitfile-target-missing)<br>[q18-io-control-permission](#q18-io-control-permission)<br>[q18-ignore-source-is-directory](#q18-ignore-source-is-directory)<br>[src-rules-too-large](#src-rules-too-large)<br>[src-gitdir-file](#src-gitdir-file)<br>[src-gitdir-file-linux](#src-gitdir-file-linux)<br>[src-by-hand-linux](#src-by-hand-linux) |
| `text:const:FIX_WRITE_UNCONFIRMED:agent-coexist-check` | 常量 `FIX_WRITE_UNCONFIRMED`（`crates/core/src/diag/catalog.rs`），由 `crates/cli/src/main.rs` 的 `fn write_uncertain_report` 组装 | 未覆盖：只在 fsx::commit 的最后一步 sync_data 失败（写入已完整）或恢复原内容本身失败（commit.rs Undo::restore）时显示。[native-io-classes-linux](#native-io-classes-linux) 的夹具都没有走到这两步：只读挂载在打开文件时就失败，提交从未开始；文件大小上限和磁盘已满时 post 失败，文件前后的 SHA-256 和大小（磁盘已满时还有 inode）都相同，也就是文件保持原样，夹具不能说明恢复步骤是否执行过、是否成功。产品里没有故障注入开关，也没有单元测试让 sync_data 或恢复失败（native） |
| `text:const:FIX_WRITE_UNCONFIRMED:run-check-on-it-by-hand` | 常量 `FIX_WRITE_UNCONFIRMED`（`crates/core/src/diag/catalog.rs`），由 `crates/cli/src/main.rs` 的 `fn write_uncertain_report` 组装 | 未覆盖：只在 fsx::commit 的最后一步 sync_data 失败（写入已完整）或恢复原内容本身失败（commit.rs Undo::restore）时显示。[native-io-classes-linux](#native-io-classes-linux) 的夹具都没有走到这两步：只读挂载在打开文件时就失败，提交从未开始；文件大小上限和磁盘已满时 post 失败，文件前后的 SHA-256 和大小（磁盘已满时还有 inode）都相同，也就是文件保持原样，夹具不能说明恢复步骤是否执行过、是否成功。产品里没有故障注入开关，也没有单元测试让 sync_data 或恢复失败，且需要无法写成可复制命令的文件路径（native） |
| `text:const:FIX_WRITE_MIXED:agent-coexist-check` | 常量 `FIX_WRITE_MIXED`（`crates/core/src/diag/catalog.rs`），由 `crates/cli/src/main.rs` 的 `fn write_uncertain_report` 组装 | 未覆盖：只在 fsx::commit 的最后一步 sync_data 失败（写入已完整）或恢复原内容本身失败（commit.rs Undo::restore）时显示。[native-io-classes-linux](#native-io-classes-linux) 的夹具都没有走到这两步：只读挂载在打开文件时就失败，提交从未开始；文件大小上限和磁盘已满时 post 失败，文件前后的 SHA-256 和大小（磁盘已满时还有 inode）都相同，也就是文件保持原样，夹具不能说明恢复步骤是否执行过、是否成功。产品里没有故障注入开关，也没有单元测试让 sync_data 或恢复失败（native） |
| `text:const:FIX_WRITE_MIXED:run-check-on-it-by-hand` | 常量 `FIX_WRITE_MIXED`（`crates/core/src/diag/catalog.rs`），由 `crates/cli/src/main.rs` 的 `fn write_uncertain_report` 组装 | 未覆盖：只在 fsx::commit 的最后一步 sync_data 失败（写入已完整）或恢复原内容本身失败（commit.rs Undo::restore）时显示。[native-io-classes-linux](#native-io-classes-linux) 的夹具都没有走到这两步：只读挂载在打开文件时就失败，提交从未开始；文件大小上限和磁盘已满时 post 失败，文件前后的 SHA-256 和大小（磁盘已满时还有 inode）都相同，也就是文件保持原样，夹具不能说明恢复步骤是否执行过、是否成功。产品里没有故障注入开关，也没有单元测试让 sync_data 或恢复失败，且需要无法写成可复制命令的文件路径（native） |
| `text:value_tip:after-starts-with-so-it-was-read-as-an-option-takes-which-never-` | `crates/cli/src/args.rs` 的 `fn value_tip` | [usage-number-negative](#usage-number-negative)<br>[usage-dash-value-nick](#usage-dash-value-nick)<br>[usage-dash-value-status](#usage-dash-value-status)<br>[usage-dash-value-since](#usage-dash-value-since)<br>[usage-dash-value-duration](#usage-dash-value-duration) |
| `text:value_tip:the-value-of-is-not-valid-unicode-text-retype-it-or-re-encode-th` | `crates/cli/src/args.rs` 的 `fn value_tip` | [native-dash-value-invalid-unicode-windows](#native-dash-value-invalid-unicode-windows)<br>[native-dash-value-invalid-utf8-linux](#native-dash-value-invalid-utf8-linux) |
| `text:value_tip:the-word-after-starts-with-so-it-was-read-as-an-option-to-pass-i` | `crates/cli/src/args.rs` 的 `fn value_tip` | [usage-value-looks-like-flag](#usage-value-looks-like-flag) |
| `text:value_tip:the-word-after-starts-with-so-it-was-read-as-an-option-to-pass-i-2` | `crates/cli/src/args.rs` 的 `fn value_tip` | [usage-value-looks-like-flag-invisible](#usage-value-looks-like-flag-invisible) |
| `text:value_tip:the-word-after-starts-with-so-it-was-read-as-an-option-to-pass-i-3` | `crates/cli/src/args.rs` 的 `fn value_tip` | [usage-value-looks-like-flag-multiline](#usage-value-looks-like-flag-multiline) |
| `text:unknown_option_tip:version-v-is-accepted-only-before-a-command-agent-coexist-versio` | `crates/cli/src/args.rs` 的 `fn unknown_option_tip` | [info-version](#info-version) |
| `text:unknown_option_tip:has-one-so-it-was-read-as-the-short-option-the-long-option-is-wr` | `crates/cli/src/args.rs` 的 `fn unknown_option_tip` | [usage-single-dash-long](#usage-single-dash-long) |
| `text:unknown_option_tip:has-no-option-agent-coexist-help-lists-its-options` | `crates/cli/src/args.rs` 的 `fn unknown_option_tip` | [usage-unknown-flag-invisible](#usage-unknown-flag-invisible)<br>[usage-option-not-on-command](#usage-option-not-on-command)<br>[usage-brief-grammar-multiline-usage](#usage-brief-grammar-multiline-usage) |
| `text:unknown_option_tip:only-if-is-the-itself-write-it-last-after` | `crates/cli/src/args.rs` 的 `fn unknown_option_tip` | [info-help-overrides](#info-help-overrides)<br>[usage-unknown-flag](#usage-unknown-flag) |
| `text:edge_space_fix:remove-the-before-or-after` | `crates/core/src/names.rs` 的 `fn edge_space_fix` | [usage-brief-target-newline](#usage-brief-target-newline)<br>[usage-guide-topic-edge-space](#usage-guide-topic-edge-space)<br>[usage-type-edge-space](#usage-type-edge-space)<br>[usage-nick-edge-space](#usage-nick-edge-space)<br>[usage-status-edge-space](#usage-status-edge-space)<br>[usage-section-edge-space](#usage-section-edge-space)<br>[usage-nick-trailing-cr](#usage-nick-trailing-cr)<br>[usage-status-trailing-cr](#usage-status-trailing-cr)<br>[usage-section-trailing-double-space](#usage-section-trailing-double-space)<br>[usage-duration-edge-space](#usage-duration-edge-space)<br>[usage-since-edge-space](#usage-since-edge-space)<br>[usage-env-now-trailing-cr](#usage-env-now-trailing-cr)<br>[usage-env-lock-wait-edge-space](#usage-env-lock-wait-edge-space)<br>[usage-entry-trailing-cr](#usage-entry-trailing-cr)<br>[usage-entry-trailing-double-space](#usage-entry-trailing-double-space) |
| `text:invalid_unicode_message:the-value-of-is-not-valid-unicode-text-retype-it-or-re-encode-th` | `crates/cli/src/args.rs` 的 `fn invalid_unicode_message` | [native-invalid-unicode-windows](#native-invalid-unicode-windows)<br>[native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `text:invalid_unicode_message:the-guide-topic-is-not-valid-unicode-text-retype-it-or-re-encode` | `crates/cli/src/args.rs` 的 `fn invalid_unicode_message` | [native-invalid-unicode-windows](#native-invalid-unicode-windows)<br>[native-invalid-utf8-linux](#native-invalid-utf8-linux) |
| `text:invalid_unicode_message:an-argument-is-not-valid-unicode-text-retype-it-or-re-encode-the` | `crates/cli/src/args.rs` 的 `fn invalid_unicode_message` | 未覆盖：当前语法下不可达：clap 只对文本值报 InvalidUtf8，而文本值只有选项值（分开写、--long=值、-m值都能认出选项）和 guide 主题（包括 -- 之后）；PATH/DIR 接受任意原生名称；非 Unicode 的命令名、help 主题或选项名分别报 unrecognized subcommand / unexpected argument（args.rs invalid_unicode_subject）（information） |
| `text:plain_value:must-not-contain-control-characters` | `crates/cli/src/args.rs` 的 `fn plain_value` | [usage-plain-control](#usage-plain-control)<br>[usage-nick-trailing-cr](#usage-nick-trailing-cr)<br>[usage-status-trailing-cr](#usage-status-trailing-cr)<br>[usage-entry-trailing-cr](#usage-entry-trailing-cr) |
| `text:plain_value:must-not-contain-two-consecutive-spaces` | `crates/cli/src/args.rs` 的 `fn plain_value` | [usage-plain-double-space](#usage-plain-double-space)<br>[usage-section-trailing-double-space](#usage-section-trailing-double-space)<br>[usage-entry-trailing-double-space](#usage-entry-trailing-double-space) |
| `text:nick_value:write-without-the` | `crates/cli/src/args.rs` 的 `fn nick_value` | [usage-nick-at-sign](#usage-nick-at-sign) |
| `text:regex_reason:the-compiled-pattern-exceeds-the-byte-size-limit-use-a-shorter-o` | `crates/core/src/names.rs` 的 `fn regex_reason` | [usage-brief-regex-size](#usage-brief-regex-size) |
| `text:ensure_parent_dir:not-a-directory` | `crates/core/src/wait.rs` 的 `fn ensure_parent_dir` | [native-wait-start-parent-not-dir](#native-wait-start-parent-not-dir) |
