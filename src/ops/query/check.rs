//! `check <PATH>` — the single-file report (spec §7.2, §8.2 `check`).
//!
//! Row order per §8.1.13: `file` → `type` (when the H1 kind survived)
//! → every applicable §9 diagnostic in anchor order → the per-type
//! counts and previews. Without an H1 there is nothing to preview
//! (§2.2.3, §8.2 check example 3). First-line status comes from the
//! structured diagnostic set plus the §8.1.11 facts, never from
//! formatted text.

use std::collections::HashSet;
use std::path::Path;

use crate::Time;
use crate::consts;
use crate::diag;
use crate::doc::{self, Document};
use crate::error::{Diagnostic, Fault};
use crate::hash;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::{CheckParams, reason_report};
use crate::report::{Report, Status};
use crate::time;
use crate::types;

use super::{
    LoadError, blocking_report, body_excerpt, file_missing_report, load_file, sort_diagnostics,
};

pub fn check(ctx: &mut Ctx, p: &CheckParams) -> Result<Report, Fault> {
    let object = context::object_of_path(&p.path);
    let dctx = context::diag_context_for_file(ctx, "check", &p.path);
    let file_display = context::display_file(&p.path);
    match load_file(&p.path, ctx.now, consts::READ_WAIT_MS) {
        Err(LoadError::Missing) => Ok(file_missing_report(&dctx, "check", &object, &p.path)),
        Err(LoadError::Busy { id, params }) => {
            Ok(reason_report(&dctx, "check", &object, id, params))
        }
        Err(LoadError::Blocking(diags)) => Ok(blocking_report(
            "check",
            &object,
            &file_display,
            None,
            diags,
        )),
        Ok(document) => match document.kind {
            None => Ok(blocking_report(
                "check",
                &object,
                &file_display,
                None,
                document.diagnostics,
            )),
            Some(kind) => Ok(report_for(
                ctx,
                &p.path,
                &object,
                &file_display,
                kind,
                document,
            )),
        },
        Err(LoadError::Fault(f)) => Err(f),
    }
}

fn report_for(
    ctx: &Ctx,
    path: &Path,
    object: &str,
    file_display: &str,
    kind: doc::Kind,
    document: Document,
) -> Report {
    let mut report = Report::new("check", object);
    report.kv("file", vec![file_display.to_string()]);
    report.kv("type", vec![kind.as_str().to_string()]);

    let mut warn_floor = false;
    let (has_error, has_warn) = match kind {
        doc::Kind::Post => {
            let flags = push_diagnostics(&mut report, document.diagnostics.clone());
            post_rows(&mut report, &document);
            flags
        }
        doc::Kind::Brief => {
            // §4.4: one verdict per entry through the shared decision
            // order; verify supplies the target-read diagnostics.
            let verifications: Vec<hash::Verification> = document
                .brief
                .entries
                .iter()
                .map(|entry| hash::verify(path, entry))
                .collect();
            let mut diags = document.diagnostics.clone();
            for verification in &verifications {
                diags.extend(verification.diagnostics.iter().cloned());
            }
            let flags = push_diagnostics(&mut report, diags);
            warn_floor = brief_rows(&mut report, &document, &verifications);
            flags
        }
        doc::Kind::Profile => {
            let flags = push_diagnostics(&mut report, document.diagnostics.clone());
            profile_rows(&mut report, &document);
            flags
        }
        doc::Kind::State => {
            // Expand the complete pairwise conflict report at emission,
            // alongside preview marks over the same selected declarations.
            let mut diags = document.diagnostics.clone();
            let (count, rows) = state_preview(ctx.now, &document, &mut diags);
            let flags = push_diagnostics(&mut report, diags);
            report.kv("entries", vec![count.to_string()]);
            for (status, values) in rows {
                report.record(status, values, 2, consts::KEY_WIDTH);
            }
            flags
        }
    };

    // §8.1.11 from structured facts. `unchecked` without a non-blocking
    // `E-*` (target too large) is the only preview-level warn source.
    if has_error {
        report.status = Status::Error;
        report.exit = 1;
    } else if has_warn || warn_floor {
        report.warn();
    }
    report
}

/// Push the sorted diagnostic rows, reporting whether any error-level
/// and any warn-level diagnostic was seen (§9 supplement 4 order).
fn push_diagnostics(report: &mut Report, diags: Vec<Diagnostic>) -> (bool, bool) {
    let mut has_error = false;
    let mut has_warn = false;
    for diagnostic in sort_diagnostics(diags) {
        if diag::is_error(diagnostic.id) {
            has_error = true;
        } else {
            has_warn = true;
        }
        report.diagnostic(diagnostic);
    }
    (has_error, has_warn)
}

fn post_rows(report: &mut Report, document: &Document) {
    let post = &document.post;
    report.kv("comments", vec![post.comments.len().to_string()]);
    let mut authors: Vec<&str> = Vec::new();
    let mut seen = HashSet::new();
    for comment in &post.comments {
        if seen.insert(comment.nick.as_str()) {
            authors.push(comment.nick.as_str());
        }
    }
    report.kv(
        "authors",
        vec![if authors.is_empty() {
            "none".to_string()
        } else {
            authors.join(", ")
        }],
    );
    if let Some(first) = post.comments.first() {
        let mut values = vec![format!("#{}", first.seq), first.nick.clone()];
        if let Some(excerpt) = body_excerpt(document, first.section) {
            values.push(excerpt);
        }
        report.kv("topic", values);
    }
    if let Some(last) = post.comments.last() {
        let mut values = vec![
            format!("#{}", last.seq),
            last.nick.clone(),
            time::format(last.activity()),
        ];
        if let Some(excerpt) = body_excerpt(document, last.section) {
            values.push(excerpt);
        }
        report.kv("latest", values);
    }
    report.kv(
        "subscribers",
        vec![if post.subscribers.is_empty() {
            "none".to_string()
        } else {
            post.subscribers.join(", ")
        }],
    );
}

/// Emit the brief previews; returns the §8.1.11 `unchecked` warn floor.
fn brief_rows(
    report: &mut Report,
    document: &Document,
    verifications: &[hash::Verification],
) -> bool {
    if let Some(body) = &document.brief.description {
        let excerpt = crate::names::excerpt(document.text(&body.content));
        if excerpt.is_empty() {
            report.kv("description", Vec::new());
        } else {
            report.kv("description", vec![excerpt]);
        }
    }
    let mut fresh = 0usize;
    let mut shifted = 0usize;
    let mut stale = 0usize;
    let mut unchecked = 0usize;
    for verification in verifications {
        match verification.state {
            "fresh" => fresh += 1,
            "shifted" => shifted += 1,
            "stale" => stale += 1,
            _ => unchecked += 1,
        }
    }
    report.kv("entries", vec![document.brief.entries.len().to_string()]);
    report.kv("fresh", vec![fresh.to_string()]);
    report.kv("shifted", vec![shifted.to_string()]);
    report.kv("stale", vec![stale.to_string()]);
    if unchecked > 0 {
        report.kv("unchecked", vec![unchecked.to_string()]);
    }
    for (entry, verification) in document.brief.entries.iter().zip(verifications.iter()) {
        let mut values = vec![
            names::display_text(&entry.title),
            verification.state.to_string(),
            entry
                .path
                .as_ref()
                .map(|property| {
                    crate::names::display_text(&hash::normalized_target(&property.value))
                })
                .unwrap_or_else(|| "?".to_string()),
        ];
        if let Some(reason) = verification.reason {
            values.push(reason.to_string());
        }
        if let Some(excerpt) = body_excerpt(document, entry.section) {
            values.push(excerpt);
        }
        report.kv("entry", values);
    }
    unchecked > 0
}

fn profile_rows(report: &mut Report, document: &Document) {
    report.kv("sections", vec![document.profile.len().to_string()]);
    for section in &document.profile {
        let mut values = vec![names::display_text(&section.name)];
        if let Some(excerpt) = body_excerpt(document, section.section) {
            values.push(excerpt);
        }
        report.kv("section", values);
    }
}

/// Same-file state preview (§6.2.7, §8.2 check state): display uses
/// [`crate::types::state::selected`] across the file (all parseable
/// declarations stay candidates, dirty-line fragments included) and
/// renders the §8.1.11 `conflict` / `expired` marks for that same
/// selected set. The scan supplies expiry diagnostics; this check-only
/// boundary adds the complete pairwise conflict diagnostics while marking
/// their preview rows, without expanding discarded reports during parsing.
#[allow(clippy::type_complexity)]
fn state_preview(
    now: Time,
    document: &Document,
    diagnostics: &mut Vec<Diagnostic>,
) -> (usize, Vec<(String, Vec<String>)>) {
    let flat = types::state::selected(&document.state);
    types::state::append_conflict_diagnostics(&flat, now, diagnostics);
    // Selection is unique per (status, nick): every valid locked entry
    // conflicts exactly when at least two remain, without another pair walk.
    let has_conflicts = flat
        .iter()
        .filter(|(section, declaration)| {
            section.status == "locked" && section.valid_name && declaration.valid(now)
        })
        .count()
        > 1;
    let rows = flat
        .iter()
        .map(|(section, decl)| {
            let mut values = vec![
                decl.nick.clone(),
                format!("{}..{}", time::format(decl.start), time::format(decl.until)),
            ];
            let valid = decl.valid(now);
            if has_conflicts && valid && section.status == "locked" && section.valid_name {
                values.push("conflict".to_string());
            } else if !valid {
                values.push("expired".to_string());
            }
            (names::display_text(&section.status), values)
        })
        .collect();
    (flat.len(), rows)
}

#[cfg(test)]
mod tests {
    use crate::report::{Report, Row};

    use super::*;
    use crate::doc;
    use std::path::Path;

    fn state_report(src: &str, now: &str) -> Report {
        let now = crate::time::parse(now).unwrap();
        let path = Path::new("state.agent-coexist.md");
        let document = doc::parse(src.as_bytes().to_vec(), path, now).expect("parses");
        report_for(
            &context::test_ctx(now),
            path,
            "state",
            "state.agent-coexist.md",
            doc::Kind::State,
            document,
        )
    }

    fn ids(report: &Report) -> Vec<(&'static str, usize)> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Diagnostic(diagnostic) => Some((diagnostic.id, diagnostic.line)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn state_expiry_and_conflicts_use_selected_declarations_across_sections() {
        // The renewal supersedes alice's expired declaration in an earlier
        // section. Dave's legal fragment on a dirty line still conflicts.
        let report = state_report(
            concat!(
                "# .state\n",
                "\n",
                "## locked\n",
                "alice 2026-09-17T08:00:00Z..2026-09-17T09:00:00Z\n",
                "## locked\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
                "bob 2026-09-17T08:00:00Z..2026-09-17T09:30:00Z\n",
                "dave 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, broken entry\n",
                "gina 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            ),
            "2026-09-17T10:00:00Z",
        );
        assert_eq!(
            ids(&report),
            vec![
                ("E-STATUS-DUP", 5),
                ("W-STATE-DUP", 6),
                ("W-STATE-EXPIRED", 7),
                ("W-STATE-ENTRY", 8),
                ("W-STATE-CONFLICT", 8),
                ("W-STATE-CONFLICT", 9),
                ("W-STATE-CONFLICT", 9),
            ]
        );
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 1);
        assert_eq!(
            field_values(&report, "entries"),
            vec![vec!["4".to_string()]]
        );
        let rows = field_values(&report, "locked");
        let marks: Vec<(&str, Option<&str>)> = rows
            .iter()
            .map(|values| (values[0].as_str(), values.get(2).map(String::as_str)))
            .collect();
        assert_eq!(
            marks,
            vec![
                ("alice", Some("conflict")),
                ("bob", Some("expired")),
                ("dave", Some("conflict")),
                ("gina", Some("conflict")),
            ]
        );
    }

    #[test]
    fn state_conflicts_warn_with_every_pair_in_anchor_order() {
        // Same-line pairs preserve declaration order, not nickname order;
        // sorting by anchor must not interleave the later dave pairs.
        let report = state_report(
            concat!(
                "# .state\n",
                "\n",
                "## locked\n",
                "bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, ",
                "carol 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z, ",
                "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
                "dave 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            ),
            "2026-09-17T10:00:00Z",
        );
        assert_eq!(report.status, Status::Warn);
        assert_eq!(report.exit, 0);
        let diagnostics: Vec<_> = report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Diagnostic(diagnostic) => {
                    let param = |key| {
                        diagnostic
                            .params
                            .iter()
                            .find(|(name, _)| *name == key)
                            .unwrap()
                            .1
                            .to_str()
                            .unwrap()
                    };
                    Some((diagnostic.id, diagnostic.line, param("a"), param("b")))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            diagnostics,
            vec![
                ("W-STATE-CONFLICT", 4, "bob", "carol"),
                ("W-STATE-CONFLICT", 4, "bob", "alice"),
                ("W-STATE-CONFLICT", 4, "carol", "alice"),
                ("W-STATE-CONFLICT", 5, "bob", "dave"),
                ("W-STATE-CONFLICT", 5, "carol", "dave"),
                ("W-STATE-CONFLICT", 5, "alice", "dave"),
            ]
        );
    }

    #[test]
    fn state_reversed_range_marks_expired_without_a_second_warning() {
        // §6.2.5: start > until counts as expired — the row is marked,
        // but the only diagnostic is the per-entry W-STATE-RANGE.
        let report = state_report(
            concat!(
                "# .state\n",
                "\n",
                "## locked\n",
                "fred 2026-09-17T14:00:00Z..2026-09-17T13:00:00Z\n",
            ),
            "2026-09-17T10:00:00Z",
        );
        assert_eq!(ids(&report), vec![("W-STATE-RANGE", 4)]);
        assert_eq!(
            field_values(&report, "entries"),
            vec![vec!["1".to_string()]]
        );
        assert_eq!(field_values(&report, "locked")[0][2], "expired");
    }

    #[test]
    fn state_esc_status_headings_show_replacement_without_merging_raw_identities() {
        // Handwritten ESC/BEL headings remain distinct raw statuses;
        // display_text sanitizes them equally. The preview row shown to
        // consumers must carry U+FFFD and must not leak the controls.
        // Selection and locked matching stay on the raw identities, so
        // the headings are not merged and are never valid locked status.
        let report = state_report(
            concat!(
                "# .state\n",
                "\n",
                "## locked\u{1b}\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
                "## locked\u{07}\n",
                "alice 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
                "bob 2026-09-17T09:00:00Z..2026-09-17T12:00:00Z\n",
            ),
            "2026-09-17T10:00:00Z",
        );
        assert_eq!(
            field_values(&report, "entries"),
            vec![vec!["3".to_string()]]
        );
        let rows = field_values(&report, "locked\u{FFFD}");
        let preview: Vec<(&str, Option<&str>)> = rows
            .iter()
            .map(|values| (values[0].as_str(), values.get(2).map(String::as_str)))
            .collect();
        assert_eq!(
            preview,
            vec![("alice", None), ("alice", None), ("bob", None)]
        );
        assert_eq!(
            ids(&report),
            vec![("W-STATUS-NAME", 3), ("W-STATUS-NAME", 5)]
        );
    }

    fn field_values(report: &Report, key: &str) -> Vec<Vec<String>> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key: k, values, .. } if k == key => Some(values.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn brief_entry_titles_show_replacement_without_merging_raw_identities() {
        let document = doc::parse(
            concat!(
                "# notes.brief\n",
                "\n",
                "## t\u{1b}itle\n",
                "- path: a.txt\n",
                "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
                "## t\u{07}itle\n",
                "- path: b.txt\n",
                "- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
            )
            .as_bytes()
            .to_vec(),
            Path::new("notes.agent-coexist.md"),
            crate::time::parse("2026-09-17T10:00:00Z").unwrap(),
        )
        .expect("parses");
        let verifications: Vec<_> = document
            .brief
            .entries
            .iter()
            .map(|entry| hash::verify(Path::new("notes.agent-coexist.md"), entry))
            .collect();
        let mut report = Report::new("check", "notes");
        brief_rows(&mut report, &document, &verifications);
        let titles: Vec<String> = field_values(&report, "entry")
            .into_iter()
            .map(|values| values[0].clone())
            .collect();
        assert_eq!(
            titles,
            vec!["t\u{FFFD}itle".to_string(), "t\u{FFFD}itle".to_string()]
        );
        assert_eq!(document.brief.entries.len(), 2);
        assert_ne!(
            document.brief.entries[0].title,
            document.brief.entries[1].title
        );
        assert!(
            document
                .brief
                .entries
                .iter()
                .all(|entry| entry.title.contains('\u{1b}') || entry.title.contains('\u{07}'))
        );
    }

    #[test]
    fn profile_section_titles_show_replacement_without_merging_raw_identities() {
        let document = doc::parse(
            concat!(
                "# a.agent-profile\n",
                "\n",
                "## role\u{1b}\n",
                "\n",
                "```md\n",
                "dev\n",
                "```\n",
                "## role\u{07}\n",
                "\n",
                "```md\n",
                "ops\n",
                "```\n",
            )
            .as_bytes()
            .to_vec(),
            Path::new("a.agent-coexist.md"),
            crate::time::parse("2026-09-17T10:00:00Z").unwrap(),
        )
        .expect("parses");
        let mut report = Report::new("check", "a");
        profile_rows(&mut report, &document);
        let titles: Vec<String> = field_values(&report, "section")
            .into_iter()
            .map(|values| values[0].clone())
            .collect();
        assert_eq!(
            titles,
            vec!["role\u{FFFD}".to_string(), "role\u{FFFD}".to_string()]
        );
        assert_eq!(document.profile.len(), 2);
        assert_ne!(document.profile[0].name, document.profile[1].name);
        assert!(
            document
                .profile
                .iter()
                .all(|section| section.name.contains('\u{1b}') || section.name.contains('\u{07}'))
        );
    }

    #[test]
    fn post_authors_keep_first_occurrence_order() {
        let document = doc::parse(
            concat!(
                "# foo.post\n",
                "\n",
                "## #1 alice (2026-09-17T10:00:00Z)\n",
                "\n",
                "```md\n",
                "one\n",
                "```\n",
                "## #2 bob (2026-09-17T10:00:00Z)\n",
                "\n",
                "```md\n",
                "two\n",
                "```\n",
                "## #3 alice (2026-09-17T10:00:00Z)\n",
                "\n",
                "```md\n",
                "three\n",
                "```\n",
                "## #4 carol (2026-09-17T10:00:00Z)\n",
                "\n",
                "```md\n",
                "four\n",
                "```\n",
            )
            .as_bytes()
            .to_vec(),
            Path::new("foo.agent-coexist.md"),
            crate::time::parse("2026-09-17T10:00:00Z").unwrap(),
        )
        .expect("parses");
        let mut report = Report::new("check", "foo");
        post_rows(&mut report, &document);
        assert_eq!(
            field_values(&report, "authors"),
            vec![vec!["alice, bob, carol".to_string()]]
        );
    }

    /// `check` through the real filesystem: the W-NAME stem is the
    /// directory entry's spelling, the `file` row the operand as typed.
    #[cfg(any(windows, target_os = "macos"))]
    mod listed_spelling {
        use super::*;
        use crate::ops::CheckParams;

        const NOW: &str = "2026-09-17T10:00:00Z";

        fn check_at(path: std::path::PathBuf) -> Report {
            let mut ctx = context::test_ctx(crate::time::parse(NOW).unwrap());
            check(&mut ctx, &CheckParams { path }).expect("check runs")
        }

        fn wname_stems(report: &Report) -> Vec<String> {
            report
                .rows
                .iter()
                .filter_map(|row| match row {
                    Row::Diagnostic(d) if d.id == "W-NAME" => d
                        .params
                        .iter()
                        .find(|(k, _)| *k == "stem")
                        .map(|(_, v)| v.to_string_lossy().into_owned()),
                    _ => None,
                })
                .collect()
        }

        #[test]
        fn case_variant_operand_uses_the_on_disk_name() {
            let root = tempfile::tempdir().unwrap();
            std::fs::create_dir(root.path().join("room")).unwrap();
            std::fs::write(
                root.path().join("room").join("standup.agent-coexist.md"),
                "# standup.post\n",
            )
            .unwrap();
            let typed = root.path().join("ROOM").join("STANDUP.agent-coexist.md");
            let report = check_at(typed);
            assert_eq!(wname_stems(&report), Vec::<String>::new());
            assert_eq!(report.status, Status::Ok);
            let file = field_values(&report, "file");
            assert!(file[0][0].ends_with("STANDUP.agent-coexist.md"), "{file:?}");

            // A real mismatch still warns, naming the on-disk stem so the
            // offered H1 is the one that silences the canonical spelling.
            std::fs::write(
                root.path().join("room").join("standup.agent-coexist.md"),
                "# retro.post\n",
            )
            .unwrap();
            let typed = root.path().join("ROOM").join("STANDUP.agent-coexist.md");
            assert_eq!(wname_stems(&check_at(typed)), vec!["standup".to_string()]);
        }

        #[test]
        fn on_disk_case_mismatch_warns_even_when_the_operand_matches_the_h1() {
            let root = tempfile::tempdir().unwrap();
            std::fs::write(
                root.path().join("Standup.agent-coexist.md"),
                "# standup.post\n",
            )
            .unwrap();
            let typed = root.path().join("standup.agent-coexist.md");
            assert_eq!(wname_stems(&check_at(typed)), vec!["Standup".to_string()]);
        }

        #[test]
        fn symlink_alias_keeps_its_own_name() {
            let root = tempfile::tempdir().unwrap();
            let target = root.path().join("standup.agent-coexist.md");
            std::fs::write(&target, "# standup.post\n").unwrap();
            let alias = root.path().join("alias.agent-coexist.md");
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &alias).unwrap();
            #[cfg(windows)]
            if std::os::windows::fs::symlink_file(&target, &alias).is_err() {
                // Creating symlinks needs Developer Mode or elevation.
                return;
            }
            let typed = root.path().join("ALIAS.agent-coexist.md");
            assert_eq!(wname_stems(&check_at(typed)), vec!["alias".to_string()]);
        }
    }
}
