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
use crate::ops::{ReadBriefParams, reason_report};
use crate::report::Report;

use super::{LoadError, blocking_report, file_missing_report, load_file, type_report};

pub fn read_brief(ctx: &mut Ctx, p: &ReadBriefParams) -> Result<Report, Fault> {
    let object = context::object_of_path(&p.path);
    let dctx = context::diag_context_for_file(ctx, "read-brief", &p.path);
    let file_display = context::display_file(&p.path);
    match load_file(&p.path, ctx.now, consts::READ_WAIT_MS) {
        Err(LoadError::Missing) => Ok(file_missing_report(&dctx, "read-brief", &object, &p.path)),
        Err(LoadError::Busy { id, params }) => {
            Ok(reason_report(&dctx, "read-brief", &object, id, params))
        }
        Err(LoadError::Blocking(diags)) => Ok(blocking_report(
            "read-brief",
            &object,
            &file_display,
            None,
            diags,
        )),
        Ok(document) => {
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
                    if let Some(title) = p.entry.as_deref()
                        && !document
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
                                ("title", title.into()),
                                ("path_arg", p.path.as_os_str().to_os_string()),
                            ],
                        ));
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
        Err(LoadError::Fault(f)) => Err(f),
    }
}

fn render(
    path: &Path,
    object: &str,
    document: &doc::Document,
    entry_filter: Option<&str>,
) -> Report {
    let mut report = Report::new("read-brief", object);
    // §10.10.1: `--entry` renders only that entry, no description.
    if entry_filter.is_none()
        && let Some(body) = &document.brief.description
    {
        report.text("description", document.text(&body.content).to_string());
    }
    let mut warn = false;
    for entry in &document.brief.entries {
        if let Some(title) = entry_filter
            && entry.title != title
        {
            continue;
        }
        let verification = hash::verify(path, entry);
        // Renderable when the in-memory target and its content span are
        // present: empty files have content_span 0..0 and no line range
        // (§10.10.2 "full text"). Oversized no-regex targets stream the hash
        // with content = None (§10.4.2 / §10.10.3).
        let content = verification
            .target
            .as_ref()
            .and_then(|target| target.content.as_deref());
        let rendered = verification.content_span.is_some() && content.is_some();
        if matches!(verification.state, "shifted" | "stale" | "unchecked") || !rendered {
            warn = true;
        }
        let mut values = vec![
            names::display_text(&entry.title),
            verification.state.to_string(),
            entry
                .path
                .as_ref()
                .map(|property| names::display_text(&hash::normalized_target(&property.value)))
                .unwrap_or_else(|| "?".to_string()),
        ];
        if let Some(reason) = verification.reason {
            values.push(reason.to_string());
        } else if !rendered
            && verification.target.as_ref().is_some_and(|target| {
                target.content.is_none() && target.bytes > consts::MAX_TARGET_BYTES as u64
            })
        {
            values.push("target too large".to_string());
        }
        if rendered && let Some((first, last)) = verification.range {
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
            let target = content.unwrap_or_default();
            let span = verification.content_span.clone().unwrap_or_default();
            report.text("content", target[span].to_string());
        }
    }
    if warn {
        report.warn();
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::context::test_ctx;
    use crate::report::{Row, Status};
    use crate::time;
    use std::io::Write;
    use std::path::PathBuf;

    const NOW: &str = "2026-09-25T12:00:00Z";

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-read-brief-tests")
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

    fn text_blocks(report: &Report, key: &str) -> Vec<String> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Text { key: k, content } if k == key => Some(content.clone()),
                _ => None,
            })
            .collect()
    }

    fn write_brief(dir: &std::path::Path, body: &str) -> PathBuf {
        let path = dir.join("plan.agent-besties.md");
        std::fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn empty_target_is_a_complete_empty_render() {
        let dir = scratch("empty");
        std::fs::write(dir.join("empty.txt"), b"").unwrap();
        let path = write_brief(
            &dir,
            "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## empty.txt\n\n- path: empty.txt\n- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
        );
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = read_brief(&mut ctx, &ReadBriefParams { path, entry: None }).unwrap();
        assert_eq!(report.status, Status::Ok);
        assert_eq!(report.exit, 0);
        assert_eq!(
            field_values(&report, "entry"),
            vec![vec![
                "empty.txt".to_string(),
                "fresh".to_string(),
                "empty.txt".to_string()
            ]]
        );
        assert_eq!(text_blocks(&report, "content"), vec![String::new()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_target_path_column_uses_slash() {
        let dir = scratch("slash");
        let nested = dir.join("src").join("parser");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("mod.rs"), b"").unwrap();
        let path = write_brief(
            &dir,
            "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## mod.rs\n\n- path: src\\parser\\mod.rs\n- hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
        );
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = read_brief(&mut ctx, &ReadBriefParams { path, entry: None }).unwrap();
        assert_eq!(report.status, Status::Ok);
        assert_eq!(
            field_values(&report, "entry"),
            vec![vec![
                "mod.rs".to_string(),
                "fresh".to_string(),
                "src/parser/mod.rs".to_string()
            ]]
        );
        assert_eq!(text_blocks(&report, "content"), vec![String::new()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversize_no_regex_keeps_hash_state_and_explains_too_large() {
        let dir = scratch("oversize");
        let name = "big.bin";
        let full = dir.join(name);
        let mut pattern = [0u8; 64 * 1024];
        for (i, byte) in pattern.iter_mut().enumerate() {
            *byte = b'a' + (i % 26) as u8;
        }
        {
            let mut file = std::fs::File::create(&full).unwrap();
            let limit = consts::MAX_TARGET_BYTES as u64;
            let mut written: u64 = 0;
            while written < limit + 1 {
                let take = pattern.len().min((limit + 1 - written) as usize);
                file.write_all(&pattern[..take]).unwrap();
                written += take as u64;
            }
        }
        let sha = hash::read_target(&full).unwrap().sha256;
        let path = write_brief(
            &dir,
            &format!(
                "# plan.brief\n\n> paths below are relative to this file's directory; hashes are SHA-256 of the target's bytes.\n\n## big.bin\n\n- path: big.bin\n- hash: {sha}\n"
            ),
        );
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = read_brief(&mut ctx, &ReadBriefParams { path, entry: None }).unwrap();
        assert_eq!(report.status, Status::Warn);
        assert_eq!(report.exit, 0);
        assert_eq!(
            field_values(&report, "entry"),
            vec![vec![
                "big.bin".to_string(),
                "fresh".to_string(),
                "big.bin".to_string(),
                "target too large".to_string()
            ]]
        );
        assert!(text_blocks(&report, "content").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn entry_title_column_replaces_controls_without_touching_content_or_filter_identity() {
        let dir = scratch("title-cc");
        std::fs::write(dir.join("note.txt"), b"hello\x1bworld").unwrap();
        let sha = hash::read_target(&dir.join("note.txt")).unwrap().sha256;
        let path = write_brief(
            &dir,
            &format!(
                "# plan.brief\n\n## t\u{1b}itle\n- path: note.txt\n- hash: {sha}\n## t\u{07}itle\n- path: note.txt\n- hash: {sha}\n"
            ),
        );
        let mut ctx = test_ctx(time::parse(NOW).unwrap());
        let report = read_brief(
            &mut ctx,
            &ReadBriefParams {
                path: path.clone(),
                entry: None,
            },
        )
        .unwrap();
        let entries = field_values(&report, "entry");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0][0], "t\u{FFFD}itle");
        assert_eq!(entries[1][0], "t\u{FFFD}itle");
        assert!(
            entries
                .iter()
                .all(|values| !values[0].contains('\u{1b}') && !values[0].contains('\u{07}'))
        );
        let contents = text_blocks(&report, "content");
        assert_eq!(contents.len(), 2);
        assert!(
            contents.iter().all(|content| content.contains('\u{1b}')),
            "full content must keep raw Cc: {contents:?}"
        );

        let filtered = read_brief(
            &mut ctx,
            &ReadBriefParams {
                path,
                entry: Some("t\u{1b}itle".to_string()),
            },
        )
        .unwrap();
        assert_eq!(field_values(&filtered, "entry").len(), 1);
        assert_eq!(field_values(&filtered, "entry")[0][0], "t\u{FFFD}itle");
        let display_filter = read_brief(
            &mut ctx,
            &ReadBriefParams {
                path: write_brief(
                    &dir,
                    &format!("# plan.brief\n\n## t\u{1b}itle\n- path: note.txt\n- hash: {sha}\n"),
                ),
                entry: Some("t\u{FFFD}itle".to_string()),
            },
        )
        .unwrap();
        assert!(
            field_values(&display_filter, "entry").is_empty(),
            "filter identity stays on the raw folded title"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
