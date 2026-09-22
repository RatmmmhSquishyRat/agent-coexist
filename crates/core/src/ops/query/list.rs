//! `list [DIR] [--type <type>]... [WALK]` — the collaboration-file tree
//! under DIR (spec §7.2, §8.2 `list`, §10.1).
//!
//! Unreadable files stay visible as leaves with type `?`, lock-busy
//! files as `busy` — both counted and both warn (§1, §10.1.1); `list`
//! never emits file-level `skipped` rows. The type column is one
//! tree-wide start column (§8.2): `max(TREE_TYPE_COL, max over shown
//! leaves of (indent + stem chars) + 2)`, computed after collecting so
//! the rows are built last.

use std::path::Path;

use crate::consts;
use crate::doc;
use crate::error::Fault;
use crate::fsx;
use crate::ops::context::{self, Ctx};
use crate::ops::ListParams;
use crate::report::Report;

use super::dir_error;

/// A leaf that will be rendered (post type filtering).
struct Leaf {
    /// Directory components between DIR and the file, lexical.
    components: Vec<String>,
    stem: String,
    indent: usize,
    type_col: String,
    /// `busy` or `?`: the type is unknown — warns (§8.1.11).
    unreadable: bool,
}

enum LeafKind {
    Known(doc::Kind),
    Unknown,
    Busy,
}

pub fn list(ctx: &mut Ctx, p: &ListParams) -> Result<Report, Fault> {
    fsx::walk::validate(&p.walk)?;
    let object = context::object_of_dir(&p.dir);
    if let Some(rep) = dir_error(ctx, "list", &p.dir)? {
        return Ok(rep);
    }
    let scanned = {
        let mut sink = ctx.walk_sink();
        fsx::walk::scan(&p.dir, &p.walk, &mut sink)?
    };
    // §8.4.1: the file counter counts files READ (the loop below,
    // failures included), not files merely found during traversal.
    ctx.progress(scanned.dirs, 0);

    let mut report = Report::new("list", object.clone());
    let ignore_value = if p.walk.no_ignore {
        "--no-ignore".to_string()
    } else if scanned.ignore_sources.is_empty() {
        "none".to_string()
    } else {
        scanned.ignore_sources.join(", ")
    };
    report.kv("ignore", vec![ignore_value]);

    let dir_display = object;
    let mut leaves: Vec<Leaf> = Vec::new();
    for file in &scanned.files {
        ctx.file_read();
        let file_display = context::display_file(file);
        let components = relative_components(&file_display, &dir_display);
        if components.is_empty() {
            continue;
        }
        let file_name = components.last().map(|c| c.as_str()).unwrap_or_default();
        let stem = stem_of(file_name).to_string();
        let indent = 2 + 2 * (components.len() - 1);
        let kind = classify(ctx, file);
        // §7.1.8: `--type` is a union; non-matching leaves are hidden
        // entirely (no row, not counted). A leaf whose type could not be
        // determined (`busy` / `?`) cannot be proven non-matching and
        // stays visible — `list` never silently loses files.
        let shown = match &kind {
            LeafKind::Known(k) => p.types.is_empty() || p.types.contains(k),
            LeafKind::Unknown | LeafKind::Busy => true,
        };
        if !shown {
            continue;
        }
        let (type_col, unreadable) = match kind {
            LeafKind::Known(k) => (k.as_str().to_string(), false),
            LeafKind::Unknown => ("?".to_string(), true),
            LeafKind::Busy => ("busy".to_string(), true),
        };
        leaves.push(Leaf {
            components,
            stem,
            indent,
            type_col,
            unreadable,
        });
    }

    // One type-column start for the whole tree (§8.2; Unicode scalar
    // counts per §13 Q14), so the rows are built only after collecting.
    let mut type_start = consts::TREE_TYPE_COL;
    for leaf in &leaves {
        type_start = type_start.max(leaf.indent + leaf.stem.chars().count() + 2);
    }

    let mut prev_dirs: Vec<String> = Vec::new();
    for leaf in &leaves {
        let dirs = &leaf.components[..leaf.components.len() - 1];
        let mut common = 0usize;
        while common < dirs.len() && common < prev_dirs.len() && prev_dirs[common] == dirs[common] {
            common += 1;
        }
        for i in common..dirs.len() {
            report.record(format!("{}/", dirs[i]), Vec::new(), 2 + 2 * i, 0);
        }
        prev_dirs = dirs.to_vec();
        report.record(
            leaf.stem.clone(),
            vec![leaf.type_col.clone()],
            leaf.indent,
            type_start - leaf.indent - 2,
        );
    }

    let mut warn = leaves.iter().any(|leaf| leaf.unreadable);
    let mut count = leaves.len();

    // §10.1.5: unreadable directories — `skipped` rows, counted.
    let mut skipped: Vec<String> = scanned
        .unreadable_dirs
        .iter()
        .map(|dir| context::display_file(dir))
        .collect();
    skipped.sort();
    for dir in skipped {
        report.record(
            "skipped",
            vec![dir, "unreadable".to_string()],
            2,
            consts::KEY_WIDTH,
        );
        count += 1;
        warn = true;
    }

    // §10.1.3 / §8.2: include globs that matched nothing while ignore
    // rules were active (not counted, §8.1.10).
    if !p.walk.no_ignore {
        for glob in &scanned.zero_includes {
            report.record(
                "include",
                vec![
                    glob.clone(),
                    "matched 0 files; ignored directories are not reopened (pass --no-ignore)"
                        .to_string(),
                ],
                2,
                consts::KEY_WIDTH,
            );
            warn = true;
        }
    }

    report.total(count);
    if warn {
        report.warn();
    }
    Ok(report)
}

/// Read one scanned file just far enough to learn its type (§10.2 scan
/// read: shared lock, `SCAN_READ_WAIT_MS`).
fn classify(ctx: &Ctx, file: &Path) -> LeafKind {
    match fsx::read(file, consts::SCAN_READ_WAIT_MS, None) {
        Err(Fault::Reason { .. }) => LeafKind::Busy,
        Err(_) => LeafKind::Unknown,
        Ok(bytes) => match doc::parse(bytes, file, ctx.now) {
            Ok(parsed) => match parsed.kind {
                Some(kind) => LeafKind::Known(kind),
                None => LeafKind::Unknown,
            },
            Err(_) => LeafKind::Unknown,
        },
    }
}

/// Lexical components of `file_display` relative to `dir_display`.
fn relative_components(file_display: &str, dir_display: &str) -> Vec<String> {
    let strip = |s: &str| s.strip_prefix("./").unwrap_or(s);
    let file = strip(file_display);
    let dir = strip(dir_display);
    let rel = if dir == "." {
        file
    } else {
        let prefix = format!("{dir}/");
        file.strip_prefix(&prefix).unwrap_or(file)
    };
    rel.split('/')
        .filter(|component| !component.is_empty())
        .map(str::to_string)
        .collect()
}

/// Stem of a collaboration file name: everything before the suffix
/// (matched case-insensitively, §10.1.4; the display keeps the actual
/// file name's stem bytes).
fn stem_of(file_name: &str) -> &str {
    let suffix = consts::SUFFIX;
    if file_name.len() >= suffix.len()
        && file_name[file_name.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
    {
        &file_name[..file_name.len() - suffix.len()]
    } else {
        file_name
    }
}
