//! Diagnostic classification and §9 rendering.
//!
//! `catalog.rs` is the single source of the §9 template text; this module
//! classifies ids, orders them for output, and turns a `Diagnostic` into the
//! `message` / `why` / `fix` triple. Title placeholders (`status`, `title`,
//! `name`, `section`, `entry`) and file stems are display-sanitized only
//! in human text.
//! Commands keep their original identity; headings with controls cannot
//! be addressed through the CLI and receive manual repair guidance instead.
//! Other params are substituted as given.
//! Runtime values come from the [`Context`]. Each catalog fix declares
//! whether its substitutions are command operands (§8.3.4) or prose/file
//! content. Only command operands are POSIX-quoted; already-quoted `{argv}`
//! retries embed verbatim. Copyable headings and other literal content
//! never receive shell quoting ([`fix_text`], [`heading_fix`]). Invalid
//! stored statuses receive manual heading repair before any CLI operation.

mod catalog;

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::error::Diagnostic;
use crate::fsx;
use crate::names;

pub use catalog::{
    FIX_CANONICALIZE, FIX_CHECK_STATE_UNREAD, FIX_CHECK_STATE_UNREAD_MANUAL, FIX_FILE_CHECK,
    FIX_FILE_CHECK_MANUAL, FIX_LIST_UNREAD, FIX_LIST_UNREAD_MANUAL, FIX_LOCK_SCAN_INCOMPLETE,
    FIX_LOCK_SCAN_INCOMPLETE_MANUAL, FIX_NATIVE_OPERAND, FIX_NOTIFY_UNREAD,
    FIX_NOTIFY_UNREAD_MANUAL, FIX_STATE_SCAN_INCOMPLETE, FIX_STATE_SCAN_INCOMPLETE_MANUAL,
    FIX_STATE_SCAN_STOPPED, FIX_STATE_SCAN_STOPPED_MANUAL, FIX_WAIT_RESTART,
    FIX_WAIT_RESTART_MANUAL, FIX_WAIT_RESTORE_DIR_GONE, FIX_WAIT_RESTORE_DIR_REPLACED,
    FIX_WAIT_RESTORE_NOT_REGULAR, FIX_WRITE_MIXED, FIX_WRITE_UNCONFIRMED, MANUAL_CHECK_LIST_LEAVES,
    MANUAL_CHECK_SCAN_FILES, MANUAL_CHECK_STATE_DECLARED, MANUAL_CHECK_STATE_FILES,
    MANUAL_CHECK_STATE_UNREADABLE, MANUAL_CHECK_STATE_WRITTEN_FILE, MANUAL_LIST_UNREADABLE,
    MESSAGE_WRITE_MIXED, MESSAGE_WRITE_UNCONFIRMED, REREAD_FILE, REREAD_STATE,
    SCAN_CANONICALIZE_FIRST, SCAN_UNREADABLE_FIRST,
};

/// Error level: every id from the catalog starting with `E-`.
pub fn is_error(id: &str) -> bool {
    id.starts_with("E-")
}

/// The blocking subset (spec §1): these errors make the file structure
/// undecidable and refuse every write command.
pub fn is_blocking(id: &str) -> bool {
    matches!(
        id,
        "E-UTF8"
            | "E-TOO-LARGE"
            | "E-H1"
            | "E-FENCE"
            | "E-HEAD"
            | "E-SEQ-DUP"
            | "E-ENTRY-DUP"
            | "E-SECTION-DUP"
            | "E-STATUS-DUP"
    )
}

/// Position in the §9 table; the tiebreak for diagnostics on the same line
/// (spec §9 supplement 4). Unknown ids sort last.
pub fn order(id: &str) -> usize {
    catalog::CATALOG
        .iter()
        .position(|t| t.id == id)
        .unwrap_or(usize::MAX)
}

/// Runtime context for template substitution. `path` and `dir` are display
/// values only; `path_arg` and `dir_arg` retain the native operands for
/// commands. `state_file_selected` records the existing reserved-state
/// lookup once for the inspected file; false includes unknown selection.
/// `argv` must be built with [`Context::argv_from_args`] for per-argument
/// POSIX quoting. `None` records an unrepresentable command, not an absent
/// placeholder.
#[derive(Debug, Clone)]
pub struct Context {
    pub path: String,
    pub dir: String,
    pub path_arg: PathBuf,
    pub dir_arg: PathBuf,
    /// True only when the native inspected path is the selected state file.
    pub state_file_selected: bool,
    pub stem: String,
    pub argv: Option<String>,
    pub command: String,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            path: String::new(),
            dir: String::new(),
            path_arg: PathBuf::new(),
            dir_arg: PathBuf::new(),
            state_file_selected: false,
            stem: String::new(),
            argv: Some(String::new()),
            command: String::new(),
        }
    }
}

impl Context {
    /// The `{argv}` value: every argument POSIX-quoted per §8.3.4, joined by
    /// single spaces (§9: `{argv}` = the original command line). `None` means
    /// at least one native argument cannot be represented in a single-line
    /// POSIX recipe.
    pub fn argv_from_args<I, A>(args: I) -> Option<String>
    where
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let mut out = String::new();
        for (index, arg) in args.into_iter().enumerate() {
            let quoted = names::shell_quote(arg.as_ref())?;
            if index != 0 {
                out.push(' ');
            }
            out.push_str(&quoted);
        }
        Some(out)
    }
}

/// A rendered diagnostic: the diagnostic-row text plus its `why` / `fix`
/// lines (§8.1.6). For reason-form failures the `message` is the reason
/// text (§8.3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    pub message: String,
    pub why: String,
    pub fix: String,
}

/// Render an anchored diagnostic (spec §9 templates + §8.3.4 quoting).
pub fn render(d: &Diagnostic, ctx: &Context) -> Rendered {
    parts(d.id, &d.params, ctx)
}

/// Render a reason-form failure (spec §8.3.1): the same catalog row, with
/// the catalog message column serving as the reason text.
pub fn reason(id: &str, params: &[(&'static str, OsString)], ctx: &Context) -> Rendered {
    parts(id, params, ctx)
}

fn parts(id: &str, params: &[(&'static str, OsString)], ctx: &Context) -> Rendered {
    match catalog::find(id) {
        Some(t) => {
            let message = match id {
                "E-STATE-NOTFOUND" if get(params, "status").is_some() => {
                    catalog::MESSAGE_ESTATE_NOTFOUND_STATUS
                }
                "E-FILE-MISSING" if get(params, "given_dir").is_some() => {
                    catalog::MESSAGE_EFILEMISSING_DIR
                }
                "E-TOO-LARGE" if get(params, "result").is_some() => {
                    catalog::MESSAGE_ETOOLARGE_WRITE
                }
                "W-SEQ-GAP" if get(params, "first").is_some() => catalog::MESSAGE_WSEQGAP_FIRST,
                _ => t.message,
            };
            let first = get(params, "first").is_some();
            let why = match id {
                "W-SEQ-GAP" if get(params, "later").is_some() => {
                    if first {
                        catalog::WHY_WSEQGAP_FIRST_LATER
                    } else {
                        catalog::WHY_WSEQGAP_LATER
                    }
                }
                "W-SEQ-GAP" if get(params, "malformed").is_some() => catalog::WHY_WSEQGAP_MALFORMED,
                "W-SEQ-GAP" if first => catalog::WHY_WSEQGAP_FIRST,
                // The same unselected gate as `fix_text`'s state recipes:
                // these whys promise the next state write, and no state
                // command writes this file. W-STATE-CONFLICT's why names
                // no state write, so it keeps the catalog row.
                "W-STATE-DUP" if !ctx.state_file_selected => catalog::WHY_WSTATEDUP_UNSELECTED,
                "W-STATE-RANGE" if !ctx.state_file_selected => catalog::WHY_WSTATERANGE_UNSELECTED,
                "W-STATE-EXPIRED" if !ctx.state_file_selected => {
                    catalog::WHY_WSTATEEXPIRED_UNSELECTED
                }
                _ => t.why,
            };
            Rendered {
                message: subst(message, params, ctx, false).unwrap_or_else(str::to_string),
                why: subst(why, params, ctx, false).unwrap_or_else(str::to_string),
                fix: fix_text(id, t, params, ctx),
            }
        }
        None => Rendered {
            message: format!("unknown diagnostic id {id}"),
            why: String::new(),
            fix: String::new(),
        },
    }
}

/// Fix text with context-sensitive overrides (§9 supplements), using the
/// catalog's explicit command-versus-literal substitution mode. Every
/// variant keeps the reader's original task: a recipe either repeats or
/// corrects the failed command, or inspects before that retry; it never
/// substitutes an unrelated write.
fn fix_text(
    id: &str,
    t: &catalog::Template,
    params: &[(&'static str, OsString)],
    ctx: &Context,
) -> String {
    let state_recipe = matches!(
        id,
        "W-STATE-DUP" | "W-STATE-RANGE" | "W-STATE-EXPIRED" | "W-STATE-CONFLICT"
    );
    if state_recipe && !ctx.state_file_selected {
        return catalog::FIX_UNSELECTED_STATE.to_string();
    }
    let wait = matches!(ctx.command.as_str(), "wait" | "wait-post" | "wait-state");
    let template: &str = match id {
        // A command that needs a state file met another type under the
        // reserved name: only renaming that file lets the same command
        // succeed. Every other mismatch asks for a path of the needed type.
        // Creating write forms may also use a new unused path; query/edit/
        // delete/refresh/wait cannot, so they keep the catalog row.
        "E-TYPE" if get(params, "expected").is_some_and(|e| e == "state") => {
            catalog::FIX_ETYPE_RESERVED
        }
        "E-TYPE" if etype_can_create(ctx) => catalog::FIX_ETYPE_CREATE,
        "E-TOO-LARGE" if get(params, "result").is_some() => catalog::FIX_ETOOLARGE_WRITE,
        // No number above the highest is left, and the allocator never
        // wraps: the comment keeps its place under an unused lower number
        // the reader chooses by hand. E-HEAD also prints the known-original
        // restore branch. Both are plain-text recipes.
        "E-HEAD" | "E-SEQ-DUP" if get(params, "exhausted").is_some() => {
            let fix = if id == "E-HEAD" {
                let fill = if get(params, "fill").is_some() {
                    catalog::FIX_EXHAUSTED_FILL
                } else {
                    ""
                };
                catalog::FIX_EHEAD_EXHAUSTED.replace("{fill}", fill)
            } else {
                catalog::FIX_ESEQDUP_EXHAUSTED.to_string()
            };
            let fix = format!("{fix}{}", catalog::FIX_EXHAUSTED_TAIL);
            return finish_fix(&fix, params, ctx, false);
        }
        "E-HEAD" => return ehead_fix(t, params, ctx),
        "E-ENTRY-TITLE-TAKEN" => return title_taken_fix(t, params, ctx),
        "E-ENTRY-HASH" if get(params, "no_path").is_some() => catalog::FIX_EENTRYHASH_NO_PATH,
        // A target another process holds locked is contention, not
        // damage: the entry stays as written and the same read is
        // retried once the holder releases.
        "E-ENTRY-TARGET" if get(params, "is_locked").is_some() => catalog::FIX_EENTRYTARGET_LOCKED,
        "E-ENTRY-TARGET" if get(params, "is_dir").is_some() => catalog::FIX_EENTRYTARGET_DIR,
        "E-STATE-NOTFOUND" if get(params, "status").is_some() => {
            catalog::FIX_ESTATE_NOTFOUND_STATUS
        }
        "E-FILE-MISSING" if get(params, "given_dir").is_some() => catalog::FIX_EFILEMISSING_DIR,
        "E-FILE-MISSING" if get(params, "missing_dir").is_some() => {
            catalog::FIX_EFILEMISSING_ANCESTOR
        }
        // These commands need the directory itself; listing its parent
        // would not show a directory that holds no collaboration files.
        "E-DIR-MISSING"
            if matches!(
                ctx.command.as_str(),
                "state" | "wait" | "wait-post" | "wait-state"
            ) =>
        {
            catalog::FIX_EDIRMISSING_CREATE
        }
        "E-DIR-MISSING" if get(params, "missing_dir").is_some() => {
            catalog::FIX_EDIRMISSING_ANCESTOR
        }
        // A restarted wait takes a new baseline. Manual recovery names
        // the by-hand restart before the re-read, not after it.
        "E-LOCKED" | "E-INUSE" if wait => {
            let manual = ctx.argv.is_none();
            let fix = match (id, manual) {
                ("E-LOCKED", false) => catalog::FIX_WAIT_LOCKED,
                ("E-LOCKED", true) => catalog::FIX_WAIT_LOCKED_MANUAL,
                (_, false) => catalog::FIX_WAIT_INUSE,
                (_, true) => catalog::FIX_WAIT_INUSE_MANUAL,
            };
            let reread = if ctx.command == "wait-state" {
                catalog::REREAD_STATE
            } else {
                catalog::REREAD_FILE
            };
            let mut fix = finish_fix(&fix.replace("{reread}", reread), params, ctx, false);
            if manual {
                let why = FIX_NATIVE_OPERAND
                    .split_once("; ")
                    .map_or(FIX_NATIVE_OPERAND, |(_, why)| why);
                fix.push_str("; ");
                fix.push_str(why);
            }
            return fix;
        }
        // A wait on a state file refuses a snapshot with duplicate
        // identities: the duplicate must be gone before the wait can
        // start, and occurrences on dirty lines take the by-hand
        // deletion the template names.
        "W-STATE-DUP" if wait => catalog::FIX_WSTATEDUP_WAIT,
        // With --as already given, adding another would be a usage error.
        "E-FLOCKED" if argv_has_as(ctx) => catalog::FIX_EFLOCKED_AS,
        // Copies of one status are merged, never renamed apart.
        "W-STATUS-NAME" if get(params, "repeated").is_some() => catalog::FIX_WSTATUSNAME_REPEATED,
        "W-STRAY" if get(params, "type").is_some_and(|kind| kind == "post") => {
            catalog::FIX_WSTRAY_POST
        }
        "W-STRAY" if get(params, "type").is_some_and(|kind| kind == "state") => {
            catalog::FIX_WSTRAY_STATE
        }
        "E-NOT-DIR" => return enotdir_fix(t, params, ctx),
        "W-SEQ-GAP" if get(params, "malformed").is_some() => catalog::FIX_WSEQGAP_MALFORMED,
        // An invalid stored regex kept any drift since the stored hash
        // invisible (the regex verdict comes first), and rerunning the
        // command records the target's current hash: require the review
        // the shifted/stale siblings do. A path-less entry keeps the
        // base row, which also records a hash once a target is set.
        "E-ENTRY-REGEX" if get(params, "target_arg").is_some() => catalog::FIX_EENTRYREGEX_REVIEW,
        "W-ENTRY-STALE" if get(params, "reason").is_some_and(|r| r == "target missing") => {
            catalog::FIX_WSTALE_TARGET_MISSING
        }
        // The merge stays the repair; only its manual form changes, because
        // a control-bearing heading cannot be copied as literal content.
        // A distinct name would split one status into two.
        "E-STATUS-DUP"
            if get(params, "status")
                .and_then(OsStr::to_str)
                .is_some_and(|s| s.chars().any(char::is_control)) =>
        {
            catalog::FIX_ESTATUSDUP_CONTROL
        }
        "E-H1" => return eh1_fix(t, params, ctx),
        "W-NAME" => return wname_fix(t, params, ctx),
        "W-BODY" => return wbody_fix(params, ctx).unwrap_or_else(str::to_string),
        "W-SECTION-NAME" if get(params, "suggested").is_none() => {
            return catalog::FIX_SECTION_NAME_UNIQUE.to_string();
        }
        _ => t.fix,
    };
    finish_fix(template, params, ctx, t.command_fix)
}

/// Substitute a fix template. When its command cannot be printed
/// losslessly, keep the template's own advice (the text before the
/// command) and the command's operation, instead of replacing
/// cause-specific guidance with a generic sentence.
fn finish_fix(
    template: &str,
    params: &[(&'static str, OsString)],
    ctx: &Context,
    quote: bool,
) -> String {
    match subst(template, params, ctx, quote) {
        Ok(fix) => fix,
        Err(manual) if manual == FIX_NATIVE_OPERAND => manual_fix(template, params, ctx),
        Err(other) => other.to_string(),
    }
}

/// The manual form of a fix whose command cannot be printed losslessly.
/// A retry of the original command line (`: {argv}`) keeps the advice and
/// is done "by hand": the reader already has that command. A composed
/// recovery (`: agent-coexist …` or a template that is only a command)
/// keeps its advice and the whole command (subcommand, options, printable
/// operands) with a marker for each value that cannot be printed
/// ([`manual_command`]).
fn manual_fix(template: &str, params: &[(&'static str, OsString)], ctx: &Context) -> String {
    let composed = template
        .rfind(": agent-coexist ")
        .map(|c| (c, c + 2))
        .or_else(|| template.starts_with("agent-coexist ").then_some((0, 0)));
    let retry = template.rfind(": {argv}");
    match (composed, retry) {
        (Some((c, _)), Some(r)) if r > c => manual_retry(&template[..r], params, ctx),
        (Some((at, start)), _) => match subst(&template[..at], params, ctx, false) {
            Ok(advice) => manual_command(&advice, &template[start..], params, ctx),
            Err(_) => FIX_NATIVE_OPERAND.to_string(),
        },
        (None, Some(r)) => manual_retry(&template[..r], params, ctx),
        (None, None) => FIX_NATIVE_OPERAND.to_string(),
    }
}

/// `advice` (the text before `: {argv}`) completed by "by hand".
fn manual_retry(advice: &str, params: &[(&'static str, OsString)], ctx: &Context) -> String {
    let why = FIX_NATIVE_OPERAND
        .split_once("; ")
        .map_or(FIX_NATIVE_OPERAND, |(_, why)| why);
    match subst(advice, params, ctx, false) {
        Ok(advice) if !advice.is_empty() => format!("{advice} by hand; {why}"),
        _ => FIX_NATIVE_OPERAND.to_string(),
    }
}

/// `command` (a template starting with `agent-coexist`) with every operand
/// that cannot be printed replaced by a `<path of …>` marker, preceded by
/// the already-substituted `advice` ([`catalog::FIX_MANUAL_COMMAND`]).
fn manual_command(
    advice: &str,
    command: &str,
    params: &[(&'static str, OsString)],
    ctx: &Context,
) -> String {
    let mut markers = Vec::new();
    match subst_with(command, params, ctx, true, Some(&mut markers)) {
        Ok(command) if !markers.is_empty() => {
            // The advice ends where its command would follow (`…, then`,
            // `… from`), so the manual instruction takes the command's place.
            let advice = if advice.is_empty() {
                String::new()
            } else {
                format!("{advice}: ")
            };
            let fill = [
                ("advice", OsString::from(advice)),
                ("markers", OsString::from(join_and(&markers))),
                ("command", OsString::from(command)),
            ];
            subst(catalog::FIX_MANUAL_COMMAND, &fill, ctx, false)
                .unwrap_or_else(|_| FIX_NATIVE_OPERAND.to_string())
        }
        Err(other) if other != FIX_NATIVE_OPERAND => other.to_string(),
        _ => FIX_NATIVE_OPERAND.to_string(),
    }
}

/// Reader-typed marker for a command operand that cannot be printed.
fn operand_marker(key: &str) -> String {
    match key {
        "path_arg" => "<path of this file>".to_string(),
        "dir_arg" => "<path of this directory>".to_string(),
        "parent_arg" => "<path of the parent directory>".to_string(),
        "target_arg" => "<path of the target>".to_string(),
        other => format!("<{} shown above>", other.trim_end_matches("_arg")),
    }
}

/// `a`, `a and b`, `a, b and c`.
fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The final `refused` row of a write refused by structural errors
/// (§2.7.4, §8.3.2): nothing was written, and which `ID at L<n>` rows
/// block it, so a reader who just ran another row's fix knows what to fix
/// first. `None` when no structural blocker is present, e.g. a write whose
/// result alone would exceed the size limit (its fix already says that
/// nothing was written). Write refusals only; check reports never call it.
pub fn write_refusal(diags: &[Diagnostic]) -> Option<String> {
    let mut blocking: Vec<&Diagnostic> = diags
        .iter()
        .filter(|d| {
            is_blocking(d.id) && !(d.id == "E-TOO-LARGE" && get(&d.params, "result").is_some())
        })
        .collect();
    if blocking.is_empty() {
        return None;
    }
    let mixed = blocking.len() < diags.len();
    blocking.sort_by_key(|d| (d.line, order(d.id)));
    let mut rows: Vec<String> = blocking
        .iter()
        .map(|d| format!("{} at L{}", d.id, d.line))
        .collect();
    rows.dedup();
    let template = if mixed {
        catalog::MESSAGE_WRITE_REFUSED_MIXED
    } else {
        catalog::MESSAGE_WRITE_REFUSED
    };
    Some(template.replace("{blocking}", &join_and(&rows)))
}

/// E-NOT-DIR fix: the original command line with its directory operand
/// replaced by that path's parent, so every other option survives. When
/// the operand cannot be located exactly once, the `<same flags>` template
/// stays.
fn enotdir_fix(
    t: &catalog::Template,
    params: &[(&'static str, OsString)],
    ctx: &Context,
) -> String {
    match enotdir_retry(params, ctx) {
        Some(retry) => catalog::FIX_ENOTDIR_RETRY.replace("{retry}", &retry),
        None => finish_fix(t.fix, params, ctx, t.command_fix),
    }
}

/// E-HEAD fix: the suggested header (with the reader-filled variant when
/// it still shows `<nick>` / `<time>`). A repair numbered `n` that differs
/// from the header's own number `shown`, or that gives a number to a
/// header showing none (`n` without `shown`), also says which replies to
/// change: an old `@#` would otherwise point at another comment or none.
/// Choose `#{n}` before rewriting if no reply already names it; otherwise
/// another number that no header carries, that no other repair in this
/// report gives a header, and that no reply in this post already names,
/// and retarget replies to that number. The unnumbered branch also says a
/// known original number may be restored when no header carries it and no
/// other repair is given it.
fn ehead_fix(t: &catalog::Template, params: &[(&'static str, OsString)], ctx: &Context) -> String {
    let header = if get(params, "fill").is_some() {
        catalog::FIX_EHEAD_FILL
    } else {
        t.fix
    };
    let references = if get(params, "shown").is_some() {
        catalog::FIX_EHEAD_RENUMBERED
    } else if get(params, "n").is_some() {
        catalog::FIX_EHEAD_UNNUMBERED
    } else {
        ""
    };
    finish_fix(&format!("{header}{references}"), params, ctx, t.command_fix)
}

/// Whether the original command line already gives `--as` (as its own word
/// or `--as=<nick>`, quoted or not) before any `--` separator.
fn argv_has_as(ctx: &Context) -> bool {
    argv_has_option(ctx, "--as")
}

/// `true` when `{argv}` has `option` or `option=` as its own word before
/// any `--` separator. Empty or unprintable argv is not a match.
fn argv_has_option(ctx: &Context, option: &str) -> bool {
    let Some(argv) = ctx.argv.as_deref().filter(|s| !s.is_empty()) else {
        return false;
    };
    let equals = format!("{option}=");
    shell_words(argv)
        .into_iter()
        .skip(1)
        .take_while(|word| *word != "--")
        .any(|word| {
            let word = word.strip_prefix('\'').unwrap_or(word);
            word == option || word.starts_with(&equals)
        })
}

/// E-TYPE: only write forms that would create a missing file may be
/// retried on a new unused path. Inspects `{argv}` options (equivalent to
/// a `can_create` param) so query/edit/remove/refresh/wait/subscribe never
/// get the create clause. Missing argv stays on the catalog row.
fn etype_can_create(ctx: &Context) -> bool {
    match ctx.command.as_str() {
        "post" => !argv_has_option(ctx, "--edit"),
        "profile" => !argv_has_option(ctx, "--remove"),
        "brief" => {
            !argv_has_option(ctx, "--refresh")
                && !argv_has_option(ctx, "--remove")
                && (argv_has_option(ctx, "--target") || !argv_has_option(ctx, "--entry"))
        }
        _ => false,
    }
}

/// E-ENTRY-TITLE-TAKEN fix: the original command line with
/// `--entry '<title>'` added before any `--` separator (after it the words
/// would be operands), so `--as`, `-m`, `--regex` and every other option
/// survive. Without a printable command line the catalog row says to rerun
/// it by hand with that option added.
fn title_taken_fix(
    t: &catalog::Template,
    params: &[(&'static str, OsString)],
    ctx: &Context,
) -> String {
    match ctx.argv.as_deref().filter(|argv| !argv.is_empty()) {
        Some(argv) => catalog::FIX_EENTRYTITLETAKEN_RETRY.replace("{retry}", &entry_retry(argv)),
        None => finish_fix(t.fix, params, ctx, t.command_fix),
    }
}

/// `{argv}` with `--entry '<title>'` inserted before the first `--` word,
/// or appended when there is none. The parser reads a bare `--` word as
/// the separator, never as an option value (`-m=--` passes that text).
fn entry_retry(argv: &str) -> String {
    const ENTRY: &str = "--entry '<title>'";
    let words = shell_words(argv);
    let at = words
        .iter()
        .skip(1)
        .position(|word| *word == "--")
        .map_or(words.len(), |p| p + 1);
    let mut out = String::with_capacity(argv.len() + ENTRY.len() + 1);
    for (i, word) in words.iter().enumerate() {
        if i == at {
            out.push(' ');
            out.push_str(ENTRY);
        }
        if i != 0 {
            out.push(' ');
        }
        out.push_str(word);
    }
    if at == words.len() {
        out.push(' ');
        out.push_str(ENTRY);
    }
    out
}

/// `{argv}` with the one word spelled exactly like the quoted directory
/// operand replaced by the quoted parent. `None` when argv or either
/// operand is unrepresentable, or the operand is absent or ambiguous.
fn enotdir_retry(params: &[(&'static str, OsString)], ctx: &Context) -> Option<String> {
    let argv = ctx.argv.as_deref()?;
    let operand = names::shell_quote(ctx.dir_arg.as_os_str())?;
    let parent = quote_value("parent_arg", get(params, "parent_arg")?)?;
    let words = shell_words(argv);
    let mut hits = words
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, w)| **w == operand);
    let (at, _) = hits.next()?;
    if hits.next().is_some() {
        return None;
    }
    let mut out = String::with_capacity(argv.len() + parent.len());
    for (i, word) in words.iter().enumerate() {
        if i != 0 {
            out.push(' ');
        }
        out.push_str(if i == at { parent.as_str() } else { word });
    }
    Some(out)
}

/// Split an `{argv}` line built by [`Context::argv_from_args`] into its
/// quoted words: spaces separate words only outside quotes. Outside quotes
/// `\` escapes the next character (the `'\''` form); the Unix octal form
/// nests `'…'` inside `"$(…)"`.
fn shell_words(line: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let (mut start, mut single, mut double, mut escaped) = (0, false, false, false);
    for (i, c) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if !single && !double => escaped = true,
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            ' ' if !single && !double => {
                words.push(&line[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    words.push(&line[start..]);
    words
}

/// Shared composer for fixes that suggest an H1 built from the file stem
/// (E-H1, W-NAME; 2026-09-25 repair): the stem is display-folded, and the
/// suggestion substitutes as plain text — it is file content, not a
/// command argument, so §8.3.4 quoting must never touch it. When the
/// folded stem is not a valid NAME the `fallback` template is used: an
/// invalid stem must never be presented as a valid H1. A stem that only
/// folds to a valid name still yields a copyable suggestion.
fn heading_fix(
    base: &'static str,
    fallback: &'static str,
    params: &[(&'static str, OsString)],
    ctx: &Context,
) -> String {
    let folded = resolve("stem", params, ctx)
        .map(|s| names::display_text(&s.to_string_lossy()))
        .unwrap_or_default();
    let template = if names::valid_name(&folded) {
        base
    } else {
        fallback
    };
    subst(template, params, ctx, false).unwrap_or_else(str::to_string)
}

/// E-H1 fix: under the reserved state file name (platform rule of
/// [`fsx::walk::is_state_file_name`]) `# .state` leads, because another
/// type there would be refused by every state command; otherwise the base
/// suggestion when the stem is a valid NAME, else the any-valid-name
/// guidance (never an invalid or quoted stem as the H1).
fn eh1_fix(t: &catalog::Template, params: &[(&'static str, OsString)], ctx: &Context) -> String {
    if ctx
        .path_arg
        .file_name()
        .is_some_and(fsx::walk::is_state_file_name)
    {
        return catalog::FIX_EH1_STATE_NAME.to_string();
    }
    heading_fix(t.fix, catalog::FIX_EH1_BAD_STEM, params, ctx)
}

/// W-NAME fix: both recoveries with plain substitution when the stem is a
/// valid NAME; otherwise rename-only — the H1 name is already valid, the
/// stem can never become a valid H1. The mismatch itself stays a warning.
fn wname_fix(t: &catalog::Template, params: &[(&'static str, OsString)], ctx: &Context) -> String {
    heading_fix(t.fix, catalog::FIX_WNAME_BAD_STEM, params, ctx)
}

/// `W-BODY` fix (§9 supplement 2): the `set_body` command for the file type
/// that owns the body-less unit. Only posts (`seq` + `author`) and
/// profiles (`name`) emit W-BODY; a brief entry's note is optional. A
/// comment may be rewritten only by its author, so the post fix names who
/// runs it and what everyone else does instead. Unaddressable headings
/// receive manual repair guidance instead of a command the CLI rejects.
fn wbody_fix(params: &[(&'static str, OsString)], ctx: &Context) -> Result<String, &'static str> {
    if get(params, "seq").is_some() && get(params, "author").is_some() {
        let advice = subst(catalog::FIX_WBODY_COMMENT, params, ctx, false)?;
        return Ok(command_fix(
            &advice,
            "agent-coexist post {path_arg} --as={author} --edit={seq} -m '<text>'",
            params,
            ctx,
        ));
    }
    let name = get(params, "name").ok_or(FIX_NATIVE_OPERAND)?;
    let text = name.to_str().ok_or(FIX_NATIVE_OPERAND)?;
    if text.chars().any(char::is_control) {
        return Err(catalog::FIX_CONTROL_HEADING);
    }
    if !names::valid_section(text) {
        return Err(catalog::FIX_WBODY_SECTION_NAME);
    }
    Ok(command_fix(
        "",
        "agent-coexist profile {path_arg} --section={name} -m '<text>'",
        params,
        ctx,
    ))
}

/// `advice: command` with every operand quoted, or its manual form when an
/// operand cannot be printed ([`manual_command`]).
fn command_fix(
    advice: &str,
    command: &str,
    params: &[(&'static str, OsString)],
    ctx: &Context,
) -> String {
    match subst(command, params, ctx, true) {
        Ok(command) if advice.is_empty() => command,
        Ok(command) => format!("{advice}: {command}"),
        Err(manual) if manual == FIX_NATIVE_OPERAND => manual_command(advice, command, params, ctx),
        Err(other) => other.to_string(),
    }
}

/// The `fix` row for a completed post write whose new body replied to a
/// comment number that no comment in this post carries: the comment is
/// already written and only its author may rewrite it, so the recipe
/// edits the comment the report just announced, keeping the reply.
/// `{path_arg}` is §8.3.4-quoted; an operand that cannot be printed
/// takes the manual-command fallback ([`catalog::FIX_MANUAL_COMMAND`]).
pub fn reply_target_missing_fix(ctx: &Context, nick: &str, seq: u64) -> String {
    let params = [
        ("nick", OsString::from(nick)),
        ("seq", OsString::from(seq.to_string())),
    ];
    finish_fix(catalog::FIX_WREPLYDANGLING_WRITE, &params, ctx, true)
}

fn get<'a>(params: &'a [(&'static str, OsString)], key: &str) -> Option<&'a OsStr> {
    params
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.as_os_str())
}

fn is_title_param(key: &str) -> bool {
    matches!(key, "status" | "title" | "name" | "section" | "entry")
}

/// Human display only; executable operands must not use this lossy value.
fn display_param<'a>(key: &str, value: &'a OsStr) -> Cow<'a, str> {
    let value = value.to_string_lossy();
    if is_title_param(key) || key == "stem" {
        Cow::Owned(names::display_text(&value))
    } else {
        value
    }
}

/// Positional paths must survive both shell and CLI option parsing. Native
/// spelling is otherwise left alone; option values use `--key={value}` in
/// the templates and need only shell quoting.
fn quote_value(key: &str, value: &OsStr) -> Option<String> {
    if matches!(key, "path_arg" | "dir_arg" | "parent_arg")
        && value.as_encoded_bytes().starts_with(b"-")
    {
        let mut operand = OsString::from("./");
        operand.push(value);
        names::shell_quote(&operand)
    } else {
        names::shell_quote(value)
    }
}

/// A positional path operand for a composed recovery command (§8.3.4): the
/// same rule the catalog uses for `{dir_arg}`. `None` = not representable.
pub fn path_operand(value: &OsStr) -> Option<String> {
    quote_value("dir_arg", value)
}

/// Substitute `{key}` placeholders. Explicit params win (first occurrence),
/// then the context keys; unknown placeholders stay literal so a missing
/// value is visible instead of silently dropped. Human titles and stems
/// are sanitized, but a command must never address the sanitized sibling.
fn subst(
    template: &str,
    params: &[(&'static str, OsString)],
    ctx: &Context,
    quote: bool,
) -> Result<String, &'static str> {
    subst_with(template, params, ctx, quote, None)
}

/// [`subst`]; with `markers`, a command operand (`*_arg`) that cannot be
/// quoted is written as its [`operand_marker`] and recorded there instead
/// of failing the whole substitution.
fn subst_with(
    template: &str,
    params: &[(&'static str, OsString)],
    ctx: &Context,
    quote: bool,
    mut markers: Option<&mut Vec<String>>,
) -> Result<String, &'static str> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) => {
                let key = &after[..close];
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    if key == "argv" && get(params, key).is_none() && ctx.argv.is_none() {
                        return Err(FIX_NATIVE_OPERAND);
                    }
                    if let Some(v) = resolve(key, params, ctx) {
                        // §9: `{argv}` is the command line with every
                        // argument already §8.3.4-quoted and joined by
                        // single spaces (`Context::argv_from_args`); it
                        // embeds verbatim. Re-quoting would collapse the
                        // whole line into one giant single-quoted program
                        // name.
                        if key == "argv" {
                            let argv = v
                                .to_str()
                                .filter(|s| !s.contains(['\0', '\r', '\n']))
                                .ok_or(FIX_NATIVE_OPERAND)?;
                            out.push_str(argv);
                        } else if quote {
                            if is_title_param(key) {
                                let text = v.to_str().ok_or(FIX_NATIVE_OPERAND)?;
                                if text.chars().any(char::is_control) {
                                    return Err(catalog::FIX_CONTROL_HEADING);
                                }
                                // Stored statuses can be readable but not
                                // addressable through the CLI grammar.
                                if key == "status" && !names::valid_status(text) {
                                    return Err(catalog::FIX_INVALID_STATUS);
                                }
                            }
                            match (quote_value(key, v), markers.as_deref_mut()) {
                                (Some(quoted), _) => out.push_str(&quoted),
                                (None, Some(found)) if key.ends_with("_arg") => {
                                    let marker = operand_marker(key);
                                    out.push_str(&marker);
                                    if !found.contains(&marker) {
                                        found.push(marker);
                                    }
                                }
                                (None, _) => return Err(FIX_NATIVE_OPERAND),
                            }
                        } else {
                            out.push_str(&display_param(key, v));
                        }
                        rest = &after[close + 1..];
                        continue;
                    }
                    if key == "target_arg" {
                        // A path-less brief entry supplies no runtime target.
                        // Only that absence yields a reader-fillable marker;
                        // a real target/title equal to "<file>" is quoted.
                        out.push_str("<file>");
                        rest = &after[close + 1..];
                        continue;
                    }
                }
                out.push('{');
                rest = after;
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    Ok(out)
}

fn resolve<'a>(
    key: &str,
    params: &'a [(&'static str, OsString)],
    ctx: &'a Context,
) -> Option<&'a OsStr> {
    if let Some(v) = get(params, key) {
        return Some(v);
    }
    let value = match key {
        "path_arg" => ctx.path_arg.as_os_str(),
        "dir_arg" => ctx.dir_arg.as_os_str(),
        "path" => OsStr::new(&ctx.path),
        "dir" => OsStr::new(&ctx.dir),
        "stem" => OsStr::new(&ctx.stem),
        "argv" => OsStr::new(ctx.argv.as_deref()?),
        "command" => OsStr::new(&ctx.command),
        _ => return None,
    };
    // An unset context field stays as the literal placeholder rather than
    // silently vanishing (§9 supplement 7 spirit).
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        Context {
            path: "docs/standup".into(),
            dir: "docs".into(),
            path_arg: "docs/standup.agent-coexist.md".into(),
            dir_arg: "docs".into(),
            state_file_selected: false,
            stem: "standup".into(),
            argv: Some("agent-coexist check docs/standup".into()),
            command: "check".into(),
        }
    }

    #[test]
    fn classification_sets() {
        assert!(is_error("E-H1"));
        assert!(!is_error("W-STRAY"));
        assert!(is_blocking("E-STATUS-DUP"));
        assert!(!is_blocking("E-ENTRY-PATH"));
        assert!(!is_blocking("W-BOM"));
        // reason-level errors of the 2026-09-25 repair: errors, but never
        // structural write-blockers
        assert!(is_error("E-NAME"));
        assert!(is_error("E-STATE-DIRTY"));
        assert!(!is_blocking("E-NAME"));
        assert!(!is_blocking("E-STATE-DIRTY"));
        // unknown ids sort last in the same-line tiebreak
        assert_eq!(order("E-NOSUCH"), usize::MAX);
    }

    #[test]
    fn argv_has_as_respects_word_boundaries() {
        for (argv, has_as) in [
            ("agent-coexist brief docs/h --as bob -m x", true),
            ("agent-coexist brief docs/h '--as=bob' -m x", true),
            ("agent-coexist brief docs/h -m x -- --as", false),
            ("agent-coexist brief docs/h -m '--as bob'", false),
            ("agent-coexist brief docs/h --ask -m x", false),
        ] {
            let mut c = ctx();
            c.argv = Some(argv.into());
            assert_eq!(argv_has_as(&c), has_as, "{argv}");
        }
    }

    #[test]
    fn wname_fix_invalid_stem_recommends_rename_only() {
        // valid stem: both recoveries, plain substitution of the H1 form
        let d = Diagnostic::new("W-NAME", 3)
            .with("stem", "bob")
            .with("name", "alice")
            .with("type", "post");
        let r = render(&d, &ctx());
        assert!(
            r.fix.contains("alice.agent-coexist.md"),
            "rename target missing: {s}",
            s = r.fix
        );
        assert!(
            r.fix.contains("# bob.post"),
            "stem H1 missing: {s}",
            s = r.fix
        );

        // invalid stem: rename to the already-valid H1 name only; the
        // stem is never offered as an H1, quoted or not
        for bad in ["my docs", "it's;rm"] {
            let d = Diagnostic::new("W-NAME", 3)
                .with("stem", bad)
                .with("name", "alice")
                .with("type", "post");
            let r = render(&d, &ctx());
            assert!(
                r.fix.contains("alice.agent-coexist.md"),
                "rename target missing: {s}",
                s = r.fix
            );
            assert!(
                !r.fix.contains(&format!("# {bad}")),
                "invalid stem suggested: {s}",
                s = r.fix
            );
            assert!(!r.fix.contains("{stem}"), "unresolved stem: {s}", s = r.fix);
        }
    }

    #[test]
    fn eh1_fix_never_suggests_an_invalid_stem() {
        // a stem that is not a valid NAME must not appear as the
        // supposedly valid H1, quoted or not
        for bad in ["my docs", "it's;rm"] {
            let mut c = ctx();
            c.stem = bad.into();
            let d = Diagnostic::new("E-H1", 1).with("line", "# Notes");
            let r = render(&d, &c);
            assert!(!r.fix.contains("{stem}"), "unresolved stem in: {}", r.fix);
            assert!(
                !r.fix.contains(&format!("# {bad}")),
                "invalid stem suggested: {}",
                r.fix
            );
            assert!(r.fix.contains(".post"), "no H1 form offered: {}", r.fix);
            assert!(
                r.fix.contains(".state"),
                "no state fallback offered: {}",
                r.fix
            );
        }

        // no stem at all: same honest guidance, no placeholder leak
        let d = Diagnostic::new("E-H1", 1).with("line", "");
        let c = Context::default();
        let r = render(&d, &c);
        assert!(!r.fix.contains("{stem}"), "unresolved stem in: {}", r.fix);
        assert!(r.fix.contains(".post") && r.fix.contains(".state"));

        // spaces in a stem must not become a supposedly valid H1
        let d = Diagnostic::new("E-H1", 1).with("line", "# Notes");
        let mut c = ctx();
        c.stem = "stand up".into(); // invalid: space
        let r = render(&d, &c);
        assert!(!r.fix.contains("# stand up.post"));

        // stem that folds to a valid name still yields a valid H1
        // suggestion (whitespace is display-folded, not copied)
        let mut c = ctx();
        c.stem = "  standup  ".into();
        let r = render(&d, &c);
        assert!(
            r.fix.contains("# standup.post"),
            "folded stem not suggested: {}",
            r.fix
        );
    }

    #[test]
    fn title_placeholders_replace_controls_before_interpolation() {
        let d = Diagnostic::new("W-STATE-EXPIRED", 4)
            .with("nick", "alice")
            .with("status", "custom\u{1b}[2J")
            .with("until", "2026-09-25T11:00:00Z")
            .with("dir", "diagnostic");
        let r = render(&d, &ctx());
        assert!(r.message.contains("custom\u{FFFD}[2J"));
        assert!(
            !r.message.contains('\u{1b}') && !r.why.contains('\u{1b}') && !r.fix.contains('\u{1b}'),
            "status title must not leak ESC: {r:?}"
        );

        let esc = render(
            &Diagnostic::new("E-STATUS-DUP", 3).with("status", "locked\u{1b}"),
            &ctx(),
        );
        let bel = render(
            &Diagnostic::new("E-STATUS-DUP", 5).with("status", "locked\u{07}"),
            &ctx(),
        );
        assert!(esc.message.contains("locked\u{FFFD}"));
        assert_eq!(esc.message, bel.message);

        let profile = Diagnostic::new("W-BODY", 6)
            .with("unit", "section \"role\"")
            .with("n", "6")
            .with("name", "role\u{1b}");
        let r = render(&profile, &ctx());
        assert!(
            !r.fix.contains('\u{1b}'),
            "W-BODY section title must not leak ESC: {}",
            r.fix
        );
    }

    #[test]
    fn comment_body_fix_is_run_by_its_author_only() {
        let d = Diagnostic::new("W-BODY", 3)
            .with("unit", "comment #1")
            .with("n", "3")
            .with("seq", "1")
            .with("author", "alice");
        let mut c = ctx();
        c.argv = Some("agent-coexist post docs/standup --as bob -m hi".into());
        let fix = render(&d, &c).fix;
        // bob ran the command, but only alice may edit her comment: the
        // recipe is attributed to her and others get a non-edit route.
        let (advice, command) = fix.rsplit_once(": ").expect("command");
        assert_eq!(
            command,
            "agent-coexist post docs/standup.agent-coexist.md --as=alice --edit=1 -m '<text>'"
        );
        assert!(advice.contains("alice") && advice.contains("@#1"), "{fix}");
    }

    #[test]
    fn wait_retry_keeps_original_command_last() {
        let mut c = ctx();
        for command in ["wait", "wait-post", "wait-state"] {
            c.command = command.into();
            c.argv = Some(format!("agent-coexist {command} docs"));
            for id in ["E-LOCKED", "E-INUSE"] {
                let fix = render(&Diagnostic::new(id, 0), &c).fix;
                assert!(
                    fix.ends_with(&format!(": agent-coexist {command} docs")),
                    "{fix}"
                );
            }
        }
    }

    #[test]
    fn title_taken_retry_keeps_the_original_options() {
        let d = Diagnostic::new("E-ENTRY-TITLE-TAKEN", 0)
            .with("title", "mod.rs")
            .with("existing", "src/parser/mod.rs")
            .with("target", "src/render/mod.rs");
        let mut c = ctx();
        c.command = "brief".into();
        let command = |fix: &str| fix.split_once(": ").expect("command").1.to_string();
        c.argv = Some(
            "agent-coexist brief onboarding --as alice --target src/render/mod.rs -m 'Render notes' --regex 'fn render'"
                .into(),
        );
        assert_eq!(
            command(&render(&d, &c).fix),
            "agent-coexist brief onboarding --as alice --target src/render/mod.rs -m 'Render notes' --regex 'fn render' --entry '<title>'"
        );
        // After `--` the option would be read as an operand.
        c.argv = Some(
            "agent-coexist brief --as alice --target src/render/mod.rs -m 'a -- b' -- onboarding"
                .into(),
        );
        assert_eq!(
            command(&render(&d, &c).fix),
            "agent-coexist brief --as alice --target src/render/mod.rs -m 'a -- b' --entry '<title>' -- onboarding"
        );
        // No printable command line: nothing to copy, nothing invented.
        c.argv = None;
        assert!(!render(&d, &c).fix.contains("agent-coexist"));
    }

    #[cfg(windows)]
    #[test]
    fn unpaired_utf16_keeps_the_operation_with_a_marker() {
        use std::os::windows::ffi::OsStringExt;

        let native = OsString::from_wide(&[b'r' as u16, 0xd800]);
        assert!(names::shell_quote(&native).is_none());
        let mut c = ctx();
        c.path_arg = PathBuf::from(&native);
        // The repair operation and its options survive; only the path is
        // typed by the reader.
        let body = render(&Diagnostic::new("W-BODY", 3).with("name", "role"), &c).fix;
        assert!(
            body.ends_with(
                ": agent-coexist profile <path of this file> --section=role -m '<text>'"
            ),
            "{body}"
        );

        // Explicit params override an otherwise representable context; only
        // the operand is a marker.
        let missing = Diagnostic::new("E-DIR-MISSING", 0).with("parent_arg", &native);
        let fix = render(&missing, &ctx()).fix;
        assert!(
            fix.ends_with(": agent-coexist list <path of the parent directory>"),
            "{fix}"
        );

        // A retry of an unprintable original command line stays by hand.
        c.argv = Context::argv_from_args([OsStr::new("agent-coexist"), native.as_os_str()]);
        assert!(c.argv.is_none());
        let dirty = render(&Diagnostic::new("E-STATE-DIRTY", 0).with("line", "4"), &c).fix;
        assert!(!dirty.contains("agent-coexist"), "{dirty}");
    }

    #[test]
    fn unknown_placeholder_stays_literal() {
        let d = Diagnostic::new("E-LOCKED", 0);
        let c = Context::default();
        let r = render(&d, &c);
        assert!(r.fix.contains("{argv}")); // argv empty -> stays literal, visible
    }
}
