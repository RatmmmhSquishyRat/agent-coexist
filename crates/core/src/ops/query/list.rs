//! `list [DIR] [--type <type>]... [WALK]` — the collaboration-file tree
//! under DIR (spec §7.2, §8.2 `list`, §10.1).
//!
//! Unreadable files stay visible as leaves with type `?`, lock-busy
//! files as `busy` — both counted and both warn (§1, §10.1.1); `list`
//! never emits file-level `skipped` rows. The type column is one
//! tree-wide start column (§8.2): `max(TREE_TYPE_COL, max over shown
//! leaves of (indent + stem chars) + 2)`, computed after collecting so
//! the rows are built last.

use std::ffi::OsString;
use std::path::Path;

use crate::consts;
use crate::doc;
use crate::error::Fault;
use crate::fsx;
use crate::names;
use crate::ops::context::{self, Ctx};
use crate::ops::ListParams;
use crate::report::Report;

use super::{dir_error, include_hints};

/// Native directory identity; `display` is folded once for emit.
struct DirKey {
    native: OsString,
    display: String,
}

impl DirKey {
    fn new(native: OsString) -> Self {
        let display = names::fold(&native.to_string_lossy());
        Self { native, display }
    }
}

/// A leaf that will be rendered (post type filtering).
struct Leaf {
    /// Native directories between the requested DIR and the file.
    dirs: Vec<DirKey>,
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

    let mut report = Report::new("list", object.clone());
    let ignore_value = if p.walk.no_ignore {
        "--no-ignore".to_string()
    } else if scanned.ignore_sources.is_empty() {
        "none".to_string()
    } else {
        scanned.ignore_sources.join(", ")
    };
    report.kv("ignore", vec![ignore_value]);

    let mut leaves: Vec<Leaf> = Vec::new();
    for file in &scanned.files {
        ctx.file_read();
        let Some((native_dirs, file_name)) = fsx::walk::relative_native_parts(file, &p.dir) else {
            continue;
        };
        let stem = names::fold(stem_of(&file_name.to_string_lossy()));
        let dirs: Vec<DirKey> = native_dirs.into_iter().map(DirKey::new).collect();
        let indent = 2 + 2 * dirs.len();
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
            dirs,
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

    let mut prev_dirs: Vec<OsString> = Vec::new();
    for leaf in &leaves {
        let mut common = 0usize;
        while common < leaf.dirs.len()
            && common < prev_dirs.len()
            && prev_dirs[common] == leaf.dirs[common].native
        {
            common += 1;
        }
        for (i, dir) in leaf.dirs.iter().enumerate().skip(common) {
            report.record(format!("{}/", dir.display), Vec::new(), 2 + 2 * i, 0);
        }
        prev_dirs = leaf.dirs.iter().map(|d| d.native.clone()).collect();
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
        warn |= include_hints(&mut report, &scanned.zero_includes);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::context::test_ctx;
    use crate::report::Row;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("coexist-list-tests")
            .join(format!(
                "{}-{}-{}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn list_no_ignore(dir: &Path) -> Report {
        let mut ctx = test_ctx(crate::time::parse("2026-09-25T12:00:00Z").unwrap());
        list(
            &mut ctx,
            &ListParams {
                dir: dir.to_path_buf(),
                types: Vec::new(),
                walk: fsx::walk::Options {
                    no_ignore: true,
                    ..Default::default()
                },
            },
        )
        .unwrap()
    }

    fn tree_rows(report: &Report) -> Vec<(usize, &str, Vec<&str>)> {
        report
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Fields {
                    key,
                    values,
                    indent,
                    ..
                } if key != "ignore" && key != "total" && key != "skipped" => Some((
                    *indent,
                    key.as_str(),
                    values.iter().map(String::as_str).collect(),
                )),
                _ => None,
            })
            .collect()
    }

    fn write_post(dir: &Path, stem: &str) {
        std::fs::write(
            dir.join(format!("{stem}{}", consts::SUFFIX)),
            format!("# {stem}.post\n"),
        )
        .unwrap();
    }

    #[test]
    fn folded_sibling_directories_keep_separate_tree_groups() {
        let dir = scratch("folded-tree-groups");
        let double = dir.join("a  b");
        let single = dir.join("a b");
        std::fs::create_dir(&double).unwrap();
        std::fs::create_dir(&single).unwrap();
        write_post(&double, "one");
        write_post(&double, "three");
        write_post(&single, "four");
        write_post(&single, "two");
        let report = list_no_ignore(&dir);
        assert_eq!(
            tree_rows(&report),
            vec![
                (2, "a b/", vec![]),
                (4, "one", vec!["post"]),
                (4, "three", vec!["post"]),
                (2, "a b/", vec![]),
                (4, "four", vec!["post"]),
                (4, "two", vec!["post"]),
            ]
        );
        assert!(report.rows.iter().any(|row| match row {
            Row::Fields { key, values, .. } => {
                key == "total" && values.first().map(String::as_str) == Some("4")
            }
            _ => false,
        }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn list_tree_is_relative_to_the_literal_requested_dir() {
        let parent = scratch("literal-root");
        let dir = parent.join("room ");
        std::fs::create_dir(&dir).unwrap();
        write_post(&dir, "one");
        let report = list_no_ignore(&dir);
        assert_eq!(tree_rows(&report), vec![(2, "one", vec!["post"])]);
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn relative_parts_skip_leading_curdir_and_keep_trailing_space_dir() {
        let trailing = PathBuf::from("room ").join("one.agent-besties.md");
        let (dirs, name) = fsx::walk::relative_native_parts(&trailing, Path::new("room "))
            .expect("trailing-space DIR");
        assert!(dirs.is_empty());
        assert_eq!(name, OsString::from("one.agent-besties.md"));

        let (dirs, name) = fsx::walk::relative_native_parts(
            Path::new("./-/note.agent-besties.md"),
            Path::new("-"),
        )
        .expect("dash DIR vs ./- file");
        assert!(dirs.is_empty(), "extra dir level: {dirs:?}");
        assert_eq!(name, OsString::from("note.agent-besties.md"));

        let (dirs, name) = fsx::walk::relative_native_parts(
            Path::new("-/note.agent-besties.md"),
            Path::new("./-"),
        )
        .expect("./- DIR vs - file");
        assert!(dirs.is_empty(), "extra dir level: {dirs:?}");
        assert_eq!(name, OsString::from("note.agent-besties.md"));
    }
}
