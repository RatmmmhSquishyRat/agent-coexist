//! The `brief` command (spec §4): all six mutually exclusive forms of
//! §4.5 / §7.2 — bare (create skeleton / `unchanged`), description-only
//! `-m`, `--target`, note-only `--entry … -m`, `--entry … --remove`, and
//! `--refresh`. Creation forms write the complete resulting content in
//! one shot; everything else splices the original bytes via `text::apply`.
//!
//! Every byte-writing form runs the functional-lock verdict inside the
//! exclusive fastfail lock (§10.8); the `unchanged` form never judges one
//! (§10.8.1). After every successful write the new bytes are re-parsed and
//! any surviving non-blocking `E-*` is reported as `remaining` rows — the
//! last rows of the output, with a `warn` first line (§2.7.4).

use std::io::ErrorKind;
use std::path::Path;

use crate::consts;
use crate::diag;
use crate::doc;
use crate::error::{Diagnostic, Fault};
use crate::fsx::{self, Mutation};
use crate::hash;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::report::Report;
use crate::text::{self, Splice};
use crate::types::brief::Entry;
use crate::types::property_parts;
use crate::Time;

use super::post::{
    after_line_and_blank, checked_doc, fence_block, finish, flock_blocked, guarded_apply, is_blank,
    line_end, removed_lines_label, require_valid_stem, section_delete_splice, stash_flock,
    strip_term, unchanged_done, Applied, Done, Early, FlockStash, Step,
};
use super::{reason_report, BriefForm, BriefParams, BriefTarget, RegexOpt};

/// What the target form needs from its target, resolved before the brief
/// is ever opened so a missing target never creates a file or directory
/// (§4.5.1).
struct TargetInfo {
    sha: String,
    /// `--entry` value (already folded by the CLI) or the target's folded
    /// file-name part.
    title: String,
}

pub fn run(ctx: &mut Ctx, p: &BriefParams) -> Result<Report, Fault> {
    let mut target: Option<TargetInfo> = None;
    if let BriefForm::Target(t) = &p.form {
        let resolved = hash::resolve_target(&p.path, &t.target);
        match hash::read_target(&resolved) {
            Ok(read) => {
                let title = match &t.entry {
                    Some(title) => title.clone(),
                    None => default_title(&resolved, &t.target),
                };
                target = Some(TargetInfo {
                    sha: read.sha256,
                    title,
                });
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let dctx = context::diag_context_for_file(ctx, "brief", &p.path);
                let object = context::object_of_path(&p.path);
                return Ok(reason_report(
                    &dctx,
                    "brief",
                    &object,
                    "E-TARGET-MISSING",
                    vec![("target", display_stored_path(&t.target))],
                ));
            }
            Err(err) => return Err(Fault::io("read", resolved, err)),
        }
    }
    let creates = !matches!(
        p.form,
        BriefForm::NoteOnly { .. } | BriefForm::RemoveEntry { .. } | BriefForm::Refresh { .. }
    );
    let policy = fsx::WritePolicy {
        wait_ms: consts::FASTFAIL_GRACE_MS,
        create: creates,
        create_parents: creates,
    };
    let mut stash: Option<FlockStash> = None;
    let result = fsx::mutate(&p.path, policy, |bytes, created| {
        step(ctx, p, target.as_ref(), &mut stash, bytes, created)
    });
    finish(ctx, "brief", &p.path, stash, result)
}

fn step(
    ctx: &mut Ctx,
    p: &BriefParams,
    target: Option<&TargetInfo>,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    created: bool,
) -> Result<(Mutation, Step), Fault> {
    match &p.form {
        BriefForm::Bare => bare_step(ctx, p, stash, bytes, created),
        BriefForm::SetDescription(description) => {
            description_step(ctx, p, stash, bytes, created, description)
        }
        BriefForm::Target(t) => target_step(ctx, p, stash, bytes, created, t, target),
        BriefForm::NoteOnly { entry, note } => note_step(ctx, p, stash, bytes, entry, note),
        BriefForm::RemoveEntry { entry } => remove_entry_step(ctx, p, stash, bytes, entry),
        BriefForm::Refresh { entry } => refresh_step(ctx, p, stash, bytes, entry),
    }
}

/// §4.5.1–2: create the bare skeleton; an existing file stays `unchanged`.
fn bare_step(
    ctx: &mut Ctx,
    p: &BriefParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    created: bool,
) -> Result<(Mutation, Step), Fault> {
    if created {
        let stem = require_valid_stem(&p.path)?;
        if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
            return flock_blocked();
        }
        return Ok((
            Mutation::Write(skeleton(&stem).into_bytes()),
            Step::Done(Done::empty()),
        ));
    }
    match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Brief) {
        Ok(_) => Ok((Mutation::Unchanged, Step::Done(unchanged_done(&p.path)))),
        Err(Early::Step(step)) => Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => Err(fault),
    }
}

/// §4.5.3: replace the description fence, or insert one after the comment
/// line / H1 when absent.
fn description_step(
    ctx: &mut Ctx,
    p: &BriefParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    created: bool,
    description: &str,
) -> Result<(Mutation, Step), Fault> {
    let mut done = Done::empty();
    done.kv("description", vec!["set".to_string()]);
    if created {
        let stem = require_valid_stem(&p.path)?;
        if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
            return flock_blocked();
        }
        let mut content = skeleton(&stem);
        content.push('\n');
        content.push_str(&fence_block(description));
        return Ok((Mutation::Write(content.into_bytes()), Step::Done(done)));
    }
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Brief) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let splices = match &doc.brief.description {
        Some(body) => {
            let span = body.span.clone();
            vec![Splice {
                range: span,
                replacement: text::fenced(description).into_bytes(),
            }]
        }
        None => {
            let anchor = super::post::preamble_anchor(&doc);
            let (pos, inserted) =
                after_line_and_blank(&doc.source, anchor, &fence_block(description));
            vec![Splice {
                range: pos..pos,
                replacement: inserted.into_bytes(),
            }]
        }
    };
    if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
        return flock_blocked();
    }
    match guarded_apply(&bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
        Applied::Bytes(new_bytes) => {
            append_remaining(&mut done, &new_bytes, &p.path, ctx.now)?;
            Ok((Mutation::Write(new_bytes), Step::Done(done)))
        }
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// §4.5.4: add or overwrite an entry. The hash comes from the target's
/// current bytes (resolved before the brief was opened).
fn target_step(
    ctx: &mut Ctx,
    p: &BriefParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    created: bool,
    t: &BriefTarget,
    target: Option<&TargetInfo>,
) -> Result<(Mutation, Step), Fault> {
    let info = target.expect("the target form resolves its target before mutate");
    let mut done = Done::empty();
    done.kv(
        "entry",
        vec![
            names::display_text(&info.title),
            "added".to_string(),
            display_stored_path(&t.target),
        ],
    );
    done.kv("hash", vec![info.sha.clone()]);
    if t.note.is_some() {
        done.kv("note", vec!["set".to_string()]);
    }
    if created {
        let stem = require_valid_stem(&p.path)?;
        if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
            return flock_blocked();
        }
        let mut content = skeleton(&stem);
        content.push('\n');
        content.push_str(&entry_block(&info.title, t, &info.sha));
        return Ok((Mutation::Write(content.into_bytes()), Step::Done(done)));
    }
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Brief) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    match find_entry(&doc, &info.title) {
        None => {
            let gap = text::append_gap(&doc.source);
            let tail = format!("{gap}{}", entry_block(&info.title, t, &info.sha));
            let splices = vec![Splice {
                range: doc.source.len()..doc.source.len(),
                replacement: tail.into_bytes(),
            }];
            if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
                return flock_blocked();
            }
            match guarded_apply(&bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
                Applied::Bytes(new_bytes) => {
                    append_remaining(&mut done, &new_bytes, &p.path, ctx.now)?;
                    Ok((Mutation::Write(new_bytes), Step::Done(done)))
                }
                Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
            }
        }
        Some(idx) => {
            let entry = &doc.brief.entries[idx];
            if t.entry.is_none() {
                // Default title is reserved for this target (§4.5.4).
                // TITLE-TAKEN only when an existing path, after `\`→`/`
                // normalize, points at a different target. A path-less
                // entry is the §4.5.4 / §4.5.8 repair, not a conflict.
                if let Some(prop) = &entry.path {
                    let new_norm = hash::normalized_target(&t.target);
                    let existing_norm = hash::normalized_target(&prop.value);
                    if existing_norm != new_norm {
                        return Err(Fault::reason("E-ENTRY-TITLE-TAKEN")
                            .with("title", info.title.clone())
                            .with("existing", display_stored_path(&prop.value))
                            .with("target", display_stored_path(&t.target))
                            .with("target_arg", t.target.clone())
                            .with("path_arg", p.path.to_string_lossy().into_owned()));
                    }
                }
            }
            if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
                return flock_blocked();
            }
            let splices = target_update_splices(&doc, entry, t, &info.sha);
            let mut done = Done::empty();
            done.kv(
                "entry",
                vec![
                    names::display_text(&info.title),
                    "updated".to_string(),
                    display_stored_path(&t.target),
                ],
            );
            done.kv("hash", vec![info.sha.clone()]);
            if t.note.is_some() {
                done.kv("note", vec!["set".to_string()]);
            }
            match guarded_apply(&bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
                Applied::Bytes(new_bytes) => {
                    append_remaining(&mut done, &new_bytes, &p.path, ctx.now)?;
                    Ok((Mutation::Write(new_bytes), Step::Done(done)))
                }
                Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
            }
        }
    }
}

/// §4.5.5: replace or insert an existing entry's note fence.
fn note_step(
    ctx: &mut Ctx,
    p: &BriefParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    entry_title: &str,
    note: &str,
) -> Result<(Mutation, Step), Fault> {
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Brief) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let idx = match find_entry(&doc, entry_title) {
        Some(idx) => idx,
        None => return Err(entry_notfound(entry_title, &p.path)),
    };
    if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
        return flock_blocked();
    }
    let entry = &doc.brief.entries[idx];
    let splices = match &doc.sections[entry.section].body {
        Some(body) => {
            let span = body.span.clone();
            vec![Splice {
                range: span,
                replacement: text::fenced(note).into_bytes(),
            }]
        }
        None => {
            let (pos, inserted) = note_insert(&doc, entry, note);
            vec![Splice {
                range: pos..pos,
                replacement: inserted.into_bytes(),
            }]
        }
    };
    let mut done = Done::empty();
    done.kv("note", vec!["set".to_string()]);
    match guarded_apply(&bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
        Applied::Bytes(new_bytes) => {
            append_remaining(&mut done, &new_bytes, &p.path, ctx.now)?;
            Ok((Mutation::Write(new_bytes), Step::Done(done)))
        }
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// §4.5.6: delete the whole entry section (§10.3.5 range).
fn remove_entry_step(
    ctx: &mut Ctx,
    p: &BriefParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    entry_title: &str,
) -> Result<(Mutation, Step), Fault> {
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Brief) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let idx = match find_entry(&doc, entry_title) {
        Some(idx) => idx,
        None => return Err(entry_notfound(entry_title, &p.path)),
    };
    if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
        return flock_blocked();
    }
    let entry = &doc.brief.entries[idx];
    let splice = section_delete_splice(&doc.source, &doc.sections[entry.section]);
    let label = removed_lines_label(&doc.source, splice.range.start, splice.range.end);
    let mut done = Done::empty();
    done.kv(
        "entry",
        vec![names::display_text(&entry.title), "removed".to_string()],
    );
    done.kv("removed-lines", vec![label]);
    match guarded_apply(
        &bytes,
        vec![splice],
        doc.kind.as_ref().map(doc::Kind::as_str),
    )? {
        Applied::Bytes(new_bytes) => {
            append_remaining(&mut done, &new_bytes, &p.path, ctx.now)?;
            Ok((Mutation::Write(new_bytes), Step::Done(done)))
        }
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// §4.5.7: refresh `- hash:` values for the selected (default: all)
/// entries from each target's current bytes. Missing targets and entries
/// without a `path` line are skipped (first line `warn`).
fn refresh_step(
    ctx: &mut Ctx,
    p: &BriefParams,
    stash: &mut Option<FlockStash>,
    bytes: Vec<u8>,
    entry_title: &Option<String>,
) -> Result<(Mutation, Step), Fault> {
    let doc = match checked_doc(&bytes, &p.path, ctx.now, doc::Kind::Brief) {
        Ok(doc) => doc,
        Err(Early::Step(step)) => return Ok((Mutation::Unchanged, step)),
        Err(Early::Fault(fault)) => return Err(fault),
    };
    let selected: Vec<usize> = match entry_title {
        Some(title) => vec![match find_entry(&doc, title) {
            Some(idx) => idx,
            None => return Err(entry_notfound(title, &p.path)),
        }],
        None => (0..doc.brief.entries.len()).collect(),
    };
    if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
        return flock_blocked();
    }
    let mut done = Done::empty();
    let mut splices: Vec<Splice> = Vec::new();
    let mut inserts: Vec<(usize, Vec<u8>)> = Vec::new();
    for &idx in &selected {
        let entry = &doc.brief.entries[idx];
        let path_prop = match &entry.path {
            Some(prop) => prop,
            None => {
                done.kv(
                    "entry",
                    vec![
                        names::display_text(&entry.title),
                        "skipped".to_string(),
                        "no path".to_string(),
                    ],
                );
                done.warn = true;
                continue;
            }
        };
        let resolved = hash::resolve_target(&p.path, &path_prop.value);
        let sha = match hash::read_target(&resolved) {
            Ok(read) => read.sha256,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                done.kv(
                    "entry",
                    vec![
                        names::display_text(&entry.title),
                        "skipped".to_string(),
                        "target missing".to_string(),
                    ],
                );
                done.warn = true;
                continue;
            }
            Err(err) => return Err(Fault::io("read", resolved, err)),
        };
        match &entry.hash {
            Some(prop) => splices.push(Splice {
                range: prop.value_span.clone(),
                replacement: sha.clone().into_bytes(),
            }),
            None => inserts.push((
                line_end(&doc.source, path_prop.span.start),
                format!("- hash: {sha}\n").into_bytes(),
            )),
        }
        done.kv(
            "entry",
            vec![
                names::display_text(&entry.title),
                "refreshed".to_string(),
                display_stored_path(&path_prop.value),
            ],
        );
    }
    let splices = merge_inserts(&doc.source, splices, inserts);
    if splices.is_empty() {
        return Ok((Mutation::Unchanged, Step::Done(done)));
    }
    match guarded_apply(&bytes, splices, doc.kind.as_ref().map(doc::Kind::as_str))? {
        Applied::Bytes(new_bytes) => {
            append_remaining(&mut done, &new_bytes, &p.path, ctx.now)?;
            Ok((Mutation::Write(new_bytes), Step::Done(done)))
        }
        Applied::Refused(step) => Ok((Mutation::Unchanged, step)),
    }
}

/// Splice set for overwriting an existing entry with `--target`: `path`
/// and `hash` value spans (or their insertion geometries when the lines
/// are missing, §4.5.4), the `--regex` tri-state, and the optional note
/// fence.
fn target_update_splices(
    doc: &doc::Document,
    entry: &Entry,
    t: &BriefTarget,
    sha: &str,
) -> Vec<Splice> {
    let mut splices: Vec<Splice> = Vec::new();
    let mut inserts: Vec<(usize, Vec<u8>)> = Vec::new();
    match (&entry.path, &entry.hash) {
        (Some(path_prop), Some(hash_prop)) => {
            splices.push(Splice {
                range: path_prop.value_span.clone(),
                replacement: t.target.clone().into_bytes(),
            });
            splices.push(Splice {
                range: hash_prop.value_span.clone(),
                replacement: sha.as_bytes().to_vec(),
            });
        }
        (Some(path_prop), None) => {
            splices.push(Splice {
                range: path_prop.value_span.clone(),
                replacement: t.target.clone().into_bytes(),
            });
            // §4.5.4: `- hash:` goes right after the path line's terminator.
            inserts.push((
                line_end(&doc.source, path_prop.span.start),
                format!("- hash: {sha}\n").into_bytes(),
            ));
        }
        (None, Some(hash_prop)) => {
            // §4.5.4: insert `- path:` after the H2 (and its following
            // blank); replace the existing hash value so a second `- hash:`
            // is not left in the property area.
            let head = doc.sections[entry.section].head.start;
            let (pos, inserted) =
                after_line_and_blank(&doc.source, head, &format!("- path: {}\n", t.target));
            inserts.push((pos, inserted.into_bytes()));
            splices.push(Splice {
                range: hash_prop.value_span.clone(),
                replacement: sha.as_bytes().to_vec(),
            });
        }
        (None, None) => {
            let head = doc.sections[entry.section].head.start;
            let (pos, inserted) = after_line_and_blank(
                &doc.source,
                head,
                &format!("- path: {}\n- hash: {sha}\n", t.target),
            );
            inserts.push((pos, inserted.into_bytes()));
        }
    }
    match &t.regex {
        RegexOpt::Keep => {}
        RegexOpt::Set(pattern) => match &entry.regex {
            Some(prop) => splices.push(Splice {
                range: prop.value_span.clone(),
                replacement: pattern.clone().into_bytes(),
            }),
            None => inserts.push((
                attrs_anchor(doc, entry),
                format!("- regex: {pattern}\n").into_bytes(),
            )),
        },
        RegexOpt::Delete => splices.extend(regex_delete_splices(doc, entry)),
    }
    if let Some(note) = &t.note {
        match &doc.sections[entry.section].body {
            Some(body) => {
                let span = body.span.clone();
                splices.push(Splice {
                    range: span,
                    replacement: text::fenced(note).into_bytes(),
                });
            }
            None => {
                let (pos, inserted) = note_insert(doc, entry, note);
                inserts.push((pos, inserted.into_bytes()));
            }
        }
    }
    merge_inserts(&doc.source, splices, inserts)
}

/// Line-start offset where new property lines of `entry` belong: the
/// parser's `attrs_end` (start of the first non-property non-blank line
/// after the H2, or the section end), clamped to the source length.
fn attrs_anchor(doc: &doc::Document, entry: &Entry) -> usize {
    entry.attrs_end.min(doc.source.len())
}

/// `--regex ''`: drop every syntactically active `- regex:` line in the
/// selected entry's property area (H2 body through `attrs_end`). Duplicate
/// keys never live on `Entry` (first-wins), so this scans source lines
/// instead of `Entry.regex`'s span. Isolated CR is not a property.
fn regex_delete_splices(doc: &doc::Document, entry: &Entry) -> Vec<Splice> {
    let src = &doc.source;
    let mut out = Vec::new();
    let end = entry.attrs_end.min(src.len());
    let mut pos = line_end(src, doc.sections[entry.section].head.start);
    while pos < end {
        let start = pos;
        let next = line_end(src, start);
        let line = strip_term(&src[start..next]);
        if !line.contains('\r') && matches!(property_parts(line), Some(("regex", _))) {
            out.push(Splice {
                range: start..next,
                replacement: Vec::new(),
            });
        }
        pos = next;
    }
    out
}

/// Insertion point and bytes for an entry's note fence when it has none:
/// after the attribute area, adding one blank line when the area is not
/// already followed by one.
fn note_insert(doc: &doc::Document, entry: &Entry, note: &str) -> (usize, String) {
    let pos = attrs_anchor(doc, entry);
    if pos < doc.source.len() {
        let next_end = line_end(&doc.source, pos);
        if is_blank(strip_term(&doc.source[pos..next_end])) {
            return (next_end, fence_block(note));
        }
    }
    (pos, format!("\n{}", fence_block(note)))
}

/// Merge position-sorted insertions into one splice per offset (stable, so
/// the construction order — hash, then regex, then note — is preserved at
/// equal offsets) and append them to the replacement splices.
///
/// The first insert at an offset that is not already a line boundary
/// (previous source byte is not `\n`, including an unterminated last line
/// at EOF, §2.1.2) gets a leading `\n`, unless that insert already starts
/// with one (`after_line_and_blank` / `note_insert`). Coincident hash and
/// regex inserts therefore share a single terminator and stay separate
/// property lines (§2.6.1, §4.5.4).
fn merge_inserts(
    src: &str,
    splices: Vec<Splice>,
    mut inserts: Vec<(usize, Vec<u8>)>,
) -> Vec<Splice> {
    inserts.sort_by(|a, b| a.0.cmp(&b.0));
    let mut merged: Vec<Splice> = Vec::new();
    for (pos, bytes) in inserts {
        match merged.last_mut() {
            Some(tail) if tail.range.start == pos && tail.range.is_empty() => {
                tail.replacement.extend_from_slice(&bytes)
            }
            _ => {
                let mut replacement = Vec::new();
                if needs_line_before(src, pos) && !bytes.starts_with(b"\n") {
                    replacement.push(b'\n');
                }
                replacement.extend_from_slice(&bytes);
                merged.push(Splice {
                    range: pos..pos,
                    replacement,
                });
            }
        }
    }
    let mut all = splices;
    all.extend(merged);
    all
}

fn needs_line_before(src: &str, at: usize) -> bool {
    at > 0 && src.as_bytes()[at - 1] != b'\n'
}

/// §2.7.4: after a successful write, re-parse the new bytes and list every
/// surviving non-blocking `E-*` as `remaining  <E-ID>  L<n>` rows (line
/// ascending) — the last rows of the report, with a `warn` first line.
fn append_remaining(
    done: &mut Done,
    new_bytes: &[u8],
    path: &Path,
    now: Time,
) -> Result<(), Fault> {
    let doc = doc::parse(new_bytes.to_vec(), path, now)?;
    let mut diags: Vec<&Diagnostic> = doc
        .diagnostics
        .iter()
        .filter(|d| diag::is_error(d.id) && !diag::is_blocking(d.id))
        .collect();
    diags.sort_by_key(|d| (d.line, diag::order(d.id)));
    let any = !diags.is_empty();
    for d in diags {
        done.kv("remaining", vec![d.id.to_string(), format!("L{}", d.line)]);
    }
    if any {
        done.warn = true;
    }
    Ok(())
}

fn find_entry(doc: &doc::Document, title: &str) -> Option<usize> {
    doc.brief.entries.iter().position(|e| e.title == title)
}

fn entry_notfound(title: &str, path: &Path) -> Fault {
    Fault::reason("E-ENTRY-NOTFOUND")
        .with("title", title.to_string())
        .with("path_arg", path.to_string_lossy().into_owned())
}

/// Skeleton written by the three creation forms (§4.5.1): H1, blank line,
/// and the fixed reader hint (comment line, §2.6.2).
fn skeleton(stem: &str) -> String {
    format!(
        "# {stem}.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n"
    )
}

/// Entry section appended / written by the target form (§4.1 layout).
fn entry_block(title: &str, t: &BriefTarget, sha: &str) -> String {
    let mut block = format!("## {title}\n\n- path: {}\n- hash: {sha}\n", t.target);
    if let RegexOpt::Set(pattern) = &t.regex {
        block.push_str(&format!("- regex: {pattern}\n"));
    }
    if let Some(note) = &t.note {
        block.push('\n');
        block.push_str(&fence_block(note));
    }
    block
}

/// Default entry title: the target's file-name part, folded for output and
/// matching (§4.3.1).
fn default_title(resolved: &Path, target: &str) -> String {
    let last = target.rsplit('/').next().unwrap_or("");
    let name = if last.is_empty() {
        resolved
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        last.to_string()
    };
    names::fold(&name)
}

/// Stored path values may use `\` separators (accepted on parse); output
/// columns are `/`-separated with folded whitespace (§8.1.9, §10.4.3).
fn display_stored_path(value: &str) -> String {
    names::fold(&value.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash;
    use crate::ops::context::test_ctx;
    use crate::report::{Report, Row, Status};
    use crate::time;
    use std::path::PathBuf;

    const NOW: &str = "2026-09-25T12:00:00Z";
    const LOCKED_BY_OTHER: &str =
        "# .state\n\n## locked\n\nother 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n";

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-brief-tests")
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
    fn create_rejects_invalid_stem_without_leaving_a_trace() {
        let dir = scratch("ename");
        let target = dir.join("bad/stem!.agent-besties.md");
        let p = BriefParams {
            path: target.clone(),
            as_nick: None,
            form: BriefForm::Bare,
        };
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &p).unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 1);
        // Neither the file nor the parent this call created survives.
        assert!(!target.exists());
        assert!(!dir.join("bad").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flock_refusal_cleans_brief_creation() {
        let dir = scratch("flock-clean");
        std::fs::write(dir.join("state.agent-besties.md"), LOCKED_BY_OTHER).unwrap();
        let target = dir.join("sub/plan.agent-besties.md");
        let p = BriefParams {
            path: target.clone(),
            as_nick: Some("me".to_string()),
            form: BriefForm::SetDescription("the plan".to_string()),
        };
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &p).unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 3);
        assert!(!target.exists());
        assert!(!dir.join("sub").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn row_values(report: &Report, key: &str) -> Vec<Vec<String>> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields { key: k, values, .. } if k == key => Some(values.clone()),
                _ => None,
            })
            .collect()
    }

    fn property_count(text: &str, key: &str) -> usize {
        let prefix = format!("- {key}:");
        text.lines()
            .filter(|line| line.starts_with(&prefix))
            .count()
    }

    fn target_form(
        path: PathBuf,
        target: &str,
        entry: Option<String>,
        regex: RegexOpt,
    ) -> BriefParams {
        BriefParams {
            path,
            as_nick: None,
            form: BriefForm::Target(BriefTarget {
                target: target.to_string(),
                entry,
                regex,
                note: None,
            }),
        }
    }

    #[test]
    fn default_title_repairs_pathless_entry_and_replaces_existing_props() {
        let dir = scratch("pathless-repair");
        let brief = dir.join("plan.agent-besties.md");
        let foo = dir.join("foo.rs");
        std::fs::write(&foo, b"hello\n").unwrap();
        std::fs::write(
            &brief,
            "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## foo.rs\n\n- regex: oldpat\n- extra: keep-me\n- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
        )
        .unwrap();
        let sha = hash::read_target(&foo).unwrap().sha256;
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(
            &mut ctx,
            &target_form(
                brief.clone(),
                "foo.rs",
                None,
                RegexOpt::Set("newpat".to_string()),
            ),
        )
        .unwrap();
        assert_eq!(report.status, Status::Ok);
        assert_eq!(report.exit, 0);
        assert_eq!(
            row_values(&report, "entry"),
            vec![vec![
                "foo.rs".to_string(),
                "updated".to_string(),
                "foo.rs".to_string()
            ]]
        );
        assert_eq!(row_values(&report, "hash"), vec![vec![sha.clone()]]);
        let text = std::fs::read_to_string(&brief).unwrap();
        assert_eq!(property_count(&text, "path"), 1);
        assert_eq!(property_count(&text, "hash"), 1);
        assert_eq!(property_count(&text, "regex"), 1);
        assert!(text.contains("- path: foo.rs\n"));
        assert!(text.contains(&format!("- hash: {sha}\n")));
        assert!(text.contains("- regex: newpat\n"));
        assert!(text.contains("- extra: keep-me\n"));
        assert!(!text.contains("- regex: oldpat"));
        assert!(!text.contains("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_title_keeps_regex_when_flag_absent() {
        let dir = scratch("pathless-keep-re");
        let brief = dir.join("plan.agent-besties.md");
        let foo = dir.join("foo.rs");
        std::fs::write(&foo, b"hello\n").unwrap();
        std::fs::write(
            &brief,
            "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## foo.rs\n\n- regex: oldpat\n",
        )
        .unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(
            &mut ctx,
            &target_form(brief.clone(), "foo.rs", None, RegexOpt::Keep),
        )
        .unwrap();
        assert_eq!(report.status, Status::Ok);
        let text = std::fs::read_to_string(&brief).unwrap();
        assert_eq!(property_count(&text, "path"), 1);
        assert_eq!(property_count(&text, "hash"), 1);
        assert_eq!(property_count(&text, "regex"), 1);
        assert!(text.contains("- regex: oldpat\n"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_title_rejects_a_different_existing_target() {
        let dir = scratch("title-taken");
        let brief = dir.join("plan.agent-besties.md");
        let foo = dir.join("foo.rs");
        std::fs::write(&foo, b"hello\n").unwrap();
        let original = "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## foo.rs\n\n- path: other.rs\n- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n";
        std::fs::write(&brief, original).unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(
            &mut ctx,
            &target_form(brief.clone(), "foo.rs", None, RegexOpt::Keep),
        )
        .unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 1);
        assert_eq!(std::fs::read_to_string(&brief).unwrap(), original);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unterminated_last_property_keeps_composed_path_hash_regex_lines() {
        let dir = scratch("prop-insert-eof");
        let brief = dir.join("plan.agent-besties.md");
        let target = dir.join("empty.txt");
        std::fs::write(&target, b"").unwrap();
        std::fs::write(
            &brief,
            "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## empty.txt\n- extra: keep-me\n- path: empty.txt",
        )
        .unwrap();
        let sha = hash::read_target(&target).unwrap().sha256;
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(
            &mut ctx,
            &target_form(
                brief.clone(),
                "empty.txt",
                Some("empty.txt".to_string()),
                RegexOpt::Set("foo".to_string()),
            ),
        )
        .unwrap();
        assert_eq!(report.status, Status::Ok);
        assert_eq!(report.exit, 0);
        assert_eq!(
            row_values(&report, "entry"),
            vec![vec![
                "empty.txt".to_string(),
                "updated".to_string(),
                "empty.txt".to_string()
            ]]
        );
        assert!(row_values(&report, "remaining").is_empty());
        let text = std::fs::read_to_string(&brief).unwrap();
        assert_eq!(
            text,
            format!(
                "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## empty.txt\n- extra: keep-me\n- path: empty.txt\n- hash: {sha}\n- regex: foo\n"
            )
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn regex_delete_removes_every_active_line_and_leaves_the_rest() {
        let dir = scratch("dup-regex-del");
        let target = dir.join("target.txt");
        let other = dir.join("other.txt");
        std::fs::write(&target, b"visible body\n").unwrap();
        std::fs::write(&other, b"other body\n").unwrap();
        let sha = hash::read_target(&target).unwrap().sha256;
        let sha_other = hash::read_target(&other).unwrap().sha256;
        let brief = dir.join("plan.agent-besties.md");
        let original = format!(
            "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## item\n\n- path: target.txt\n- extra: keep-me\n- hash: {sha}\n- regex: first-never-matches\n- regex: second-never-matches\n\n```md\nkeep-note\n```\n\n## other\n\n- path: other.txt\n- hash: {sha_other}\n- regex: other\n"
        );
        std::fs::write(&brief, &original).unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(
            &mut ctx,
            &target_form(
                brief.clone(),
                "target.txt",
                Some("item".to_string()),
                RegexOpt::Delete,
            ),
        )
        .unwrap();
        assert_eq!(report.status, Status::Ok);
        assert_eq!(report.exit, 0);
        let text = std::fs::read_to_string(&brief).unwrap();
        assert_eq!(
            text,
            format!(
                "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## item\n\n- path: target.txt\n- extra: keep-me\n- hash: {sha}\n\n```md\nkeep-note\n```\n\n## other\n\n- path: other.txt\n- hash: {sha_other}\n- regex: other\n"
            )
        );
        let read = crate::ops::query::read_brief(
            &mut ctx,
            &crate::ops::ReadBriefParams {
                path: brief,
                entry: Some("item".to_string()),
            },
        )
        .unwrap();
        assert_eq!(read.status, Status::Ok);
        let entries = row_values(&read, "entry");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0][1], "fresh");
        assert!(!entries[0].iter().any(|v| v == "regex not found"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
