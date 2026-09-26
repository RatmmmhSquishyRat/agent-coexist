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
        fix: r###"fix the header to "## #{next} <nick> ({now})", or drop the leading "#" to make it a plain heading"###,
    },
    Template {
        id: "E-SEQ-DUP",
        command_fix: false,
        message: r#"comment number #{n} appears twice"#,
        why: r#"@#{n} and --edit {n} cannot tell the two apart"#,
        fix: r#"renumber the later one to #{next} and update any @#{n} that meant it"#,
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
        fix: r#"agent-coexist brief {path_arg} --target <file> --entry={title}"#,
    },
    Template {
        id: "E-ENTRY-HASH",
        command_fix: true,
        message: r#"entry "{title}" has no valid "- hash:" line"#,
        why: r#"the hash is how check tells fresh from shifted; without it drift is invisible"#,
        fix: r#"agent-coexist brief {path_arg} --entry={title} --refresh"#,
    },
    Template {
        id: "E-ENTRY-REGEX",
        command_fix: true,
        message: r#"entry "{title}" regex does not compile: {err}"#,
        why: r#"read-brief and check use the regex to find the lines the note is about"#,
        fix: r#"agent-coexist brief {path_arg} --target={target_arg} --entry={title} --regex '<pattern>'"#,
    },
    Template {
        id: "E-ENTRY-TARGET",
        command_fix: true,
        message: r#"entry "{title}" target {target} cannot be read: {err}"#,
        why: r#"the target exists but its bytes are not readable, so neither hash nor regex can be checked"#,
        fix: r#"fix its permissions, or point the entry elsewhere: agent-coexist brief {path_arg} --target <file> --entry={title}"#,
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
        why: r#"entries under a status must live in one place, or upsert and cleanup would touch the wrong one"#,
        fix: r###"merge the two "## {status}" sections into one"###,
    },
    Template {
        id: "E-TYPE",
        command_fix: true,
        message: r#"file is a {actual}, not a {expected}"#,
        why: r#"each command works on one file type; the H1 decides which"#,
        fix: r#"use agent-coexist {cmd_for_actual} for this file, or choose another path"#,
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
        fix: r#"close it, then retry: {argv}"#,
    },
    Template {
        id: "E-FLOCKED",
        command_fix: true,
        message: r#"{dir} is locked by {holder} until {until}"#,
        why: r#"{holder} asked that nothing under {dir} be changed; profile and brief writes fast-fail on such locks"#,
        fix: r#"wait for the lock to expire or ask its holder to release it; if the lock is yours, pass --as={holder}"#,
    },
    Template {
        id: "E-FILE-MISSING",
        command_fix: true,
        message: r#"{file} does not exist"#,
        why: r#"read commands and forms that only act on existing content (--edit, --cancel, --remove, --refresh, note-only -m) never create files, so a typo shows up here instead of as an empty result"#,
        fix: r#"check the path: agent-coexist list {dir_arg}"#,
    },
    Template {
        id: "E-DIR-MISSING",
        command_fix: true,
        message: r#"directory {dir} does not exist"#,
        why: r#"an empty result for a missing directory would look like "nothing happened""#,
        fix: r#"check the path: agent-coexist list {parent_arg}"#,
    },
    Template {
        id: "E-NOT-DIR",
        command_fix: true,
        message: r#"{path} is not a directory"#,
        why: r#"state and the scanning commands describe directories, not files"#,
        fix: r#"re-run against the directory: agent-coexist {cmd} {parent_arg} <same flags>"#,
    },
    Template {
        id: "E-TARGET-MISSING",
        command_fix: false,
        message: r#"target {target} does not exist"#,
        why: r#"a brief entry must point at something that can be hashed now"#,
        fix: r#"create the target first, or fix the path"#,
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
        fix: r#"inspect declarations before correcting the requested removal; keep its intended status selection: agent-coexist check-state {dir_arg}"#,
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
        fix: r#"add a new comment instead: agent-coexist post {path_arg} --as={nick} -m '<text>'"#,
    },
    Template {
        id: "E-ENTRY-TITLE-TAKEN",
        command_fix: true,
        message: r#"entry "{title}" already points at {existing}, not {target}"#,
        why: r#"the default title is the target's file name; two different files with the same name would silently overwrite each other"#,
        fix: r#"give the entry its own title: agent-coexist brief {path_arg} --target={target_arg} --entry '<title>'"#,
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
        why: r#"the BOM is skipped when reading; appends and splices leave it in place, so it stays until someone re-saves the file"#,
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
        fix: r##"rename the file to {name}.agent-besties.md or set line 1 to "# {stem}.{type}""##,
    },
    Template {
        id: "W-NAME-STATE",
        command_fix: false,
        message: r#"state file is named "{stem}.agent-besties.md", not "state.agent-besties.md""#,
        why: r#"check-state and state look only for state.agent-besties.md (case-insensitive on Windows/macOS); other names are ignored"#,
        fix: r##"inspect this file and any state.agent-besties.md already in its directory; resolve their intended declarations before choosing which file should use that name, and never overwrite a separate existing state file (keep line 1 as "# .state")"##,
    },
    Template {
        id: "W-STRAY",
        command_fix: false,
        message: r#"text outside any fence at L{n} is not part of the record"#,
        why: r#"the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown"#,
        fix: r#"move it into the nearest fenced block, or delete it"#,
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
        fix: r#"keep a single "- subscribers:" line with each nick once"#,
    },
    Template {
        id: "W-SEQ-GAP",
        command_fix: false,
        message: r#"comment numbers jump from #{a} to #{b}"#,
        why: r#"@#{m} can no longer be resolved; a comment was probably deleted by hand"#,
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
        why: r#"the brief only verifies on this machine; on any other checkout the target will be stale"#,
        fix: r#"agent-coexist brief {path_arg} --target <relative path> --entry={title}"#,
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
        fix: r#"review the target, then: agent-coexist brief {path_arg} --entry={title} --refresh"#,
    },
    Template {
        id: "W-ENTRY-STALE",
        command_fix: true,
        message: r#"entry "{title}" is stale: {reason}"#,
        why: r#"the note points at something that is gone or no longer matches its regex"#,
        fix: r#"agent-coexist brief {path_arg} --target <file> --entry={title} --regex '<pattern>'"#,
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
        why: r#"--section cannot address it; allowed: letters, digits, _ and -, no spaces"#,
        fix: r###"rename the heading, e.g. "## {suggested}""###,
    },
    Template {
        id: "W-STATUS-NAME",
        command_fix: false,
        message: r#"status "{name}" at L{n} is not a valid status name"#,
        why: r#"entries under it are shown but never treated as locked or observing; allowed: lowercase letters, digits and -"#,
        fix: r###"rename the heading by hand to a valid, distinct status name, e.g. "## {lowercased}" only if that name is unused, then run check again"###,
    },
    Template {
        id: "W-STATE-ENTRY",
        command_fix: false,
        message: r#""{fragment}" under {status} is not "<nick> <start>..<until>""#,
        why: r#"the fragment is kept but ignored by conflict checks and cleanup"#,
        fix: r#"rewrite it as "<nick> <start>..<until>" or delete it"#,
    },
    Template {
        id: "W-STATE-DUP",
        command_fix: true,
        message: r#"{nick} appears twice under {status}"#,
        why: r#"the latest end time wins; the next state write merges them into one unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"agent-coexist state {dir_arg} --as={nick} --status={status}"#,
    },
    Template {
        id: "W-STATE-RANGE",
        command_fix: true,
        message: r#"{nick} under {status} ends before it starts"#,
        why: r#"treated as expired: it never counts as a lock; the next state write removes it unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"agent-coexist state {dir_arg} --as={nick} --status={status}"#,
    },
    Template {
        id: "W-STATE-CONFLICT",
        command_fix: true,
        message: r#"locked is held by both {a} and {b}"#,
        why: r#"two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues"#,
        fix: r#"agree who continues, then the other runs: agent-coexist state {dir_arg} --remove <nick> --status=locked"#,
    },
    Template {
        id: "W-STATE-EXPIRED",
        command_fix: true,
        message: r#"{nick} under {status} expired at {until}"#,
        why: r#"expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"renew if still relevant: agent-coexist state {dir_arg} --as={nick} --status={status}"#,
    },
    Template {
        id: "E-NAME",
        command_fix: false,
        message: r#"file name "{name}" is not a valid name"#,
        why: r##"the H1 is generated from the file name; use 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-""##,
        fix: r#"rename the file to a valid name and re-run the same command with the new path; the CLI never renames, truncates or replaces the name you gave"#,
    },
    Template {
        id: "E-STATE-DIRTY",
        command_fix: false,
        message: r#"{nick} is still declared on line {line} of the state file in {dir}, mixed with unparseable fragments"#,
        why: r#"removal would rewrite that whole line and could drop the fragments it cannot parse, so nothing is written"#,
        fix: r#"make line {line} parseable by hand (or delete {nick}'s declaration from it), then re-run: {argv}"#,
    },
];

/// A status-filtered removal only establishes absence within that status,
/// not absence from every declaration in the directory.
pub(crate) const MESSAGE_ESTATE_NOTFOUND_STATUS: &str =
    r#"{nick} has no entry under status "{status}" in {dir}"#;

/// §9 supplement 1: E-TYPE fix when actual type is state.
pub(crate) const FIX_ETYPE_STATE: &str =
    r#"this is a state file: agent-coexist state {dir_arg} --as <nick> --status <name>"#;

/// A directory-level state recipe is safe only for the inspected file
/// selected by the existing reserved-state lookup. Failed selection is
/// not authority to mutate a possibly different file.
pub(crate) const FIX_UNSELECTED_STATE: &str =
    "edit this inspected file by hand and run check again; do not use directory-level state commands for this repair, because this file was not confirmed as their selected state.agent-besties.md target";

/// A concrete section repair must be unique across stored headings and
/// all suggestions emitted for the same profile.
pub(crate) const FIX_SECTION_NAME_UNIQUE: &str =
    "rename this heading by hand to a valid section name distinct from every other section and any other planned heading repair, then run check again";

/// A control-bearing heading cannot be passed through the CLI's existing
/// input grammar. Never substitute a sanitized sibling as its identity.
pub(crate) const FIX_CONTROL_HEADING: &str =
    "edit the heading by hand to remove control characters and give it a valid, distinct title or name, then run check again";

/// A stored status may be readable but not addressable by `--status`.
/// Repair its heading first; never redirect an operation to a normalized sibling.
pub(crate) const FIX_INVALID_STATUS: &str =
    "rename the status heading by hand to a valid, distinct status name, then run check again before renewing or changing its declarations";

/// 2026-09-25 repair: E-H1 fix when the file stem is not a valid NAME.
/// The base fix suggests `# {stem}.<type>`, which would re-trigger E-H1
/// the moment it is copied in, so this variant suggests any valid name
/// instead. It carries no command argument, so it substitutes as plain
/// text (§8.3.4 quoting stays on command-form fixes).
pub(crate) const FIX_EH1_BAD_STEM: &str = r##"set line 1 to "# <name>.post" (or .brief / .agent-profile): use 1-100 letters, digits, "_", "-" or ".", starting with a letter, digit, "_" or "-"; use "# .state" for a state file; renaming the file as well keeps the H1 and the file name in agreement"##;

/// 2026-09-25 repair: W-NAME fix when the file stem is not a valid NAME.
/// The base template also offers `set line 1 to "# {stem}.{type}"`, which
/// would be an invalid H1; this variant keeps only the other recovery:
/// rename the file to the H1 name, which parsing already proved to be a
/// valid NAME. Plain text, like the E-H1 variant.
pub(crate) const FIX_WNAME_BAD_STEM: &str = r#"rename the file to {name}.agent-besties.md; the file stem is not a valid name, so no valid H1 can be built from it"#;

/// §9 supplement 3: E-HEAD fix when numbering is exhausted.
pub(crate) const FIX_EHEAD_EXHAUSTED: &str =
    r##"drop the leading "#" to make this a plain heading; numbering is exhausted"##;

/// §9 supplement 3: E-SEQ-DUP fix when numbering is exhausted.
pub(crate) const FIX_ESEQDUP_EXHAUSTED: &str =
    r#"move the duplicate comment to a new post and update references to it"#;

/// Look up a §9 template by diagnostic id.
pub(crate) fn find(id: &str) -> Option<&'static Template> {
    CATALOG.iter().find(|t| t.id == id)
}
