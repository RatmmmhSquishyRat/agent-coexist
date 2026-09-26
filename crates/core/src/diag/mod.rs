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

use std::path::PathBuf;

use crate::error::Diagnostic;
use crate::names;

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
/// `now` is the command clock; `argv` must be built with
/// [`Context::argv_from_args`] for per-argument POSIX quoting.
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub path: String,
    pub dir: String,
    pub path_arg: PathBuf,
    pub dir_arg: PathBuf,
    /// True only when the native inspected path is the selected state file.
    pub state_file_selected: bool,
    pub stem: String,
    pub argv: String,
    pub command: String,
    pub now: String,
}

impl Context {
    /// The `{argv}` value: every argument POSIX-quoted per §8.3.4, joined by
    /// single spaces (§9: `{argv}` = 原命令行).
    pub fn argv_from_args(args: &[String]) -> String {
        args.iter()
            .map(|a| names::shell_quote(a))
            .collect::<Vec<_>>()
            .join(" ")
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
/// the catalog `文案` column serving as the reason text.
pub fn reason(id: &str, params: &[(&'static str, String)], ctx: &Context) -> Rendered {
    parts(id, params, ctx)
}

fn parts(id: &str, params: &[(&'static str, String)], ctx: &Context) -> Rendered {
    match catalog::find(id) {
        Some(t) => Rendered {
            message: subst(
                if id == "E-STATE-NOTFOUND" && get(params, "status").is_some() {
                    catalog::MESSAGE_ESTATE_NOTFOUND_STATUS
                } else {
                    t.message
                },
                params,
                ctx,
                false,
            ),
            why: subst(t.why, params, ctx, false),
            fix: fix_text(id, t, params, ctx),
        },
        None => Rendered {
            message: format!("unknown diagnostic id {id}"),
            why: String::new(),
            fix: String::new(),
        },
    }
}

/// Fix text with context-sensitive overrides (§9 supplements 1 and 3),
/// using the catalog's explicit command-versus-literal substitution mode.
fn fix_text(
    id: &str,
    t: &catalog::Template,
    params: &[(&'static str, String)],
    ctx: &Context,
) -> String {
    let state_recipe = matches!(
        id,
        "W-STATE-DUP" | "W-STATE-RANGE" | "W-STATE-EXPIRED" | "W-STATE-CONFLICT"
    ) || (id == "E-TYPE" && get(params, "actual").is_some_and(|a| a == "state"));
    if state_recipe && !ctx.state_file_selected {
        return catalog::FIX_UNSELECTED_STATE.to_string();
    }
    let template: &str = match id {
        // §9 supplement 1: the state-variant fix applies because the file
        // IS a state file. Presence of a `dir` param must not select it —
        // post/brief/profile E-TYPE faults carry `dir` too, and the state
        // wording would lie about the actual type (2026-09-25 repair).
        "E-TYPE" => {
            if get(params, "actual").is_some_and(|a| a == "state") {
                catalog::FIX_ETYPE_STATE
            } else {
                t.fix
            }
        }
        "E-HEAD" => {
            if get(params, "exhausted").is_some() {
                catalog::FIX_EHEAD_EXHAUSTED
            } else {
                t.fix
            }
        }
        "E-SEQ-DUP" => {
            if get(params, "exhausted").is_some() {
                catalog::FIX_ESEQDUP_EXHAUSTED
            } else {
                t.fix
            }
        }
        // This fix describes literal headings, but a control-bearing
        // identity still needs manual repair, not a display-folded sibling.
        "E-STATUS-DUP"
            if get(params, "status").is_some_and(|s| s.chars().any(char::is_control)) =>
        {
            return catalog::FIX_CONTROL_HEADING.to_string();
        }
        "E-H1" => return eh1_fix(t, params, ctx),
        "W-NAME" => return wname_fix(t, params, ctx),
        "W-BODY" => return wbody_fix(params, ctx),
        "W-SECTION-NAME" if get(params, "suggested").is_none() => {
            return catalog::FIX_SECTION_NAME_UNIQUE.to_string();
        }
        _ => t.fix,
    };
    subst(template, params, ctx, t.command_fix)
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
    params: &[(&'static str, String)],
    ctx: &Context,
) -> String {
    let folded = resolve("stem", params, ctx)
        .map(|s| names::display_text(&s))
        .unwrap_or_default();
    let template = if names::valid_name(&folded) {
        base
    } else {
        fallback
    };
    let mut effective: Vec<(&'static str, String)> = Vec::with_capacity(params.len() + 1);
    effective.push(("stem", folded));
    effective.extend(params.iter().cloned());
    subst(template, &effective, ctx, false)
}

/// E-H1 fix: base suggestion when the stem is a valid NAME, otherwise the
/// any-valid-name guidance (never an invalid or quoted stem as the H1).
fn eh1_fix(t: &catalog::Template, params: &[(&'static str, String)], ctx: &Context) -> String {
    heading_fix(t.fix, catalog::FIX_EH1_BAD_STEM, params, ctx)
}

/// W-NAME fix: both recoveries with plain substitution when the stem is a
/// valid NAME; otherwise rename-only — the H1 name is already valid, the
/// stem can never become a valid H1. The mismatch itself stays a warning.
fn wname_fix(t: &catalog::Template, params: &[(&'static str, String)], ctx: &Context) -> String {
    heading_fix(t.fix, catalog::FIX_WNAME_BAD_STEM, params, ctx)
}

/// `W-BODY` fix (§9 supplement 2): the `set_body` command for the file type
/// that owns the body-less unit. Post variants carry `seq`/`author` params,
/// profile `name`, brief `title`. Unaddressable headings receive manual
/// repair guidance instead of a command that the CLI must reject.
fn wbody_fix(params: &[(&'static str, String)], ctx: &Context) -> String {
    let path = subst("{path_arg}", params, ctx, true);
    if let Some(seq) = get(params, "seq") {
        let author = get(params, "author")
            .map(|a| names::shell_quote(a))
            .unwrap_or_else(|| "<nick>".to_string());
        format!(
            "agent-coexist post {path} --as={author} --edit={} -m '<text>'",
            names::shell_quote(seq)
        )
    } else if let Some(name) = get(params, "name") {
        if name.chars().any(char::is_control) {
            return catalog::FIX_CONTROL_HEADING.to_string();
        }
        if !names::valid_section(name) {
            return "rename the heading by hand to a valid, distinct section name and add a fenced body beneath it, then run check again".to_string();
        }
        format!(
            "agent-coexist profile {path} --section={} -m '<text>'",
            names::shell_quote(name)
        )
    } else if let Some(title) = get(params, "title") {
        if title.chars().any(char::is_control) {
            return catalog::FIX_CONTROL_HEADING.to_string();
        }
        format!(
            "agent-coexist brief {path} --entry={} -m '<note>'",
            names::shell_quote(title)
        )
    } else {
        format!("agent-coexist profile {path} --section <name> -m '<text>'")
    }
}

fn get<'a>(params: &'a [(&'static str, String)], key: &str) -> Option<&'a String> {
    params.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
}

fn is_title_param(key: &str) -> bool {
    matches!(key, "status" | "title" | "name" | "section" | "entry")
}

/// Human display only; executable operands must not use this lossy value.
fn display_param(key: &str, value: String) -> String {
    if is_title_param(key) || key == "stem" {
        names::display_text(&value)
    } else {
        value
    }
}

/// Positional paths must survive both shell and CLI option parsing. Native
/// spelling is otherwise left alone; option values use `--key={value}` in
/// the templates and need only shell quoting.
fn quote_value(key: &str, value: &str) -> String {
    if matches!(key, "path_arg" | "dir_arg" | "parent_arg") && value.starts_with('-') {
        names::shell_quote(&format!("./{value}"))
    } else {
        names::shell_quote(value)
    }
}

/// Substitute `{key}` placeholders. Explicit params win (first occurrence),
/// then the context keys; unknown placeholders stay literal so a missing
/// value is visible instead of silently dropped. Human titles and stems
/// are sanitized, but a command must never address the sanitized sibling.
fn subst(template: &str, params: &[(&'static str, String)], ctx: &Context, quote: bool) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) => {
                let key = &after[..close];
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    if let Some(v) = resolve(key, params, ctx) {
                        // §9: `{argv}` is the command line with every
                        // argument already §8.3.4-quoted and joined by
                        // single spaces (`Context::argv_from_args`); it
                        // embeds verbatim. Re-quoting would collapse the
                        // whole line into one giant single-quoted program
                        // name.
                        if quote && key != "argv" {
                            if is_title_param(key) && v.chars().any(char::is_control) {
                                return catalog::FIX_CONTROL_HEADING.to_string();
                            }
                            // Readable stored statuses need not satisfy CLI
                            // admission. Do not emit a rejected operand or
                            // substitute another, normalized status identity.
                            if key == "status" && !names::valid_status(&v) {
                                return catalog::FIX_INVALID_STATUS.to_string();
                            }
                            out.push_str(&quote_value(key, &v));
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
    out
}

fn resolve(key: &str, params: &[(&'static str, String)], ctx: &Context) -> Option<String> {
    if let Some(v) = get(params, key) {
        return Some(v.clone());
    }
    let operand = match key {
        "path_arg" => Some(&ctx.path_arg),
        "dir_arg" => Some(&ctx.dir_arg),
        _ => None,
    };
    if let Some(operand) = operand {
        return (!operand.as_os_str().is_empty()).then(|| operand.to_string_lossy().into_owned());
    }
    let v = match key {
        "path" => &ctx.path,
        "dir" => &ctx.dir,
        "stem" => &ctx.stem,
        "argv" => &ctx.argv,
        "command" => &ctx.command,
        "now" => &ctx.now,
        _ => return None,
    };
    // an unset context field stays as the literal placeholder rather than
    // silently vanishing (§9 supplement 7 spirit)
    if v.is_empty() {
        None
    } else {
        Some(v.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        Context {
            path: "docs/standup".into(),
            dir: "docs".into(),
            path_arg: "docs/standup.agent-besties.md".into(),
            dir_arg: "docs".into(),
            state_file_selected: false,
            stem: "standup".into(),
            argv: "agent-coexist check docs/standup".into(),
            command: "check".into(),
            now: "2026-09-17T10:00:00Z".into(),
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
    fn etype_exhausted_variant_takes_over() {
        // the exhausted param selects the exhausted variant, not the
        // renumber form (whose {next} key has no value here)
        let d = Diagnostic::new("E-SEQ-DUP", 9)
            .with("n", "3")
            .with("exhausted", "1");
        let r = render(&d, &ctx());
        assert!(
            r.message.contains("#3"),
            "seq param dropped: {s}",
            s = r.message
        );
        assert!(
            r.fix.contains("new post"),
            "exhausted recovery missing: {s}",
            s = r.fix
        );
        assert!(!r.fix.contains("{next}"), "unresolved key: {s}", s = r.fix);
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
            r.fix.contains("alice.agent-besties.md"),
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
                r.fix.contains("alice.agent-besties.md"),
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

        let brief = Diagnostic::new("W-BODY", 8)
            .with("unit", "entry \"lexer\"")
            .with("n", "8")
            .with("title", "t\u{1b}itle");
        let r = render(&brief, &ctx());
        assert!(
            !r.fix.contains('\u{1b}'),
            "W-BODY entry title must not leak ESC: {}",
            r.fix
        );
    }

    #[test]
    fn unknown_placeholder_stays_literal() {
        let d = Diagnostic::new("E-LOCKED", 0);
        let c = Context::default();
        let r = render(&d, &c);
        assert!(r.fix.contains("{argv}")); // argv empty -> stays literal, visible
    }
}
