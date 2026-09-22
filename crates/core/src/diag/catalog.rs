//! Spec §9 diagnostic catalog: all 50 templates transcribed byte-verbatim
//! from the normative spec table (columns 文案 → message, why → fix).
//!
//! `{placeholder}` sequences are PLAIN TEXT replaced at runtime; never pass
//! these strings through any formatter.

/// One §9 catalog row; message/why/fix are byte-verbatim template text.
pub(crate) struct Template {
    pub(crate) id: &'static str,
    pub(crate) message: &'static str,
    pub(crate) why: &'static str,
    pub(crate) fix: &'static str,
}

/// All 50 diagnostics in §9 table order (this order is the same-line tiebreak).
pub(crate) const CATALOG: &[Template] = &[
    Template {
        id: "E-UTF8",
        message: r#"file is not valid UTF-8"#,
        why: r#"the CLI cannot locate lines or headings in it"#,
        fix: r#"re-save the file as UTF-8"#,
    },
    Template {
        id: "E-TOO-LARGE",
        message: r#"file is larger than {limit} MiB"#,
        why: r#"files this size are never collaboration files; parsing it would stall every list and notification scan"#,
        fix: r#"move the large content elsewhere and keep only a short post or brief here"#,
    },
    Template {
        id: "E-H1",
        message: r##"first line is "{line}", not "# <name>.<type>""##,
        why: r#"the H1 is the only place the file type is recorded; without it nothing else can be parsed"#,
        fix: r##"set line 1 to "# {stem}.post" (or .brief / .agent-profile), or "# .state" for a state file"##,
    },
    Template {
        id: "E-FENCE",
        message: r#"code fence opened at L{n} is never closed"#,
        why: r#"everything after it is treated as fence content, so no later comment, entry or section can be found"#,
        fix: r#"add a closing line of {ticks} backticks where the block should end"#,
    },
    Template {
        id: "E-HEAD",
        message: r#""{line}" looks like a comment header but is malformed"#,
        why: r###"comment headers must be "## #<n> <nick> (<time>)"; a malformed one makes numbering and @#n references ambiguous"###,
        fix: r###"fix the header to "## #{next} <nick> ({now})", or drop the leading "#" to make it a plain heading"###,
    },
    Template {
        id: "E-SEQ-DUP",
        message: r#"comment number #{n} appears twice"#,
        why: r#"@#{n} and --edit {n} cannot tell the two apart"#,
        fix: r#"renumber the later one to #{next} and update any @#{n} that meant it"#,
    },
    Template {
        id: "E-ENTRY-DUP",
        message: r#"entry "{title}" appears twice"#,
        why: r#"--entry {title} cannot tell the two apart"#,
        fix: r#"merge the two sections or retitle one of them"#,
    },
    Template {
        id: "E-ENTRY-PATH",
        message: r#"entry "{title}" has no "- path:" line"#,
        why: r#"without a path there is nothing to hash or read; the entry cannot be verified"#,
        fix: r#"agent-coexist brief {path} --target <file> --entry {title}"#,
    },
    Template {
        id: "E-ENTRY-HASH",
        message: r#"entry "{title}" has no valid "- hash:" line"#,
        why: r#"the hash is how check tells fresh from shifted; without it drift is invisible"#,
        fix: r#"agent-coexist brief {path} --entry {title} --refresh"#,
    },
    Template {
        id: "E-ENTRY-REGEX",
        message: r#"entry "{title}" regex does not compile: {err}"#,
        why: r#"read-brief and check use the regex to find the lines the note is about"#,
        fix: r#"agent-coexist brief {path} --target {target} --entry {title} --regex '<pattern>'"#,
    },
    Template {
        id: "E-ENTRY-TARGET",
        message: r#"entry "{title}" target {target} cannot be read: {err}"#,
        why: r#"the target exists but its bytes are not readable, so neither hash nor regex can be checked"#,
        fix: r#"fix its permissions, or point the entry elsewhere: agent-coexist brief {path} --target <file> --entry {title}"#,
    },
    Template {
        id: "E-SECTION-DUP",
        message: r#"section "{name}" appears twice"#,
        why: r#"--section {name} cannot tell the two apart"#,
        fix: r#"merge the two sections or rename one"#,
    },
    Template {
        id: "E-STATUS-DUP",
        message: r#"status "{status}" appears twice"#,
        why: r#"entries under a status must live in one place, or upsert and cleanup would touch the wrong one"#,
        fix: r###"merge the two "## {status}" sections into one"###,
    },
    Template {
        id: "E-TYPE",
        message: r#"file is a {actual}, not a {expected}"#,
        why: r#"each command works on one file type; the H1 decides which"#,
        fix: r#"use agent-coexist {cmd_for_actual} for this file, or choose another path"#,
    },
    Template {
        id: "E-LOCKED",
        message: r#"file is locked by another process"#,
        why: r#"another agent-coexist command is writing it right now"#,
        fix: r#"retry shortly: {argv}"#,
    },
    Template {
        id: "E-INUSE",
        message: r#"file is in use by another program"#,
        why: r#"an editor or another tool holds it open without sharing"#,
        fix: r#"close it, then retry: {argv}"#,
    },
    Template {
        id: "E-FLOCKED",
        message: r#"{dir} is locked by {holder} until {until}"#,
        why: r#"{holder} asked that nothing under {dir} be changed; profile and brief writes fast-fail on such locks"#,
        fix: r#"wait for the lock to expire or ask {holder} to release it; if the lock is yours, pass --as {holder}"#,
    },
    Template {
        id: "E-FILE-MISSING",
        message: r#"{file} does not exist"#,
        why: r#"read commands and forms that only act on existing content (--edit, --cancel, --remove, --refresh, note-only -m) never create files, so a typo shows up here instead of as an empty result"#,
        fix: r#"check the path: agent-coexist list {dir}"#,
    },
    Template {
        id: "E-DIR-MISSING",
        message: r#"directory {dir} does not exist"#,
        why: r#"an empty result for a missing directory would look like "nothing happened""#,
        fix: r#"check the path: agent-coexist list {parent}"#,
    },
    Template {
        id: "E-NOT-DIR",
        message: r#"{path} is not a directory"#,
        why: r#"state and the scanning commands describe directories, not files"#,
        fix: r#"re-run against the directory: agent-coexist {cmd} {parent} <same flags>"#,
    },
    Template {
        id: "E-TARGET-MISSING",
        message: r#"target {target} does not exist"#,
        why: r#"a brief entry must point at something that can be hashed now"#,
        fix: r#"create the target first, or fix the path"#,
    },
    Template {
        id: "E-ENTRY-NOTFOUND",
        message: r#"no entry titled "{title}""#,
        why: r#"titles are how brief entries are addressed; the command will not guess which one you meant"#,
        fix: r#"pick an existing title from: agent-coexist check {path}"#,
    },
    Template {
        id: "E-SECTION-NOTFOUND",
        message: r#"no section named "{name}""#,
        why: r#"only existing sections can be removed; adding one takes -m"#,
        fix: r#"pick an existing name from: agent-coexist check {path}"#,
    },
    Template {
        id: "E-STATE-NOTFOUND",
        message: r#"{nick} has no entry in {dir}"#,
        why: r#"the state file only lists current declarations; a nick that is not there has nothing to release"#,
        fix: r#"pick a listed nick from: agent-coexist check-state {dir}, then: agent-coexist state {dir} --remove <nick>; if nobody is listed, nothing needs releasing"#,
    },
    Template {
        id: "E-EDIT-AUTHOR",
        message: r#"comment #{n} was written by {author}, not {nick}"#,
        why: r#"comments stay attributable; only the author may rewrite one"#,
        fix: r#"reply instead: agent-coexist post {path} --as {nick} -m '@#{n} <your reply>'"#,
    },
    Template {
        id: "E-COMMENT-NOTFOUND",
        message: r#"comment #{n} does not exist"#,
        why: r#"--edit rewrites an existing comment; the highest number in this post is #{max}"#,
        fix: r#"add a new comment instead: agent-coexist post {path} --as {nick} -m '<text>'"#,
    },
    Template {
        id: "E-ENTRY-TITLE-TAKEN",
        message: r#"entry "{title}" already points at {existing}, not {target}"#,
        why: r#"the default title is the target's file name; two different files with the same name would silently overwrite each other"#,
        fix: r#"give the entry its own title: agent-coexist brief {path} --target {target} --entry '<title>'"#,
    },
    Template {
        id: "E-SEQ-EXHAUSTED",
        message: r#"comment numbering is exhausted: #{max} is the highest number a header can carry"#,
        why: r#"the header grammar stops at #{max}; nothing was written"#,
        fix: r#"start a new post: agent-coexist post <new path> --as {nick} -m '<text>'"#,
    },
    Template {
        id: "W-BOM",
        message: r#"file starts with a UTF-8 BOM"#,
        why: r#"the BOM is skipped when reading; appends and splices leave it in place, so it stays until someone re-saves the file"#,
        fix: r#"re-save the file without BOM"#,
    },
    Template {
        id: "W-CRLF",
        message: r#"file uses CRLF line endings"#,
        why: r#"lines the CLI adds use LF, so the file will end up mixed"#,
        fix: r#"convert the file to LF"#,
    },
    Template {
        id: "W-NAME",
        message: r#"file name "{stem}" does not match the H1 name "{name}""#,
        why: r#"list shows the file name, the file says another name; readers will be confused"#,
        fix: r##"rename the file to {name}.agent-besties.md or set line 1 to "# {stem}.{type}""##,
    },
    Template {
        id: "W-NAME-STATE",
        message: r#"state file is named "{stem}.agent-besties.md", not "state.agent-besties.md""#,
        why: r#"check-state and state look only for state.agent-besties.md (case-insensitive on Windows/macOS); other names are ignored"#,
        fix: r##"rename the file to state.agent-besties.md (keep line 1 as "# .state")"##,
    },
    Template {
        id: "W-STRAY",
        message: r#"text outside any fence at L{n} is not part of the record"#,
        why: r#"the CLI only reads fenced bodies, attribute lines and headings; this text is kept but never shown"#,
        fix: r#"move it into the nearest fenced block, or delete it"#,
    },
    Template {
        id: "W-BODY",
        message: r#"{unit} at L{n} has no fenced body"#,
        why: r#"there is nothing to show as its content"#,
        fix: r#"{set_body}"#,
    },
    Template {
        id: "W-SUB-DUP",
        message: r#"subscribers line repeated or contains duplicates"#,
        why: r#"only the first line counts and duplicates are ignored; the extra text is noise for readers"#,
        fix: r#"keep a single "- subscribers:" line with each nick once"#,
    },
    Template {
        id: "W-SEQ-GAP",
        message: r#"comment numbers jump from #{a} to #{b}"#,
        why: r#"@#{m} can no longer be resolved; a comment was probably deleted by hand"#,
        fix: r#"leave the numbering as is; other files may already reference #{b}"#,
    },
    Template {
        id: "W-SEQ-ORDER",
        message: r#"comment #{n} appears after #{m}"#,
        why: r#"readers assume comments are in order; @#n still resolves"#,
        fix: r#"move the section so numbers increase, or leave it if the order is intentional"#,
    },
    Template {
        id: "W-REPLY-DANGLING",
        message: r#"comment #{n} replies to @#{m} which does not exist"#,
        why: r#"the reply cannot be followed to its target"#,
        fix: r#"ask the author of #{n} which comment was meant"#,
    },
    Template {
        id: "W-ENTRY-ABS",
        message: r#"entry "{title}" uses an absolute path"#,
        why: r#"the brief only verifies on this machine; on any other checkout the target will be stale"#,
        fix: r#"agent-coexist brief {path} --target <relative path> --entry {title}"#,
    },
    Template {
        id: "W-ENTRY-KEY",
        message: r#"entry "{title}" has unknown attribute "{key}""#,
        why: r#"the line is kept but has no meaning to the CLI"#,
        fix: r#"remove the line, or move its content into the note"#,
    },
    Template {
        id: "W-ENTRY-SHIFTED",
        message: r#"entry "{title}" is shifted: {target} changed since its hash was taken"#,
        why: r#"the note may describe code that no longer looks like this"#,
        fix: r#"review the target, then: agent-coexist brief {path} --entry {title} --refresh"#,
    },
    Template {
        id: "W-ENTRY-STALE",
        message: r#"entry "{title}" is stale: {reason}"#,
        why: r#"the note points at something that is gone or no longer matches its regex"#,
        fix: r#"agent-coexist brief {path} --target <file> --entry {title} --regex '<pattern>'"#,
    },
    Template {
        id: "W-PROFILE-EMPTY",
        message: r#"profile has no sections"#,
        why: r#"nothing tells other agents who this is or what they own"#,
        fix: r#"agent-coexist profile {path} --section role -m '<what you do>'"#,
    },
    Template {
        id: "W-SECTION-NAME",
        message: r#"section "{name}" at L{n} is not a valid section name"#,
        why: r#"--section cannot address it; allowed: letters, digits, _ and -, no spaces"#,
        fix: r###"rename the heading, e.g. "## {suggested}""###,
    },
    Template {
        id: "W-STATUS-NAME",
        message: r#"status "{name}" at L{n} is not a valid status name"#,
        why: r#"entries under it are shown but never treated as locked or observing; allowed: lowercase letters, digits and -"#,
        fix: r###"rename the heading to "## {lowercased}""###,
    },
    Template {
        id: "W-STATE-ENTRY",
        message: r#""{fragment}" under {status} is not "<nick> <start>..<until>""#,
        why: r#"the fragment is kept but ignored by conflict checks and cleanup"#,
        fix: r#"rewrite it as "<nick> <start>..<until>" or delete it"#,
    },
    Template {
        id: "W-STATE-DUP",
        message: r#"{nick} appears twice under {status}"#,
        why: r#"the latest end time wins; the next state write merges them into one unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"agent-coexist state {dir} --as {nick} --status {status}"#,
    },
    Template {
        id: "W-STATE-RANGE",
        message: r#"{nick} under {status} ends before it starts"#,
        why: r#"treated as expired: it never counts as a lock; the next state write removes it unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"agent-coexist state {dir} --as {nick} --status {status}"#,
    },
    Template {
        id: "W-STATE-CONFLICT",
        message: r#"locked is held by both {a} and {b}"#,
        why: r#"two agents asked for exclusive changes in the same subtree; neither is blocked, so they must agree who continues"#,
        fix: r#"agree who continues, then the other runs: agent-coexist state {dir} --remove <nick>"#,
    },
    Template {
        id: "W-STATE-EXPIRED",
        message: r#"{nick} under {status} expired at {until}"#,
        why: r#"expired entries are ignored by conflict checks and removed on the next state write unless the line also has unparseable fragments (W-STATE-ENTRY)"#,
        fix: r#"renew if still relevant: agent-coexist state {dir} --as {nick} --status {status}"#,
    },
];

/// §9 supplement 1: E-TYPE fix when actual type is state.
pub(crate) const FIX_ETYPE_STATE: &str =
    r#"this is a state file: agent-coexist state {dir} --as <nick> --status <name>"#;

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
