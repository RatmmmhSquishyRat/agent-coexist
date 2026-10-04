# agent-coexist 运行时输出示例

这里按读者遇到的情况分组，展示 `agent-coexist` 实际打印的文字：帮助与指南、参数错误、各类报告、诊断与恢复建议、等待输出、平台差异，以及它写进协作文件的说明。所有输出都是真实运行捕获的逐字节记录，不是手写的预期输出。每组情况一页：

## 信息命令：guide、help 与版本

[guide：内置指南的五个主题页](runtime-examples/information.md#info-guide-pages) · [顶层帮助](runtime-examples/information.md#info-help-top) · [写命令的帮助](runtime-examples/information.md#info-help-write) · [读命令的帮助](runtime-examples/information.md#info-help-read) · [等待命令的帮助](runtime-examples/information.md#info-help-wait) · [guide 与 help 自身的帮助；help 不接受帮助选项](runtime-examples/information.md#info-help-guide-and-help) · [版本号：--version 与 -V](runtime-examples/information.md#info-version)

## 参数与入口失败

[只输入程序名，没有命令](runtime-examples/arguments.md#arg-no-command) · [命令名拼错](runtime-examples/arguments.md#arg-unknown-command) · [选项不被这个命令接受](runtime-examples/arguments.md#arg-unknown-option) · [缺参数或缺选项值](runtime-examples/arguments.md#arg-missing-arguments) · [固定取值的词写错](runtime-examples/arguments.md#arg-enum-values) · [名称不符合书写规则](runtime-examples/arguments.md#arg-name-values) · [编号不是数字](runtime-examples/arguments.md#arg-numbers) · [时间与时长写错](runtime-examples/arguments.md#arg-time-durations) · [过滤模式与正则写错](runtime-examples/arguments.md#arg-glob-regex) · [缺少配套选项](runtime-examples/arguments.md#arg-target-required) · [多余的参数与冲突的组合](runtime-examples/arguments.md#arg-conflicting-flags) · [环境变量 AGENT_COEXIST_NOW 与 AGENT_COEXIST_LOCK_WAIT_MS](runtime-examples/arguments.md#arg-env-overrides) · [以短横线开头的值](runtime-examples/arguments.md#arg-dash-values) · [看不见的空白混进了值](runtime-examples/arguments.md#arg-invisible-whitespace) · [PATH 给了目录](runtime-examples/arguments.md#arg-path-lookup)

## 查询与写入报告

[list：查看目录下有什么](runtime-examples/reports.md#list-dir) · [list：类型读不出的文件](runtime-examples/reports.md#list-unknown-leaves) · [list：--include 结果为空](runtime-examples/reports.md#list-include-empty) · [check：单份文件的概览](runtime-examples/reports.md#check-previews) · [lookfor-comment：查找评论](runtime-examples/reports.md#lookfor-comment) · [read-brief：阅读导读](runtime-examples/reports.md#read-brief) · [read-notification：收通知](runtime-examples/reports.md#read-notification) · [check-state：查看目录的锁](runtime-examples/reports.md#check-state) · [路径写错时的错误与建议](runtime-examples/reports.md#query-path-errors) · [post：发出与管理评论](runtime-examples/reports.md#write-post) · [profile：建立与维护档案](runtime-examples/reports.md#write-profile) · [subscribe：订阅](runtime-examples/reports.md#write-subscribe) · [brief：维护导读](runtime-examples/reports.md#write-brief) · [state：声明与删除状态](runtime-examples/reports.md#write-state) · [对另一种类型的文件写入](runtime-examples/reports.md#write-type-mismatch)

## 诊断与恢复

[文件读不了、太大或第一行不是标题](runtime-examples/diagnostics.md#diag-file-unusable) · [文件名不能直接抄成标题时的改名修复](runtime-examples/diagnostics.md#diag-h1-rename) · [评论标题坏了](runtime-examples/diagnostics.md#diag-comment-headers) · [评论编号重复或已用尽](runtime-examples/diagnostics.md#diag-numbering) · [帖子里的顺序、回复与游离文字告警](runtime-examples/diagnostics.md#diag-post-warnings) · [BOM、CRLF、重复订阅行与文件名不一致](runtime-examples/diagnostics.md#diag-post-encoding) · [导读条目的问题](runtime-examples/diagnostics.md#diag-brief-entries) · [档案节的问题](runtime-examples/diagnostics.md#diag-profile-sections) · [标题里的控制字符](runtime-examples/diagnostics.md#diag-control-headings) · [state 文件里的冲突、重复、倒置与过期](runtime-examples/diagnostics.md#diag-state-warnings) · [状态内容写在了别的文件名里](runtime-examples/diagnostics.md#diag-state-unselected) · [要删的声明和读不懂的内容在同一行](runtime-examples/diagnostics.md#diag-state-dirty) · [功能锁 locked 挡住了写入](runtime-examples/diagnostics.md#diag-functional-locks) · [ignore 来源里的规则写错](runtime-examples/diagnostics.md#diag-ignore-syntax) · [ignore 来源读不了：不是语法错](runtime-examples/diagnostics.md#diag-ignore-unreadable) · [.git 文件的问题](runtime-examples/diagnostics.md#diag-git-file) · [state 声明已写入，但写后的冲突扫描失败](runtime-examples/diagnostics.md#diag-state-scan-after-write) · [ignore 规则合起来超过上限](runtime-examples/diagnostics.md#diag-ignore-too-large)

## 等待：wait、wait-post、wait-state

[wait-post：就绪行与命中报告](runtime-examples/wait.md#wait-ready-and-hit) · [wait：文件被创建与被删除](runtime-examples/wait.md#wait-file-created-removed) · [wait-state：过滤外的变化不结束，解除命中](runtime-examples/wait.md#wait-state-hit) · [wait：对已有内容的修改报告修改前后](runtime-examples/wait.md#wait-modified-report) · [wait：只改字节不改内容](runtime-examples/wait.md#wait-byte-only) · [在真实终端按 Ctrl-C 取消等待](runtime-examples/wait.md#wait-cancel) · [Windows：采样时遇到锁或独占打开，以退出 3 结束](runtime-examples/wait.md#wait-busy-windows) · [就绪后路径被破坏：三种修复指令](runtime-examples/wait.md#wait-restore-after-ready)

## 平台差异：Windows 与 Linux

[Windows：进程锁与独占打开是两种错误](runtime-examples/platform.md#plat-lock-vs-sharing-windows) · [写入成功但写后核对读不到：warn 而不是 error](runtime-examples/platform.md#plat-write-scan-warning) · [Linux：权限不足的读取失败](runtime-examples/platform.md#plat-denied-io-linux) · [Windows：待删除的文件报告为普通 I/O 失败](runtime-examples/platform.md#plat-denied-delete-pending-windows) · [Linux：只读文件系统上的写入失败](runtime-examples/platform.md#plat-readonly-fs-linux) · [路径写法与大小写：Windows 接受，Linux 是文件名的一部分](runtime-examples/platform.md#plat-operands) · [Linux：非 UTF-8 名字的显示、恢复命令与手工分支](runtime-examples/platform.md#plat-nonutf8-quoting) · [参数或环境变量不是有效 Unicode / UTF-8](runtime-examples/platform.md#plat-args-unicode) · [扫描进度行：只在 stderr 是终端时出现](runtime-examples/platform.md#plat-progress-lines)

## 写入文件的说明文字

[brief：新建空导读](runtime-examples/files.md#brief-new-file) · [state：新建状态文件](runtime-examples/files.md#state-new-file)

另见[命令参考](usage.md)与[文件格式说明](file-format.md)。
