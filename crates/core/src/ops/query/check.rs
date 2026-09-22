//! `check <PATH>` — the single-file report (spec §7.2, §8.2 `check`).
//!
//! Row order per §8.1.13: `file` → `type` (when the H1 kind survived)
//! → every applicable §9 diagnostic in anchor order → the per-type
//! counts and previews. Without an H1 there is nothing to preview
//! (§2.2.3, §8.2 check example 3). First-line status comes from the
//! structured diagnostic set plus the §8.1.11 facts, never from
//! formatted text.

use std::path::Path;

use crate::consts;
use crate::diag;
use crate::doc::{self, Document};
use crate::error::{Diagnostic, Fault};
use crate::hash;
use crate::ops::context::{self, Ctx};
use crate::ops::{reason_report, CheckParams};
use crate::report::{Report, Status};
use crate::time;
use crate::types;
use crate::Time;

use super::{
    blocking_report, body_excerpt, file_missing_report, load_file, Loaded, parent_display,
    sort_diagnostics,
};

pub fn check(ctx: &mut Ctx, p: &CheckParams) -> Result<Report, Fault> {
    let object = context::object_of_path(&p.path);
    let dctx = context::diag_context_for_file(ctx, "check", &p.path);
    let file_display = context::display_file(&p.path);
    match load_file(&p.path, ctx.now, consts::READ_WAIT_MS)? {
        Loaded::Missing => Ok(file_missing_report(&dctx, "check", &object, &p.path)),
        Loaded::Busy { id, params } => Ok(reason_report(&dctx, "check", &object, id, params)),
        Loaded::Blocking(diags) => Ok(blocking_report("check", &object, &file_display, None, diags)),
        Loaded::Doc(document) => match document.kind {
            None => Ok(blocking_report(
                "check",
                &object,
                &file_display,
                None,
                document.diagnostics,
            )),
            Some(kind) => Ok(report_for(ctx, &p.path, &object, &file_display, kind, document)),
        },
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
            let mut diags = document.diagnostics.clone();
            let (count, rows) = state_preview(ctx.now, object, &document, &mut diags);
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
/// and any warn-level diagnostic was seen (§9 补充4 order).
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
    for comment in &post.comments {
        if !authors.contains(&comment.nick.as_str()) {
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
            entry.title.clone(),
            verification.state.to_string(),
            entry
                .path
                .as_ref()
                .map(|property| crate::names::fold(&property.value))
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
        let mut values = vec![section.name.clone()];
        if let Some(excerpt) = body_excerpt(document, section.section) {
            values.push(excerpt);
        }
        report.kv("section", values);
    }
}

/// Same-file state preview (§6.2.7, §8.2 check state): display uses
/// [`crate::types::state::selected`] per section (all parseable
/// declarations stay candidates, dirty-line fragments included), the
/// derived diagnostics anchor on the later / expiring declaration.
#[allow(clippy::type_complexity)]
fn state_preview(
    now: Time,
    object: &str,
    document: &Document,
    derived: &mut Vec<Diagnostic>,
) -> (usize, Vec<(String, Vec<String>)>) {
    let dir_display = parent_display(object);
    let flat: Vec<(&types::state::StateSection, &types::state::Declaration)> = document
        .state
        .iter()
        .flat_map(|section| {
            types::state::selected(section)
                .into_iter()
                .map(move |decl| (section, decl))
        })
        .collect();
    let mut conflict = vec![false; flat.len()];
    let mut expired = vec![false; flat.len()];
    for (i, (section, decl)) in flat.iter().enumerate() {
        if !decl.valid(now) {
            expired[i] = true;
            derived.push(
                Diagnostic::new("W-STATE-EXPIRED", decl.line)
                    .with("nick", decl.nick.as_str())
                    .with("status", section.status.as_str())
                    .with("until", time::format(decl.until))
                    .with("dir", dir_display.clone()),
            );
        }
    }
    // §9 补充6: {a} is the earlier declaration's nick, {b} the later
    // one's; the diagnostic anchors on the later declaration's line.
    for x in 0..flat.len() {
        for y in (x + 1)..flat.len() {
            let (section_x, decl_x) = flat[x];
            let (section_y, decl_y) = flat[y];
            if section_x.status == "locked"
                && section_y.status == "locked"
                && decl_x.nick != decl_y.nick
                && decl_x.valid(now)
                && decl_y.valid(now)
            {
                conflict[x] = true;
                conflict[y] = true;
                derived.push(
                    Diagnostic::new("W-STATE-CONFLICT", decl_y.line)
                        .with("a", decl_x.nick.as_str())
                        .with("b", decl_y.nick.as_str())
                        .with("dir", dir_display.clone()),
                );
            }
        }
    }
    let rows = flat
        .iter()
        .enumerate()
        .map(|(i, (section, decl))| {
            let mut values = vec![
                decl.nick.clone(),
                format!("{}..{}", time::format(decl.start), time::format(decl.until)),
            ];
            if conflict[i] {
                values.push("conflict".to_string());
            } else if expired[i] {
                values.push("expired".to_string());
            }
            (section.status.clone(), values)
        })
        .collect();
    (flat.len(), rows)
}
