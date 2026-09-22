//! `read-brief <PATH> [--entry <title>]` — render brief descriptions,
//! entry notes and target content (spec §7.2, §8.2, §10.10).
//!
//! `content` is complete (no truncation, T6); any entry whose content
//! was not rendered — whatever its state column — makes the first line
//! `warn` (§8.1.11), and so does any shifted / stale / unchecked entry.

use std::path::Path;

use crate::consts;
use crate::doc;
use crate::error::Fault;
use crate::hash;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::{reason_report, ReadBriefParams};
use crate::report::Report;

use super::{blocking_report, file_missing_report, load_file, Loaded, type_report};

pub fn read_brief(ctx: &mut Ctx, p: &ReadBriefParams) -> Result<Report, Fault> {
    let object = context::object_of_path(&p.path);
    let dctx = context::diag_context_for_file(ctx, "read-brief", &p.path);
    let file_display = context::display_file(&p.path);
    match load_file(&p.path, ctx.now, consts::READ_WAIT_MS)? {
        Loaded::Missing => Ok(file_missing_report(&dctx, "read-brief", &object, &p.path)),
        Loaded::Busy { id, params } => Ok(reason_report(&dctx, "read-brief", &object, id, params)),
        Loaded::Blocking(diags) => Ok(blocking_report("read-brief", &object, &file_display, None, diags)),
        Loaded::Doc(document) => {
            // Blocking structure (duplicate titles make `--entry`
            // ambiguous, an unclosed fence the rest unjudgeable): the
            // §8.3.2 diagnostic form instead of a rendering.
            let blocked = document.kind.is_none() || !document.blocking().is_empty();
            if blocked {
                return Ok(blocking_report(
                    "read-brief",
                    &object,
                    &file_display,
                    document.kind,
                    document.diagnostics,
                ));
            }
            match document.kind {
                Some(doc::Kind::Brief) => {
                    if let Some(title) = p.entry.as_deref() {
                        if !document
                            .brief
                            .entries
                            .iter()
                            .any(|entry| entry.title == title)
                        {
                            return Ok(reason_report(
                                &dctx,
                                "read-brief",
                                &object,
                                "E-ENTRY-NOTFOUND",
                                vec![
                                    ("title", title.to_string()),
                                    ("path", object.clone()),
                                ],
                            ));
                        }
                    }
                    Ok(render(&p.path, &object, &document, p.entry.as_deref()))
                }
                Some(kind) => Ok(type_report(&dctx, "read-brief", &object, kind, "brief")),
                None => Ok(blocking_report(
                    "read-brief",
                    &object,
                    &file_display,
                    None,
                    document.diagnostics,
                )),
            }
        }
    }
}

fn render(path: &Path, object: &str, document: &doc::Document, entry_filter: Option<&str>) -> Report {
    let mut report = Report::new("read-brief", object);
    // §10.10.1: `--entry` renders only that entry, no description.
    if entry_filter.is_none() {
        if let Some(body) = &document.brief.description {
            report.text("description", document.text(&body.content).to_string());
        }
    }
    let mut warn = false;
    for entry in &document.brief.entries {
        if let Some(title) = entry_filter {
            if entry.title != title {
                continue;
            }
        }
        let verification = hash::verify(path, entry);
        // Content is renderable exactly when the verification produced
        // the line range, the span within the lossy target text, and the
        // in-memory target itself (oversized targets stream the hash
        // with `content = None`, §10.4.2).
        let rendered = verification.range.is_some()
            && verification.content_span.is_some()
            && verification
                .target
                .as_ref()
                .and_then(|target| target.content.as_deref())
                .is_some();
        if matches!(verification.state, "shifted" | "stale" | "unchecked") || !rendered {
            warn = true;
        }
        let mut values = vec![
            entry.title.clone(),
            verification.state.to_string(),
            entry
                .path
                .as_ref()
                .map(|property| names::fold(&property.value))
                .unwrap_or_else(|| "?".to_string()),
        ];
        if let Some(reason) = verification.reason {
            values.push(reason.to_string());
        }
        if rendered {
            let (first, last) = verification.range.unwrap();
            values.push(format!("L{first}..L{last}"));
        }
        report.kv("entry", values);
        // §10.9.2: group values are folded with control replacement.
        for (name, value) in &verification.groups {
            report.kv("group", vec![name.clone(), names::display_text(value)]);
        }
        if let Some(note) = document.body(entry.section) {
            report.text("note", note.to_string());
        }
        if rendered {
            let target = verification
                .target
                .as_ref()
                .and_then(|t| t.content.as_deref())
                .unwrap_or_default();
            let span = verification.content_span.clone().unwrap_or_default();
            report.text("content", target[span].to_string());
        }
    }
    if warn {
        report.warn();
    }
    report
}
