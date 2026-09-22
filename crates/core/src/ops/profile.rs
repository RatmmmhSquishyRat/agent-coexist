//! The `profile` command (spec §5): the three mutually exclusive forms of
//! §5.3 — bare (create-if-missing / `unchanged`), `--section <name> -m`
//! (set or replace a section's fenced body), and
//! `--section <name> --remove` (delete the whole section).
//!
//! Byte-writing forms take the exclusive fastfail lock and run the
//! functional-lock verdict inside it (§10.8); the `unchanged` form never
//! judges a functional lock (§10.8.1). The remove form never creates a
//! file or directories (§7.3.2).

use crate::consts;
use crate::doc;
use crate::error::Fault;
use crate::fsx::{self, Mutation};
use crate::ops::context::{self, Ctx};
use crate::report::Report;
use crate::text::{self, Splice};

use super::post::{
    after_line_and_blank, checked_doc, fence_block, finish, flock_blocked, guarded_apply,
    removed_lines_label, section_delete_splice, stash_flock, stem_of, unchanged_done, Applied,
    Done, Early, FlockStash, Step,
};
use super::{ProfileAction, ProfileParams};

pub fn run(ctx: &mut Ctx, p: &ProfileParams) -> Result<Report, Fault> {
    let creates = !matches!(p.action, ProfileAction::Remove);
    let policy = fsx::WritePolicy {
        wait_ms: consts::FASTFAIL_GRACE_MS,
        create: creates,
        create_parents: creates,
    };
    let mut stash: Option<FlockStash> = None;
    let result = fsx::mutate(&p.path, policy, |bytes, created| {
        step(ctx, p, &mut stash, bytes, created)
    });
    finish(ctx, "profile", &p.path, stash, result)
}

fn step(
    ctx: &mut Ctx,
    p: &ProfileParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    created: bool,
) -> Result<(Mutation, Step), Fault> {
    match &p.action {
        ProfileAction::None => bare_step(ctx, p, stash, bytes, created),
        ProfileAction::SetBody(content) => set_step(ctx, p, stash, bytes, created, content),
        ProfileAction::Remove => remove_step(ctx, p, stash, bytes),
    }
}

/// §5.3.1: create a file holding only the H1 when missing; report
/// `unchanged` for an existing one (no write, no functional-lock verdict).
fn bare_step(
    ctx: &mut Ctx,
    p: &ProfileParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    created: bool,
) -> Result<(Mutation, Step), Fault> {
    if created {
        if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
            return flock_blocked(true);
        }
        let content = format!("# {}.agent-profile\n", stem_of(&p.path));
        return Ok((
            Mutation::Write(content.into_bytes()),
            Step::Done(Done::empty()),
        ));
    }
    match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Profile) {
        Ok(_) => Ok((Mutation::Unchanged, Step::Done(unchanged_done(&p.path)))),
        Err(Early::Step(step)) => Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => Err(fault),
    }
}

/// §5.3.2: append the section at EOF when absent, else replace its first
/// fence block — or insert one after the H2 line and its following blank
/// line when the section has no fenced body (`W-BODY`).
fn set_step(
    ctx: &mut Ctx,
    p: &ProfileParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    created: bool,
    content: &str,
) -> Result<(Mutation, Step), Fault> {
    // The CLI grammar pairs `--section` with exactly one of `-m`/`--remove`.
    let name = p
        .section
        .as_deref()
        .expect("--section is paired with -m or --remove");
    let mut done = Done::empty();
    done.kv("section", vec![name.to_string(), "set".to_string()]);
    if created {
        if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
            return flock_blocked(true);
        }
        let mut text = format!("# {}.agent-profile\n\n## {name}\n\n", stem_of(&p.path));
        text.push_str(&fence_block(content));
        return Ok((Mutation::Write(text.into_bytes()), Step::Done(done)));
    }
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Profile) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let kind = doc.kind.as_ref().map(doc::Kind::as_str);
    let splices = match doc.profile.iter().find(|ps| ps.name == name) {
        Some(ps) => {
            let section = &doc.sections[ps.section];
            match &section.body {
                Some(body) => {
                    let span = body.span.clone();
                    vec![Splice {
                        range: span,
                        replacement: text::fenced(content).into_bytes(),
                    }]
                }
                None => {
                    let (pos, inserted) = after_line_and_blank(
                        &doc.source,
                        section.head.start,
                        &fence_block(content),
                    );
                    vec![Splice {
                        range: pos..pos,
                        replacement: inserted.into_bytes(),
                    }]
                }
            }
        }
        None => {
            let gap = text::append_gap(&doc.source);
            let mut tail = format!("{gap}## {name}\n\n");
            tail.push_str(&fence_block(content));
            vec![Splice {
                range: doc.source.len()..doc.source.len(),
                replacement: tail.into_bytes(),
            }]
        }
    };
    if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
        return flock_blocked(false);
    }
    match guarded_apply(&bytes, splices, kind)? {
        Applied::Bytes(new_bytes) => Ok((Mutation::Write(new_bytes), Step::Done(done))),
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// §5.3.3: delete the whole section (§10.3.5 range); `E-SECTION-NOTFOUND`
/// when absent, `E-FILE-MISSING` for a missing file (fsx, never creates).
fn remove_step(
    ctx: &mut Ctx,
    p: &ProfileParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
) -> Result<(Mutation, Step), Fault> {
    let name = p
        .section
        .as_deref()
        .expect("--section is paired with -m or --remove");
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Profile) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let section_idx = match doc.profile.iter().find(|ps| ps.name == name) {
        Some(ps) => ps.section,
        None => {
            return Err(Fault::reason("E-SECTION-NOTFOUND")
                .with("name", name.to_string())
                .with("path", context::object_of_path(&p.path)));
        }
    };
    if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
        return flock_blocked(false);
    }
    let splice = section_delete_splice(&doc.source, &doc.sections[section_idx]);
    let label = removed_lines_label(&doc.source, splice.range.start, splice.range.end);
    let mut done = Done::empty();
    done.kv("section", vec![name.to_string(), "removed".to_string()]);
    done.kv("removed-lines", vec![label]);
    match guarded_apply(
        &bytes,
        vec![splice],
        doc.kind.as_ref().map(doc::Kind::as_str),
    )? {
        Applied::Bytes(new_bytes) => Ok((Mutation::Write(new_bytes), Step::Done(done))),
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}
