/*!
The gitignore module provides a way to match globs from a gitignore file
against file paths.

Note that this module implements the specification as described in the
`gitignore` man page from scratch. That is, this module does *not* shell out to
the `git` command line tool.
*/

use std::{
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use {
    globset::{Candidate, GlobBuilder, GlobSet, GlobSetBuilder},
    regex_automata::util::pool::Pool,
};

use crate::{
    Error, Match, PartialErrorBuilder, open_regular_file,
    pathutil::{is_file_name, strip_prefix},
};

/// Glob represents a single glob in a gitignore file.
///
/// This is used to report information about the highest precedent glob that
/// matched in one or more gitignore files.
#[derive(Clone, Debug)]
pub struct Glob {
    /// The file path that this glob was extracted from.
    from: Option<PathBuf>,
    /// The original glob string.
    original: String,
    /// The actual glob string used to convert to a regex.
    actual: String,
    /// Whether this is a whitelisted glob or not.
    is_whitelist: bool,
    /// Whether this glob should only match directories or not.
    is_only_dir: bool,
}

impl Glob {
    /// Returns the file path that defined this glob.
    pub fn from(&self) -> Option<&Path> {
        self.from.as_ref().map(|p| &**p)
    }

    /// The original glob as it was defined in a gitignore file.
    pub fn original(&self) -> &str {
        &self.original
    }

    /// The actual glob that was compiled to respect gitignore
    /// semantics.
    pub fn actual(&self) -> &str {
        &self.actual
    }

    /// Whether this was a whitelisted glob or not.
    pub fn is_whitelist(&self) -> bool {
        self.is_whitelist
    }

    /// Whether this glob must match a directory or not.
    pub fn is_only_dir(&self) -> bool {
        self.is_only_dir
    }

    /// Returns true if and only if this glob has a `**/` prefix.
    fn has_doublestar_prefix(&self) -> bool {
        self.actual.starts_with("**/") || self.actual == "**"
    }
}

/// Gitignore is a matcher for the globs in one or more gitignore files
/// in the same directory.
#[derive(Clone, Debug)]
pub struct Gitignore {
    set: GlobSet,
    root: PathBuf,
    globs: Vec<Glob>,
    num_ignores: u64,
    num_whitelists: u64,
    matches: Option<Arc<Pool<Vec<usize>>>>,
    loaded: bool,
}

impl Gitignore {
    /// Creates a new gitignore matcher from the gitignore file path given.
    ///
    /// If it's desirable to include multiple gitignore files in a single
    /// matcher, or read gitignore globs from a different source, then
    /// use `GitignoreBuilder`.
    ///
    /// This always returns a valid matcher, even if it's empty. In particular,
    /// a Gitignore file can be partially valid, e.g., when one glob is invalid
    /// but the rest aren't.
    ///
    /// Missing ignore files are ignored. Other I/O errors are returned.
    /// For more granular control over errors, use `GitignoreBuilder`.
    pub fn new<P: AsRef<Path>>(
        gitignore_path: P,
    ) -> (Gitignore, Option<Error>) {
        let path = gitignore_path.as_ref();
        let parent = path.parent().unwrap_or(Path::new("/"));
        let mut builder = GitignoreBuilder::new(parent);
        let mut errs = PartialErrorBuilder::default();
        errs.maybe_push_ignore_io(builder.add(path));
        match builder.build() {
            Ok(gi) => (gi, errs.into_error_option()),
            Err(err) => {
                errs.push(err);
                (Gitignore::empty(), errs.into_error_option())
            }
        }
    }

    /// Creates a new gitignore matcher from the global ignore file, if one
    /// exists.
    ///
    /// The global config file path is specified by git's `core.excludesFile`
    /// config option.
    ///
    /// # Behavior
    ///
    /// This routine does its best to discover any global git exclude files.
    /// This will try to parse out the `excludesFile` config option in your
    /// global git configuration, if necessary.
    ///
    /// The specific things this routine tries (which are subject to change
    /// based on how git behaves) are:
    ///
    ///
    ///
    /// Git's config file location is `$HOME/.gitconfig`. If `$HOME/.gitconfig`
    /// does not exist or does not specify `core.excludesFile`, then
    /// `$XDG_CONFIG_HOME/git/ignore` is read. If `$XDG_CONFIG_HOME` is not
    /// set or is empty, then `$HOME/.config/git/ignore` is used instead.
    pub fn global() -> (Gitignore, Option<Error>) {
        match std::env::current_dir() {
            Ok(cwd) => GitignoreBuilder::new(cwd).build_global(),
            Err(err) => (Gitignore::empty(), Some(err.into())),
        }
    }

    /// Creates a new empty gitignore matcher that never matches anything.
    ///
    /// Its path is empty.
    pub fn empty() -> Gitignore {
        Gitignore {
            set: GlobSet::empty(),
            root: PathBuf::from(""),
            globs: vec![],
            num_ignores: 0,
            num_whitelists: 0,
            matches: None,
            loaded: false,
        }
    }

    /// Returns the directory containing this gitignore matcher.
    ///
    /// All matches are done relative to this path.
    pub fn path(&self) -> &Path {
        &*self.root
    }

    /// Returns true if and only if this gitignore has zero globs, and
    /// therefore never matches any file path.
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Returns true if an ignore file was successfully read to EOF.
    ///
    /// Empty files count as loaded. Line-only construction does not.
    pub(crate) fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Returns the total number of globs, which should be equivalent to
    /// `num_ignores + num_whitelists`.
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// Returns the total number of ignore globs.
    pub fn num_ignores(&self) -> u64 {
        self.num_ignores
    }

    /// Returns the total number of whitelisted globs.
    pub fn num_whitelists(&self) -> u64 {
        self.num_whitelists
    }

    /// Returns whether the given path (file or directory) matched a pattern in
    /// this gitignore matcher.
    ///
    /// `is_dir` should be true if the path refers to a directory and false
    /// otherwise.
    ///
    /// The given path is matched relative to the path given when building
    /// the matcher. Specifically, before matching `path`, its prefix (as
    /// determined by a common suffix of the directory containing this
    /// gitignore) is stripped. If there is no common suffix/prefix overlap,
    /// then `path` is assumed to be relative to this matcher.
    pub fn matched<P: AsRef<Path>>(
        &self,
        path: P,
        is_dir: bool,
    ) -> Match<&Glob> {
        if self.is_empty() {
            return Match::None;
        }
        self.matched_stripped(self.strip(path.as_ref()), is_dir)
    }

    /// Returns whether the given path (file or directory, and expected to be
    /// under the root) or any of its parent directories (up to the root)
    /// matched a pattern in this gitignore matcher.
    ///
    /// NOTE: This method is more expensive than walking the directory hierarchy
    /// top-to-bottom and matching the entries. But, is easier to use in cases
    /// when a list of paths are available without a hierarchy.
    ///
    /// `is_dir` should be true if the path refers to a directory and false
    /// otherwise.
    ///
    /// The given path is matched relative to the path given when building
    /// the matcher. Specifically, before matching `path`, its prefix (as
    /// determined by a common suffix of the directory containing this
    /// gitignore) is stripped. If there is no common suffix/prefix overlap,
    /// then `path` is assumed to be relative to this matcher.
    ///
    /// # Panics
    ///
    /// This method panics if the given file path is not under the root path
    /// of this matcher.
    pub fn matched_path_or_any_parents<P: AsRef<Path>>(
        &self,
        path: P,
        is_dir: bool,
    ) -> Match<&Glob> {
        if self.is_empty() {
            return Match::None;
        }
        let mut path = self.strip(path.as_ref());
        assert!(!path.has_root(), "path is expected to be under the root");

        match self.matched_stripped(path, is_dir) {
            Match::None => (), // walk up
            a_match => return a_match,
        }
        while let Some(parent) = path.parent() {
            match self.matched_stripped(parent, /* is_dir */ true) {
                Match::None => path = parent, // walk up
                a_match => return a_match,
            }
        }
        Match::None
    }

    /// Like matched, but takes a path that has already been stripped.
    fn matched_stripped<P: AsRef<Path>>(
        &self,
        path: P,
        is_dir: bool,
    ) -> Match<&Glob> {
        if self.is_empty() {
            return Match::None;
        }
        let path = path.as_ref();
        let mut matches = self.matches.as_ref().unwrap().get();
        let candidate = Candidate::from_bytes(path.as_os_str().as_encoded_bytes());
        self.set.matches_candidate_into(&candidate, &mut *matches);
        for &i in matches.iter().rev() {
            let glob = &self.globs[i];
            if !glob.is_only_dir() || is_dir {
                return if glob.is_whitelist() {
                    Match::Whitelist(glob)
                } else {
                    Match::Ignore(glob)
                };
            }
        }
        Match::None
    }

    /// Strips the given path such that it's suitable for matching with this
    /// gitignore matcher.
    fn strip<'a, P: 'a + AsRef<Path> + ?Sized>(
        &'a self,
        path: &'a P,
    ) -> &'a Path {
        let mut path = path.as_ref();
        // A leading ./ is completely superfluous. We also strip it from
        // our gitignore root path, so we need to strip it from our candidate
        // path too.
        if let Some(p) = strip_prefix("./", path) {
            path = p;
        }
        // Relativize against the matcher root, keeping the raw remainder
        // spelling. Unix byte-prefix stripping is required for a leftover
        // `/./` body, but it must stop at a native component boundary so
        // `/T/a` is not treated as a parent of `/T/ab/...`. A bare file
        // name must not be rewritten, and root `.` is left alone so
        // candidates that begin with `.` are not stripped.
        if self.root != Path::new(".") && !is_file_name(path) {
            if let Some(p) = strip_prefix_at_component_boundary(&self.root, path)
            {
                path = p;
                // If we're left with a leading slash, get rid of it.
                if let Some(p) = strip_prefix("/", path) {
                    path = p;
                }
            }
        }
        path
    }
}

/// Builds a matcher for a single set of globs from a .gitignore file.
#[derive(Clone, Debug)]
pub struct GitignoreBuilder {
    builder: GlobSetBuilder,
    root: PathBuf,
    globs: Vec<Glob>,
    case_insensitive: bool,
    allow_unclosed_class: bool,
    loaded: bool,
}

impl GitignoreBuilder {
    /// Create a new builder for a gitignore file.
    ///
    /// The path given should be the path at which the globs for this gitignore
    /// file should be matched. Note that paths are always matched relative
    /// to the root path given here. Generally, the root path should correspond
    /// to the *directory* containing a `.gitignore` file.
    pub fn new<P: AsRef<Path>>(root: P) -> GitignoreBuilder {
        let root = root.as_ref();
        GitignoreBuilder {
            builder: GlobSetBuilder::new(),
            root: strip_prefix("./", root).unwrap_or(root).to_path_buf(),
            globs: vec![],
            case_insensitive: false,
            allow_unclosed_class: true,
            loaded: false,
        }
    }

    /// Builds a new matcher from the globs added so far.
    ///
    /// Once a matcher is built, no new globs can be added to it.
    pub fn build(&self) -> Result<Gitignore, Error> {
        let nignore = self.globs.iter().filter(|g| !g.is_whitelist()).count();
        let nwhite = self.globs.iter().filter(|g| g.is_whitelist()).count();
        let set = self
            .builder
            .build()
            .map_err(|err| Error::Glob { glob: None, err: err.to_string() })?;
        Ok(Gitignore {
            set,
            root: self.root.clone(),
            globs: self.globs.clone(),
            num_ignores: nignore as u64,
            num_whitelists: nwhite as u64,
            matches: Some(Arc::new(
                Pool::with_available_parallelism_capacity(|| vec![]),
            )),
            loaded: self.loaded,
        })
    }

    /// Build a global gitignore matcher using the configuration in this
    /// builder.
    ///
    /// This consumes ownership of the builder unlike `build` because it
    /// must mutate the builder to add the global gitignore globs.
    ///
    /// Note that this ignores the path given to this builder's constructor
    /// and instead derives the path automatically from git's global
    /// configuration.
    ///
    /// Missing optional sources are skipped. Other discovery or load errors
    /// are returned. A configured excludes path is not replaced by the
    /// default path.
    pub fn build_global(mut self) -> (Gitignore, Option<Error>) {
        match gitconfig_excludes_path() {
            Err(err) => (Gitignore::empty(), Some(err)),
            Ok(None) => (Gitignore::empty(), None),
            Ok(Some(path)) => {
                let mut errs = PartialErrorBuilder::default();
                errs.maybe_push_ignore_io(self.add(&path));
                match self.build() {
                    Ok(gi) => (gi, errs.into_error_option()),
                    Err(err) => {
                        errs.push(err);
                        (Gitignore::empty(), errs.into_error_option())
                    }
                }
            }
        }
    }

    /// Add each glob from the file path given.
    ///
    /// The file given should be formatted as a `gitignore` file.
    ///
    /// Note that partial errors can be returned. For example, if there was
    /// a problem adding one glob, an error for that will be returned, but
    /// all other valid globs will still be added.
    pub fn add<P: AsRef<Path>>(&mut self, path: P) -> Option<Error> {
        let path = path.as_ref();
        let file = match open_regular_file(path) {
            Err(err) => return Some(Error::Io(err).with_path(path)),
            Ok(file) => file,
        };
        log::debug!("opened gitignore file: {}", path.display());
        let rdr = BufReader::new(file);
        let mut errs = PartialErrorBuilder::default();
        let mut complete = true;
        for (i, line) in rdr.lines().enumerate() {
            let lineno = (i + 1) as u64;
            let line = match line {
                Ok(line) => line,
                Err(err) => {
                    complete = false;
                    errs.push(Error::Io(err).tagged(path, lineno));
                    break;
                }
            };

            // Match Git's handling of .gitignore files that begin with the Unicode BOM
            const UTF8_BOM: &str = "\u{feff}";
            let line =
                if i == 0 { line.trim_start_matches(UTF8_BOM) } else { &line };

            if let Err(err) = self.add_line(Some(path.to_path_buf()), &line) {
                errs.push(err.tagged(path, lineno));
            }
        }
        if complete {
            self.loaded = true;
        }
        errs.into_error_option()
    }

    /// Add each glob line from the string given.
    ///
    /// If this string came from a particular `gitignore` file, then its path
    /// should be provided here.
    ///
    /// The string given should be formatted as a `gitignore` file.
    #[cfg(test)]
    fn add_str(
        &mut self,
        from: Option<PathBuf>,
        gitignore: &str,
    ) -> Result<&mut GitignoreBuilder, Error> {
        for line in gitignore.lines() {
            self.add_line(from.clone(), line)?;
        }
        Ok(self)
    }

    /// Add a line from a gitignore file to this builder.
    ///
    /// If this line came from a particular `gitignore` file, then its path
    /// should be provided here.
    ///
    /// If the line could not be parsed as a glob, then an error is returned.
    pub fn add_line(
        &mut self,
        from: Option<PathBuf>,
        mut line: &str,
    ) -> Result<&mut GitignoreBuilder, Error> {
        #![allow(deprecated)]

        if line.starts_with("#") {
            return Ok(self);
        }
        if !line.ends_with("\\ ") {
            line = line.trim_right();
        }
        if line.is_empty() {
            return Ok(self);
        }
        let mut glob = Glob {
            from,
            original: line.to_string(),
            actual: String::new(),
            is_whitelist: false,
            is_only_dir: false,
        };
        let mut is_absolute = false;
        if line.starts_with("\\!") || line.starts_with("\\#") {
            line = &line[1..];
            is_absolute = line.chars().nth(0) == Some('/');
        } else {
            if line.starts_with("!") {
                glob.is_whitelist = true;
                line = &line[1..];
            }
            if line.starts_with("/") {
                // `man gitignore` says that if a glob starts with a slash,
                // then the glob can only match the beginning of a path
                // (relative to the location of gitignore). We achieve this by
                // simply banning wildcards from matching /.
                line = &line[1..];
                is_absolute = true;
            }
        }
        // If it ends with a slash, then this should only match directories,
        // but the slash should otherwise not be used while globbing.
        if line.as_bytes().last() == Some(&b'/') {
            glob.is_only_dir = true;
            line = &line[..line.len() - 1];
            // If the slash was escaped, then remove the escape.
            // See: https://github.com/BurntSushi/ripgrep/issues/2236
            if line.as_bytes().last() == Some(&b'\\') {
                line = &line[..line.len() - 1];
            }
        }
        glob.actual = line.to_string();
        // If there is a literal slash, then this is a glob that must match the
        // entire path name. Otherwise, we should let it match anywhere, so use
        // a **/ prefix.
        if !is_absolute && !line.chars().any(|c| c == '/') {
            // ... but only if we don't already have a **/ prefix.
            if !glob.has_doublestar_prefix() {
                glob.actual = format!("**/{}", glob.actual);
            }
        }
        // If the glob ends with `/**`, then we should only match everything
        // inside a directory, but not the directory itself. Standard globs
        // will match the directory. So we add `/*` to force the issue.
        if glob.actual.ends_with("/**") {
            glob.actual = format!("{}/*", glob.actual);
        }
        let parsed = GlobBuilder::new(&glob.actual)
            .literal_separator(true)
            .case_insensitive(self.case_insensitive)
            .backslash_escape(true)
            .allow_unclosed_class(self.allow_unclosed_class)
            .build()
            .map_err(|err| Error::Glob {
                glob: Some(glob.original.clone()),
                err: err.kind().to_string(),
            })?;
        self.builder.add(parsed);
        self.globs.push(glob);
        Ok(self)
    }

    /// Toggle whether the globs should be matched case insensitively or not.
    ///
    /// When this option is changed, only globs added after the change will be
    /// affected.
    ///
    /// This is disabled by default.
    pub fn case_insensitive(
        &mut self,
        yes: bool,
    ) -> Result<&mut GitignoreBuilder, Error> {
        // TODO: This should not return a `Result`. Fix this in the next semver
        // release.
        self.case_insensitive = yes;
        Ok(self)
    }

    /// Toggle whether unclosed character classes are allowed. When allowed,
    /// a `[` without a matching `]` is treated literally instead of resulting
    /// in a parse error.
    ///
    /// For example, if this is set then the glob `[abc` will be treated as the
    /// literal string `[abc` instead of returning an error.
    ///
    /// By default, this is true in order to match established `gitignore`
    /// semantics. Generally speaking, enabling this leads to worse failure
    /// modes since the glob parser becomes more permissive. You might want to
    /// enable this when compatibility (e.g., with POSIX glob implementations)
    /// is more important than good error messages.
    pub fn allow_unclosed_class(
        &mut self,
        yes: bool,
    ) -> &mut GitignoreBuilder {
        self.allow_unclosed_class = yes;
        self
    }
}

/// Return the file path of the current environment's global gitignore file.
///
/// Note that the file path returned may not exist.
///
/// Missing optional config files are skipped. Any other error encountered
/// while opening or reading a selected config source is returned. When a
/// config file selects `core.excludesFile`, that path is returned even if
/// the ignore file itself is missing; the default path is not used as a
/// fallback.
pub fn gitconfig_excludes_path() -> Result<Option<PathBuf>, Error> {
    // When GIT_CONFIG_GLOBAL is set, it replaces both $HOME/.gitconfig and
    // $XDG_CONFIG_HOME/git/config (per git 2.32+). A selected override with
    // no core.excludesFile does not fall through to those replaced files.
    // Otherwise, git supports $HOME/.gitconfig and
    // $XDG_CONFIG_HOME/git/config simultaneously, where $HOME/.gitconfig
    // takes precedence. Stop at the first successfully parsed excludesFile
    // setting. The default ignore file is used only when no selected config
    // provides a setting.
    if gitconfig_global_env_path().is_some() {
        if let Some(path) = gitconfig_global_env_contents()?
            .and_then(|x| parse_excludes_file(&x))
        {
            return Ok(Some(path));
        }
    } else {
        if let Some(path) =
            gitconfig_home_contents()?.and_then(|x| parse_excludes_file(&x))
        {
            return Ok(Some(path));
        }
        if let Some(path) =
            gitconfig_xdg_contents()?.and_then(|x| parse_excludes_file(&x))
        {
            return Ok(Some(path));
        }
    }
    // System-level config has the lowest priority for core.excludesFile.
    // GIT_CONFIG_SYSTEM overrides the default /etc/gitconfig path.
    if let Some(path) =
        gitconfig_system_contents()?.and_then(|x| parse_excludes_file(&x))
    {
        return Ok(Some(path));
    }
    Ok(excludes_file_default())
}

/// Reads a git config file if present.
///
/// `NotFound` and `NotADirectory` are treated as absent. Every other open
/// or read error is returned.
fn read_optional_gitconfig(path: &Path) -> Result<Option<Vec<u8>>, Error> {
    let file = match open_regular_file(path) {
        Err(err)
            if err.kind() == std::io::ErrorKind::NotFound
                || err.kind() == std::io::ErrorKind::NotADirectory =>
        {
            return Ok(None);
        }
        Err(err) => return Err(Error::Io(err).with_path(path)),
        Ok(file) => file,
    };
    let mut contents = vec![];
    match BufReader::new(file).read_to_end(&mut contents) {
        Ok(_) => Ok(Some(contents)),
        Err(err) => Err(Error::Io(err).with_path(path)),
    }
}

/// Returns the path specified by `GIT_CONFIG_GLOBAL` when the variable is
/// set to a non-empty value.
fn gitconfig_global_env_path() -> Option<PathBuf> {
    match std::env::var_os("GIT_CONFIG_GLOBAL").map(PathBuf::from) {
        Some(path) if !path.as_os_str().is_empty() => Some(path),
        _ => None,
    }
}

/// Returns the file contents of git's global config file from the path
/// specified by the `GIT_CONFIG_GLOBAL` environment variable.
fn gitconfig_global_env_contents() -> Result<Option<Vec<u8>>, Error> {
    let Some(path) = gitconfig_global_env_path() else {
        return Ok(None);
    };
    read_optional_gitconfig(&path)
}

/// Returns the file contents of git's system-level config file.
///
/// Checks `GIT_CONFIG_SYSTEM` first, then falls back to `/etc/gitconfig`.
fn gitconfig_system_contents() -> Result<Option<Vec<u8>>, Error> {
    let path = std::env::var_os("GIT_CONFIG_SYSTEM")
        .map(PathBuf::from)
        .filter(|x| !x.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from("/etc/gitconfig"));
    read_optional_gitconfig(&path)
}

/// Returns the file contents of git's global config file, if one exists, in
/// the user's home directory.
fn gitconfig_home_contents() -> Result<Option<Vec<u8>>, Error> {
    let Some(home) = home_dir() else {
        return Ok(None);
    };
    read_optional_gitconfig(&home.join(".gitconfig"))
}

/// Returns the file contents of git's global config file, if one exists, in
/// the user's XDG_CONFIG_HOME directory.
fn gitconfig_xdg_contents() -> Result<Option<Vec<u8>>, Error> {
    let Some(path) = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|x| !x.as_os_str().is_empty())
        .or_else(|| home_dir().map(|p| p.join(".config")))
        .map(|x| x.join("git/config"))
    else {
        return Ok(None);
    };
    read_optional_gitconfig(&path)
}

/// Returns the default file path for a global .gitignore file.
///
/// Specifically, this respects XDG_CONFIG_HOME.
fn excludes_file_default() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|x| !x.as_os_str().is_empty())
        .or_else(|| home_dir().map(|p| p.join(".config")))
        .map(|x| x.join("git/ignore"))
}

/// Extract git's `core.excludesfile` config setting from the raw file contents
/// given.
fn parse_excludes_file(data: &[u8]) -> Option<PathBuf> {
    use std::sync::OnceLock;

    use regex_automata::{meta::Regex, util::syntax};

    // N.B. This is the lazy approach, and isn't technically correct, but
    // probably works in more circumstances. I guess we would ideally have
    // a full INI parser. Yuck.
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::builder()
            .configure(Regex::config().utf8_empty(false))
            .syntax(syntax::Config::new().utf8(false))
            .build(r#"(?im-u)^\s*excludesfile\s*=\s*"?\s*(\S+?)\s*"?\s*$"#)
            .unwrap()
    });
    // We don't care about amortizing allocs here I think. This should only
    // be called ~once per traversal or so? (Although it's not guaranteed...)
    let mut caps = re.create_captures();
    re.captures(data, &mut caps);
    let span = caps.get_group(1)?;
    let candidate = &data[span];
    std::str::from_utf8(candidate).ok().map(expand_tilde)
}

/// Expands a leading `~` path component to the native home directory.
///
/// Later tildes and `~user` paths are left unchanged.
fn expand_tilde(path: &str) -> PathBuf {
    let Some(rest) = path.strip_prefix('~') else {
        return PathBuf::from(path);
    };
    if !rest.is_empty() && !rest.starts_with(std::path::is_separator) {
        return PathBuf::from(path);
    }
    let Some(home) = home_dir() else {
        return PathBuf::from(path);
    };
    let mut expanded = home.into_os_string();
    expanded.push(rest);
    PathBuf::from(expanded)
}

fn strip_prefix_at_component_boundary<'a>(
    root: &'a Path,
    path: &'a Path,
) -> Option<&'a Path> {
    let stripped = strip_prefix(root, path)?;
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        if !stripped.as_os_str().is_empty() {
            let root_bytes = root.as_os_str().as_bytes();
            if root_bytes.last() != Some(&b'/')
                && stripped.as_os_str().as_bytes().first() != Some(&b'/')
            {
                return None;
            }
        }
    }
    Some(stripped)
}

/// Returns the location of the user's home directory.
fn home_dir() -> Option<PathBuf> {
    // We're fine with using std::env::home_dir for now. Its bugs are, IMO,
    // pretty minor corner cases.
    #![allow(deprecated)]
    std::env::home_dir()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Gitignore, GitignoreBuilder};

    fn gi_from_str<P: AsRef<Path>>(root: P, s: &str) -> Gitignore {
        let mut builder = GitignoreBuilder::new(root);
        builder.add_str(None, s).unwrap();
        builder.build().unwrap()
    }

    macro_rules! ignored {
        ($name:ident, $root:expr, $gi:expr, $path:expr) => {
            ignored!($name, $root, $gi, $path, false);
        };
        ($name:ident, $root:expr, $gi:expr, $path:expr, $is_dir:expr) => {
            #[test]
            fn $name() {
                let gi = gi_from_str($root, $gi);
                assert!(gi.matched($path, $is_dir).is_ignore());
            }
        };
    }

    macro_rules! not_ignored {
        ($name:ident, $root:expr, $gi:expr, $path:expr) => {
            not_ignored!($name, $root, $gi, $path, false);
        };
        ($name:ident, $root:expr, $gi:expr, $path:expr, $is_dir:expr) => {
            #[test]
            fn $name() {
                let gi = gi_from_str($root, $gi);
                assert!(!gi.matched($path, $is_dir).is_ignore());
            }
        };
    }

    const ROOT: &'static str = "/home/foobar/rust/rg";

    ignored!(ig1, ROOT, "months", "months");
    ignored!(ig2, ROOT, "*.lock", "Cargo.lock");
    ignored!(ig3, ROOT, "*.rs", "src/main.rs");
    ignored!(ig4, ROOT, "src/*.rs", "src/main.rs");
    ignored!(ig5, ROOT, "/*.c", "cat-file.c");
    ignored!(ig6, ROOT, "/src/*.rs", "src/main.rs");
    ignored!(ig7, ROOT, "!src/main.rs\n*.rs", "src/main.rs");
    ignored!(ig8, ROOT, "foo/", "foo", true);
    ignored!(ig9, ROOT, "**/foo", "foo");
    ignored!(ig10, ROOT, "**/foo", "src/foo");
    ignored!(ig11, ROOT, "**/foo/**", "src/foo/bar");
    ignored!(ig12, ROOT, "**/foo/**", "wat/src/foo/bar/baz");
    ignored!(ig13, ROOT, "**/foo/bar", "foo/bar");
    ignored!(ig14, ROOT, "**/foo/bar", "src/foo/bar");
    ignored!(ig15, ROOT, "abc/**", "abc/x");
    ignored!(ig16, ROOT, "abc/**", "abc/x/y");
    ignored!(ig17, ROOT, "abc/**", "abc/x/y/z");
    ignored!(ig18, ROOT, "a/**/b", "a/b");
    ignored!(ig19, ROOT, "a/**/b", "a/x/b");
    ignored!(ig20, ROOT, "a/**/b", "a/x/y/b");
    ignored!(ig21, ROOT, r"\!xy", "!xy");
    ignored!(ig22, ROOT, r"\#foo", "#foo");
    ignored!(ig23, ROOT, "foo", "./foo");
    ignored!(ig24, ROOT, "target", "grep/target");
    ignored!(ig25, ROOT, "Cargo.lock", "./tabwriter-bin/Cargo.lock");
    ignored!(ig26, ROOT, "/foo/bar/baz", "./foo/bar/baz");
    ignored!(ig27, ROOT, "foo/", "xyz/foo", true);
    ignored!(ig28, "./src", "/llvm/", "./src/llvm", true);
    ignored!(ig29, ROOT, "node_modules/ ", "node_modules", true);
    ignored!(ig30, ROOT, "**/", "foo/bar", true);
    ignored!(ig31, ROOT, "path1/*", "path1/foo");
    ignored!(ig32, ROOT, ".a/b", ".a/b");
    ignored!(ig33, "./", ".a/b", ".a/b");
    ignored!(ig34, ".", ".a/b", ".a/b");
    ignored!(ig35, "./.", ".a/b", ".a/b");
    ignored!(ig36, "././", ".a/b", ".a/b");
    ignored!(ig37, "././.", ".a/b", ".a/b");
    ignored!(ig38, ROOT, "\\[", "[");
    ignored!(ig39, ROOT, "\\?", "?");
    ignored!(ig40, ROOT, "\\*", "*");
    ignored!(ig41, ROOT, "\\a", "a");
    ignored!(ig42, ROOT, "s*.rs", "sfoo.rs");
    ignored!(ig43, ROOT, "**", "foo.rs");
    ignored!(ig44, ROOT, "**/**/*", "a/foo.rs");

    not_ignored!(ignot1, ROOT, "amonths", "months");
    not_ignored!(ignot2, ROOT, "monthsa", "months");
    not_ignored!(ignot3, ROOT, "/src/*.rs", "src/grep/src/main.rs");
    not_ignored!(ignot4, ROOT, "/*.c", "mozilla-sha1/sha1.c");
    not_ignored!(ignot5, ROOT, "/src/*.rs", "src/grep/src/main.rs");
    not_ignored!(ignot6, ROOT, "*.rs\n!src/main.rs", "src/main.rs");
    not_ignored!(ignot7, ROOT, "foo/", "foo", false);
    not_ignored!(ignot8, ROOT, "**/foo/**", "wat/src/afoo/bar/baz");
    not_ignored!(ignot9, ROOT, "**/foo/**", "wat/src/fooa/bar/baz");
    not_ignored!(ignot10, ROOT, "**/foo/bar", "foo/src/bar");
    not_ignored!(ignot11, ROOT, "#foo", "#foo");
    not_ignored!(ignot12, ROOT, "\n\n\n", "foo");
    not_ignored!(ignot13, ROOT, "foo/**", "foo", true);
    not_ignored!(
        ignot14,
        "./third_party/protobuf",
        "m4/ltoptions.m4",
        "./third_party/protobuf/csharp/src/packages/repositories.config"
    );
    not_ignored!(ignot15, ROOT, "!/bar", "foo/bar");
    not_ignored!(ignot16, ROOT, "*\n!**/", "foo", true);
    not_ignored!(ignot17, ROOT, "src/*.rs", "src/grep/src/main.rs");
    not_ignored!(ignot18, ROOT, "path1/*", "path2/path1/foo");
    not_ignored!(ignot19, ROOT, "s*.rs", "src/foo.rs");

    #[test]
    fn root_prefix_collision_requires_component_boundary() {
        let gi = gi_from_str("/T/a", "/b/note.agent-besties.md");
        assert!(!gi.matched("/T/ab/note.agent-besties.md", false).is_ignore());
        assert!(gi.matched("/T/a/b/note.agent-besties.md", false).is_ignore());

        let gi = gi_from_str("a", "/b/note.agent-besties.md");
        assert!(!gi.matched("ab/note.agent-besties.md", false).is_ignore());
        assert!(gi.matched("a/b/note.agent-besties.md", false).is_ignore());
        assert!(!gi.matched("./ab/note.agent-besties.md", false).is_ignore());
        assert!(gi.matched("./a/b/note.agent-besties.md", false).is_ignore());

        let gi = gi_from_str("/", "/b/note.agent-besties.md");
        assert!(gi.matched("/b/note.agent-besties.md", false).is_ignore());
        assert!(!gi.matched("/ab/note.agent-besties.md", false).is_ignore());

        #[cfg(unix)]
        {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            let root = Path::new(OsStr::from_bytes(b"/T/\xff"));
            let gi = gi_from_str(root, "/b/note.agent-besties.md");
            let sibling =
                Path::new(OsStr::from_bytes(b"/T/\xffb/note.agent-besties.md"));
            let descendant =
                Path::new(OsStr::from_bytes(b"/T/\xff/b/note.agent-besties.md"));
            assert!(!gi.matched(sibling, false).is_ignore());
            assert!(gi.matched(descendant, false).is_ignore());

            let gi = gi_from_str("/T/a", "/foo");
            assert!(gi.matched("/T/a/foo", false).is_ignore());
            assert!(!gi.matched("/T/a/./foo", false).is_ignore());
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_volume_prefix_equivalence_relativizes_anchored_rules() {
        let gi = gi_from_str(r"C:\T\room", "/note.agent-besties.md");
        assert!(gi.matched(r"C:\T\room\note.agent-besties.md", false).is_ignore());
        assert!(gi
            .matched(r"\\?\C:\T\room\note.agent-besties.md", false)
            .is_ignore());
        assert!(!gi
            .matched(r"C:\T\roomx\note.agent-besties.md", false)
            .is_ignore());
        assert!(!gi
            .matched(r"\\?\C:\T\roomx\note.agent-besties.md", false)
            .is_ignore());
        assert!(gi
            .matched(r"\\?\c:\T\room\note.agent-besties.md", false)
            .is_ignore());
        assert!(!gi
            .matched(r"\\?\D:\T\room\note.agent-besties.md", false)
            .is_ignore());
        assert!(!gi
            .matched(r"\\.\C:\T\room\note.agent-besties.md", false)
            .is_ignore());

        let gi = gi_from_str(r"\\?\C:\T\room", "/note.agent-besties.md");
        assert!(gi.matched(r"C:\T\room\note.agent-besties.md", false).is_ignore());
        assert!(gi
            .matched(r"\\?\C:\T\room\note.agent-besties.md", false)
            .is_ignore());

        let gi = gi_from_str(r"C:\T\a", "/b/note.agent-besties.md");
        assert!(!gi
            .matched(r"\\?\C:\T\ab\note.agent-besties.md", false)
            .is_ignore());
        assert!(gi
            .matched(r"\\?\C:\T\a\b\note.agent-besties.md", false)
            .is_ignore());

        let gi = gi_from_str(r"\\server\share\room", "/note.agent-besties.md");
        assert!(gi
            .matched(r"\\server\share\room\note.agent-besties.md", false)
            .is_ignore());
        assert!(gi
            .matched(r"\\?\UNC\server\share\room\note.agent-besties.md", false)
            .is_ignore());
        assert!(!gi
            .matched(r"\\?\UNC\server\share\roomx\note.agent-besties.md", false)
            .is_ignore());

        let gi = gi_from_str(
            r"\\?\UNC\server\share\room",
            "/note.agent-besties.md",
        );
        assert!(gi
            .matched(r"\\server\share\room\note.agent-besties.md", false)
            .is_ignore());
    }

    #[cfg(windows)]
    #[test]
    fn windows_shorter_candidate_keeps_global_directory_rule() {
        let gi = gi_from_str(r"C:\T\tree\hidden\work", "hidden/");
        assert!(gi.matched(r"C:\T\tree\hidden", true).is_ignore());
        assert!(gi.matched(r"\\?\C:\T\tree\hidden", true).is_ignore());
        assert!(gi.matched(r"\\?\c:\T\tree\hidden", true).is_ignore());

        let gi = gi_from_str(r"\\?\C:\T\tree\hidden\work", "hidden/");
        assert!(gi.matched(r"C:\T\tree\hidden", true).is_ignore());
        assert!(gi.matched(r"\\?\C:\T\tree\hidden", true).is_ignore());

        let gi = gi_from_str(r"C:\T\tree", "hidden/");
        assert!(gi.matched(r"C:\T\tree\hidden", true).is_ignore());
        assert!(gi.matched(r"\\?\C:\T\tree\hidden", true).is_ignore());

        let gi = gi_from_str(r"\\server\share\tree\hidden\work", "hidden/");
        assert!(gi.matched(r"\\server\share\tree\hidden", true).is_ignore());
        assert!(gi
            .matched(r"\\?\UNC\server\share\tree\hidden", true)
            .is_ignore());

        let gi = gi_from_str(r"\\?\UNC\server\share\tree", "hidden/");
        assert!(gi
            .matched(r"\\server\share\tree\hidden", true)
            .is_ignore());
        assert!(gi
            .matched(r"\\?\UNC\server\share\tree\hidden", true)
            .is_ignore());
    }

    fn bytes(s: &str) -> Vec<u8> {
        s.to_string().into_bytes()
    }

    fn path_string<P: AsRef<Path>>(path: P) -> String {
        path.as_ref().to_str().unwrap().to_string()
    }

    #[test]
    fn parse_excludes_file1() {
        let data = bytes("[core]\nexcludesFile = /foo/bar");
        let got = super::parse_excludes_file(&data).unwrap();
        assert_eq!(path_string(got), "/foo/bar");
    }

    #[test]
    fn parse_excludes_file2() {
        let data = bytes("[core]\nexcludesFile = ~/foo/bar");
        let got = super::parse_excludes_file(&data).unwrap();
        let expected = super::home_dir()
            .map(|home| home.join("foo/bar"))
            .unwrap_or_else(|| PathBuf::from("~/foo/bar"));
        assert_eq!(got, expected);
    }

    #[test]
    fn parse_excludes_file3() {
        let data = bytes("[core]\nexcludeFile = /foo/bar");
        assert!(super::parse_excludes_file(&data).is_none());
    }

    #[test]
    fn parse_excludes_file4() {
        let data = bytes("[core]\nexcludesFile = \"~/foo/~/bar\"");
        let got = super::parse_excludes_file(&data).unwrap();
        let expected = super::home_dir()
            .map(|home| home.join("foo/~/bar"))
            .unwrap_or_else(|| PathBuf::from("~/foo/~/bar"));
        assert_eq!(got, expected);
    }

    #[test]
    fn parse_excludes_file5() {
        let data = bytes("[core]\nexcludesFile = \" \"~/foo/bar \" \"");
        assert!(super::parse_excludes_file(&data).is_none());
    }

    // See: https://github.com/BurntSushi/ripgrep/issues/106
    #[test]
    fn regression_106() {
        gi_from_str("/", " ");
    }

    #[test]
    fn loaded_false_without_file_read() {
        assert!(!Gitignore::empty().is_loaded());
        let gi = GitignoreBuilder::new(ROOT)
            .add_str(None, "*.rs")
            .unwrap()
            .build()
            .unwrap();
        assert!(!gi.is_loaded());
    }

    #[test]
    fn case_insensitive() {
        let gi = GitignoreBuilder::new(ROOT)
            .case_insensitive(true)
            .unwrap()
            .add_str(None, "*.html")
            .unwrap()
            .build()
            .unwrap();
        assert!(gi.matched("foo.html", false).is_ignore());
        assert!(gi.matched("foo.HTML", false).is_ignore());
        assert!(!gi.matched("foo.htm", false).is_ignore());
        assert!(!gi.matched("foo.HTM", false).is_ignore());
    }

    ignored!(cs1, ROOT, "*.html", "foo.html");
    not_ignored!(cs2, ROOT, "*.html", "foo.HTML");
    not_ignored!(cs3, ROOT, "*.html", "foo.htm");
    not_ignored!(cs4, ROOT, "*.html", "foo.HTM");

    /// Spawn this test binary under an isolated git/home environment. The
    /// child sees `root_key` and runs the assertions; the parent never
    /// mutates process-global environment.
    fn run_env_isolated(
        test: &str,
        root_key: &str,
        root: &Path,
        vars: &[(&str, &Path)],
    ) -> bool {
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
            if let Some(status) = child.try_wait().expect("observe isolated child")
            {
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
    fn git_config_global_without_excludesfile_skips_replaced_home() {
        const ROOT_KEY: &str = "IGNORE_GITCONFIG_GLOBAL_NO_EXCLUDES";
        if let Some(base) = std::env::var_os(ROOT_KEY) {
            let base = PathBuf::from(base);
            let got = super::gitconfig_excludes_path()
                .expect("override without excludesFile must not open home");
            assert_eq!(got, Some(base.join("xdg").join("git").join("ignore")));
            return;
        }

        let td = crate::tests::TempDir::new().unwrap();
        let base = td.path();
        let home = base.join("home");
        let xdg = base.join("xdg");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&xdg).unwrap();
        std::fs::create_dir_all(home.join(".gitconfig")).unwrap();
        std::fs::write(base.join("global.cfg"), "[user]\nname = isolated\n")
            .unwrap();

        let ok = run_env_isolated(
            "gitignore::tests::git_config_global_without_excludesfile_skips_replaced_home",
            ROOT_KEY,
            base,
            &[
                ("HOME", &home),
                ("USERPROFILE", &home),
                ("XDG_CONFIG_HOME", &xdg),
                ("GIT_CONFIG_GLOBAL", &base.join("global.cfg")),
                ("GIT_CONFIG_SYSTEM", &base.join("missing-system")),
            ],
        );
        assert!(ok, "isolated GIT_CONFIG_GLOBAL replacement child failed");
    }

    #[test]
    fn git_config_global_unreadable_remains_error() {
        const ROOT_KEY: &str = "IGNORE_GITCONFIG_GLOBAL_UNREADABLE";
        if std::env::var_os(ROOT_KEY).is_some() {
            assert!(
                super::gitconfig_excludes_path().is_err(),
                "selected override failures must remain errors"
            );
            return;
        }

        let td = crate::tests::TempDir::new().unwrap();
        let base = td.path();
        let home = base.join("home");
        let xdg = base.join("xdg");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&xdg).unwrap();
        std::fs::create_dir_all(base.join("global.cfg")).unwrap();

        let ok = run_env_isolated(
            "gitignore::tests::git_config_global_unreadable_remains_error",
            ROOT_KEY,
            base,
            &[
                ("HOME", &home),
                ("USERPROFILE", &home),
                ("XDG_CONFIG_HOME", &xdg),
                ("GIT_CONFIG_GLOBAL", &base.join("global.cfg")),
                ("GIT_CONFIG_SYSTEM", &base.join("missing-system")),
            ],
        );
        assert!(ok, "isolated GIT_CONFIG_GLOBAL failure child failed");
    }

    #[test]
    fn git_config_home_excludesfile_used_without_override() {
        const ROOT_KEY: &str = "IGNORE_GITCONFIG_HOME_PRECEDENCE";
        if let Some(base) = std::env::var_os(ROOT_KEY) {
            let base = PathBuf::from(base);
            let got = super::gitconfig_excludes_path()
                .expect("home config must be selected without override");
            assert_eq!(got, Some(base.join("from-home")));
            return;
        }

        let td = crate::tests::TempDir::new().unwrap();
        let base = td.path().join("native~literal");
        let home = base.join("home");
        let xdg = base.join("xdg");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(xdg.join("git")).unwrap();
        std::fs::write(
            home.join(".gitconfig"),
            format!(
                "[core]\nexcludesFile = {}\n",
                base.join("from-home").display()
            ),
        )
        .unwrap();
        std::fs::write(
            xdg.join("git").join("config"),
            format!(
                "[core]\nexcludesFile = {}\n",
                base.join("from-xdg").display()
            ),
        )
        .unwrap();

        let ok = run_env_isolated(
            "gitignore::tests::git_config_home_excludesfile_used_without_override",
            ROOT_KEY,
            &base,
            &[
                ("HOME", &home),
                ("USERPROFILE", &home),
                ("XDG_CONFIG_HOME", &xdg),
                ("GIT_CONFIG_SYSTEM", &base.join("missing-system")),
            ],
        );
        assert!(ok, "isolated home-precedence child failed");
    }
}
