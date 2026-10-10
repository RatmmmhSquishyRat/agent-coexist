//! Spec §9 diagnostic catalog: the 50 spec-table diagnostics plus the two
//! reason-level additions of the 2026-09-25 repair contract (`E-NAME`,
//! `E-STATE-DIRTY`) appended after the spec rows so every spec row keeps
//! its §9 table position.
//!
//! `{placeholder}` sequences are replaced at runtime. Each fix explicitly
//! distinguishes file content/prose from command operands. Command operands
//! use native `*_arg` keys, separate from human display keys; attached option
//! values preserve the CLI boundary after shell parsing.

/// One §9 catalog row; message/why/fix are literal template text.
pub(crate) struct Template {
    pub(crate) id: &'static str,
    pub(crate) message: &'static str,
    pub(crate) why: &'static str,
    pub(crate) fix: &'static str,
    /// Runtime values need shell-operand quoting rather than plain text.
    /// Already-quoted `{argv}` embeds unchanged in either mode.
    pub(crate) command_fix: bool,
}

/// All 52 diagnostics: 50 in §9 table order (this order is the same-line
/// tiebreak), then the two repair-contract reason errors (`E-NAME`,
/// `E-STATE-DIRTY`), which are reason-form only and never anchored rows.
pub(crate) const CATALOG: &[Template] = &[
    Template {
        id: "E-UTF8",
        command_fix: false,
        message: r#"file is not valid UTF-8"#,
        why: r#"the CLI cannot locate lines or headings in it"#,
        fix: r#"re-save the file as UTF-8"#,
    },
    Template {
        id: "E-TOO-LARGE",
        command_fix: false,
        message: r#"file is larger than {limit} MiB"#,
        why: r#"files this size are never collaboration files; parsing it would stall every list and notification scan"#,
        fix: r#"move the large content elsewhere and keep only a short post or brief here"#,
    },
    Template {
        id: "E-H1",
        command_fix: false,
        message: r##"first line is "{line}", not "# <name>.<type>""##,
        why: r#"the H1 is the only place the file type is recorded; without it nothing else can be parsed"#,
        fix: r##"set line 1 to "# {stem}.post" (or .brief / .agent-profile), or "# .state" for a state file"##,
    },
    Template {
        id: "E-FENCE",
        command_fix: false,
        message: r#"code fence opened at L{n} is never closed"#,
        why: r#"everything after it is treated as fence content, so no later comment, entry or section can be found"#,
        fix: r#"add a closing line of {ticks} backticks where the block should end"#,
    },
    Template {
        id: "E-HEAD",
        command_fix: false,
        message: r#""{line}" looks like a comment header but is malformed"#,
        why: r###"comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous"###,
        fix: r###"rewrite the header as "{header}""###,
    },
    Template {
        id: "E-SEQ-DUP",
        command_fix: false,
        message: r#"comment number #{n} appears twice"#,
        why: r#"@#{n} and --edit {n} cannot tell the two apart"#,
        fix: r#"renumber the later one to #{next} if no reply in this post already names @#{next}, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply in this post already names. Change any @#{n} that meant it to that number"#,
    },
    Template {
        id: "E-ENTRY-DUP",
        command_fix: false,
        message: r#"entry "{title}" appears twice"#,
        why: r#"--entry {title} cannot tell the two apart"#,
        fix: r#"merge the two sections or retitle one of them"#,
    },
    Template {
        id: "E-ENTRY-PATH",
        command_fix: true,
        message: r#"entry "{title}" has no "- path:" line"#,
        why: r#"without a path there is nothing to hash or read; the entry cannot be verified"#,
        fix: r#"choose a file you know is the right target, review that file and the note, and update the note if needed. Then give the entry its path; this command also records the target's current hash: agent-coexist brief {path_arg} --target <file> --entry={title}"#,
    },
    Template {
        id: "E-ENTRY-HASH",
        command_fix: true,
        message: r#"entry "{title}" has no valid "- hash:" line"#,
        why: r#"the hash is how check tells fresh from shifted; without it drift is invisible"#,
        fix: r#"review the target and the note, and update the note if needed. Then refresh the entry's hash; this command records the target's current hash: agent-coexist brief {path_arg} --entry={title} --refresh"#,
    },
    Template {
        id: "E-ENTRY-REGEX",
        command_fix: true,
        message: r#"entry "{title}" regex does not compile: {err}"#,
        why: r#"read-brief and check use the regex to find the lines the note is about"#,
        fix: r#"choose a file you know is the right target, review that file and the note, and update the note if needed. Then give the entry its path and a regex that compiles and matches it; this command also records the target's current hash: agent-coexist brief {path_arg} --target=<file> --entry={title} --regex '<pattern>'"#,
    },
    Template {
        id: "E-ENTRY-TARGET",
        command_fix: true,
        message: r#"entry "{title}" target {target} cannot be read: {err}"#,
        why: r#"the target exists but its bytes are not readable, so neither hash nor regex can be checked"#,
        fix: r#"the path must name a readable file: fix its permissions and run check again without changing the entry. Otherwise choose a file you know is the right target: review that file and the note, update the note if needed, and point the entry at it; this command also records the target's current hash: agent-coexist brief {path_arg} --target <file> --entry={title}"#,
    },
    Template {
        id: "E-SECTION-DUP",
        command_fix: false,
        message: r#"section "{name}" appears twice"#,
        why: r#"--section {name} cannot tell the two apart"#,
        fix: r#"merge the two sections or rename one"#,
    },
    Template {
        id: "E-STATUS-DUP",
        command_fix: false,
        message: r#"status "{status}" appears twice"#,
        why: r#"entries for one status must share one section; state writes are refused until the duplicates are merged"#,
        fix: r###"merge the two "## {status}" sections into one"###,
    },
    Template {
        id: "E-TYPE",
        command_fix: true,
        message: r#"file type is {actual}, not {expected}"#,
        why: r#"each command works on one file type; the H1 decides which"#,
        fix: r#"this command needs a file of type {expected}; rerun it with such a path; files of that type under this directory: agent-coexist list {dir_arg} --type={expected}"#,
    },
    Template {
        id: "E-LOCKED",
        command_fix: false,
        message: r#"file is locked by another process"#,
        why: r#"the required file lock, stable identity, or stable contents could not be obtained or maintained"#,
        fix: r#"retry shortly: {argv}"#,
    },
    Template {
        id: "E-INUSE",
        command_fix: false,
        message: r#"file is in use by another program"#,
        why: r#"an editor or another tool holds it open without sharing"#,
        fix: r#"close the other program, then retry: {argv}"#,
    },
    Template {
        id: "E-FLOCKED",
        command_fix: true,
        message: r#"{dir} is locked by {holder} until {until}"#,
        why: r#"{holder} asked that nothing under {dir} be changed; profile and brief writes fast-fail on such locks"#,
        fix: r#"if the lock is yours, add --as={holder} before any -- separator. Otherwise wait for it to expire or ask its holder to release it, then retry: {argv}"#,
    },
    Template {
        id: "E-FILE-MISSING",
        command_fix: true,
        message: r#"{file} does not exist"#,
        why: r#"read commands and forms that only act on existing content (--edit, --cancel, --remove, --refresh, note-only -m) never create files, so a typo shows up here instead of as an empty result"#,
        fix: r#"correct the path and rerun (this command never creates the file); collaboration files under that directory: agent-coexist list {dir_arg}"#,
    },
    Template {
        id: "E-DIR-MISSING",
        command_fix: true,
        message: r#"directory {dir} does not exist"#,
        why: r#"an empty result for a missing directory would look like "nothing happened""#,
        fix: r#"correct the directory and rerun; collaboration files under its parent: agent-coexist list {parent_arg}"#,
    },
    Template {
        id: "E-NOT-DIR",
        command_fix: true,
        message: r#"{path} is not a directory"#,
        why: r#"state and the scanning commands describe directories, not files"#,
        fix: r#"rerun on its directory with your other options: agent-coexist {cmd} {parent_arg} <same flags>"#,
    },
    Template {
        id: "E-TARGET-MISSING",
        command_fix: false,
        message: r#"target {target} does not exist; looked for {resolved}"#,
        why: r#"a brief entry must point at something that can be hashed now"#,
        fix: r#"the target path is relative to the brief's directory; correct it and rerun, or create the target, then retry: {argv}"#,
    },
    Template {
        id: "E-ENTRY-NOTFOUND",
        command_fix: true,
        message: r#"no entry titled "{title}""#,
        why: r#"titles are how brief entries are addressed; the command will not guess which one you meant"#,
        fix: r#"pick an existing title from: agent-coexist check {path_arg}"#,
    },
    Template {
        id: "E-SECTION-NOTFOUND",
        command_fix: true,
        message: r#"no section named "{name}""#,
        why: r#"only existing sections can be removed; adding one takes -m"#,
        fix: r#"pick an existing name from: agent-coexist check {path_arg}"#,
    },
    Template {
        id: "E-STATE-NOTFOUND",
        command_fix: true,
        message: r#"{nick} has no entry in {dir}"#,
        why: r#"no declaration matched the requested nick and status selection, so nothing was changed"#,
        fix: r#"check which nicks hold entries here, then rerun with the right nick or directory: agent-coexist check-state {dir_arg}"#,
    },
    Template {
        id: "E-EDIT-AUTHOR",
        command_fix: true,
        message: r#"comment #{n} was written by {author}, not {nick}"#,
        why: r#"comments stay attributable; only the author may rewrite one"#,
        fix: r#"reply instead: agent-coexist post {path_arg} --as={nick} -m '@#{n} <your reply>'"#,
    },
    Template {
        id: "E-COMMENT-NOTFOUND",
        command_fix: true,
        message: r#"comment #{n} does not exist"#,
        why: r#"--edit rewrites an existing comment; the highest number in this post is #{max}"#,
        fix: r#"find the number of the comment to edit, then rerun --edit with it: agent-coexist lookfor-comment {path_arg} --by={nick}"#,
    },
    Template {
        id: "E-ENTRY-TITLE-TAKEN",
        command_fix: false,
        message: r#"entry "{title}" already points at {existing}, not {target}"#,
        why: r#"the default title is the target's file name; two different files with the same name would silently overwrite each other"#,
        fix: r#"give the entry its own title: rerun the original command by hand with --entry '<title>' added before any -- separator, keeping all its other options; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly"#,
    },
    Template {
        id: "E-SEQ-EXHAUSTED",
        command_fix: true,
        message: r#"comment numbering is exhausted: #{max} is the highest number a header can carry"#,
        why: r#"the header grammar stops at #{max}; nothing was written"#,
        fix: r#"start a new post: agent-coexist post <new path> --as={nick} -m '<text>'"#,
    },
    Template {
        id: "W-BOM",
        command_fix: false,
        message: r#"file starts with a UTF-8 BOM"#,
        why: r#"the BOM is skipped when reading; the CLI leaves it in place when adding or changing content"#,
        fix: r#"re-save the file without BOM"#,
    },
    Template {
        id: "W-CRLF",
        command_fix: false,
        message: r#"file uses CRLF line endings"#,
        why: r#"lines the CLI adds use LF, so the file will end up mixed"#,
        fix: r#"convert the file to LF"#,
    },
    Template {
        id: "W-NAME",
        command_fix: false,
        message: r#"file name "{stem}" does not match the H1 name "{name}""#,
        why: r#"list shows the file name, the file says another name; readers will be confused"#,
        fix: r##"rename the file to {name}.agent-besties.md if no file has that name yet (never overwrite an existing file), or set line 1 to "# {stem}.{type}""##,
    },
    Template {
        id: "W-NAME-STATE",
        command_fix: false,
        message: r#"state file is named "{stem}.agent-besties.md", not "state.agent-besties.md""#,
        why: r#"check-state and state look only for state.agent-besties.md (case-insensitive on Windows/macOS); other names are ignored"#,
        fix: r##"inspect this file and any state.agent-besties.md already in this directory. Resolve intended declarations before choosing which file should use that name, and never overwrite an existing state file (keep line 1 as "# .state")"##,
    },
    Template {
        id: "W-STRAY",
        command_fix: false,
        message: r#"text outside any fence at L{n} is not part of the record"#,
        why: r#"this text stays in the file but is not included in CLI reports"#,
        fix: r#"the writer may move this text into a fenced body or delete it; others must ask the writer, not infer who wrote it from a nearby heading"#,
    },
    Template {
        id: "W-BODY",
        command_fix: true,
        message: r#"{unit} at L{n} has no fenced body"#,
        why: r#"there is nothing to show as its content"#,
        fix: r#"{set_body}"#,
    },
    Template {
        id: "W-SUB-DUP",
        command_fix: false,
        message: r#"subscribers line repeated or contains duplicates"#,
        why: r#"only the first line counts and duplicates are ignored; the extra text is noise for readers"#,
        fix: r#"keep the first "- subscribers:" line, each nick once; delete later lines without merging them. Nicks found only there are not subscribed; those agents decide whether to subscribe"#,
    },
    Template {
        id: "W-SEQ-GAP",
        command_fix: false,
        message: r#"comment numbers jump from #{a} to #{b}"#,
        why: r#"no comment in this post has number #{m}, so @#{m} cannot be resolved; that comment may have been deleted or renumbered by hand, or a new comment was written with the highest number plus one after numbers were already out of order"#,
        fix: r#"leave the numbering as is; other files may already reference #{b}"#,
    },
    Template {
        id: "W-SEQ-ORDER",
        command_fix: false,
        message: r#"comment #{n} appears after #{m}"#,
        why: r#"readers assume comments are in order; @#n still resolves"#,
        fix: r#"move the section so numbers increase, or leave it if the order is intentional"#,
    },
    Template {
        id: "W-REPLY-DANGLING",
        command_fix: false,
        message: r#"comment #{n} replies to @#{m} which does not exist"#,
        why: r#"the reply cannot be followed to its target"#,
        fix: r#"ask the author of #{n} which comment was meant"#,
    },
    Template {
        id: "W-ENTRY-ABS",
        command_fix: true,
        message: r#"entry "{title}" uses an absolute path"#,
        why: r#"an absolute path stays fixed when the brief moves; in another checkout it may read a file outside that checkout or point to a missing file"#,
        fix: r#"choose a file you know is the right target, review that file and the note, and update the note if needed. Then point the entry at a path relative to the brief's directory; this command also records the target's current hash: agent-coexist brief {path_arg} --target <relative path> --entry={title}"#,
    },
    Template {
        id: "W-ENTRY-KEY",
        command_fix: false,
        message: r#"entry "{title}" has unknown attribute "{key}""#,
        why: r#"the line is kept but has no meaning to the CLI"#,
        fix: r#"remove the line, or move its content into the note"#,
    },
    Template {
        id: "W-ENTRY-SHIFTED",
        command_fix: true,
        message: r#"entry "{title}" is shifted: {target} changed since its hash was taken"#,
        why: r#"the note may describe code that no longer looks like this"#,
        fix: r#"review the target and the note, update the note if it no longer describes the target, then refresh the entry's hash; this command records the target's current hash: agent-coexist brief {path_arg} --entry={title} --refresh"#,
    },
    Template {
        id: "W-ENTRY-STALE",
        command_fix: true,
        message: r#"entry "{title}" is stale: {reason}"#,
        why: r#"the note points at something that is gone or no longer matches its regex"#,
        fix: r#"review the target and the note, update the note if it no longer describes the target, then give the entry a regex that matches it; this command also records the target's current hash: agent-coexist brief {path_arg} --target={target_arg} --entry={title} --regex '<pattern>'"#,
    },
    Template {
        id: "W-PROFILE-EMPTY",
        command_fix: true,
        message: r#"profile has no sections"#,
        why: r#"nothing tells other agents who this is or what they own"#,
        fix: r#"agent-coexist profile {path_arg} --section role -m '<what you do>'"#,
    },
    Template {
        id: "W-SECTION-NAME",
        command_fix: false,
        message: r#"section "{name}" at L{n} is not a valid section name"#,
        why: r#"--section cannot address it; allowed: starts with a letter, digit or _, then letters, digits, _ and -, at most 64 characters"#,
        fix: r###"rename the heading, e.g. "## {suggested}""###,
    },
    Template {
        id: "W-STATUS-NAME",
        command_fix: false,
        message: r#"status "{name}" at L{n} is not a valid status name"#,
        why: r#"entries under it are shown but never treated as locked or observing; allowed: starts with a lowercase letter, then lowercase letters, digits and -, at most 32 characters"#,
        fix: r###"choose a valid, unused status name and rename the heading by hand; "## {lowercased}" is only an example, not a guess at your intended name. Then run check again"###,
    },
    Template {
        id: "W-STATE-ENTRY",
        command_fix: false,
        message: r#""{fragment}" under {status} is not "<nick> <start>..<until>""#,
        why: r#"the fragment is kept but ignored by conflict checks and cleanup"#,
        fix: r#"this file does not record who wrote this fragment, even when it shares a line with another agent's entries; whoever wrote it rewrites it as "<nick> <start>..<until>" or deletes it; anyone else asks who wrote it before changing it"#,
    },
    Template {
        id: "W-STATE-DUP",
        command_fix: true,
        message: r#"{nick} appears twice under {status}"#,
        why: r#"the latest end time wins. The next state write that rewrites the {status} section (a declaration, removal, or expiry cleanup in it) merges them into one, unless either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"optional; to merge now, that agent declares again to replace both with one entry from now (add --for to choose its length), but only when both entries are on clean lines. When either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is: inspect the file, {nick} deletes its own entries from each such line by hand keeping other content, then declares again to merge remaining duplicates or start one entry from now: agent-coexist state {dir_arg} --as={nick} --status={status}"#,
    },
    Template {
        id: "W-STATE-RANGE",
        command_fix: true,
        message: r#"{nick} under {status} ends before it starts"#,
        why: r#"treated as expired: it never counts as a lock; the next state write removes it unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"if still needed, that agent declares it again: agent-coexist state {dir_arg} --as={nick} --status={status}"#,
    },
    Template {
        id: "W-STATE-CONFLICT",
        command_fix: true,
        message: r#"locked is held by both {a} and {b}"#,
        why: r#"two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues"#,
        fix: r#"agree who continues, then the other runs the removal; it is refused with E-STATE-DIRTY when that declaration's line also has unparseable fragments (W-STATE-ENTRY), and that error's fix names the by-hand deletion: agent-coexist state {dir_arg} --remove <nick> --status=locked"#,
    },
    Template {
        id: "W-STATE-EXPIRED",
        command_fix: true,
        message: r#"{nick} under {status} expired at {until}"#,
        why: r#"expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"if still needed, that agent renews it: agent-coexist state {dir_arg} --as={nick} --status={status}"#,
    },
    Template {
        id: "E-NAME",
        command_fix: false,
        message: r#"file name "{name}" is not a valid name"#,
        why: r##"the H1 is generated from the file name; use 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-""##,
        fix: r##"nothing was created; choose a name of 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-", and run the same command with that name"##,
    },
    Template {
        id: "E-STATE-DIRTY",
        command_fix: false,
        message: r#"{nick} is still declared on line {line} of the state file in {dir}, mixed with unparseable fragments"#,
        why: r#"removal would rewrite that whole line and could drop the fragments it cannot parse, so nothing is written"#,
        fix: r#"either delete {nick}'s declaration from line {line} by hand, which is the removal itself (re-run then only if {nick} has more entries this command should remove), or have whoever wrote that line's unparseable fragments make them parseable, then re-run: {argv}"#,
    },
];

/// A status-filtered removal only establishes absence within that status,
/// not absence from every declaration in the directory.
pub(crate) const MESSAGE_ESTATE_NOTFOUND_STATUS: &str =
    r#"{nick} has no entry under status "{status}" in {dir}"#;

/// E-FILE-MISSING when the PATH without its suffix names an existing
/// directory: say so, and point at the files inside it (`dir_arg` is that
/// directory). `{given_dir}` is display only; the fix never embeds it.
pub(crate) const MESSAGE_EFILEMISSING_DIR: &str = r#"{file} does not exist; {given_dir} is a directory, and this command needs one collaboration file"#;

/// See [`MESSAGE_EFILEMISSING_DIR`].
pub(crate) const FIX_EFILEMISSING_DIR: &str = r#"rerun with the path of one collaboration file (the .agent-besties.md suffix may be omitted); collaboration files in that directory: agent-coexist list {dir_arg}"#;

/// E-STATE-NOTFOUND fix when the removal named `--status`: keep that
/// selection, because a bare `--remove` would drop every status.
pub(crate) const FIX_ESTATE_NOTFOUND_STATUS: &str = r#"check which statuses hold this nick, then rerun with the right --status (without --status, --remove deletes every status of the nick): agent-coexist check-state {dir_arg}"#;

/// E-TYPE fix when the command needs a state file: the reserved name is
/// occupied by another type, so no state command can succeed until that
/// file is renamed. Like every rename recipe, it never overwrites another
/// file. After the rename, line 1 must use that same name and keep this
/// file's type (`{actual}`), matching the E-H1 reserved-name recipe. The
/// retry repeats the original declaration or wait.
pub(crate) const FIX_ETYPE_RESERVED: &str = r##"state.agent-besties.md in this directory has type {actual}, so it cannot hold state. Rename that file to <name>.agent-besties.md with an unused valid name (never overwrite an existing file), set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name, then retry: {argv}"##;

/// E-TYPE fix when the failed command is a write form that creates a
/// missing file (`post` without `--edit`, `profile` without `--remove`,
/// `brief` Bare / SetDescription / Target). Query, edit, remove, refresh,
/// note-only, subscribe, cancel and wait forms keep the catalog row: they
/// cannot finish the original task on a new unused path. The list stays
/// read-only.
pub(crate) const FIX_ETYPE_CREATE: &str = r#"this command needs a file of type {expected}; rerun it with such a path, or with a new path that no file uses yet to create one; files of that type under this directory: agent-coexist list {dir_arg} --type={expected}"#;

/// E-NOT-DIR fix when the directory operand is exactly one word of the
/// original command line: `{retry}` is that line with the operand
/// replaced by its parent, so no other option is lost.
pub(crate) const FIX_ENOTDIR_RETRY: &str = "rerun on its directory: {retry}";

/// E-ENTRY-TITLE-TAKEN fix when the original command line is printable:
/// `{retry}` is that line with `--entry '<title>'` added before any `--`
/// separator, so `--as`, `-m`, `--regex` and every other option survive.
pub(crate) const FIX_EENTRYTITLETAKEN_RETRY: &str = "give the entry its own title: {retry}";

/// E-DIR-MISSING fix for commands that need the directory to exist before
/// they can act (`state` and the waits). They never create directories, and
/// `list` would not show a directory without collaboration files.
pub(crate) const FIX_EDIRMISSING_CREATE: &str = r#"if the directory name is wrong, rerun with the correct one; if it is new, create it first (this command never creates directories), then retry: {argv}"#;

/// E-DIR-MISSING when the lexical parent is missing too (`missing_dir`
/// selects this variant for `list` / `read-notification` / `check-state`):
/// `{parent_arg}` is the nearest existing ancestor, never a missing
/// directory. `state` and the waits keep [`FIX_EDIRMISSING_CREATE`].
pub(crate) const FIX_EDIRMISSING_ANCESTOR: &str = r#"correct the directory and rerun; its parent does not exist either; collaboration files under the nearest existing directory: agent-coexist list {parent_arg}"#;

/// W-ENTRY-STALE fix when the target is gone: do not assume it was
/// moved. The reader names a file they know is the right target, or
/// removes or rewrites the entry if it no longer exists. A guessed
/// replacement is not a recovery. The entry's regex, if any, is kept
/// rather than replaced. Setting the target also records its current
/// hash, so the review of target and note that the shifted/stale
/// siblings require comes first, updating the note when it no longer
/// describes the target, before the note is silently re-anchored.
pub(crate) const FIX_WSTALE_TARGET_MISSING: &str = r#"choose a file you know is the right target, review that file and the note, update the note if needed, then point the entry at it; if it no longer exists, remove or rewrite the entry. This command also records the target's current hash: agent-coexist brief {path_arg} --target <file> --entry={title}"#;

/// E-ENTRY-REGEX fix when the entry has a path: the regex verdict
/// precedes the hash comparison, so drift since the stored hash is
/// invisible while the regex does not compile, and rerunning the
/// command records the target's current hash. The review prefix is the
/// one W-ENTRY-SHIFTED and W-ENTRY-STALE carry. The base catalog row
/// is the path-less branch: setting a target also records a hash.
pub(crate) const FIX_EENTRYREGEX_REVIEW: &str = r#"review the target and the note, and update the note if it no longer describes the target. Then give the entry a regex that compiles and matches it; this command also records the target's current hash: agent-coexist brief {path_arg} --target={target_arg} --entry={title} --regex '<pattern>'"#;

/// A directory-level state recipe is safe only for the inspected file
/// selected by the existing reserved-state lookup. Failed selection is
/// not authority to mutate a possibly different file, so the fix gates
/// every declaration change behind two steps instead: settle which file
/// is this directory's state.agent-besties.md (a W-NAME-STATE row, when
/// one is shown, carries the matching inspection advice), then re-check
/// — and only the declaring agent ever changes, renews or removes its
/// own entry by hand.
pub(crate) const FIX_UNSELECTED_STATE: &str =
    "this file is not confirmed as this directory's state.agent-besties.md; settle which file should be, without overwriting another state file, then run check on this file again before editing declarations by hand. Only the declaring agent may change, renew or remove its entry";

/// W-STATE-DUP why under the same unselected gate as
/// [`FIX_UNSELECTED_STATE`]: the catalog row's merge happens on the next
/// state write, and no state command writes this file, so the duplicate
/// stays until someone edits the file by hand.
pub(crate) const WHY_WSTATEDUP_UNSELECTED: &str = r#"the latest end time wins; no state write merges them, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md"#;

/// W-STATE-RANGE why under the same unselected gate as
/// [`FIX_UNSELECTED_STATE`]: the removal the catalog row promises cannot
/// happen, because no state command writes this file.
pub(crate) const WHY_WSTATERANGE_UNSELECTED: &str = r#"treated as expired: it never counts as a lock; no state write removes it, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md"#;

/// W-STATE-EXPIRED why under the same unselected gate as
/// [`FIX_UNSELECTED_STATE`]: same non-removal as
/// [`WHY_WSTATERANGE_UNSELECTED`], for an entry that has ended.
pub(crate) const WHY_WSTATEEXPIRED_UNSELECTED: &str = r#"expired entries are ignored by conflict checks; no state write removes them, because state commands do not write this file until it is confirmed as this directory's state.agent-besties.md"#;

/// W-STRAY in a post: a fenced body remains a possible destination,
/// but a comment may be changed only by that comment's author.
pub(crate) const FIX_WSTRAY_POST: &str = r#"the writer may move this text into a fenced body or delete it. Others must ask the writer, not infer who wrote it from a nearby heading; only a comment's author edits that comment"#;

/// W-STRAY in a state file's preamble: a reader note on a '> ' line and
/// a misplaced entry under its status are both possible destinations.
pub(crate) const FIX_WSTRAY_STATE: &str = r#"the writer may keep it as a reader note on a line starting with '> ', put a misplaced state entry under its status, or delete the text. Others must ask the writer, not infer who wrote it from a nearby heading"#;

/// A concrete section repair must be unique across stored headings and
/// all suggestions emitted for the same profile.
pub(crate) const FIX_SECTION_NAME_UNIQUE: &str =
    "rename this heading by hand to a valid section name distinct from every other section and any other planned heading repair, then run check again";

/// Replace the entire recipe when its native operands cannot be emitted
/// losslessly in one POSIX UTF-8 fix record. Never substitute this as a path.
pub const FIX_NATIVE_OPERAND: &str =
    "repair the original file or retry the original command by hand; no copyable command can be printed because a path or argument contains a line break or characters this text cannot reproduce exactly";

/// A command recipe whose operands cannot all be printed ready to copy:
/// the command keeps its subcommand, options and printable operands, and
/// each unprintable value is a `<path of …>` marker the reader types in.
/// `{advice}` is empty or the template's own advice followed by `: `.
pub(crate) const FIX_MANUAL_COMMAND: &str = "{advice}run this command by hand, typing {markers} yourself, because no copyable command can be printed when a path or argument contains a line break or characters this text cannot reproduce exactly: {command}";

/// Final row of a write refused by structural errors (§2.7.4, §8.3.2):
/// the report repeats check's rows, so this row says that nothing was
/// written and which rows block the write. `{blocking}` lists `ID at L<n>`.
pub const MESSAGE_WRITE_REFUSED: &str = "nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix {blocking} first, as the fix rows above say, then run this command again";

/// [`MESSAGE_WRITE_REFUSED`] when the report also shows rows that do not
/// block writes.
pub const MESSAGE_WRITE_REFUSED_MIXED: &str = "nothing was written: writes are refused while the file has errors that make its structure ambiguous; fix {blocking} first, as the fix rows above say, then run this command again; the other rows above do not block this write";

/// A control-bearing heading cannot be passed through the CLI's existing
/// input grammar. Never substitute a sanitized sibling as its identity.
pub(crate) const FIX_CONTROL_HEADING: &str =
    "edit the heading by hand to remove control characters and give it a valid, distinct title or name, then run check again";

/// E-STATUS-DUP whose status holds control characters: the repair is still
/// the merge of [`CATALOG`]'s row, done by hand after removing the controls,
/// not [`FIX_CONTROL_HEADING`]'s distinct name, which would split one status
/// into two. `{status}` is the display-sanitized title.
pub(crate) const FIX_ESTATUSDUP_CONTROL: &str = r###"edit the two "## {status}" headings by hand: remove their control characters and merge the two sections into one, together with any section that already uses the cleaned name, then run check again"###;

/// A stored status may be readable but not addressable by `--status`.
/// Repair its heading first; never redirect an operation to a normalized sibling.
pub(crate) const FIX_INVALID_STATUS: &str =
    "rename the status heading by hand to a valid, distinct status name, then run check again before renewing or changing its declarations";

/// E-FLOCKED when the command already has `--as`: holders exclude that
/// nick, so the lock is the caller's only if `--as` named the wrong nick;
/// a second `--as` would be a usage error, so the fix says to replace it.
pub(crate) const FIX_EFLOCKED_AS: &str = r#"if the lock is yours and the --as in this command names someone else, rerun it with --as={holder} in place of that --as. Otherwise wait for it to expire or ask its holder to release it, then retry: {argv}"#;

/// W-STATUS-NAME when the same raw status title heads more than one
/// section (E-STATUS-DUP reports the later copies): the copies are one
/// status, so the repair is the merge under one valid name, never one
/// distinct name per copy, which would split it into two statuses.
pub(crate) const FIX_WSTATUSNAME_REPEATED: &str = r###"merge this status's sections by hand under one valid name you choose, including any section already using it; "## {lowercased}" is only an example. Keep one status, then run check again"###;

/// 2026-09-25 repair: E-H1 fix when the file stem is not a valid NAME.
/// The base fix suggests `# {stem}.<type>`, which would re-trigger E-H1
/// the moment it is copied in, so this variant suggests any valid name
/// instead. It carries no command argument, so it substitutes as plain
/// text (§8.3.4 quoting stays on command-form fixes). The optional rename
/// carries the same never-overwrite condition as every rename recipe.
pub(crate) const FIX_EH1_BAD_STEM: &str = r##"set line 1 to "# <name>.post" (or .brief / .agent-profile) using 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-"; use "# .state" for a state file. Renaming the file to <name>.agent-besties.md as well keeps the name in agreement, if no file has that name yet (never overwrite an existing file)"##;

/// E-H1 fix when the file has the reserved state file name: the name says
/// this is the directory's state file, so `# .state` comes first. A post,
/// brief or profile H1 under this name would make every state command
/// refuse the file (E-TYPE) and lock scans skip it, so another type is
/// offered only together with moving the file off the reserved name.
pub(crate) const FIX_EH1_STATE_NAME: &str = r##"set line 1 to "# .state" if this file holds state declarations, because its name is reserved for this directory's state file. If it holds a post, brief or profile instead, rename it first to <name>.agent-besties.md with an unused valid name (never overwrite an existing file), then set line 1 to "# <name>.post" (or .brief / .agent-profile) with that same name"##;

/// 2026-09-25 repair: W-NAME fix when the file stem is not a valid NAME.
/// The base template also offers `set line 1 to "# {stem}.{type}"`, which
/// would be an invalid H1; this variant keeps only the other recovery:
/// rename the file to the H1 name, which parsing already proved to be a
/// valid NAME. Plain text, like the E-H1 variant.
pub(crate) const FIX_WNAME_BAD_STEM: &str = r#"rename the file to {name}.agent-besties.md if no file has that name yet (never overwrite an existing file). The file stem is not a valid name, so no valid H1 can be built from it"#;

/// §9 supplement 3: E-HEAD fix when no number above the highest legal one
/// (`{max}`) is left. The allocator never wraps or reuses numbers, so the
/// fix does not print one: the reader picks an unused lower number by hand,
/// which keeps the comment in place with its author, time and text.
/// A fresh number must not capture an existing `@#n` reply. Restoring a
/// known original number is a printed branch of this fix (and of
/// [`FIX_EHEAD_UNNUMBERED`]): that number may already be named by a reply.
/// `{header}` shows `<n>` for that number; `{fill}` is empty or
/// [`FIX_EXHAUSTED_FILL`]. [`FIX_EXHAUSTED_TAIL`] follows.
pub(crate) const FIX_EHEAD_EXHAUSTED: &str = r##"no number above #{max} is available. Rewrite the header by hand as "{header}"{fill}, keeping the comment's position, author, time and text. For <n>, restore its known original number if no header uses it or receives it in another repair; replies then stay unchanged. Otherwise choose a lower number unused by headers, other repairs and @#n replies in this post, then retarget replies meant for this comment"##;

/// `{fill}` of [`FIX_EHEAD_EXHAUSTED`] when the header still shows
/// `<nick>` or `<time>`.
pub(crate) const FIX_EXHAUSTED_FILL: &str =
    ", filling each <nick> or <time> with the original author or first-written time";

/// Appended to both exhausted repairs: a lower number is only an ordering
/// warning, and it makes no room for appends, which E-SEQ-EXHAUSTED covers.
pub(crate) const FIX_EXHAUSTED_TAIL: &str = "; W-SEQ-ORDER or W-SEQ-GAP may remain but do not block writes; this frees no append number, so use a new post when E-SEQ-EXHAUSTED stops an append";

/// E-HEAD fix when the suggested header still has reader-filled parts;
/// the parts it already shows are the header's own and stay as printed.
pub(crate) const FIX_EHEAD_FILL: &str = r###"rewrite the header as "{header}", replacing only each <nick> or <time> placeholder with the comment's original author or the time it was first written"###;

/// Appended to the E-HEAD fix when the repair gives the comment another
/// number than its header shows: replies by the old number would silently
/// point at whatever comment keeps it. Choose `#{n}` before rewriting if
/// no reply already names it; otherwise another number that no header
/// carries, that no other repair in this report gives a header, and that
/// no reply in this post already names, and retarget replies to that number.
pub(crate) const FIX_EHEAD_RENUMBERED: &str = r#"; its number changes from #{shown} to #{n} if no reply in this post already names @#{n}, otherwise to another number that no header in this post carries, that no other repair in this report gives a header, and that no reply in this post already names. Change any @#{shown} that meant this comment to that number"#;

/// Appended to the E-HEAD fix when the header shows no usable number.
/// `{n}` is the unused number the rewrite prints; the CLI does not
/// guess a prior identity. Choose `#{n}` before rewriting if no reply
/// already names it; otherwise another number that no header carries,
/// that no other repair in this report gives a header, and that no
/// reply already names, and retarget replies to that number. A reader
/// who knows the original number may restore it when no header in this
/// post carries it and no other repair is given it; that original number
/// may already be named by a reply.
pub(crate) const FIX_EHEAD_UNNUMBERED: &str = r#"; the header shows no usable number, so use #{n} if no reply in this post already names @#{n}, otherwise another number that no header in this post carries, that no other repair in this report gives a header, and that no reply in this post already names. Change any reply that meant this comment to that number. If you know this comment's original number and no header in this post carries it or is given it by another repair, use that number instead and no reply needs to change"#;

/// §9 supplement 3: E-SEQ-DUP fix when no number above `{max}` is left:
/// the same by-hand lower-number repair as [`FIX_EHEAD_EXHAUSTED`], which
/// changes only the number, then [`FIX_EXHAUSTED_TAIL`]. A fresh number
/// must not capture an existing `@#n` reply.
pub(crate) const FIX_ESEQDUP_EXHAUSTED: &str = r#"no number above #{max} is available. Change only this later #{n}'s header number by hand to a lower number unused by headers, other repairs and @#n replies in this post. Keep its position, author, time and text; retarget any @#{n} replies meant for it"#;

/// W-SEQ-GAP when a malformed comment header lies between the two
/// numbers: that header may be the missing comment, not a deletion.
pub(crate) const WHY_WSEQGAP_MALFORMED: &str = r#"no comment in this post has number #{m}, so @#{m} cannot be resolved; a malformed comment header before this one may be that comment"#;

/// W-SEQ-GAP when every number between `a` and `b` is carried by a header
/// elsewhere in the post: the comments are only out of order.
pub(crate) const WHY_WSEQGAP_LATER: &str = r#"every number between #{a} and #{b} is used elsewhere in this post, so no comment is missing and @#n still resolves; only the order is unusual"#;

/// W-SEQ-GAP at the first legal comment (`a` is the scanner's 0 seed, not
/// a comment): the message names that first comment by its position in
/// the file, because lower-numbered comments may still appear later.
/// No #0 exists or can exist.
pub(crate) const MESSAGE_WSEQGAP_FIRST: &str = r#"the first comment in this post is #{b}, not #1"#;

/// [`WHY_WSEQGAP_LATER`] at the first legal comment.
pub(crate) const WHY_WSEQGAP_FIRST_LATER: &str = r#"every number below #{b} is used elsewhere in this post, so no comment is missing and @#n still resolves; only the order is unusual"#;

/// The catalog W-SEQ-GAP why at the first legal comment: deletion or a
/// hand renumbering is one possibility, not the only one, because a post
/// may also have started above #1. `{m}` is still the first number nothing
/// carries.
pub(crate) const WHY_WSEQGAP_FIRST: &str = r#"no comment in this post has number #{m}, so @#{m} cannot be resolved; that comment may have been deleted or renumbered by hand, or the first comment was written with a higher number"#;

/// See [`WHY_WSEQGAP_MALFORMED`].
pub(crate) const FIX_WSEQGAP_MALFORMED: &str = r#"repair the malformed header first; if the gap remains, leave the numbering as is, because other files may already reference #{b}"#;

/// E-TOO-LARGE raised by a write whose result would exceed the limit;
/// the file on disk is still within it.
pub(crate) const MESSAGE_ETOOLARGE_WRITE: &str = r#"this write would make the file larger than {limit} MiB: it is {size} bytes now and would become {result} bytes"#;

/// See [`MESSAGE_ETOOLARGE_WRITE`].
pub(crate) const FIX_ETOOLARGE_WRITE: &str = r#"nothing was written; the file itself is still readable; shorten the new text, or put it in a new file (for a comment, a new post)"#;

/// E-ENTRY-HASH on an entry without a path: `--refresh` skips such an
/// entry, so only setting the target (which also writes the hash) helps.
pub(crate) const FIX_EENTRYHASH_NO_PATH: &str = r#"this entry has no path, so --refresh cannot hash it. Choose a file you know is the right target, review that file and the note, and update the note if needed. Then set its target; this command also records the target's current hash: agent-coexist brief {path_arg} --target <file> --entry={title}"#;

/// E-ENTRY-TARGET when the stored path names a directory: permissions are
/// not the problem. The review names the chosen replacement file — a
/// directory has no reviewable bytes.
pub(crate) const FIX_EENTRYTARGET_DIR: &str = r#"the path names a directory, not a file. Choose a file you know is the right target, review that file and the note, and update the note if needed. Then point the entry at it; this command also records the target's current hash: agent-coexist brief {path_arg} --target <file> --entry={title}"#;

/// E-ENTRY-TARGET when another process holds the target locked (the
/// WouldBlock marker from `fsx`'s bounded shared lock): the entry and its
/// permissions are fine and the bytes become readable once the holder
/// releases, so the only recovery reruns the read that failed. `{argv}`
/// is the §8.3.4-quoted command line and embeds verbatim.
pub(crate) const FIX_EENTRYTARGET_LOCKED: &str =
    r#"wait for the process holding the target to release it, then retry: {argv}"#;

/// W-BODY for a comment: only its author may rewrite it (E-EDIT-AUTHOR),
/// so the recipe names who runs it. `{author}` / `{seq}` are the stored
/// header values; the command follows after `: `.
pub(crate) const FIX_WBODY_COMMENT: &str = r#"only {author}, the comment's author, can add its body; anyone else asks {author} to, or replies with @#{seq} instead; the command for {author}"#;

/// W-BODY for a profile section whose heading `--section` cannot address.
pub(crate) const FIX_WBODY_SECTION_NAME: &str = "rename the heading by hand to a valid, distinct section name and add a fenced body beneath it, then run check again";

/// Fix row after a completed post write whose new body replies to a
/// comment number no comment in this post carries (the `reply-to` row
/// above says `(not found)`): the comment is already written and only
/// its author may rewrite it (E-EDIT-AUTHOR), so the recipe edits the
/// comment this report announced — keeping the reply and naming a
/// number the post carries — instead of posting a new one. `--edit`
/// replaces the whole body, so the `-m` marker stands for the comment's
/// entire body with the reference corrected, not only the reply line.
/// Composed by
/// [`super::reply_target_missing_fix`]: `{path_arg}` is §8.3.4-quoted,
/// `{nick}` / `{seq}` come from the write itself, and an unprintable
/// operand takes the manual-command fallback.
pub(crate) const FIX_WREPLYDANGLING_WRITE: &str = r#"if you meant another comment, edit this one so the reply names a number this post carries: agent-coexist post {path_arg} --as={nick} --edit={seq} -m '<the whole body, with the reference corrected>'"#;

/// E-FILE-MISSING when the file's own directory is missing too: list the
/// nearest existing ancestor (`dir_arg`) instead of a missing directory.
pub(crate) const FIX_EFILEMISSING_ANCESTOR: &str = r#"correct the path and rerun (this command never creates the file); its directory does not exist either; collaboration files under the nearest existing directory: agent-coexist list {dir_arg}"#;

/// W-STATE-DUP rendered by a wait refusal (`wait` or `wait-state` on a
/// state file): the wait cannot start until the duplicate is gone, so
/// resolving it is required, not optional. A declaration by the listed
/// agent merges only clean-line entries; every occurrence on a dirty
/// line is deleted from its line by hand first, keeping each line's
/// other content, and a declaration follows only when duplicates remain
/// or one entry from now is wanted (the refusal prints no W-STATE-ENTRY
/// row, so the fix says to inspect the file). The printed command stays
/// the declaration by the listed agent; the restart is a separate step
/// with no command here.
pub(crate) const FIX_WSTATEDUP_WAIT: &str = r#"the wait cannot start while a nick is listed twice under one status; restart only after the duplicate is gone. To merge, {nick} declares again to replace both with one entry from now (add --for to choose its length), but only when both entries are on clean lines. When either entry sits on a line that also has unparseable fragments (W-STATE-ENTRY), that line is kept as is: inspect the file, {nick} deletes its own entries from every such line by hand keeping other content, then declares again to merge remaining duplicates or start one entry from now; the command for {nick}: agent-coexist state {dir_arg} --as={nick} --status={status}"#;

/// Wait E-LOCKED: repair, restart, ready line, then the command-specific
/// re-read ([`REREAD_FILE`] / [`REREAD_STATE`]); the original argv stays last.
pub(crate) const FIX_WAIT_LOCKED: &str = r#"wait briefly; restart with this command, then {reread} after its ready line (the new wait cannot report changes made while it was stopped): {argv}"#;

/// [`FIX_WAIT_LOCKED`] when the original command cannot be printed.
/// The caller appends the explanation from [`FIX_NATIVE_OPERAND`].
pub(crate) const FIX_WAIT_LOCKED_MANUAL: &str = r#"wait briefly; restart the original wait by hand, then {reread} after its ready line (the new wait cannot report changes made while it was stopped)"#;

/// Wait E-INUSE: same re-read requirement as [`FIX_WAIT_LOCKED`].
pub(crate) const FIX_WAIT_INUSE: &str = r#"close the other program; restart with this command, then {reread} after its ready line (the new wait cannot report changes made while it was stopped): {argv}"#;

/// [`FIX_WAIT_INUSE`] when the original command cannot be printed.
/// The caller appends the explanation from [`FIX_NATIVE_OPERAND`].
pub(crate) const FIX_WAIT_INUSE_MANUAL: &str = r#"close the other program; restart the original wait by hand, then {reread} after its ready line (the new wait cannot report changes made while it was stopped)"#;

/// Wait I/O recovery: the caller puts the cause-specific repair first,
/// then this restart/read instruction, then `: {argv}` with no suffix.
pub const FIX_WAIT_RESTART: &str = "restart with this command, then {reread} after its ready line (the new wait cannot report changes made while it was stopped)";

/// [`FIX_WAIT_RESTART`] when no copyable original command can be printed.
/// The caller appends the explanation from [`FIX_NATIVE_OPERAND`].
pub const FIX_WAIT_RESTART_MANUAL: &str = "restart the original wait by hand, then {reread} after its ready line (the new wait cannot report changes made while it was stopped)";

/// `{reread}` for `wait` and `wait-post`.
pub const REREAD_FILE: &str = "read the file again";

/// `{reread}` for `wait-state`.
pub const REREAD_STATE: &str = "run check-state on the directory again";

/// Wait path-class fault that ends a wait after its ready line with
/// NotFound. The storage read reports the same kind for a deleted or
/// renamed-away directory, for a directory's place taken by a file (a
/// Windows shape) and for a dangling link surviving at the watched path,
/// so the advice states the one unconditional goal — restore the
/// intended path — and each cause as its own condition; nothing is
/// deleted blindly. `{display}` is the failing path's folded display;
/// the CLI wait dispatch composes this in place of the generic I/O
/// repair before [`FIX_WAIT_RESTART`] and the original wait command.
pub const FIX_WAIT_RESTORE_DIR_GONE: &str = "restore the intended path of {display}. If the directory is missing, recreate it or move it back if renamed away. If a file occupies the directory's place, move that file aside, keeping it. If {display} is a dangling link, restore the file it points at";

/// Wait path-class fault that ends a wait after its ready line: a
/// directory on the watched path was replaced by a regular file
/// (NotADirectory). The replacement is moved aside and kept while the
/// intended directory is re-created; nothing is deleted blindly.
/// `{display}` is the failing path's folded display.
pub const FIX_WAIT_RESTORE_DIR_REPLACED: &str = "a directory on the path of {display} is now a file; move that file aside, keeping it, and recreate the directory it replaced";

/// Wait path-class fault that ends a wait after its ready line: the
/// watched path itself no longer names a regular file (IsADirectory, or
/// the storage read's regular-file rejection). Whatever now occupies the
/// path is moved aside and kept, and nothing has to be created in its
/// place: the watch accepts an absent file as its next baseline.
/// `{display}` is the failing path's folded display.
pub const FIX_WAIT_RESTORE_NOT_REGULAR: &str = "the path of {display} itself no longer names a regular file; move whatever now occupies it aside, keeping it; the watch also accepts the file absent, so nothing needs to be created in its place";

/// Fix row after a completed profile/brief write whose functional-lock
/// check could not read everything (skipped / partial / unreadable /
/// canonicalize rows). `{unreadable}` contains any prerequisite clauses
/// from [`SCAN_UNREADABLE_FIRST`] and [`SCAN_CANONICALIZE_FIRST`];
/// `{commands}` are the read-only checks. Row keys distinguish these from
/// a brief's own `entry … skipped` rows.
pub const FIX_LOCK_SCAN_INCOMPLETE: &str = r#"changes reported above are already written; rows starting with skipped, partial or canonicalize mean the lock check may have missed a holder. Before writing again, {unreadable}run these checks, then agree with any holder they show: {commands}"#;

/// Fix row on a completed write whose report lists `entry … skipped`
/// and/or `remaining` errors. Check owns the full diagnostic text
/// (§9 supplement 5); this is a read-only pointer at this file, never a
/// rewrite and never a substitute for the lock-scan fix. Remaining rows
/// stay the last group.
pub const FIX_FILE_CHECK: &str = r#"skipped entries and remaining errors in this file need check on this file. Follow the suggestions check prints: agent-coexist check {path_arg}"#;

/// [`FIX_FILE_CHECK`] when this file's path cannot be written
/// as a copyable operand.
pub const FIX_FILE_CHECK_MANUAL: &str = r#"skipped entries and remaining errors in this file need check on this file. Follow the suggestions check prints by running agent-coexist check on this file by hand; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly"#;

/// [`FIX_LOCK_SCAN_INCOMPLETE`] when a path cannot be written as a
/// copyable operand; `{commands}` are the `MANUAL_*` descriptions.
pub const FIX_LOCK_SCAN_INCOMPLETE_MANUAL: &str = r#"changes reported above are already written; rows starting with skipped, partial or canonicalize mean the lock check may have missed a holder. Before writing again, {unreadable}run by hand {commands}, then agree with any holder shown; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly"#;

/// Fix row after a completed `state` declaration whose conflict scan could
/// not read everything. Same placeholders as [`FIX_LOCK_SCAN_INCOMPLETE`].
pub const FIX_STATE_SCAN_INCOMPLETE: &str = r#"the declaration is written; rows starting with skipped, partial or canonicalize mean the conflict check may have missed a lock. Without declaring again, {unreadable}run these checks, then agree with any holder they show: {commands}"#;

/// See [`FIX_STATE_SCAN_INCOMPLETE`] and [`FIX_LOCK_SCAN_INCOMPLETE_MANUAL`].
pub const FIX_STATE_SCAN_INCOMPLETE_MANUAL: &str = r#"the declaration is written; rows starting with skipped, partial or canonicalize mean the conflict check may have missed a lock. Without declaring again, {unreadable}run by hand {commands}, then agree with any holder shown; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly"#;

/// Fix row after a completed `state` declaration whose conflict scan stopped on an OS error (or any non-source scan fault) shown in the `scan` row. `{dir_arg}` is the §8.3.4-quoted DIR (plain str replace, done last).
pub const FIX_STATE_SCAN_STOPPED: &str = r#"the declaration above is written, but the conflict check stopped at the error in the scan row above, so a conflicting lock may have been missed. Without declaring again, resolve that error, then check for conflicts: agent-coexist check-state {dir_arg}"#;

/// See [`FIX_STATE_SCAN_STOPPED`]: DIR cannot be written as a copyable operand.
pub const FIX_STATE_SCAN_STOPPED_MANUAL: &str = r#"the declaration above is written, but the conflict check stopped at the error in the scan row above, so a conflicting lock may have been missed. Without declaring again, resolve that error, then run agent-coexist check-state on this directory by hand; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly"#;

/// `check-state` with skipped/partial rows: read each unread state file.
pub const FIX_CHECK_STATE_UNREAD: &str = r#"some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; {unreadable}see what each one holds: {commands}"#;

/// See [`FIX_CHECK_STATE_UNREAD`].
pub const FIX_CHECK_STATE_UNREAD_MANUAL: &str = r#"some state files were not read in full (skipped or partial rows above), so declarations in them may be missing here; {unreadable}see what each one holds by hand with {commands}; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly"#;

/// `read-notification` with skipped/partial rows: check each unread file.
/// check shows a file's type and the problems left in it; a wrong-type
/// file under the reserved state name can be clean, so the text says what
/// its E-TYPE row means instead of promising that check explains it.
pub const FIX_NOTIFY_UNREAD: &str = r#"some files were not read in full (skipped or partial rows above), so records in them may be missing here. check shows each file's type if recognized, and remaining problems; an E-TYPE row means that directory's state file has another type and no declarations were read from it. Before concluding nothing is new, {unreadable}check each one: {commands}"#;

/// See [`FIX_NOTIFY_UNREAD`].
pub const FIX_NOTIFY_UNREAD_MANUAL: &str = r#"some files were not read in full (skipped or partial rows above), so records in them may be missing here. check shows each file's type if recognized, and remaining problems; an E-TYPE row means that directory's state file has another type and no declarations were read from it. Before concluding nothing is new, {unreadable}check each one, by hand with {commands}; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly"#;

/// `list` with `?`/`busy` files or unreadable directories: read each again.
/// A directory is unreadable when it cannot be listed or, listed, cannot be
/// entered (Unix read without search permission).
pub const FIX_LIST_UNREAD: &str = r#"some files above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; {unreadable}read each one again to see what it holds or why it cannot be read: {commands}"#;

/// See [`FIX_LIST_UNREAD`].
pub const FIX_LIST_UNREAD_MANUAL: &str = r#"some files above have no known type (? or busy) or some directories could not be read or entered (skipped rows), so types or files may be missing here; {unreadable}read each one again to see what it holds or why it cannot be read, by hand with {commands}; no copyable command can be printed because a path contains a line break or characters this text cannot reproduce exactly"#;

/// `{unreadable}` when some scanned directory could not be listed or
/// entered: restoring only read permission leaves a directory without
/// search permission just as unreadable, so both are named.
pub const SCAN_UNREADABLE_FIRST: &str = r#"make each directory marked unreadable readable and enterable with read and search permission; "#;

/// `{unreadable}` when identity fell back to lexical paths (`canonicalize`
/// rows). Restore existence and open/read plus traverse/search permission
/// on the path and every ancestor, so Windows read/open and Unix ancestor
/// search are both named. The trailing separator composes ordered clauses.
pub const SCAN_CANONICALIZE_FIRST: &str = r#"ensure each path marked canonicalize exists and all directories on it, including ancestors, can be opened with read and traverse/search permission; "#;

/// Standalone fix after `canonicalize` rows on check-state and
/// read-notification. No placeholders, no command. Same permission clause
/// as [`SCAN_CANONICALIZE_FIRST`].
pub const FIX_CANONICALIZE: &str = r#"real paths could not be resolved for directories marked canonicalize; ensure those paths exist and all directories on them, including ancestors, can be opened with read and traverse/search permission, then run this command again"#;

/// Manual `{commands}` descriptions for the scan follow-ups above.
pub const MANUAL_CHECK_STATE_FILES: &str =
    r#"agent-coexist check <dir>/state for each <dir> in a skipped or partial row"#;

/// See [`MANUAL_CHECK_STATE_FILES`].
pub const MANUAL_CHECK_SCAN_FILES: &str = r#"agent-coexist check <path> for each <path> in a skipped or partial row (<path>/state when the row names a directory)"#;

/// See [`MANUAL_CHECK_STATE_FILES`].
pub const MANUAL_CHECK_STATE_UNREADABLE: &str =
    r#"agent-coexist check-state <dir> for each <dir> marked unreadable"#;

/// A state declaration rechecks the directory it declared, not the cwd.
pub const MANUAL_CHECK_STATE_DECLARED: &str =
    r#"agent-coexist check-state on the declared directory"#;

/// A profile/brief write rechecks the written file's parent, not the cwd.
pub const MANUAL_CHECK_STATE_WRITTEN_FILE: &str =
    r#"agent-coexist check-state on the written file's parent directory"#;

/// See [`MANUAL_CHECK_STATE_FILES`].
pub const MANUAL_CHECK_LIST_LEAVES: &str = r#"agent-coexist check <path> for each ? or busy file, <path> being DIR, the directory rows above the file and its name, joined by /"#;

/// See [`MANUAL_CHECK_STATE_FILES`].
pub const MANUAL_LIST_UNREADABLE: &str =
    r#"agent-coexist list <dir> for each <dir> marked unreadable"#;

/// Reason for a write whose new content is complete in the file but whose
/// final `sync_data` failed (`WriteLeft::Unconfirmed`). Nothing was undone.
pub const MESSAGE_WRITE_UNCONFIRMED: &str =
    r#"wrote {file}, but could not confirm that the change is stored on disk: {cause}"#;

/// See [`MESSAGE_WRITE_UNCONFIRMED`]. The command is not repeated: a post
/// or other change that is already in the file would be applied twice.
/// `{check}` is `: agent-coexist check <file>` or a by-hand variant.
pub const FIX_WRITE_UNCONFIRMED: &str = r#"the change is in the file, so do not run the same command again, which could apply it twice. {repair}, then check that the file still holds the change{check}"#;

/// Reason for a failed write whose restore of the previous content failed
/// too (`WriteLeft::Mixed`): both causes stay visible.
pub const MESSAGE_WRITE_MIXED: &str =
    r#"cannot write {file}: {cause}; putting its previous content back also failed: {rollback}"#;

/// See [`MESSAGE_WRITE_MIXED`]. `{from}`/`{len}` are byte counts: bytes
/// before `{from}` are unchanged, the previous content was `{len}` bytes.
pub const FIX_WRITE_MIXED: &str = r#"the first {from} bytes of the file are unchanged, but after them it may hold part of this change mixed with its previous content ({len} bytes before this command), so do not run the same command again. {repair}, then check the file and correct it by hand{check}"#;

/// Look up a §9 template by diagnostic id.
pub(crate) fn find(id: &str) -> Option<&'static Template> {
    CATALOG.iter().find(|t| t.id == id)
}
