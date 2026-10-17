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
use crate::ops::{Filters, LookforCommentParams, reason_report};
use crate::report::Report;
use crate::time;
use crate::types::post::Comment;

use super::{
    LoadError, blocking_report, body_excerpt, file_missing_report, load_file, type_report,
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
    let keyword = filters.keyword.as_deref().map(str::to_lowercase);
    for comment in &document.post.comments {
        if !matches_filters(document, comment, filters, keyword.as_deref()) {
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
fn matches_filters(
    document: &Document,
    comment: &Comment,
    filters: &Filters,
    keyword: Option<&str>,
) -> bool {
    if let Some(by) = &filters.by
        && &comment.nick != by
    {
        return false;
    }
    if let Some(mention) = &filters.mention
        && !comment.mentions.iter().any(|m| m == mention)
    {
        return false;
    }
    if let Some(reply_to) = filters.reply_to
        && !comment.replies.contains(&reply_to)
    {
        return false;
    }
    if let Some(seq) = filters.comment
        && comment.seq != seq
    {
        return false;
    }
    if let Some(needle) = keyword {
        let in_body = document
            .body(comment.section)
            .map(|body| body.to_lowercase().contains(needle))
            .unwrap_or(false);
        let header = document
            .text(&document.sections[comment.section].head)
            .to_lowercase();
        if !in_body && !header.contains(needle) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Row;
    use std::path::Path;

    fn parse_post(src: &str) -> Document {
        doc::parse(
            src.as_bytes().to_vec(),
            Path::new("foo.agent-coexist.md"),
            time::parse("2026-09-17T10:00:00Z").unwrap(),
        )
        .expect("parses")
    }

    fn keyword(value: &str) -> Filters {
        Filters {
            keyword: Some(value.to_string()),
            ..Filters::default()
        }
    }

    fn total(report: &Report) -> usize {
        report
            .rows
            .iter()
            .find_map(|row| match row {
                Row::Fields { key, values, .. } if key == "total" => {
                    values.first().and_then(|v| v.parse().ok())
                }
                _ => None,
            })
            .unwrap_or(0)
    }

    #[test]
    fn keyword_keeps_raw_whitespace_and_case_insensitive_substring() {
        let document = parse_post(concat!(
            "# foo.post\n",
            "\n",
            "## #1 alice (2026-09-17T10:00:00Z)\n",
            "\n",
            "```md\n",
            "keep a\u{00A0}b here\n",
            "two  spaces\n",
            "```\n",
        ));
        assert_eq!(total(&rows("foo", &document, &keyword("a\u{00A0}b"))), 1);
        assert_eq!(total(&rows("foo", &document, &keyword("a b"))), 0);
        assert_eq!(total(&rows("foo", &document, &keyword("two  spaces"))), 1);
        assert_eq!(total(&rows("foo", &document, &keyword("two spaces"))), 0);
        assert_eq!(total(&rows("foo", &document, &keyword("A\u{00A0}B"))), 1);
        assert_eq!(total(&rows("foo", &document, &keyword("ALICE"))), 1);
    }
}
