//! Diagnostic classification and §9 rendering.
//!
//! `catalog.rs` is the single source of the §9 template text; this module
//! classifies ids, orders them for output, and turns a `Diagnostic` into the
//! `message` / `why` / `fix` triple. Placeholder values coming from file
//! content are already folded by whoever created the diagnostic (§10.9.4);
//! runtime values are substituted from the [`Context`]. In `fix` templates —
//! the command form (§8.3.4) — every runtime value is POSIX-quoted here, so
//! no caller ever writes its own quoting.

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

/// Fix text with the context-sensitive overrides (§9 supplements 1 and 3).
fn fix_text(
    id: &str,
    t: &catalog::Template,
    params: &[(&'static str, String)],
    ctx: &Context,
) -> String {
    let template: &str = match id {
        "E-TYPE" => {
            if get(params, "dir").is_some() {
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
        "W-BODY" => return wbody_fix(params, ctx),
        _ => t.fix,
    };
    subst(template, params, ctx, true)
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
        assert!(order("E-UTF8") < order("W-BOM"));
        assert!(order("W-STRAY") < order("W-SUB-DUP"));
        assert_eq!(order("W-STATE-EXPIRED") - order("E-UTF8"), 49); // 50 rows
    }

    #[test]
    fn render_substitutes_params_and_context() {
        let d = Diagnostic::new("E-H1", 1)
            .with("line", "# Notes")
            .with("stem", "notes");
        let r = render(&d, &ctx());
        assert_eq!(
            r.message,
            "first line is \"# Notes\", not \"# <name>.<type>\""
        );
        assert_eq!(
            r.fix,
            "set line 1 to \"# notes.post\" (or .brief / .agent-profile), or \"# .state\" for a state file"
        );

        let d = Diagnostic::new("W-STRAY", 20).with("n", "20");
        let r = render(&d, &ctx());
        assert_eq!(
            r.message,
            "text outside any fence at L20 is not part of the record"
        );
    }

    #[test]
    fn fix_quotes_runtime_values() {
        let d = Diagnostic::new("E-ENTRY-NOTFOUND", 0).with("title", "my entry");
        let r = render(&d, &ctx());
        assert_eq!(
            r.fix,
            "pick an existing title from: agent-coexist check docs/standup"
        );
        let mut c = ctx();
        c.path = "my docs/stand up".into();
        let r = render(&d, &c);
        assert!(r.fix.contains("check 'my docs/stand up'"));
    }

    #[test]
    fn etype_state_override_and_exhaustion() {
        let r = reason(
            "E-TYPE",
            &[
                ("actual", "state".into()),
                ("expected", "post".into()),
                ("dir", "src/parser".into()),
            ],
            &ctx(),
        );
        assert_eq!(
            r.fix,
            "this is a state file: agent-coexist state src/parser --as <nick> --status <name>"
        );
        assert_eq!(r.message, "file is a state, not a post");

        let d = Diagnostic::new("E-SEQ-DUP", 9)
            .with("n", "3")
            .with("exhausted", "1");
        let r = render(&d, &ctx());
        assert_eq!(
            r.fix,
            "move the duplicate comment to a new post and update references to it"
        );
        assert_eq!(r.message, "comment number #3 appears twice");
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
        assert_eq!(r.fix, "retry shortly: {argv}"); // argv empty -> unresolved
    }
}
