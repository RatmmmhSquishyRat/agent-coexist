//! agent-profile section view (spec §5).
//!
//! Every H2 is a section; the name must match SECTION. Sections without a
//! fence get W-BODY. Property lines have no meaning here (§2.6.1): any
//! non-blank line outside a fence is stray.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::doc::Section;
use crate::error::Diagnostic;
use crate::names;
use crate::types::{is_blank_line, ScanCtx};

/// One profile section (spec §5.2).
#[derive(Debug, Clone)]
pub struct ProfileSection {
    /// Folded H2 title.
    pub name: String,
    /// Index into `Document::sections`.
    pub section: usize,
    /// Whether `name` matches SECTION — only valid names can be addressed
    /// by `--section` or participate in lock semantics.
    pub valid_name: bool,
}

/// §9 supplement 8: suggest a valid section name — fold, spaces to `-`,
/// delete characters outside the SECTION alphabet, drop leading `-`
/// (SECTION cannot start with it), truncate to 64 chars, fall back to
/// `section`.
pub(crate) fn suggested_section_name(title: &str) -> String {
    static NON_SECTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^\p{L}\p{N}_-]").unwrap());
    let dashed = names::fold(title).replace(' ', "-");
    let stripped = NON_SECTION.replace_all(&dashed, "");
    let trimmed = stripped.trim_start_matches('-');
    let mut out: String = trimmed.chars().take(64).collect();
    if out.is_empty() {
        out.push_str("section");
    }
    out
}

pub(crate) fn scan(ctx: &mut ScanCtx, sections: &[Section]) -> Vec<ProfileSection> {
    let suggestions: Vec<_> = sections
        .iter()
        .map(|sec| (!names::valid_section(&sec.title)).then(|| suggested_section_name(&sec.title)))
        .collect();
    // Reserve every stored folded heading, including later sections. A
    // candidate is safe only if no stored heading or other repair uses it.
    let mut unique_names: HashMap<&str, bool> = sections
        .iter()
        .map(|sec| (sec.title.as_str(), false))
        .collect();
    for suggested in suggestions.iter().flatten() {
        unique_names
            .entry(suggested.as_str())
            .and_modify(|unique| *unique = false)
            .or_insert(true);
    }
    let mut seen: HashSet<&str> = HashSet::new();
    let mut out = Vec::with_capacity(sections.len());
    for (si, (sec, suggested)) in sections.iter().zip(&suggestions).enumerate() {
        let head_idx = ctx.idx_at(sec.head.start);
        let head_line = head_idx + 1;
        if !seen.insert(sec.title.as_str()) {
            ctx.push(
                Diagnostic::new("E-SECTION-DUP", head_line)
                    .with("name", names::display_text(&sec.title)),
            );
        }
        let valid = suggested.is_none();
        if let Some(suggested) = suggested {
            let mut diagnostic = Diagnostic::new("W-SECTION-NAME", head_line)
                .with("name", names::display_text(&sec.title))
                .with("n", head_line.to_string());
            if unique_names.get(suggested.as_str()) == Some(&true) {
                diagnostic = diagnostic.with("suggested", suggested.clone());
            }
            ctx.push(diagnostic);
        }
        if sec.body.is_none() && !ctx.holds_unclosed_fence(sec) {
            ctx.push(
                Diagnostic::new("W-BODY", head_line)
                    .with(
                        "unit",
                        format!("section \"{}\"", names::display_text(&sec.title)),
                    )
                    .with("n", head_line.to_string())
                    .with("name", sec.title.clone()),
            );
        }
        for idx in ctx.section_body_lines(sec) {
            if ctx.fenced(idx) {
                continue;
            }
            let line = ctx.line_str(idx);
            if is_blank_line(line) {
                continue;
            }
            ctx.push(Diagnostic::new("W-STRAY", idx + 1).with("n", (idx + 1).to_string()));
        }
        out.push(ProfileSection {
            name: sec.title.clone(),
            section: si,
            valid_name: valid,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::suggested_section_name;
    use crate::doc::{self, Document};
    use std::path::Path;

    fn parse_profile(src: &str) -> Document {
        doc::parse(
            src.as_bytes().to_vec(),
            Path::new("a"),
            "2026-09-17T10:00:00Z".parse().unwrap(),
        )
        .unwrap()
    }

    fn ids(d: &Document) -> Vec<&'static str> {
        d.diagnostics.iter().map(|x| x.id).collect()
    }
    #[test]
    fn invalid_names_get_suggestions_and_bodies_flagged() {
        let src = "# a.agent-profile\n\n## my role 9!\n\n## empty-one\n";
        let d = parse_profile(src);
        // same line 3: W-BODY precedes W-SECTION-NAME in catalog order;
        // `empty-one` is a valid SECTION, its section just has no body
        assert_eq!(ids(&d), vec!["W-BODY", "W-SECTION-NAME", "W-BODY"]);
        assert_eq!(d.diagnostics[0].line, 3);
        let w = &d.diagnostics[1];
        assert_eq!(w.line, 3);
        let name = w.params.iter().find(|(k, _)| *k == "name").unwrap();
        let sugg = w.params.iter().find(|(k, _)| *k == "suggested").unwrap();
        assert_eq!(name.1, "my role 9!");
        assert_eq!(sugg.1, "my-role-9");
        assert!(!d.profile[0].valid_name);
        assert!(d.profile[1].valid_name);
    }

    #[test]
    fn duplicate_sections_and_stray_property_lines() {
        let src =
            "# a.agent-profile\n\n## role\n\n```md\nx\n```\n\n## role\n- path: no semantics here\n";
        let d = parse_profile(src);
        // the duplicate section has no fence: W-BODY (§5.2.2) alongside
        // E-SECTION-DUP at the same line; the property line is W-STRAY
        // (no property semantics in profiles, §2.6.1)
        assert_eq!(ids(&d), vec!["E-SECTION-DUP", "W-BODY", "W-STRAY"]);
        assert_eq!(d.diagnostics[0].line, 9);
        assert_eq!(d.diagnostics[1].line, 9);
        assert_eq!(d.diagnostics[2].line, 10);
    }

    #[test]
    fn suggestion_algorithm_edges() {
        // dashes stay (SECTION allows them after the first char, §2.3);
        // only LEADING dashes are dropped (§9 suppl. 8)
        assert_eq!(suggested_section_name("  --- Role ---  "), "Role----");
        assert_eq!(suggested_section_name("!!!"), "section");
        assert_eq!(suggested_section_name("a b c"), "a-b-c");
    }

    #[test]
    fn preamble_stray_exempts_reader_comments_and_fences() {
        let d = parse_profile("# a.agent-profile\n\njunk preamble\n\n## role\n\n```md\ndev\n```\n");
        assert_eq!(
            d.diagnostics
                .iter()
                .map(|x| (x.id, x.line))
                .collect::<Vec<_>>(),
            vec![("W-STRAY", 3)]
        );
        assert_eq!(d.profile.len(), 1);
        assert_eq!(d.profile[0].name, "role");

        let d = parse_profile("# a.agent-profile\n\n> note\n\n## role\n\n```md\ndev\n```\n");
        assert_eq!(ids(&d), Vec::<&str>::new());

        // fences in the preamble stay opaque: inner text is not W-STRAY
        let d = parse_profile(
            "# a.agent-profile\n\n```md\nhidden junk\n```\n\n## role\n\n```md\ndev\n```\n",
        );
        assert_eq!(ids(&d), Vec::<&str>::new());
    }

    #[test]
    fn unterminated_last_section_line_is_stray_phantom_is_not() {
        let d = parse_profile("# a.agent-profile\n\n## role\nhello");
        assert_eq!(
            d.diagnostics
                .iter()
                .map(|x| (x.id, x.line))
                .collect::<Vec<_>>(),
            vec![("W-BODY", 3), ("W-STRAY", 4)]
        );

        let d = parse_profile("# a.agent-profile\n\n## role\nhello\n");
        let strays: Vec<_> = d
            .diagnostics
            .iter()
            .filter(|x| x.id == "W-STRAY")
            .map(|x| x.line)
            .collect();
        assert_eq!(strays, vec![4]);
    }
}
