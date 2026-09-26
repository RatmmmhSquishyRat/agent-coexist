//! Directory traversal and path helpers (spec §10.1, §10.5.2, §10.1.6,
//! §8.1.9, §2.4.3).
//!
//! This module only *enumerates*. Managed files are collected by name and
//! never read, never locked, and never size-checked here — the locked
//! lifecycle in [`crate::fsx`] owns all reads. Directory identity for
//! conflict bookkeeping (canonicalization) is equally not here: [`ancestors`]
//! is purely path-mechanical and works on whatever path the caller supplies
//! (§10.5.2 canonicalizes before calling it).
//!
//! Ignore handling mirrors the `ignore` crate exactly (§10.1.2): with
//! `--no-ignore` every ignore source is disabled but hidden entries stay
//! traversable; without it, `.gitignore` / `.ignore` / `.git/info/exclude` /
//! global gitignore rules prune the walk, and [`scan`] additionally reports
//! which source kinds were actually in play so `list` can render its
//! `ignore` key truthfully (§8.2). `--include` / `--exclude` (§10.1.3) are a
//! post-filter on the collected managed files only — they can never reopen a
//! directory pruned by ignore rules.

use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf, Prefix};

use globset::{Candidate, GlobBuilder};
use ignore::WalkBuilder;

use crate::error::Fault;

/// File name of the per-directory state file (spec §2.4.2, §10.1.6).
const STATE_FILE_NAME: &str = "state.agent-besties.md";

/// `FILE_ATTRIBUTE_REPARSE_POINT`: junctions and symlinks on Windows carry
/// this attribute on their directory entry (spec §10.1.1).
#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

/// Traversal filter options shared by `list` / `read-notification` /
/// `check-state` (the `WALK` grammar, spec §7.2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// `--include <glob>` arguments in CLI order.
    pub include: Vec<String>,
    /// `--exclude <glob>` arguments in CLI order.
    pub exclude: Vec<String>,
    /// `--no-ignore`: disable every ignore source (spec §10.1.2).
    pub no_ignore: bool,
}

/// Outcome of a directory walk (spec §10.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkResult {
    /// Managed files (file name ends with the managed suffix, §10.1.4),
    /// lexically sorted by path. Enumerated only — never read here.
    pub files: Vec<PathBuf>,
    /// Directories that produced walk errors, sorted; the walk continued
    /// past them (spec §10.1.5).
    pub unreadable_dirs: Vec<PathBuf>,
    /// Ignore source kinds actually in play, in spec order (§10.1.2): a
    /// subset of `.gitignore` / `.ignore` / `exclude` / `global`, `none`
    /// when empty, or `--no-ignore`.
    pub ignore_sources: Vec<String>,
    /// The original `--include` arguments when they matched nothing while
    /// ignore rules were active (drives the `include ... matched 0 files`
    /// warn line); empty otherwise.
    pub zero_includes: Vec<String>,
    /// Directories entered, root included. Pruned (`.git`, ignored) and
    /// never-entered (dir symlinks, junctions) directories do not count.
    pub dirs: usize,
}

/// One compiled `--include` / `--exclude` pattern (spec §10.1.3).
struct Pattern {
    matcher: globset::GlobMatcher,
    /// Pattern is anchored to the scan root (it contained a `/` after
    /// normalization, or carried a leading `/`): it is matched against the
    /// file's root-relative path instead of the bare file name.
    anchored: bool,
    /// Pattern ended with `/`: a directory pattern. It matches only through
    /// the file's ancestor directories — never the file's own path or name,
    /// so `foo/` excludes files *under* a `foo` directory but spares a
    /// regular file that merely happens to be named `foo`.
    dir_only: bool,
}

impl Pattern {
    /// `rel` is the file's path relative to the scan root (`/`-joined native
    /// components), `name` its file name, `anc_dirs` / `anc_names` the
    /// ancestor directories of `rel` as accumulated relative paths / bare
    /// names. Construct byte candidates explicitly: globset's Path-based
    /// candidate constructor is lossy on non-Unix platforms.
    fn matches(
        &self,
        rel: &Path,
        name: &OsStr,
        anc_dirs: &[PathBuf],
        anc_names: &[OsString],
    ) -> bool {
        if self.dir_only {
            if self.anchored {
                anc_dirs.iter().any(|d| self.matches_native(d.as_os_str()))
            } else {
                anc_names.iter().any(|n| self.matches_native(n.as_os_str()))
            }
        } else if self.anchored {
            self.matches_native(rel.as_os_str())
        } else {
            self.matches_native(name)
        }
    }

    fn matches_native(&self, path: &OsStr) -> bool {
        self.matcher
            .is_match_candidate(&Candidate::from_bytes(path.as_encoded_bytes()))
    }
}

/// Compile one pattern exactly the way [`scan`] will use it: `\` normalized
/// to `/`, one leading `/` stripping to root-anchored form (relative paths
/// carry no leading slash), one trailing `/` marking a directory pattern,
/// gitignore glob semantics (`*` / `?` never cross `/`, `**` does;
/// case-insensitive on Windows and macOS). `Err` carries the message for
/// `invalid glob '<p>': …`.
fn compile_pattern(original: &str) -> Result<Pattern, String> {
    if original.is_empty() {
        return Err("empty pattern".to_string());
    }
    let normalized = original.replace('\\', "/");
    // A leading `/` anchors the pattern to the scan root; the relative
    // paths matched against start at the root's children, so strip it and
    // remember the anchoring instead of requiring a literal leading slash.
    let root_anchored = normalized.starts_with('/');
    let unsuffixed = if root_anchored {
        &normalized[1..]
    } else {
        &normalized[..]
    };
    let dir_only = unsuffixed.ends_with('/');
    let pattern = if dir_only {
        unsuffixed[..unsuffixed.len() - 1].to_string()
    } else {
        unsuffixed.to_string()
    };
    if pattern.is_empty() {
        return Err("empty pattern".to_string());
    }
    let glob = GlobBuilder::new(&pattern)
        .literal_separator(true)
        .case_insensitive(cfg!(windows) || cfg!(target_os = "macos"))
        .build()
        .map_err(|e| e.to_string())?;
    Ok(Pattern {
        matcher: glob.compile_matcher(),
        anchored: root_anchored || pattern.contains('/'),
        dir_only,
    })
}

fn compile_patterns(patterns: &[String]) -> Result<Vec<Pattern>, Fault> {
    patterns
        .iter()
        .map(|p| {
            compile_pattern(p).map_err(|msg| Fault::Usage(format!("invalid glob '{}': {}", p, msg)))
        })
        .collect()
}

/// Validate `--include` / `--exclude` patterns (spec §10.1.3): each must be
/// non-empty and must build as a glob with the exact settings [`scan`]
/// applies. No other checks.
pub fn validate(opts: &Options) -> Result<(), Fault> {
    for pattern in opts.include.iter().chain(opts.exclude.iter()) {
        if let Err(msg) = compile_pattern(pattern) {
            return Err(Fault::Usage(format!("invalid glob '{}': {}", pattern, msg)));
        }
    }
    Ok(())
}

/// `true` when the walker may yield (and, for directories, descend into) the
/// entry.
///
/// Skips `.git` directories case-insensitively on every platform (§10.1.1).
/// On Windows additionally skips junctions — reparse-point directory
/// entries, which are not reliably `is_symlink()`. Symlinks whose target is
/// a directory are skipped at the entry layer on every platform, so a
/// managed-suffix directory link is never collected as a file and directory
/// loops are impossible. File symlinks are kept and treated as files
/// (§10.1.1). A `None` file type is treated as non-directory (no special
/// skip).
fn entry_is_enterable(entry: &ignore::DirEntry) -> bool {
    let file_type = entry.file_type();
    let is_dir = file_type.is_some_and(|ft| ft.is_dir());
    if is_dir && entry.file_name().eq_ignore_ascii_case(".git") {
        return false;
    }
    #[cfg(windows)]
    {
        if is_dir {
            if let Ok(metadata) = entry.metadata() {
                use std::os::windows::fs::MetadataExt;
                // Junctions carry the reparse-point attribute on the
                // directory entry; relying on is_symlink() alone would enter
                // them (and possibly loop).
                if (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                    return false;
                }
            }
        }
    }
    if file_type.is_some_and(|ft| ft.is_symlink()) {
        // Directory-target symlinks are skipped here on every platform so a
        // managed-suffix directory link is never collected as a file. File
        // symlinks are kept. A broken symlink (metadata fails) is kept and
        // simply never matches anything the callers care about.
        if let Ok(target) = std::fs::metadata(entry.path()) {
            if target.is_dir() {
                return false;
            }
        }
    }
    true
}

/// `true` when the yielded entry is a directory the walk entered.
///
/// Depth zero is the already-validated scan root. The dependency still
/// yields that entry with symlink `file_type` for an explicit linked
/// directory, so classification must not use `file_type` alone: the root
/// is never a managed-file leaf and is counted once as a directory.
fn is_entered_directory(entry: &ignore::DirEntry) -> bool {
    entry.depth() == 0 || entry.file_type().is_some_and(|ft| ft.is_dir())
}

/// `true` when `name` (a whole file name) ends with the managed suffix,
/// compared ASCII-case-insensitively (§10.1.4).
fn has_managed_suffix(name: &OsStr) -> bool {
    let name_bytes = name.as_encoded_bytes();
    let suffix_bytes = crate::consts::SUFFIX.as_bytes();
    name_bytes.len() >= suffix_bytes.len()
        && name_bytes[name_bytes.len() - suffix_bytes.len()..].eq_ignore_ascii_case(suffix_bytes)
}

/// One fact extracted from an `ignore::Error` tree.
enum WalkErrorFact {
    /// An I/O failure attributed to a path (a directory that could not be
    /// read): the path is reported unreadable and the scan continues with
    /// its siblings (§10.1.5).
    Unreadable(PathBuf),
    /// Anything else — a malformed ignore file, a pathless I/O error, an
    /// unrecognized definition. Surfacing as a scan fault is strictly better
    /// than returning a silently incomplete success.
    Fatal {
        path: Option<PathBuf>,
        message: String,
    },
}

/// `true` when the error is ultimately an I/O error behind the
/// partial / line / depth wrappers.
fn is_io_cause(err: &ignore::Error) -> bool {
    match err {
        ignore::Error::Io(_) => true,
        ignore::Error::WithLineNumber { err, .. }
        | ignore::Error::WithDepth { err, .. }
        | ignore::Error::WithPath { err, .. } => is_io_cause(err),
        ignore::Error::Partial(errors) => !errors.is_empty() && errors.iter().all(is_io_cause),
        _ => false,
    }
}

/// Only access failures can come from an unenterable containing directory;
/// invalid bytes or unsupported source objects remain fatal source errors.
fn is_directory_access_error(err: &ignore::Error) -> bool {
    match err {
        ignore::Error::Partial(errors) => {
            !errors.is_empty() && errors.iter().all(is_directory_access_error)
        }
        _ => err.io_error().is_some_and(|err| {
            matches!(
                err.kind(),
                ErrorKind::PermissionDenied | ErrorKind::NotFound | ErrorKind::NotADirectory
            )
        }),
    }
}

/// Flatten an `ignore::Error` recursively: `Partial` unwraps to each inner,
/// `WithLineNumber` / `WithDepth` unwrap to their inner. Only directory
/// traversal I/O (tagged by the walker with its depth) and loops are
/// unreadable-directory facts. Ignore/config source I/O and parse errors
/// are fatal, keeping their actual source path.
fn classify_error(err: &ignore::Error) -> Vec<WalkErrorFact> {
    match err {
        ignore::Error::Partial(errors) => errors.iter().flat_map(classify_error).collect(),
        ignore::Error::WithLineNumber { err, .. } => classify_error(err),
        ignore::Error::WithDepth { err, .. } => classify_error(err),
        ignore::Error::WithPath { path, err } => {
            if matches!(&**err, ignore::Error::WithDepth { err, .. } if is_io_cause(err)) {
                vec![WalkErrorFact::Unreadable(path.clone())]
            } else {
                // Source rules could not be read or compiled. Never turn
                // that into an empty-looking successful scan.
                vec![WalkErrorFact::Fatal {
                    path: Some(path.clone()),
                    message: format!("{}", &**err),
                }]
            }
        }
        ignore::Error::Loop { child, .. } => vec![WalkErrorFact::Unreadable(child.clone())],
        other => vec![WalkErrorFact::Fatal {
            path: None,
            message: format!("{other}"),
        }],
    }
}

/// Walk `root` recursively and enumerate managed files (spec §10.1).
///
/// `progress` receives cumulative `(dirs_entered, managed_files_found)` and
/// is invoked whenever either counter increments (§8.4 rendering belongs to
/// the caller; this function never prints). `managed_files_found` is an
/// enumeration count, not §8.4.1's `<m>` (files actually read, failures
/// included): callers feeding the progress display keep their own read
/// counter (`ops::context::Ctx::walk_sink` preserves it across the walk).
/// Hidden entries are traversed,
/// `.git` directories and never-entered link directories are pruned, ignore
/// rules prune subtrees unless disabled, unreadable directories are recorded
/// and skipped (§10.1.5), and every other walk error — a malformed ignore
/// file, a pathless I/O failure — surfaces as `Fault::Io` instead of a
/// silently incomplete success. The `--include` / `--exclude` post-filter
/// runs on the collected names only.
pub fn scan(
    root: &Path,
    opts: &Options,
    progress: &mut dyn FnMut(usize, usize),
) -> Result<WalkResult, Fault> {
    // Root pre-check: preserve the io::ErrorKind distinction so callers can
    // translate E-DIR-MISSING (NotFound) vs E-NOT-DIR (§7.3.3).
    let root_metadata =
        std::fs::metadata(root).map_err(|e| Fault::io("scan", root.to_path_buf(), e))?;
    if !root_metadata.is_dir() {
        return Err(Fault::io(
            "scan",
            root.to_path_buf(),
            std::io::Error::new(ErrorKind::NotADirectory, "not a directory"),
        ));
    }

    // WalkBuilder treats a bare `-` as stdin. Always pass a filesystem
    // absolute path, without resolving symlinks or changing the caller's
    // lexical paths in the result/report.
    let walk_root =
        std::path::absolute(root).map_err(|e| Fault::io("scan", root.to_path_buf(), e))?;
    let original_path = |path: &Path| match path.strip_prefix(&walk_root) {
        Ok(relative) => root.join(relative),
        Err(_) => path.to_path_buf(),
    };
    let mut builder = WalkBuilder::new(&walk_root);
    builder.hidden(false).require_git(false).follow_links(false);
    if opts.no_ignore {
        builder
            .git_ignore(false)
            .git_exclude(false)
            .git_global(false)
            .ignore(false)
            .parents(false);
    } else {
        // parents(true) makes ancestor ignore files apply (§10.1.2).
        builder
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .ignore(true)
            .parents(true);
    }
    builder.filter_entry(entry_is_enterable);

    struct Found {
        path: PathBuf,
        rel: PathBuf,
        name: OsString,
    }

    let mut dirs: usize = 0;
    let mut found: usize = 0;
    let mut managed: Vec<Found> = Vec::new();
    let mut unreadable_dirs: Vec<PathBuf> = Vec::new();

    // Absorb classified walk errors: unreadable-directory facts accumulate
    // and scanning continues; any fatal fact aborts the scan. The first
    // fatal wins; a pathless fatal is attributed to the scan root.
    let mut absorb_walk_errors = |facts: Vec<WalkErrorFact>| -> Option<Fault> {
        let mut fatal: Option<Fault> = None;
        for fact in facts {
            match fact {
                WalkErrorFact::Unreadable(path) => unreadable_dirs.push(original_path(&path)),
                WalkErrorFact::Fatal { path, message } => {
                    if fatal.is_none() {
                        fatal = Some(Fault::io(
                            "scan",
                            path.map(|p| original_path(&p))
                                .unwrap_or_else(|| root.to_path_buf()),
                            std::io::Error::new(ErrorKind::InvalidData, message),
                        ));
                    }
                }
            }
        }
        fatal
    };

    // Sequential walk: the ignore crate surfaces every problem through the
    // iterator or the per-entry deferred error, never by printing; both
    // channels go through the same classifier.
    let mut walker = builder.build();
    for entry in walker.by_ref() {
        match entry {
            Ok(e) => {
                if let Some(err) = e.error() {
                    // A directory we cannot enter can also deny attempts to
                    // open its optional ignore files. Keep that failure at
                    // the unreadable-directory boundary, not as a fatal
                    // source failure. Probe only after an actual I/O error.
                    if is_entered_directory(&e)
                        && is_directory_access_error(err)
                        && std::fs::read_dir(e.path()).is_err()
                    {
                        let _ = absorb_walk_errors(vec![WalkErrorFact::Unreadable(
                            e.path().to_path_buf(),
                        )]);
                        continue;
                    }
                    if let Some(fault) = absorb_walk_errors(classify_error(err)) {
                        return Err(fault);
                    }
                }
                if is_entered_directory(&e) {
                    dirs += 1;
                    progress(dirs, found);
                } else {
                    let name = e.file_name();
                    if has_managed_suffix(name) {
                        found += 1;
                        let rel = relative_forward_slashed(e.path(), &walk_root);
                        managed.push(Found {
                            path: original_path(e.path()),
                            rel,
                            name: name.to_os_string(),
                        });
                        progress(dirs, found);
                    }
                }
            }
            Err(err) => {
                if let Some(fault) = absorb_walk_errors(classify_error(&err)) {
                    return Err(fault);
                }
            }
        }
    }

    // Post-filter on the collected files only (§10.1.3); unreachable after
    // validate(), but a bad pattern is never silently dropped.
    let includes = compile_patterns(&opts.include)?;
    let excludes = compile_patterns(&opts.exclude)?;

    let mut files: Vec<PathBuf> = managed
        .into_iter()
        .filter(|f| {
            if includes.is_empty() && excludes.is_empty() {
                return true;
            }
            let (anc_dirs, anc_names) = relative_ancestors(&f.rel);
            let included = includes.is_empty()
                || includes
                    .iter()
                    .any(|p| p.matches(&f.rel, &f.name, &anc_dirs, &anc_names));
            let excluded = excludes
                .iter()
                .any(|p| p.matches(&f.rel, &f.name, &anc_dirs, &anc_names));
            included && !excluded
        })
        .map(|f| f.path)
        .collect();
    files.sort();
    unreadable_dirs.sort();
    unreadable_dirs.dedup();

    let zero_includes = if !opts.include.is_empty() && files.is_empty() && !opts.no_ignore {
        opts.include.clone()
    } else {
        Vec::new()
    };

    let mut ignore_sources = if opts.no_ignore {
        vec!["--no-ignore".to_string()]
    } else {
        let sources = walker.ignore_sources();
        [
            (sources.git_ignore, ".gitignore"),
            (sources.ignore, ".ignore"),
            (sources.git_exclude, "exclude"),
            (sources.git_global, "global"),
        ]
        .into_iter()
        .filter(|(loaded, _)| *loaded)
        .map(|(_, name)| name.to_string())
        .collect()
    };
    if ignore_sources.is_empty() {
        ignore_sources.push("none".to_string());
    }

    Ok(WalkResult {
        files,
        unreadable_dirs,
        ignore_sources,
        zero_includes,
        dirs,
    })
}

/// Join path components with `/` while keeping each component's native
/// `OsStr` bytes. Anchored glob matching uses this form so a Windows `\` in
/// the original path becomes `/` without a lossy Unicode conversion.
fn forward_joined<'a, I>(components: I) -> PathBuf
where
    I: IntoIterator<Item = &'a OsStr>,
{
    let mut out = OsString::new();
    for component in components {
        if !out.is_empty() {
            out.push("/");
        }
        out.push(component);
    }
    PathBuf::from(out)
}

/// `entry_path` relative to `root`, components joined with `/` (the form
/// §10.1.3 anchored patterns match against).
fn relative_forward_slashed(entry_path: &Path, root: &Path) -> PathBuf {
    let rel = entry_path.strip_prefix(root).unwrap_or(entry_path);
    forward_joined(rel.components().map(|c| c.as_os_str()))
}

/// Ancestor directories of a `/`-joined relative file path, both as
/// accumulated relative paths (for anchored directory patterns) and as
/// bare component names (for slash-free directory patterns). The file's
/// own last component is not an ancestor.
fn relative_ancestors(rel: &Path) -> (Vec<PathBuf>, Vec<OsString>) {
    let components: Vec<&OsStr> = rel.components().map(|c| c.as_os_str()).collect();
    let mut dirs = Vec::with_capacity(components.len());
    let mut names = Vec::with_capacity(components.len());
    for end in 1..components.len() {
        dirs.push(forward_joined(components[..end].iter().copied()));
        names.push(components[end - 1].to_os_string());
    }
    (dirs, names)
}

/// `path` itself first, then every parent, near to far (§10.5.2, §10.1.6).
/// Purely path-mechanical — nothing is stat'ed or canonicalized here; the
/// caller passes an already-normalized path. Exactly `Path::ancestors()`
/// order, so the filesystem root (`/`, drive root, UNC share root) is the
/// last element and nothing is skipped.
pub fn ancestors(path: &Path) -> Vec<PathBuf> {
    path.ancestors().map(Path::to_path_buf).collect()
}

/// `true` when `name` (a whole file name) is the state file name: exact
/// `state.agent-besties.md` on Linux, ASCII-case-insensitive on Windows and
/// macOS (§10.1.6 — the same platform rule [`state_file`] locates by; the
/// `W-NAME-STATE` judgement belongs to `check`).
pub fn is_state_file_name(name: &OsStr) -> bool {
    if cfg!(any(windows, target_os = "macos")) {
        name.eq_ignore_ascii_case(STATE_FILE_NAME)
    } else {
        name.eq(OsStr::new(STATE_FILE_NAME))
    }
}

/// Locate the state file inside `dir` (§10.1.6): exact `state.agent-besties.md`
/// on Linux, ASCII-case-insensitive on Windows and macOS. At most one hit is
/// expected per directory; if several case variants coexist, the first in
/// `read_dir` order wins. The returned path keeps the on-disk casing (the
/// `W-NAME-STATE` judgement belongs to `check`, not here).
pub fn state_file(dir: &Path) -> Result<Option<PathBuf>, Fault> {
    let entries = std::fs::read_dir(dir).map_err(|e| Fault::io("scan", dir.to_path_buf(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| Fault::io("scan", dir.to_path_buf(), e))?;
        let name = entry.file_name();
        if is_state_file_name(&name) {
            return Ok(Some(dir.join(name)));
        }
    }
    Ok(None)
}

fn components_joined(path: &Path) -> String {
    let mut out = String::new();
    let mut separator = false;
    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                out.extend(component.as_os_str().to_string_lossy().chars().map(|c| {
                    if c == '\\' {
                        '/'
                    } else {
                        c
                    }
                }))
            }
            Component::RootDir => out.push('/'),
            _ => {
                if separator {
                    out.push('/');
                }
                out.push_str(&component.as_os_str().to_string_lossy());
            }
        }
        // A drive-relative prefix is followed directly by its first name;
        // a root has already supplied the separator.
        separator = !matches!(component, Component::Prefix(_) | Component::RootDir);
    }
    out
}

fn forward_slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Compare volume identities without folding native directory components.
fn display_prefix_identity(prefix: Prefix<'_>) -> Prefix<'_> {
    match prefix {
        Prefix::VerbatimDisk(drive) => Prefix::Disk(drive),
        Prefix::VerbatimUNC(server, share) => Prefix::UNC(server, share),
        prefix => prefix,
    }
}

/// Number of leading volume components (`Prefix` / `RootDir`). A bare
/// verbatim UNC share is rooted even though its iterator omits `RootDir`.
fn volume_component_count(components: &[Component<'_>]) -> usize {
    components
        .iter()
        .take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
        .count()
}

/// Lexical display form of `path` (§8.1.9), with `/` separators, `.` for
/// the current directory itself, and `..` allowed. Relative inputs keep
/// their native prefix/root category, without consulting a per-drive cwd.
/// Same-volume absolute paths relativize through the longest common
/// component prefix — one `..` per cwd component below it, then the
/// remaining path components (paths under the cwd are the zero-`..` case).
/// Ordinary/verbatim drive or UNC prefixes share their native identity;
/// directory components still compare exactly. Across volumes, or when
/// the current directory cannot be read, the path is shown as given with
/// `\` normalized to `/`. Purely lexical: never canonicalizes, never
/// touches the disk for relative paths, and must never be written into files.
pub fn display_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        return ".".to_string();
    }
    if path.is_relative() {
        return components_joined(path);
    }
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(_) => return forward_slashed(path),
    };
    display_absolute_path(path, &cwd)
}

fn display_absolute_path(path: &Path, cwd: &Path) -> String {
    let path_components: Vec<Component<'_>> = path.components().collect();
    let cwd_components: Vec<Component<'_>> = cwd.components().collect();

    let same_volume = match (path_components.first(), cwd_components.first()) {
        (Some(Component::Prefix(path)), Some(Component::Prefix(cwd))) => {
            display_prefix_identity(path.kind()) == display_prefix_identity(cwd.kind())
        }
        (Some(Component::RootDir), Some(Component::RootDir)) => true,
        _ => false,
    };
    if !same_volume {
        return forward_slashed(path);
    }

    // Absolute paths already have roots. Compare their volume identities
    // above, then skip each spelling's root separators independently.
    let path_components = &path_components[volume_component_count(&path_components)..];
    let cwd_components = &cwd_components[volume_component_count(&cwd_components)..];
    let common = path_components
        .iter()
        .zip(cwd_components)
        .take_while(|(path, cwd)| path == cwd)
        .count();

    let mut parts: Vec<String> =
        Vec::with_capacity((cwd_components.len() - common) + (path_components.len() - common));
    for _ in common..cwd_components.len() {
        parts.push("..".to_string());
    }
    for component in &path_components[common..] {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    let joined = parts.join("/");
    if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// Native components of `file` relative to the requested `dir`.
///
/// Ancestry uses Path/OsStr identity. Leading `Component::CurDir` on
/// either side is a spelling of the same relative root (`-` vs `./-`),
/// not a directory level. Folding happens only at emit.
pub(crate) fn relative_native_parts(file: &Path, dir: &Path) -> Option<(Vec<OsString>, OsString)> {
    let file_comps: Vec<_> = file
        .components()
        .skip_while(|c| matches!(c, Component::CurDir))
        .collect();
    let dir_comps: Vec<_> = dir
        .components()
        .skip_while(|c| matches!(c, Component::CurDir))
        .collect();
    let rel = if dir_comps.is_empty() {
        file_comps.as_slice()
    } else if file_comps.starts_with(&dir_comps) {
        &file_comps[dir_comps.len()..]
    } else {
        return None;
    };
    let mut dirs = Vec::new();
    let mut file_name: Option<OsString> = None;
    for component in rel {
        match component {
            Component::Normal(name) => {
                if let Some(prev) = file_name.replace(name.to_os_string()) {
                    dirs.push(prev);
                }
            }
            _ => return None,
        }
    }
    Some((dirs, file_name?))
}

/// Append the managed suffix to the file name unless it is already there
/// (§2.4.3). The presence test is an exact byte compare (a differently-cased
/// name is *not* already-suffixed and gets the suffix appended); other path
/// components are never touched.
pub fn managed_path(path: &Path) -> PathBuf {
    if let Some(name) = path.file_name() {
        if name
            .as_encoded_bytes()
            .ends_with(crate::consts::SUFFIX.as_bytes())
        {
            return path.to_path_buf();
        }
    }
    let mut with_suffix = path.as_os_str().to_os_string();
    with_suffix.push(crate::consts::SUFFIX);
    PathBuf::from(with_suffix)
}

/// Create `dir` (and missing parents) if needed (§7.3.2). Only the creation
/// itself lives here: emitting the `created  <dir>/` lines and the collection
/// of created directories belong to the command layer.
pub fn ensure_dir(dir: &Path) -> Result<(), Fault> {
    std::fs::create_dir_all(dir).map_err(|e| Fault::io("create", dir.to_path_buf(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Component, Path, PathBuf};

    /// Progress sink for scans whose counters the test does not inspect.
    fn sink(_dirs: usize, _files: usize) {}

    /// Unique scratch directory under the temp dir, pre-cleared so stale
    /// leftovers from an earlier run cannot poison the test.
    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("coexist-walk-{}-{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    fn touch(root: &Path, rel: &str) -> PathBuf {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&path, b"").expect("write file");
        path
    }

    fn opts(include: Vec<&str>, exclude: Vec<&str>) -> Options {
        Options {
            include: include.into_iter().map(String::from).collect(),
            exclude: exclude.into_iter().map(String::from).collect(),
            no_ignore: true,
        }
    }

    #[test]
    fn literal_dash_root_is_a_directory_not_stdin() {
        const CHILD: &str = "COEXIST_WALK_DASH_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let result = scan(Path::new("-"), &opts(vec![], vec![]), &mut sink)
                .expect("scan literal dash directory");
            assert_eq!(
                result.files,
                vec![PathBuf::from("-").join("note.agent-besties.md")]
            );
            assert_eq!(result.dirs, 1);
            return;
        }
        let root = scratch("dash-root");
        touch(&root, "-/note.agent-besties.md");
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "fsx::walk::tests::literal_dash_root_is_a_directory_not_stdin",
            ])
            .current_dir(&root)
            .env(CHILD, "1")
            .status()
            .expect("spawn isolated cwd child");
        let _ = fs::remove_dir_all(&root);
        assert!(result.success(), "literal dash directory scan failed");
    }

    #[test]
    fn hidden_dirs_and_files_are_traversed() {
        let root = scratch("hidden");
        touch(&root, ".hidden/x.agent-besties.md");
        touch(&root, ".toplevel.agent-besties.md");
        touch(&root, "plain.md");

        let mut calls: Vec<(usize, usize)> = Vec::new();
        let mut progress = |dirs: usize, files: usize| calls.push((dirs, files));
        let res = scan(&root, &opts(vec![], vec![]), &mut progress).expect("scan ok");

        assert_eq!(res.files.len(), 2);
        assert_eq!(res.dirs, 2); // root + .hidden
        assert_eq!(
            *calls.last().expect("progress called"),
            (res.dirs, res.files.len())
        );
        assert_eq!(res.ignore_sources, vec!["--no-ignore".to_string()]);
        assert!(res.zero_includes.is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn git_directory_skipped_case_insensitively() {
        let root = scratch("gitdir");
        touch(&root, ".Git/inner.agent-besties.md");
        touch(&root, "keep.agent-besties.md");

        let res = scan(&root, &opts(vec![], vec![]), &mut sink).expect("scan ok");

        assert_eq!(res.dirs, 1); // root only: .Git pruned, not entered
        assert_eq!(res.files.len(), 1);
        assert!(display_path(&res.files[0]).ends_with("keep.agent-besties.md"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn basename_and_anchored_globs_differ() {
        let root = scratch("globs");
        touch(&root, "top.agent-besties.md");
        touch(&root, "sub/top.agent-besties.md");
        touch(&root, "sub/deep/nested.agent-besties.md");

        // Basename pattern: matches the name at any depth.
        let res = scan(
            &root,
            &opts(vec!["top.agent-besties.md"], vec![]),
            &mut sink,
        )
        .expect("basename scan ok");
        assert_eq!(res.files.len(), 2);
        assert!(res
            .files
            .iter()
            .all(|f| display_path(f).ends_with("top.agent-besties.md")));

        // Anchored pattern: `*` does not cross `/`, so only sub's direct
        // child matches — not sub/deep/nested (spec §10.1.3).
        let res = scan(&root, &opts(vec!["sub/*"], vec![]), &mut sink).expect("anchored scan ok");
        assert_eq!(res.files.len(), 1);
        assert!(display_path(&res.files[0]).ends_with("sub/top.agent-besties.md"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn directory_pattern_excludes_subtree() {
        let root = scratch("dirpat");
        touch(&root, "build/out.agent-besties.md");
        touch(&root, "build/nested/deep.agent-besties.md");
        touch(&root, "vendor/pkg/lib.agent-besties.md");
        touch(&root, "src/main.agent-besties.md");

        // `build/` is a slash-free directory pattern (any ancestor component
        // named build); `vendor/pkg/` is an anchored directory pattern (any
        // ancestor relative path matching vendor/pkg).
        let res = scan(
            &root,
            &opts(vec![], vec!["build/", "vendor/pkg/"]),
            &mut sink,
        )
        .expect("scan ok");
        assert_eq!(res.files.len(), 1);
        assert!(display_path(&res.files[0]).ends_with("src/main.agent-besties.md"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn managed_suffix_matches_case_insensitively() {
        let root = scratch("suffixcase");
        touch(&root, "UPPER.AGENT-BESTIES.MD");
        touch(&root, "lower.agent-besties.md");
        touch(&root, "unrelated.txt");

        let res = scan(&root, &opts(vec![], vec![]), &mut sink).expect("scan ok");

        let mut names: Vec<String> = res
            .files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        // Collected with the real on-disk casing preserved (§10.1.4).
        assert_eq!(
            names,
            vec![
                "UPPER.AGENT-BESTIES.MD".to_string(),
                "lower.agent-besties.md".to_string(),
            ]
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn display_path_relativizes_absolute_and_relative_paths() {
        assert_eq!(display_path(Path::new("")), ".");
        assert_eq!(display_path(Path::new(".")), ".");
        assert_eq!(display_path(Path::new("./a/../b")), "./a/../b");
        let relative = PathBuf::from("a").join("b").join("c");
        assert_eq!(display_path(&relative), "a/b/c");

        let cwd = std::env::current_dir().expect("cwd available");
        assert_eq!(display_path(&cwd), ".");
        assert_eq!(display_path(&cwd.join("x").join("y.md")), "x/y.md");

        // Same-volume sibling of the cwd: relativized with `..` (§8.1.9).
        let sibling = cwd
            .parent()
            .expect("cwd has a parent")
            .join("coexist-display-outside");
        assert_eq!(display_path(&sibling), "../coexist-display-outside");
    }

    #[cfg(windows)]
    #[test]
    fn display_path_preserves_windows_relative_categories() {
        for (path, expected) in [
            ("C:alice", "C:alice"),
            ("C:", "C:"),
            (r"C:.\alice", "C:alice"),
            (r"C:..\alice", "C:../alice"),
            (r"\alice", "/alice"),
            (r"\", "/"),
        ] {
            assert_eq!(display_path(Path::new(path)), expected, "{path}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn absolute_display_uses_native_volume_identity() {
        for (path, cwd, expected) in [
            (r"\\?\C:\work\repo\alice", r"C:\work\repo", "alice"),
            (r"C:\work\repo\alice", r"\\?\C:\work\repo", "alice"),
            (r"\\?\C:\work\alice", r"C:\work\repo", "../alice"),
            (r"\\?\C:\", r"C:\work\repo", "../.."),
            (
                r"\\?\UNC\server\share\work\alice",
                r"\\server\share\work",
                "alice",
            ),
            (
                r"\\server\share\work\alice",
                r"\\?\UNC\server\share\work",
                "alice",
            ),
            (r"\\?\UNC\server\share", r"\\server\share\work", ".."),
            (r"\\server\share", r"\\?\UNC\server\share", "."),
            (
                r"\\?\UNC\server\other\alice",
                r"\\server\share\work",
                "//?/UNC/server/other/alice",
            ),
            (r"Z:\w\f.md", r"C:\work", "Z:/w/f.md"),
            (r"\\?\Z:\w\f.md", r"C:\work", "//?/Z:/w/f.md"),
            (r"\\?\C:\WORK\alice", r"C:\work\repo", "../../WORK/alice"),
        ] {
            assert_eq!(
                display_absolute_path(Path::new(path), Path::new(cwd)),
                expected,
                "{path} from {cwd}",
            );
        }
    }

    #[test]
    fn managed_path_appends_and_is_idempotent() {
        let base = PathBuf::from("dir").join("file.md");
        let first = managed_path(&base);
        assert_eq!(
            first,
            PathBuf::from("dir").join(format!("file.md{}", crate::consts::SUFFIX))
        );
        assert_eq!(managed_path(&first), first);

        let exact = PathBuf::from(format!("x{}", crate::consts::SUFFIX));
        assert_eq!(managed_path(&exact), exact);

        // Exact byte compare: a case variant is not already-managed.
        let variant = PathBuf::from("X.AGENT-BESTIES.MD");
        assert_eq!(
            managed_path(&variant).file_name().and_then(|n| n.to_str()),
            Some("X.AGENT-BESTIES.MD.agent-besties.md")
        );
    }

    #[test]
    fn state_file_name_case_rules() {
        let exact = scratch("state-exact");
        fs::write(exact.join(STATE_FILE_NAME), b"").expect("write state file");
        let hit = state_file(&exact)
            .expect("read_dir ok")
            .expect("exact name found");
        assert_eq!(hit, exact.join(STATE_FILE_NAME));

        let cased = scratch("state-cased");
        fs::write(cased.join("STATE.agent-besties.md"), b"").expect("write state file");
        let got = state_file(&cased).expect("read_dir ok");
        if cfg!(any(windows, target_os = "macos")) {
            assert!(got.is_some());
        } else {
            assert!(got.is_none());
        }
        let _ = fs::remove_dir_all(&exact);
        let _ = fs::remove_dir_all(&cased);
    }

    #[test]
    fn is_state_file_name_matches_platform_case_rules() {
        use std::ffi::OsStr;

        assert!(is_state_file_name(OsStr::new(STATE_FILE_NAME)));
        assert!(!is_state_file_name(OsStr::new("state.agent-besties.mdx")));
        assert!(!is_state_file_name(OsStr::new("other.agent-besties.md")));
        // A case variant matches only under the Windows/macOS rule — the
        // same platform predicate state_file locates with (§10.1.6).
        if cfg!(any(windows, target_os = "macos")) {
            assert!(is_state_file_name(OsStr::new("STATE.Agent-Besties.MD")));
        } else {
            assert!(!is_state_file_name(OsStr::new("STATE.Agent-Besties.MD")));
        }
    }

    /// Lexically relativize `absolute` against the cwd (`..` segments
    /// allowed), built independently of `display_path` so tests never use
    /// the code under test to construct their own fixtures. `None` when the
    /// two sit on different volumes (no relative form exists).
    fn relative_to_cwd(absolute: &Path) -> Option<PathBuf> {
        let cwd = std::env::current_dir().expect("cwd available");
        let absolute_components: Vec<Component<'_>> = absolute.components().collect();
        let cwd_components: Vec<Component<'_>> = cwd.components().collect();
        let mut common = 0;
        while common < absolute_components.len()
            && common < cwd_components.len()
            && absolute_components[common] == cwd_components[common]
        {
            common += 1;
        }
        let volume = absolute_components
            .iter()
            .take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
            .count();
        if common < volume {
            return None;
        }
        let mut parts: Vec<String> = Vec::new();
        for _ in common..cwd_components.len() {
            parts.push("..".to_string());
        }
        for component in &absolute_components[common..] {
            parts.push(component.as_os_str().to_string_lossy().into_owned());
        }
        Some(PathBuf::from(parts.join("/")))
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_dir_recorded_and_scan_continues() {
        use std::os::unix::fs::PermissionsExt;
        let root = scratch("unreadable");
        touch(&root, "locked/x.agent-besties.md");
        touch(&root, "open/y.agent-besties.md");
        fs::set_permissions(root.join("locked"), fs::Permissions::from_mode(0o000))
            .expect("chmod locked dir");

        let res = scan(&root, &opts(vec![], vec![]), &mut sink).expect("scan ok");

        assert_eq!(res.unreadable_dirs, vec![root.join("locked")]);
        assert_eq!(res.files.len(), 1);
        assert!(display_path(&res.files[0]).ends_with("open/y.agent-besties.md"));

        // Restore so the best-effort cleanup can remove the tree.
        let _ = fs::set_permissions(root.join("locked"), fs::Permissions::from_mode(0o755));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn parent_gitignore_prunes_and_reports_source() {
        let base = scratch("gitignore-parent");
        let root = base.join("root");
        fs::create_dir_all(&root).expect("create scan root");
        fs::write(base.join(".gitignore"), "pruned/\n").expect("write gitignore");
        touch(&root, "pruned/x.agent-besties.md");
        touch(&root, "kept/y.agent-besties.md");

        // Prefer a relative scan root: parent ignore files are read along
        // the canonicalized chain, which a relative root exercises hardest.
        let scan_root = relative_to_cwd(&root).unwrap_or_else(|| root.clone());
        let walk = Options {
            include: vec![],
            exclude: vec![],
            no_ignore: false,
        };
        let res = scan(&scan_root, &walk, &mut sink).expect("scan ok");

        assert_eq!(res.files.len(), 1);
        assert!(display_path(&res.files[0]).ends_with("kept/y.agent-besties.md"));
        assert!(res.ignore_sources.contains(&".gitignore".to_string()));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn dir_only_exclude_spares_same_named_file() {
        let root = scratch("dironly");
        // A regular file that merely carries the pattern's name must be
        // spared by the directory pattern `build/`.
        touch(&root, "build.agent-besties.md");
        touch(&root, "build/inner.agent-besties.md");
        touch(&root, "keep.agent-besties.md");

        let res = scan(&root, &opts(vec![], vec!["build/"]), &mut sink).expect("scan ok");

        let mut names: Vec<String> = res
            .files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec![
                "build.agent-besties.md".to_string(),
                "keep.agent-besties.md".to_string(),
            ]
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn leading_slash_include_is_root_anchored() {
        let root = scratch("rootanchor");
        touch(&root, "top.agent-besties.md");
        touch(&root, "sub/top.agent-besties.md");

        let res = scan(
            &root,
            &opts(vec!["/top.agent-besties.md"], vec![]),
            &mut sink,
        )
        .expect("scan ok");

        assert_eq!(res.files.len(), 1);
        let dirname = root.file_name().unwrap().to_string_lossy().into_owned();
        assert!(display_path(&res.files[0]).ends_with(&format!("{}/top.agent-besties.md", dirname)));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn display_path_emits_dotdot_forms() {
        let cwd = std::env::current_dir().expect("cwd available");
        let parent = cwd.parent().expect("cwd has a parent");
        assert_eq!(display_path(parent), "..");
        let grandparent = parent.parent().expect("parent has a parent");
        assert_eq!(display_path(grandparent), "../..");

        let sibling = parent.join("coexist-display-sibling");
        assert_eq!(display_path(&sibling), "../coexist-display-sibling");
        let aunt = grandparent.join("coexist-display-aunt");
        assert_eq!(display_path(&aunt), "../../coexist-display-aunt");
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn include_exclude_keep_native_path_identity() {
        #[cfg(unix)]
        let native_name = {
            use std::os::unix::ffi::OsStringExt;
            OsString::from_vec(b"r\xFF".to_vec())
        };
        #[cfg(windows)]
        let native_name = {
            use std::os::windows::ffi::OsStringExt;
            OsString::from_wide(&[0x0072, 0xD800])
        };

        let root = scratch("native-glob");
        let raw_dir = root.join(native_name);
        let fffd_dir = root.join("r\u{FFFD}");
        if fs::create_dir_all(&raw_dir).is_err() {
            // Some filesystems refuse non-Unicode names; without two
            // distinct native identities the conflation cannot occur.
            let _ = fs::remove_dir_all(&root);
            return;
        }
        fs::create_dir_all(&fffd_dir).expect("create U+FFFD dir");
        fs::write(raw_dir.join("note.agent-besties.md"), b"# note.post\n")
            .expect("write non-Unicode note");
        fs::write(fffd_dir.join("note.agent-besties.md"), b"# note.post\n")
            .expect("write U+FFFD note");

        let excluded =
            scan(&root, &opts(vec![], vec!["r\u{FFFD}/"]), &mut sink).expect("exclude scan ok");
        assert_eq!(excluded.files, vec![raw_dir.join("note.agent-besties.md")]);

        let included =
            scan(&root, &opts(vec!["r\u{FFFD}/*"], vec![]), &mut sink).expect("include scan ok");
        assert_eq!(included.files, vec![fffd_dir.join("note.agent-besties.md")]);

        // Built-in sources use Gitignore's matcher, not the explicit-filter
        // adapter above. It must preserve the same two native identities.
        let mut builder = ignore::gitignore::GitignoreBuilder::new(&root);
        builder
            .add_line(None, "r\u{FFFD}/")
            .expect("valid ignore line");
        let matcher = builder.build().expect("compile ignore rules");
        assert!(matcher.matched(&raw_dir, true).is_none());
        assert!(matcher.matched(&fffd_dir, true).is_ignore());
        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn managed_directory_symlink_is_not_a_leaf() {
        let root = scratch("dirlink");
        fs::create_dir_all(root.join("real")).expect("create real dir");
        std::os::unix::fs::symlink("real", root.join("link.agent-besties.md"))
            .expect("create managed-suffix directory symlink");

        let res = scan(&root, &opts(vec![], vec![]), &mut sink).expect("scan ok");
        assert!(
            res.files.is_empty(),
            "directory symlink must not become a leaf: {:?}",
            res.files
        );
        assert!(
            res.unreadable_dirs.is_empty(),
            "directory symlink must not be reported unreadable: {:?}",
            res.unreadable_dirs
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// Direct linked root (Unix directory symlink / Windows junction) with
    /// the managed suffix: scan the link itself. The root is a directory
    /// even when the walker yields symlink `file_type`, so only the real
    /// managed descendant is collected — never a phantom root leaf.
    #[cfg(any(unix, windows))]
    #[test]
    fn explicit_linked_root_is_directory_not_managed_file() {
        let base = scratch("linked-root");
        fs::create_dir_all(base.join("real")).expect("create real dir");
        fs::write(
            base.join("real").join("note.agent-besties.md"),
            b"# note.post\n",
        )
        .expect("write note");

        let link = base.join("link.agent-besties.md");
        #[cfg(unix)]
        std::os::unix::fs::symlink("real", &link).expect("create managed-suffix directory symlink");
        #[cfg(windows)]
        {
            let target = base
                .join("real")
                .to_str()
                .expect("utf-8 temp path")
                .replace('/', "\\");
            let mklink = std::process::Command::new("cmd")
                .args([
                    "/C",
                    "mklink",
                    "/J",
                    "link.agent-besties.md",
                    target.as_str(),
                ])
                .current_dir(&base)
                .output()
                .expect("spawn cmd /C mklink");
            assert!(
                mklink.status.success(),
                "mklink /J failed: {}",
                String::from_utf8_lossy(&mklink.stderr)
            );
        }

        let mut trailing = link.clone().into_os_string();
        trailing.push(std::path::MAIN_SEPARATOR_STR);
        let roots = [link.clone(), PathBuf::from(trailing), link.join(".")];
        for root in &roots {
            let mut calls: Vec<(usize, usize)> = Vec::new();
            let mut progress = |dirs: usize, files: usize| calls.push((dirs, files));
            let res = scan(root, &opts(vec![], vec![]), &mut progress).expect("scan ok");
            assert_eq!(
                res.files,
                vec![root.join("note.agent-besties.md")],
                "linked root {:?} must yield only the real descendant",
                root
            );
            assert_eq!(
                res.dirs, 1,
                "linked root {:?} counted once as a directory",
                root
            );
            assert!(
                res.unreadable_dirs.is_empty(),
                "linked root {:?} must not be unreadable: {:?}",
                root,
                res.unreadable_dirs
            );
            assert!(
                calls.iter().any(|&(dirs, found)| dirs == 1 && found == 0),
                "root directory progress missing for {:?}: {:?}",
                root,
                calls
            );
            assert_eq!(
                *calls.last().expect("progress called"),
                (1, 1),
                "final progress for {:?}",
                root
            );
        }
        let _ = fs::remove_dir_all(&base);
    }

    /// Spawn this test binary under an isolated git/home environment. The
    /// child sees `root_key` and runs the scan assertions; the parent never
    /// mutates process-global environment.
    fn run_env_isolated(test: &str, root_key: &str, root: &Path, vars: &[(&str, &Path)]) -> bool {
        let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
        cmd.args(["--exact", test, "--nocapture"]);
        cmd.env_remove("GIT_CONFIG_GLOBAL");
        cmd.env(root_key, root);
        cmd.current_dir(root.parent().unwrap_or(root));
        for (key, value) in vars {
            cmd.env(key, value);
        }
        let mut child = cmd.spawn().expect("spawn env-isolated child");
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("observe isolated child") {
                return status.success();
            }
            if start.elapsed() >= std::time::Duration::from_secs(15) {
                child.kill().expect("stop blocked isolated child");
                child.wait().expect("reap isolated child");
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn source_errors_reach_all_walk_consumers() {
        const ROOT_KEY: &str = "COEXIST_WALK_SOURCE_ERRORS";
        if let Some(root) = std::env::var_os(ROOT_KEY) {
            let root = PathBuf::from(root);
            assert!(scan(&root, &Options::default(), &mut sink).is_err());
            let mut builder = WalkBuilder::new(&root);
            builder.require_git(false);
            assert!(builder.build().next().expect("global error item").is_err());
            let mut matcher = builder.build_matchers().pop().expect("root matcher");
            let (_, error) = matcher.matched_with_errors("note.agent-besties.md", false);
            assert!(error.is_some());
            let failed = std::sync::atomic::AtomicBool::new(false);
            builder.build_parallel().run(|| {
                Box::new(|entry| {
                    if entry.is_err() {
                        failed.store(true, std::sync::atomic::Ordering::Relaxed);
                        ignore::WalkState::Quit
                    } else {
                        ignore::WalkState::Continue
                    }
                })
            });
            assert!(failed.load(std::sync::atomic::Ordering::Relaxed));

            // A successfully read empty global file remains a source. A
            // malformed local file is equally fatal, not a partial success.
            fs::write("global-ignore", b"").unwrap();
            fs::write(root.join(".ignore"), "{broken\n").unwrap();
            assert!(scan(&root, &Options::default(), &mut sink).is_err());
            fs::write(root.join(".ignore"), b"").unwrap();
            let mut removed = false;
            let result = scan(&root, &Options::default(), &mut |dirs, _| {
                if dirs == 1 && !removed {
                    fs::remove_file(root.join(".ignore")).unwrap();
                    removed = true;
                }
            })
            .expect("source was read before its removal");
            assert_eq!(result.files, vec![root.join("note.agent-besties.md")]);
            assert!(result.ignore_sources.iter().any(|s| s == ".ignore"));
            assert!(result.ignore_sources.iter().any(|s| s == "global"));
            return;
        }
        let base = scratch("source-errors");
        let root = base.join("room");
        touch(&root, "note.agent-besties.md");
        fs::write(
            base.join("config"),
            "[core]\nexcludesFile = global-ignore\n",
        )
        .unwrap();
        fs::write(base.join("global-ignore"), "{broken\n").unwrap();
        let ok = run_env_isolated(
            "fsx::walk::tests::source_errors_reach_all_walk_consumers",
            ROOT_KEY,
            &root,
            &[("GIT_CONFIG_GLOBAL", &base.join("config"))],
        );
        let _ = fs::remove_dir_all(&base);
        assert!(ok, "isolated source errors/provenance child failed");
    }

    #[cfg(unix)]
    #[test]
    fn nonregular_ignore_source_fails_without_waiting_for_a_writer() {
        use std::os::unix::ffi::OsStrExt;
        const ROOT_KEY: &str = "COEXIST_WALK_FIFO_SOURCE";
        if let Some(root) = std::env::var_os(ROOT_KEY) {
            let root = PathBuf::from(root);
            assert!(scan(&root, &Options::default(), &mut sink).is_err());
            let result = scan(&root, &opts(vec![], vec![]), &mut sink).unwrap();
            assert_eq!(result.files, vec![root.join("note.agent-besties.md")]);
            return;
        }
        let base = scratch("fifo-source");
        let root = base.join("room");
        touch(&root, "note.agent-besties.md");
        let fifo = std::ffi::CString::new(root.join(".ignore").as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        let ok = run_env_isolated(
            "fsx::walk::tests::nonregular_ignore_source_fails_without_waiting_for_a_writer",
            ROOT_KEY,
            &root,
            &[
                ("HOME", &base.join("home")),
                ("XDG_CONFIG_HOME", &base.join("xdg")),
                ("GIT_CONFIG_GLOBAL", &base.join("missing-global")),
                ("GIT_CONFIG_SYSTEM", &base.join("missing-system")),
            ],
        );
        let _ = fs::remove_dir_all(&base);
        assert!(
            ok,
            "nonregular source was not rejected within the child deadline"
        );
    }

    #[cfg(unix)]
    #[test]
    fn git_config_global_prunes_and_reports_global() {
        const ROOT_KEY: &str = "COEXIST_WALK_ROOT_GIT_CONFIG_GLOBAL";
        if let Some(root) = std::env::var_os(ROOT_KEY) {
            let root = PathBuf::from(root);
            let walk = Options {
                include: vec![],
                exclude: vec![],
                no_ignore: false,
            };
            let res = scan(&root, &walk, &mut sink).expect("scan ok");
            assert!(
                res.files.is_empty(),
                "global ignore must prune hidden.agent-besties.md, got {:?}",
                res.files
            );
            assert!(
                res.ignore_sources.iter().any(|s| s == "global"),
                "expected global source, got {:?}",
                res.ignore_sources
            );
            return;
        }

        let base = scratch("git-config-global");
        let home = base.join("home");
        let xdg = base.join("xdg");
        let room = base.join("room");
        fs::create_dir_all(&home).expect("create home");
        fs::create_dir_all(&xdg).expect("create xdg");
        fs::create_dir_all(&room).expect("create room");
        let global_ignore = base.join("global-ignore");
        fs::write(&global_ignore, "hidden.agent-besties.md\n").expect("write global ignore");
        fs::write(
            base.join("config"),
            format!("[core]\nexcludesFile = {}\n", global_ignore.display()),
        )
        .expect("write GIT_CONFIG_GLOBAL");
        fs::write(room.join("hidden.agent-besties.md"), b"# hidden.post\n").expect("write hidden");

        let ok = run_env_isolated(
            "fsx::walk::tests::git_config_global_prunes_and_reports_global",
            ROOT_KEY,
            &room,
            &[
                ("HOME", &home),
                ("XDG_CONFIG_HOME", &xdg),
                ("GIT_CONFIG_GLOBAL", &base.join("config")),
                ("GIT_CONFIG_SYSTEM", &base.join("missing-system")),
            ],
        );
        let _ = fs::remove_dir_all(&base);
        assert!(ok, "isolated GIT_CONFIG_GLOBAL child failed");
    }

    #[cfg(unix)]
    #[test]
    fn configured_missing_excludesfile_does_not_claim_global() {
        const ROOT_KEY: &str = "COEXIST_WALK_ROOT_MISSING_EXCLUDES";
        if let Some(root) = std::env::var_os(ROOT_KEY) {
            let root = PathBuf::from(root);
            let walk = Options {
                include: vec![],
                exclude: vec![],
                no_ignore: false,
            };
            let res = scan(&root, &walk, &mut sink).expect("scan ok");
            assert_eq!(
                res.files,
                vec![root.join("hidden.agent-besties.md")],
                "missing configured excludesFile must not load the default global ignore"
            );
            assert!(
                !res.ignore_sources.iter().any(|s| s == "global"),
                "global must be absent, got {:?}",
                res.ignore_sources
            );
            return;
        }

        let base = scratch("missing-excludes");
        let home = base.join("home");
        let xdg = base.join("xdg");
        let room = base.join("room");
        fs::create_dir_all(&home).expect("create home");
        fs::create_dir_all(xdg.join("git")).expect("create xdg git");
        fs::create_dir_all(&room).expect("create room");
        fs::write(
            home.join(".gitconfig"),
            format!(
                "[core]\nexcludesFile = {}\n",
                base.join("missing").display()
            ),
        )
        .expect("write home gitconfig");
        fs::write(xdg.join("git").join("ignore"), "hidden.agent-besties.md\n")
            .expect("write default global ignore");
        fs::write(room.join("hidden.agent-besties.md"), b"# hidden.post\n").expect("write hidden");

        let ok = run_env_isolated(
            "fsx::walk::tests::configured_missing_excludesfile_does_not_claim_global",
            ROOT_KEY,
            &room,
            &[
                ("HOME", &home),
                ("XDG_CONFIG_HOME", &xdg),
                ("GIT_CONFIG_SYSTEM", &base.join("missing-system")),
            ],
        );
        let _ = fs::remove_dir_all(&base);
        assert!(ok, "isolated missing-excludesFile child failed");
    }

    #[cfg(unix)]
    #[test]
    fn empty_global_ignore_still_reports_global() {
        const ROOT_KEY: &str = "COEXIST_WALK_ROOT_EMPTY_GLOBAL";
        if let Some(root) = std::env::var_os(ROOT_KEY) {
            let root = PathBuf::from(root);
            let walk = Options {
                include: vec![],
                exclude: vec![],
                no_ignore: false,
            };
            let res = scan(&root, &walk, &mut sink).expect("scan ok");
            assert_eq!(res.files, vec![root.join("kept.agent-besties.md")]);
            assert!(
                res.ignore_sources.iter().any(|s| s == "global"),
                "empty loaded global ignore must still be a present source, got {:?}",
                res.ignore_sources
            );
            return;
        }

        let base = scratch("empty-global");
        let home = base.join("home");
        let xdg = base.join("xdg");
        let room = base.join("room");
        fs::create_dir_all(&home).expect("create home");
        fs::create_dir_all(&xdg).expect("create xdg");
        fs::create_dir_all(&room).expect("create room");
        let global_ignore = base.join("global-ignore");
        fs::write(&global_ignore, b"").expect("write empty global ignore");
        fs::write(
            base.join("config"),
            format!("[core]\nexcludesFile = {}\n", global_ignore.display()),
        )
        .expect("write GIT_CONFIG_GLOBAL");
        fs::write(room.join("kept.agent-besties.md"), b"# kept.post\n").expect("write kept");

        let ok = run_env_isolated(
            "fsx::walk::tests::empty_global_ignore_still_reports_global",
            ROOT_KEY,
            &room,
            &[
                ("HOME", &home),
                ("XDG_CONFIG_HOME", &xdg),
                ("GIT_CONFIG_GLOBAL", &base.join("config")),
                ("GIT_CONFIG_SYSTEM", &base.join("missing-system")),
            ],
        );
        let _ = fs::remove_dir_all(&base);
        assert!(ok, "isolated empty-global child failed");
    }
}
