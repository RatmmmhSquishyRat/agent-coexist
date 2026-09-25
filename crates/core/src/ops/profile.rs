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
    removed_lines_label, require_valid_stem, section_delete_splice, stash_flock, unchanged_done,
    Applied, Done, Early, FlockStash, Step,
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
        let stem = require_valid_stem(&p.path)?;
        if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
            return flock_blocked();
        }
        let content = format!("# {stem}.agent-profile\n");
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
        let stem = require_valid_stem(&p.path)?;
        if stash_flock(stash, ctx, &p.path, p.as_nick.as_deref()) {
            return flock_blocked();
        }
        let mut text = format!("# {stem}.agent-profile\n\n## {name}\n\n");
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
        return flock_blocked();
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
        return flock_blocked();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::context::test_ctx;
    use crate::report::{Row, Status};
    use crate::time;
    use std::path::{Path, PathBuf};

    const NOW: &str = "2026-09-25T12:00:00Z";
    const LOCKED_BY_OTHER: &str =
        "# .state\n\n## locked\n\nother 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n";
    const DUP_LOCKED_SECTIONS: &str = "# .state\n\n## locked\n\nother 2026-09-25T11:00:00Z..2026-09-25T13:00:00Z\n\n## locked\n\nfriend 2026-09-25T11:30:00Z..2026-09-25T13:30:00Z\n";
    const WRONG_KIND_STATE: &str =
        "# state.post\n\n## #1 alice (2026-09-25T10:00:00Z)\n\n```\nhello\n```\n";

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-profile-tests")
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

    fn set_section(path: PathBuf) -> ProfileParams {
        ProfileParams {
            path,
            as_nick: Some("me".to_string()),
            section: Some("role".to_string()),
            action: ProfileAction::SetBody("dev".to_string()),
        }
    }

    fn row_values(report: &Report, key: &str) -> Vec<Vec<String>> {
        report
            .rows
            .iter()
            .filter_map(|r| match r {
                Row::Fields { key: k, values, .. } if k == key => Some(values.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn flock_refusal_cleans_file_and_dirs_this_call_created() {
        let dir = scratch("flock-clean");
        std::fs::write(dir.join("state.agent-besties.md"), LOCKED_BY_OTHER).unwrap();
        let target = dir.join("sub/notes.agent-besties.md");
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &set_section(target.clone())).unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 3);
        // The refusal runs the plan-Err rollback: no file, no directory.
        assert!(!target.exists());
        assert!(!dir.join("sub").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flock_refusal_keeps_preexisting_empty_target() {
        let dir = scratch("flock-adopted");
        std::fs::write(dir.join("state.agent-besties.md"), LOCKED_BY_OTHER).unwrap();
        let target = dir.join("notes.agent-besties.md");
        std::fs::write(&target, b"").unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &set_section(target.clone())).unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 3);
        // Initial-content mode is not ownership: the adopted empty file
        // survives the refusal byte-for-byte.
        assert!(target.exists());
        assert_eq!(std::fs::read(&target).unwrap(), Vec::<u8>::new());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn successful_write_warns_when_lock_scan_skipped_etype() {
        let dir = scratch("scan-etype");
        std::fs::write(dir.join("state.agent-besties.md"), WRONG_KIND_STATE).unwrap();
        let target = dir.join("notes.agent-besties.md");
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &set_section(target.clone())).unwrap();
        // Best-effort: no recognizable holder, the write proceeds — but
        // the unread portion stays visible and the first line is warn.
        assert_eq!(report.status, Status::Warn);
        assert_eq!(report.exit, 0);
        let skipped = row_values(&report, "skipped");
        assert_eq!(skipped.len(), 1, "rows: {:?}", report.rows);
        assert_eq!(skipped[0][1], "E-TYPE");
        assert!(target.exists());
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.starts_with("# notes.agent-profile\n"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flock_still_blocks_with_duplicate_state_sections() {
        let dir = scratch("flock-dup");
        std::fs::write(dir.join("state.agent-besties.md"), DUP_LOCKED_SECTIONS).unwrap();
        let target = dir.join("sub/notes.agent-besties.md");
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &set_section(target.clone())).unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 3);
        // Parseable locks from the duplicate sections still refuse the
        // write, and the incompleteness is reported as a partial row.
        let partial = row_values(&report, "partial");
        assert_eq!(partial.len(), 1, "rows: {:?}", report.rows);
        assert_eq!(partial[0][1], "E-STATUS-DUP");
        assert!(!target.exists());
        assert!(!dir.join("sub").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_rejects_invalid_stem_without_leaving_a_trace() {
        let dir = scratch("ename");
        let target = dir.join("bad stem.agent-besties.md");
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(
            &mut ctx,
            &ProfileParams {
                path: target.clone(),
                as_nick: None,
                section: None,
                action: ProfileAction::None,
            },
        )
        .unwrap();
        assert_eq!(report.status, Status::Error);
        assert_eq!(report.exit, 1);
        assert!(!target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn existing_file_with_mismatched_h1_name_stays_writable() {
        let dir = scratch("mismatch");
        // Invalid raw stem (space): if NAME gating ever escaped the
        // initial-content branch, this existing valid file would be refused.
        let target = dir.join("hand renamed.agent-besties.md");
        std::fs::write(
            &target,
            "# other.agent-profile\n\n## role\n\n```\nold\n```\n",
        )
        .unwrap();
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &set_section(target.clone())).unwrap();
        // A valid file whose H1 name differs from the stem is check's
        // W-NAME, never a new write refusal.
        assert_eq!(report.status, Status::Ok);
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.starts_with("# other.agent-profile\n"));
        let document = crate::doc::parse(content.into_bytes(), &target, ctx.now).unwrap();
        assert_eq!(document.body(0), Some("dev"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Restore listing permission so fixture cleanup can remove the tree.
    struct ListingDenial {
        dir: PathBuf,
        #[cfg(unix)]
        orig_mode: u32,
        #[cfg(windows)]
        user: String,
    }

    impl Drop for ListingDenial {
        fn drop(&mut self) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &self.dir,
                    std::fs::Permissions::from_mode(self.orig_mode),
                );
            }
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("icacls")
                    .arg(&self.dir)
                    .arg("/remove:d")
                    .arg(&self.user)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            }
        }
    }

    /// Deny directory listing while keeping known-path create/open if the
    /// platform actually honors that split. `None` when it does not.
    fn deny_listing(dir: &Path) -> Option<ListingDenial> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let orig_mode = std::fs::metadata(dir).ok()?.permissions().mode();
            let mut perms = std::fs::metadata(dir).ok()?.permissions();
            perms.set_mode(0o300);
            std::fs::set_permissions(dir, perms).ok()?;
            let denial = ListingDenial {
                dir: dir.to_path_buf(),
                orig_mode,
            };
            if std::fs::read_dir(dir).is_ok() {
                return None;
            }
            Some(denial)
        }
        #[cfg(windows)]
        {
            let user = {
                let output = std::process::Command::new("whoami").output().ok()?;
                let name = String::from_utf8(output.stdout).ok()?;
                let name = name.trim().to_string();
                if name.is_empty() {
                    return None;
                }
                name
            };
            let ok = std::process::Command::new("icacls")
                .arg(dir)
                .arg("/deny")
                .arg(format!("{user}:(RD)"))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .ok()?
                .success();
            if !ok {
                return None;
            }
            let denial = ListingDenial {
                dir: dir.to_path_buf(),
                user,
            };
            if std::fs::read_dir(dir).is_ok() {
                return None;
            }
            Some(denial)
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = dir;
            None
        }
    }

    #[test]
    fn successful_write_warns_when_lock_scan_dir_is_unreadable() {
        let dir = scratch("scan-unreadable");
        let target = dir.join("notes.agent-besties.md");
        std::fs::write(
            &target,
            "# notes.agent-profile\n\n## role\n\n```\nold\n```\n",
        )
        .unwrap();
        let Some(_denial) = deny_listing(&dir) else {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        };
        // Listing must be denied without also denying the known-path write;
        // otherwise this is not the unreadable-directory scan case.
        if std::fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .is_err()
        {
            drop(_denial);
            let _ = std::fs::remove_dir_all(&dir);
            return;
        }
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = run(&mut ctx, &set_section(target.clone())).unwrap();
        // Best-effort: no recognizable holder, so the write proceeds — but
        // an uninspected directory must not look like a clean ok.
        assert_eq!(report.status, Status::Warn);
        assert_eq!(report.exit, 0);
        let skipped = row_values(&report, "skipped");
        assert!(
            skipped
                .iter()
                .any(|row| row.get(1).map(String::as_str) == Some("unreadable")),
            "unreadable skip missing: {:?}",
            report.rows
        );
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.starts_with("# notes.agent-profile\n"));
        let document = crate::doc::parse(content.into_bytes(), &target, ctx.now).unwrap();
        assert_eq!(document.body(0), Some("dev"));
        drop(_denial);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
