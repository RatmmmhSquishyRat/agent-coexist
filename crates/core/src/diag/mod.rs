//! Diagnostic classification and §9 rendering.
//!
//! `catalog.rs` is the single source of the §9 template text; this module
//! classifies ids, orders them for output, and turns a `Diagnostic` into the
//! `message` / `why` / `fix` triple. Placeholder values coming from file
//! content are already folded by whoever created the diagnostic (§10.9.4);
//! runtime values are substituted from the [`Context`]. In `fix` templates —
//! the command form (§8.3.4) — every runtime value is POSIX-quoted here, so
//! no caller ever writes its own quoting. The E-H1 and W-NAME fixes are
//! the exceptions (2026-09-25 repair): their suggestions are file content,
//! not command arguments, so they substitute as plain text and never quote
//! the stem ([`fix_text`], [`heading_fix`]).

mod catalog;

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

/// Runtime context for template substitution. `path` is the display path
/// WITHOUT the managed suffix; `now` is the TIME string of the command's
/// clock; `argv` must be built with [`Context::argv_from_args`] so the
/// per-argument quoting exists only here.
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub path: String,
    pub dir: String,
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
            message: subst(t.message, params, ctx, false),
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

/// Fix text with the context-sensitive overrides (§9 supplements 1 and 3;
/// E-TYPE / E-H1 / W-NAME per the 2026-09-25 repair contract).
fn fix_text(
    id: &str,
    t: &catalog::Template,
    params: &[(&'static str, String)],
    ctx: &Context,
) -> String {
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
        "E-H1" => return eh1_fix(t, params, ctx),
        "W-NAME" => return wname_fix(t, params, ctx),
        "W-BODY" => return wbody_fix(params, ctx),
        _ => t.fix,
    };
    subst(template, params, ctx, true)
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
/// profile `name`, brief `title`.
fn wbody_fix(params: &[(&'static str, String)], ctx: &Context) -> String {
    let path = names::shell_quote(&ctx.path);
    if let Some(seq) = get(params, "seq") {
        let author = get(params, "author")
            .map(|a| quote_value(a))
            .unwrap_or_else(|| "<nick>".to_string());
        format!(
            "agent-coexist post {path} --as {author} --edit {} -m '<text>'",
            quote_value(seq)
        )
    } else if let Some(name) = get(params, "name") {
        format!(
            "agent-coexist profile {path} --section {} -m '<text>'",
            quote_value(name)
        )
    } else if let Some(title) = get(params, "title") {
        format!(
            "agent-coexist brief {path} --entry {} -m '<note>'",
            quote_value(title)
        )
    } else {
        format!("agent-coexist profile {path} --section <name> -m '<text>'")
    }
}

fn get<'a>(params: &'a [(&'static str, String)], key: &str) -> Option<&'a String> {
    params.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
}

/// Reader placeholders (§9 supplement 7) stay literal even inside fixes.
fn quote_value(v: &str) -> String {
    if v == "<file>" || v == "<nick>" {
        v.to_string()
    } else {
        names::shell_quote(v)
    }
}

/// Substitute `{key}` placeholders. Explicit params win (first occurrence),
/// then the context keys; unknown placeholders stay literal so a missing
/// value is visible instead of silently dropped. `quote` applies §8.3.4
/// POSIX quoting (fix lines only).
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
                            out.push_str(&quote_value(&v));
                        } else {
                            out.push_str(&v);
                        }
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
    fn render_substitutes_params_and_context() {
        let d = Diagnostic::new("E-H1", 1)
            .with("line", "# Notes")
            .with("stem", "notes");
        let r = render(&d, &ctx());
        assert!(
            r.message.contains("# Notes"),
            "line param dropped: {s}",
            s = r.message
        );
        assert!(
            r.fix.contains("# notes.post"),
            "stem param dropped: {s}",
            s = r.fix
        );

        let d = Diagnostic::new("W-STRAY", 20).with("n", "20");
        let r = render(&d, &ctx());
        assert!(
            r.message.contains("L20"),
            "line param dropped: {s}",
            s = r.message
        );
    }

    #[test]
    fn fix_quotes_runtime_values() {
        let d = Diagnostic::new("E-ENTRY-NOTFOUND", 0).with("title", "my entry");
        let r = render(&d, &ctx());
        assert!(
            r.fix.contains("check docs/standup"),
            "plain path embedded: {s}",
            s = r.fix
        );
        let mut c = ctx();
        c.path = "my docs/stand up".into();
        let r = render(&d, &c);
        assert!(r.fix.contains("check 'my docs/stand up'"));
    }

    #[test]
    fn etype_fix_follows_actual_type_not_dir_presence() {
        // actual = state: the fix names the state command with the directory
        let r = reason(
            "E-TYPE",
            &[
                ("actual", "state".into()),
                ("expected", "post".into()),
                ("dir", "src/parser".into()),
            ],
            &ctx(),
        );
        assert!(
            r.message.contains("state"),
            "actual type missing: {s}",
            s = r.message
        );
        assert!(
            r.fix.contains("agent-coexist state src/parser"),
            "state fix missing dir: {s}",
            s = r.fix
        );

        // actual != state with a dir param present (the post.rs / query
        // fault shapes): the fix names the command for the ACTUAL type and
        // resolves every key; no state wording about a non-state file
        for (actual, cmd) in [
            ("post", "post"),
            ("brief", "brief"),
            ("agent-profile", "profile"),
        ] {
            let r = reason(
                "E-TYPE",
                &[
                    ("actual", actual.into()),
                    ("expected", "brief".into()),
                    ("cmd_for_actual", cmd.into()),
                    ("dir", "docs".into()),
                ],
                &ctx(),
            );
            assert!(
                r.message.contains(actual),
                "actual type missing: {s}",
                s = r.message
            );
            assert!(
                r.fix.contains(&format!("agent-coexist {cmd}")),
                "actual command missing: {s}",
                s = r.fix
            );
            assert!(
                !r.fix.contains("state"),
                "state wording on {actual}: {s}",
                s = r.fix
            );
            assert!(!r.fix.contains('{'), "unresolved key: {s}", s = r.fix);
        }
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
        assert!(!r.fix.contains('\''), "quoted H1 guidance: {s}", s = r.fix);

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
            assert!(!r.fix.contains('\''), "quoted H1 guidance: {s}", s = r.fix);
        }
    }

    #[test]
    fn eh1_fix_never_suggests_an_invalid_or_quoted_stem() {
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
            assert!(
                !r.fix.contains('\''),
                "quote characters in H1 guidance: {}",
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

        // a valid stem still gets the copyable suggestion, unquoted
        let d = Diagnostic::new("E-H1", 1).with("line", "# Notes");
        let mut c = ctx();
        c.stem = "stand up".into(); // invalid: space
        let r = render(&d, &c);
        assert!(!r.fix.contains("# stand up.post"));
        assert!(!r.fix.contains("# 'stand up'"));

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
    fn reason_errors_ename_and_estatedirty_resolve_fully() {
        // E-NAME: the invalid stem lands in the message; the fix carries
        // no runtime value, so nothing can stay unresolved
        let r = reason("E-NAME", &[("name", "my docs/x".into())], &ctx());
        assert!(
            r.message.contains("my docs/x"),
            "name param dropped: {s}",
            s = r.message
        );
        assert!(!r.fix.contains('{') && !r.fix.contains('}'));
        assert!(r.fix.to_lowercase().contains("rename"));

        // E-STATE-DIRTY: nick/dir/line resolve in message and fix; dir is
        // a command argument in the fix, so §8.3.4 quoting applies there
        // only when it actually carries shell-special characters
        let r = reason(
            "E-STATE-DIRTY",
            &[
                ("nick", "bob".into()),
                ("dir", "src/parser".into()),
                ("line", "12".into()),
            ],
            &ctx(),
        );
        assert!(r.message.contains("bob"));
        assert!(r.message.contains("line 12"));
        assert!(r.message.contains("src/parser"));
        assert!(r
            .fix
            .contains("agent-coexist state src/parser --remove bob"));
        assert!(!r.fix.contains('{') && !r.fix.contains('}'));

        let r = reason(
            "E-STATE-DIRTY",
            &[
                ("nick", "bob".into()),
                ("dir", "my src/parser".into()),
                ("line", "12".into()),
            ],
            &ctx(),
        );
        assert!(r
            .fix
            .contains("agent-coexist state 'my src/parser' --remove bob"));
    }

    #[test]
    fn wbody_composition_per_type() {
        let d = Diagnostic::new("W-BODY", 10)
            .with("unit", "comment #2")
            .with("n", "10")
            .with("seq", "2")
            .with("author", "bob");
        let r = render(&d, &ctx());
        assert_eq!(r.message, "comment #2 at L10 has no fenced body");
        assert_eq!(
            r.fix,
            "agent-coexist post docs/standup --as bob --edit 2 -m '<text>'"
        );

        let d = Diagnostic::new("W-BODY", 6)
            .with("unit", "section \"role\"")
            .with("n", "6")
            .with("name", "role");
        let r = render(&d, &ctx());
        assert_eq!(
            r.fix,
            "agent-coexist profile docs/standup --section role -m '<text>'"
        );

        let d = Diagnostic::new("W-BODY", 8)
            .with("unit", "entry \"lexer\"")
            .with("n", "8")
            .with("title", "lexer");
        let r = render(&d, &ctx());
        assert_eq!(
            r.fix,
            "agent-coexist brief docs/standup --entry lexer -m '<note>'"
        );
    }

    #[test]
    fn argv_from_args_quotes_each_argument() {
        let argv = Context::argv_from_args(&[
            "agent-coexist".into(),
            "brief".into(),
            "my docs/x".into(),
            "-m".into(),
            "it's".into(),
        ]);
        // §8.3.4 quotes per argument, §9 joins with single spaces: plain
        // arguments stay bare, whitespace/quote-carrying ones are
        // single-quoted with the '\'' escape, so pasting the line into a
        // POSIX shell yields the same five arguments.
        assert_eq!(argv, "agent-coexist brief 'my docs/x' -m 'it'\\''s'");

        // {argv} reaches fix templates already quoted; substitution embeds
        // it verbatim. Whatever the fix prose is, swapping only the argv
        // value must be the only difference between the two renders —
        // re-quoting fails this by wrapping the line in extra quotes.
        let d = Diagnostic::new("E-LOCKED", 0);
        let base = ctx(); // argv: "agent-coexist check docs/standup"
        let plain = render(&d, &base).fix;
        let mut c = base.clone();
        c.argv = argv;
        let r = render(&d, &c);
        assert_eq!(r.fix, plain.replace(&base.argv, &c.argv));
    }

    #[test]
    fn unknown_placeholder_stays_literal() {
        let d = Diagnostic::new("E-LOCKED", 0);
        let c = Context::default();
        let r = render(&d, &c);
        assert!(r.fix.contains("{argv}")); // argv empty -> stays literal, visible
    }
}
