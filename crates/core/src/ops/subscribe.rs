//! The `subscribe` command (spec §3.2.4): add `--as` to the post's
//! `- subscribers:` list, or cancel the subscription. Exclusive lock with
//! the same queuing budget as `post` (§7.3); never subject to the
//! functional lock (T5). The add form creates the file (H1 + blank +
//! `- subscribers: <nick>`) and missing parents when absent (§7.3.2); the
//! cancel form never creates.

use std::mem;

use crate::doc;
use crate::error::Fault;
use crate::fsx::{self, Mutation};
use crate::ops::context::Ctx;
use crate::report::Report;
use crate::text::Splice;

use super::post::{
    after_line_and_blank, checked_doc, finish, guarded_apply, line_end, preamble_anchor,
    require_valid_stem, Applied, Done, Early, Step,
};
use super::SubscribeParams;

pub fn run(ctx: &mut Ctx, p: &SubscribeParams) -> Result<Report, Fault> {
    let policy = fsx::WritePolicy {
        wait_ms: ctx.post_wait_ms,
        create: !p.cancel,
        create_parents: !p.cancel,
    };
    let result = fsx::mutate(&p.path, policy, |bytes, created| {
        step(ctx, p, bytes, created)
    });
    finish(ctx, "subscribe", &p.path, None, result)
}

fn step(
    ctx: &mut Ctx,
    p: &SubscribeParams,
    bytes: Vec<u8>,
    created: bool,
) -> Result<(Mutation, Step), Fault> {
    if created {
        let stem = require_valid_stem(&p.path)?;
        let mut content = format!("# {stem}.post\n\n");
        content.push_str("- subscribers: ");
        content.push_str(&p.nick);
        content.push('\n');
        let mut done = Done::empty();
        done.kv("subscribers", vec![p.nick.clone()]);
        return Ok((Mutation::Write(content.into_bytes()), Step::Done(done)));
    }
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Post) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let subs = &doc.post.subscribers;
    let mut done = Done::empty();
    if p.cancel {
        cancel_step(p, &doc, &bytes, subs, &mut done)
    } else {
        add_step(p, &doc, &bytes, subs, &mut done)
    }
}

/// `--cancel` removes only `--as` (§3.2.4): removing the last subscriber
/// deletes the single property line (never reported as `removed-lines`).
fn cancel_step(
    p: &SubscribeParams,
    doc: &doc::Document,
    bytes: &[u8],
    subs: &[String],
    done: &mut Done,
) -> Result<(Mutation, Step), Fault> {
    if !subs.contains(&p.nick) {
        done.kv("subscribers", subs_value(subs));
        done.kv("unchanged", vec![p.nick.clone()]);
        return Ok((
            Mutation::Unchanged,
            Step::Done(mem::replace(done, Done::empty())),
        ));
    }
    let remaining: Vec<String> = subs
        .iter()
        .filter(|nick| *nick != &p.nick)
        .cloned()
        .collect();
    if remaining.is_empty() {
        // The parser only reports subscribers it read from the property
        // line, so the line (and its span) must exist here.
        let (line_start, _) =
            subscribers_line(doc).expect("reported subscribers imply the property line");
        let splices = vec![Splice {
            range: line_start..line_end(&doc.source, line_start),
            replacement: Vec::new(),
        }];
        match guarded_apply(bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
            Applied::Bytes(new_bytes) => {
                done.kv("subscribers", vec!["none".to_string()]);
                Ok((
                    Mutation::Write(new_bytes),
                    Step::Done(mem::replace(done, Done::empty())),
                ))
            }
            Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
        }
    } else {
        let (_, value_span) =
            subscribers_line(doc).expect("reported subscribers imply the property line");
        let joined = remaining.join(", ");
        let splices = vec![Splice {
            range: value_span,
            replacement: joined.clone().into_bytes(),
        }];
        match guarded_apply(bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
            Applied::Bytes(new_bytes) => {
                done.kv("subscribers", vec![joined]);
                Ok((
                    Mutation::Write(new_bytes),
                    Step::Done(mem::replace(done, Done::empty())),
                ))
            }
            Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
        }
    }
}

/// Add form: append to the list value in place when the line exists,
/// otherwise insert a new line after the H1 (+ comment lines) and its
/// first blank line (§10.3.2).
fn add_step(
    p: &SubscribeParams,
    doc: &doc::Document,
    bytes: &[u8],
    subs: &[String],
    done: &mut Done,
) -> Result<(Mutation, Step), Fault> {
    if subs.contains(&p.nick) {
        done.kv("subscribers", subs_value(subs));
        done.kv("unchanged", vec![p.nick.clone()]);
        return Ok((
            Mutation::Unchanged,
            Step::Done(mem::replace(done, Done::empty())),
        ));
    }
    let mut appended = subs.to_vec();
    appended.push(p.nick.clone());
    let display = appended.join(", ");
    let splices = match subscribers_line(doc) {
        Some((_, value_span)) => {
            let raw_is_blank = doc.text(&value_span).trim().is_empty();
            if raw_is_blank {
                // `- subscribers: ` with an empty value: write the first nick.
                vec![Splice {
                    range: value_span,
                    replacement: p.nick.clone().into_bytes(),
                }]
            } else {
                vec![Splice {
                    range: value_span.end..value_span.end,
                    replacement: format!(", {}", p.nick).into_bytes(),
                }]
            }
        }
        None => {
            let mut block = String::from("- subscribers: ");
            block.push_str(&p.nick);
            block.push('\n');
            let anchor = preamble_anchor(doc);
            let (pos, inserted) = after_line_and_blank(&doc.source, anchor, &block);
            vec![Splice {
                range: pos..pos,
                replacement: inserted.into_bytes(),
            }]
        }
    };
    match guarded_apply(bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
        Applied::Bytes(new_bytes) => {
            done.kv("subscribers", vec![display]);
            Ok((
                Mutation::Write(new_bytes),
                Step::Done(mem::replace(done, Done::empty())),
            ))
        }
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// The `- subscribers:` property line as `(line start, value span)`.
/// `Post.subscribers_span` is the full line span (terminator included);
/// the value span is derived from the `": "` separator.
fn subscribers_line(doc: &doc::Document) -> Option<(usize, crate::text::Span)> {
    let span = doc.post.subscribers_span.clone()?;
    let line_text = doc.text(&span);
    let colon = line_text.find(':')?; // the key grammar contains no ':'
    let vstart = span.start + colon + 2; // skip ": "
    let mut vend = span.end;
    let b = doc.source.as_bytes();
    if vend > vstart && b[vend - 1] == b'\n' {
        vend -= 1;
        if vend > vstart && b[vend - 1] == b'\r' {
            vend -= 1;
        }
    }
    if vstart > vend {
        vend = vstart;
    }
    Some((span.start, vstart..vend))
}

fn subs_value(subs: &[String]) -> Vec<String> {
    if subs.is_empty() {
        vec!["none".to_string()]
    } else {
        vec![subs.join(", ")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::context::test_ctx;
    use crate::time;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-subscribe-tests")
            .join(format!(
                "{}-{}-{}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn create_rejects_invalid_stem_without_leaving_a_file() {
        let dir = scratch("ename");
        let target = dir.join("bad stem.agent-besties.md");
        let p = SubscribeParams {
            path: target.clone(),
            nick: "me".to_string(),
            cancel: false,
        };
        let mut ctx = test_ctx(time::parse("2026-09-25T12:00:00Z").unwrap());
        let report = run(&mut ctx, &p).unwrap();
        assert_eq!(report.status, crate::report::Status::Error);
        assert_eq!(report.exit, 1);
        assert!(!target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
