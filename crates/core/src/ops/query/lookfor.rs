//! `lookfor-comment <PATH> [filters]` — comment search (spec §7.2,
//! §8.2, §10.7).
//!
//! Filters intersect; no filters lists every comment. The record row is
//! `L<header line>  #<seq>  <author>  <activity time>  [excerpt]` in
//! file order, closed by `total` (§8.2 lookfor-comment).

use crate::consts;
use crate::doc::{self, Document};
use crate::error::Fault;
use crate::ops::context::{self, Ctx};
use crate::ops::{reason_report, Filters, LookforCommentParams};
use crate::report::Report;
use crate::time;
use crate::types::post::Comment;

use super::{
    blocking_report, body_excerpt, file_missing_report, load_file, type_report, LoadError,
};

pub fn lookfor_comment(ctx: &mut Ctx, p: &LookforCommentParams) -> Result<Report, Fault> {
    let object = context::object_of_path(&p.path);
    let dctx = context::diag_context_for_file(ctx, "lookfor-comment", &p.path);
    let file_display = context::display_file(&p.path);
    match load_file(&p.path, ctx.now, consts::READ_WAIT_MS) {
        Err(LoadError::Missing) => Ok(file_missing_report(
            &dctx,
            "lookfor-comment",
            &object,
            &p.path,
        )),
        Err(LoadError::Busy { id, params }) => {
            Ok(reason_report(&dctx, "lookfor-comment", &object, id, params))
        }
        Err(LoadError::Blocking(diags)) => Ok(blocking_report(
            "lookfor-comment",
            &object,
            &file_display,
            None,
            diags,
        )),
        Ok(document) => {
            // A blocking diagnostic (unclosed fence, duplicate numbers,
            // …) makes numbering and references ambiguous — §8.3.2 form.
            let blocked = document.kind.is_none() || !document.blocking().is_empty();
            if blocked {
                return Ok(blocking_report(
                    "lookfor-comment",
                    &object,
                    &file_display,
                    document.kind,
                    document.diagnostics,
                ));
            }
            match document.kind {
                Some(doc::Kind::Post) => Ok(rows(&object, &document, &p.filters)),
                Some(kind) => Ok(type_report(&dctx, "lookfor-comment", &object, kind, "post")),
                None => Ok(blocking_report(
                    "lookfor-comment",
                    &object,
                    &file_display,
                    None,
                    document.diagnostics,
                )),
            }
        }
        Err(LoadError::Fault(f)) => Err(f),
    }
}

fn rows(object: &str, document: &Document, filters: &Filters) -> Report {
    let mut report = Report::new("lookfor-comment", object);
    let mut count = 0usize;
    for comment in &document.post.comments {
        if !matches_filters(document, comment, filters) {
            continue;
        }
        let section = &document.sections[comment.section];
        let line = document.line_at(section.head.start);
        let mut values = vec![
            format!("#{}", comment.seq),
            comment.nick.clone(),
            time::format(comment.activity()),
        ];
        if let Some(excerpt) = body_excerpt(document, comment.section) {
            values.push(excerpt);
        }
        report.record(format!("L{line}"), values, 2, 0);
        count += 1;
    }
    report.total(count);
    report
}

/// §10.7.1: several filters intersect; `--keyword` is a case-insensitive
/// substring on the body AND the original (unfolded) header line, the
/// argument used as given (§7.1.6).
fn matches_filters(document: &Document, comment: &Comment, filters: &Filters) -> bool {
    if let Some(by) = &filters.by {
        if &comment.nick != by {
            return false;
        }
    }
    if let Some(mention) = &filters.mention {
        if !comment.mentions.iter().any(|m| m == mention) {
            return false;
        }
    }
    if let Some(reply_to) = filters.reply_to {
        if !comment.replies.contains(&reply_to) {
            return false;
        }
    }
    if let Some(seq) = filters.comment {
        if comment.seq != seq {
            return false;
        }
    }
    if let Some(keyword) = &filters.keyword {
        let needle = keyword.to_lowercase();
        let in_body = document
            .body(comment.section)
            .map(|body| body.to_lowercase().contains(&needle))
            .unwrap_or(false);
        let header = document
            .text(&document.sections[comment.section].head)
            .to_lowercase();
        if !in_body && !header.contains(&needle) {
            return false;
        }
    }
    true
}
