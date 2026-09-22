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

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use globset::GlobBuilder;
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
    /// `rel` is the file's path relative to the scan root (`/`-joined),
    /// `name` its file name, `anc_dirs` / `anc_names` the ancestor
    /// directories of `rel` as accumulated relative paths / bare names.
    fn matches(&self, rel: &str, name: &str, anc_dirs: &[String], anc_names: &[String]) -> bool {
        if self.dir_only {
            if self.anchored {
                anc_dirs.iter().any(|d| self.matcher.is_match(d))
            } else {
                anc_names.iter().any(|n| self.matcher.is_match(n))
            }
        } else if self.anchored {
            self.matcher.is_match(rel)
        } else {
            self.matcher.is_match(name)
        }
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
/// entries, which are not reliably `is_symlink()` — and symlinks whose
/// target is a directory; directory symlinks are never entered, so directory
/// loops are impossible. On Unix the walker already refuses to follow
/// symlinks, so file symlinks are kept and treated as files (§10.1.1). A
/// `None` file type is treated as non-directory (no special skip).
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
        if file_type.is_some_and(|ft| ft.is_symlink()) {
            // Directory symlinks are never entered; file symlinks are kept.
            // A broken symlink (metadata fails) is kept and simply never
            // matches anything the callers care about.
            if let Ok(target) = std::fs::metadata(entry.path()) {
                if target.is_dir() {
                    return false;
                }
            }
        }
    }
    true
}

/// `true` when `name` (a whole file name) ends with the managed suffix,
/// compared ASCII-case-insensitively (§10.1.4).
fn has_managed_suffix(name: &str) -> bool {
    let name_bytes = name.as_bytes();
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
        ignore::Error::WithLineNumber { err, .. } | ignore::Error::WithDepth { err, .. } => {
            is_io_cause(err)
        }
        ignore::Error::Partial(errors) => !errors.is_empty() && errors.iter().all(is_io_cause),
        _ => false,
    }
}

/// Flatten an `ignore::Error` recursively: `Partial` unwraps to each inner,
/// `WithLineNumber` / `WithDepth` unwrap to their inner; a `WithPath` whose
/// cause is I/O — and `Loop` — is an unreadable-directory fact, while every
/// other shape is fatal (keeping the path the error carries, if any).
fn classify_error(err: &ignore::Error) -> Vec<WalkErrorFact> {
    match err {
        ignore::Error::Partial(errors) => errors.iter().flat_map(classify_error).collect(),
        ignore::Error::WithLineNumber { err, .. } => classify_error(err),
        ignore::Error::WithDepth { err, .. } => classify_error(err),
        ignore::Error::WithPath { path, err } => {
            if is_io_cause(err) {
                vec![WalkErrorFact::Unreadable(path.clone())]
            } else {
                // E.g. a .gitignore line that failed to parse: the crate
                // read the file but its rules are unusable — never hide
                // that behind an empty-looking result.
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
/// the caller; this function never prints). Hidden entries are traversed,
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

    let mut builder = WalkBuilder::new(root);
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
        rel: String,
        name: String,
    }

    let mut dirs: usize = 0;
    let mut found: usize = 0;
    let mut managed: Vec<Found> = Vec::new();
    let mut unreadable_dirs: Vec<PathBuf> = Vec::new();
    let mut entered_dirs: Vec<PathBuf> = Vec::new();

    // Absorb classified walk errors: unreadable-directory facts accumulate
    // and scanning continues; any fatal fact aborts the scan. The first
    // fatal wins; a pathless fatal is attributed to the scan root.
    let mut absorb_walk_errors = |facts: Vec<WalkErrorFact>| -> Option<Fault> {
        let mut fatal: Option<Fault> = None;
        for fact in facts {
            match fact {
                WalkErrorFact::Unreadable(path) => unreadable_dirs.push(path),
                WalkErrorFact::Fatal { path, message } => {
                    if fatal.is_none() {
                        fatal = Some(Fault::io(
                            "scan",
                            path.unwrap_or_else(|| root.to_path_buf()),
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
    for entry in builder.build() {
        match entry {
            Ok(e) => {
                if let Some(err) = e.error() {
                    if let Some(fault) = absorb_walk_errors(classify_error(err)) {
                        return Err(fault);
                    }
                }
                if e.file_type().is_some_and(|ft| ft.is_dir()) {
                    dirs += 1;
                    entered_dirs.push(e.path().to_path_buf());
                    progress(dirs, found);
                } else {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if has_managed_suffix(&name) {
                        found += 1;
                        let rel = relative_forward_slashed(e.path(), root);
                        managed.push(Found {
                            path: e.path().to_path_buf(),
                            rel,
                            name,
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

    let zero_includes = if !opts.include.is_empty() && files.is_empty() && !opts.no_ignore {
        opts.include.clone()
    } else {
        Vec::new()
    };

    let mut ignore_sources = if opts.no_ignore {
        vec!["--no-ignore".to_string()]
    } else {
        detect_ignore_sources(root, &entered_dirs)
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

/// `entry_path` relative to `root`, components joined with `/` (the form
/// §10.1.3 anchored patterns match against).
fn relative_forward_slashed(entry_path: &Path, root: &Path) -> String {
    let rel = entry_path.strip_prefix(root).unwrap_or(entry_path);
    let mut out = String::new();
    for component in rel.components() {
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str(&component.as_os_str().to_string_lossy());
    }
    out
}

/// Ancestor directories of a `/`-joined relative file path, both as
/// accumulated relative path strings (for anchored directory patterns) and
/// as bare component names (for slash-free directory patterns). The file's
/// own last component is not an ancestor.
fn relative_ancestors(rel: &str) -> (Vec<String>, Vec<String>) {
    let components: Vec<&str> = rel.split('/').filter(|c| !c.is_empty()).collect();
    let mut dirs = Vec::with_capacity(components.len());
    let mut names = Vec::with_capacity(components.len());
    let mut accumulated = String::new();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        if !accumulated.is_empty() {
            accumulated.push('/');
        }
        accumulated.push_str(component);
        dirs.push(accumulated.clone());
        names.push((*component).to_string());
    }
    (dirs, names)
}

/// Determine which ignore source kinds were actually in play (§10.1.2,
/// §8.2's `ignore` key). Candidate directories are the root itself, every
/// directory the walk entered, and — mirroring the ignore crate's parent
/// reading — the *physical* ancestors of the root: the crate canonicalizes
/// the root and walks that chain near→far, and skips parent reading
/// entirely when canonicalization fails. Fixed report order `.gitignore`,
/// `.ignore`, `exclude`, `global`; a source counts only when the crate
/// could actually load it.
fn detect_ignore_sources(root: &Path, entered_dirs: &[PathBuf]) -> Vec<String> {
    let mut has_gitignore = false;
    let mut has_ignore = false;
    let mut has_exclude = false;

    let mut candidates: Vec<PathBuf> = Vec::with_capacity(entered_dirs.len() + 8);
    candidates.push(root.to_path_buf());
    candidates.extend(entered_dirs.iter().cloned());
    if let Ok(physical_root) = root.canonicalize() {
        // Near→far physical ancestors, the root itself excluded: the walk
        // already read the root's own ignore files.
        for ancestor in physical_root.ancestors().skip(1) {
            candidates.push(ancestor.to_path_buf());
        }
    }

    for dir in &candidates {
        if probe_readable(&dir.join(".gitignore")) {
            has_gitignore = true;
        }
        if probe_readable(&dir.join(".ignore")) {
            has_ignore = true;
        }
        let git_dir = dir.join(".git");
        // Only a real .git directory with a loadable info/exclude counts;
        // the `.git` *file* used by worktrees/submodules does not.
        if path_is_dir(&git_dir) && probe_readable(&git_dir.join("info").join("exclude")) {
            has_exclude = true;
        }
    }

    let mut sources = Vec::new();
    if has_gitignore {
        sources.push(".gitignore".to_string());
    }
    if has_ignore {
        sources.push(".ignore".to_string());
    }
    if has_exclude {
        sources.push("exclude".to_string());
    }
    if global_ignore_in_play() {
        sources.push("global".to_string());
    }
    sources
}

/// A source counts only when the ignore crate could actually read it: the
/// file opens and yields to a read attempt. EOF still counts (the file was
/// read, it is just empty); an I/O failure at open or read does not.
fn probe_readable(path: &Path) -> bool {
    use std::io::Read;
    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut byte = [0u8; 1];
            file.read(&mut byte).is_ok()
        }
        Err(_) => false,
    }
}

fn path_is_dir(path: &Path) -> bool {
    std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
}

/// Mirror the ignore crate's global-gitignore discovery (§10.1.2):
/// `core.excludesFile` from `$HOME/.gitconfig`, falling back to the XDG git
/// config, and only then the default `git/ignore` locations. An empty
/// `XDG_CONFIG_HOME` counts as unset, exactly like the crate.
fn global_ignore_in_play() -> bool {
    if let Some(excludes_file) = gitconfig_excludes_file() {
        if probe_readable(&excludes_file) {
            return true;
        }
    }
    if let Some(ignore_file) = default_global_ignore_file() {
        if probe_readable(&ignore_file) {
            return true;
        }
    }
    false
}

fn gitconfig_excludes_file() -> Option<PathBuf> {
    // Same precedence as the ignore crate: $HOME/.gitconfig first, then the
    // XDG git config — a home config that simply lacks the key does not
    // stop the search.
    if let Some(home) = home() {
        if let Ok(bytes) = std::fs::read(home.join(".gitconfig")) {
            if let Some(path) = parse_excludes_file(&bytes) {
                return Some(path);
            }
        }
    }
    let config_dir = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => home()?.join(".config"),
    };
    let bytes = std::fs::read(config_dir.join("git").join("config")).ok()?;
    parse_excludes_file(&bytes)
}

/// Home directory the way the ignore crate resolves it (`env::home_dir`:
/// `USERPROFILE`-aware on Windows), so "global" detection agrees with what
/// the crate actually read.
fn home() -> Option<PathBuf> {
    std::env::home_dir()
}

/// Naive gitconfig scan for `core.excludesFile`, matching the ignore crate:
/// the first `excludesfile = <path>` line wins (key compared
/// case-insensitively, optional surrounding quotes stripped, `~` expanded).
fn parse_excludes_file(data: &[u8]) -> Option<PathBuf> {
    for line in data.split(|&b| b == b'\n') {
        let line = String::from_utf8_lossy(line);
        let mut parts = line.split('=');
        let key = parts.next().unwrap_or("").trim();
        if !key.eq_ignore_ascii_case("excludesfile") {
            continue;
        }
        if let Some(value) = parts.next() {
            let value = value.trim().trim_matches('"').trim();
            if value.is_empty() {
                continue;
            }
            return Some(expand_tilde(value));
        }
    }
    None
}

fn expand_tilde(value: &str) -> PathBuf {
    if let Some(home) = home() {
        if value == "~" {
            return home;
        }
        if let Some(rest) = value.strip_prefix("~/") {
            return home.join(rest);
        }
    }
    PathBuf::from(value)
}

fn default_global_ignore_file() -> Option<PathBuf> {
    match std::env::var_os("XDG_CONFIG_HOME") {
        Some(value) if !value.is_empty() => Some(PathBuf::from(value).join("git").join("ignore")),
        _ => home().map(|home| home.join(".config").join("git").join("ignore")),
    }
}

/// `path` itself first, then every parent, near to far (§10.5.2, §10.1.6).
/// Purely path-mechanical — nothing is stat'ed or canonicalized here; the
/// caller passes an already-normalized path. Exactly `Path::ancestors()`
/// order, so the filesystem root (`/`, drive root, UNC share root) is the
/// last element and nothing is skipped.
pub fn ancestors(path: &Path) -> Vec<PathBuf> {
    path.ancestors().map(Path::to_path_buf).collect()
}

/// Locate the state file inside `dir` (§10.1.6): exact `state.agent-besties.md`
/// on Linux, ASCII-case-insensitive on Windows and macOS. At most one hit is
/// expected per directory; if several case variants coexist, the first in
/// `read_dir` order wins. The returned path keeps the on-disk casing (the
/// `W-NAME-STATE` judgement belongs to `check`, not here).
pub fn state_file(dir: &Path) -> Result<Option<PathBuf>, Fault> {
    let entries = std::fs::read_dir(dir).map_err(|e| Fault::io("scan", dir.to_path_buf(), e))?;
    let case_insensitive = cfg!(any(windows, target_os = "macos"));
    for entry in entries {
        let entry = entry.map_err(|e| Fault::io("scan", dir.to_path_buf(), e))?;
        let name = entry.file_name();
        let hit = if case_insensitive {
            name.as_os_str().eq_ignore_ascii_case(STATE_FILE_NAME)
        } else {
            name.as_os_str().eq(OsStr::new(STATE_FILE_NAME))
        };
        if hit {
            return Ok(Some(dir.join(name)));
        }
    }
    Ok(None)
}

fn components_joined(path: &Path) -> String {
    let mut out = String::new();
    for component in path.components() {
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str(&component.as_os_str().to_string_lossy());
    }
    out
}

fn forward_slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Number of leading volume components (`Prefix` / `RootDir`) of an
/// absolute path: 1 on Unix (`/`), 2 on Windows (drive or UNC prefix plus
/// root).
fn volume_component_count(components: &[Component<'_>]) -> usize {
    components
        .iter()
        .take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
        .count()
}

/// Lexical display form of `path` (§8.1.9): everything the CLI prints is
/// relative to the current working directory with `/` separators, `.` for
/// the current directory itself, `..` allowed. Same-volume paths outside
/// the cwd relativize through the longest common component prefix — one
/// `..` per cwd component below it, then the remaining path components
/// (paths under the cwd are the zero-`..` case). Relativization is
/// impossible only across volumes — a mismatch at the `Prefix` / `RootDir`
/// level (different drive, UNC vs local on Windows) — or when the current
/// directory cannot be read; then the path is shown as given with `\`
/// normalized to `/`. Purely lexical: never canonicalizes, never touches
/// the disk for relative paths, and must never be written into files.
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
    let path_components: Vec<Component<'_>> = path.components().collect();
    let cwd_components: Vec<Component<'_>> = cwd.components().collect();

    // Longest common component prefix of the two absolute paths.
    let mut common = 0;
    while common < path_components.len()
        && common < cwd_components.len()
        && path_components[common] == cwd_components[common]
    {
        common += 1;
    }
    // The common prefix must reach past both volumes' identity components;
    // otherwise the paths sit on different volumes and no relative form
    // exists.
    if common < volume_component_count(&path_components)
        || common < volume_component_count(&cwd_components)
    {
        return forward_slashed(path);
    }

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

/// Strip one trailing managed suffix from the file name, then [`display_path`]
/// the result (the inverse of §2.4.3 for the first output line's object).
/// When the whole file name was the suffix, the directory part is displayed
/// (`.` for a bare name).
pub fn object_path(path: &Path) -> String {
    let mut stripped = path.to_path_buf();
    if let Some(name) = path.file_name() {
        let name_bytes = name.as_encoded_bytes();
        let suffix_bytes = crate::consts::SUFFIX.as_bytes();
        if name_bytes.ends_with(suffix_bytes) {
            let stem = &name_bytes[..name_bytes.len() - suffix_bytes.len()];
            if stem.is_empty() {
                stripped.pop();
            } else {
                // SUFFIX is exactly two dot segments (`.agent-besties.md`);
                // clearing the extension twice peels both natively at the
                // OsStr level, so native name bytes stay opaque.
                stripped.set_extension("");
                stripped.set_extension("");
            }
        }
    }
    display_path(&stripped)
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
    fn display_path_relativizes_and_falls_back() {
        assert_eq!(display_path(Path::new("")), ".");
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

        #[cfg(windows)]
        {
            let other_drive = Path::new(r"Z:\w\f.md");
            assert_eq!(display_path(other_drive), "Z:/w/f.md");
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
}
