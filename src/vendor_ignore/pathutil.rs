use std::{ffi::OsStr, path::Path};

use crate::vendor_ignore::walk::DirEntry;

/// Returns true if and only if this path is considered to be hidden.
///
/// # Platform behavior
///
/// ## Windows
///
/// This returns true if one of the following is true:
///
/// * The base name of the path starts with a `.`.
/// * The file attributes have the `HIDDEN` property set.
///
/// ## All other platforms
///
/// This only returns true if the base name of the path starts with a `.`.
pub(crate) fn is_hidden_path(dent: &Path) -> bool {
    #[cfg(not(windows))]
    fn imp(path: &Path) -> bool {
        is_hidden_path_only(path)
    }

    #[cfg(windows)]
    fn imp(path: &Path) -> bool {
        use std::os::windows::fs::MetadataExt;
        use winapi_util::file;

        if let Ok(md) = path.metadata() {
            if file::is_hidden(md.file_attributes() as u64) {
                return true;
            }
        }
        is_hidden_path_only(path)
    }

    imp(dent)
}

/// Returns true if and only if this directory entry is considered to be
/// hidden.
///
/// # Platform behavior
///
/// ## Windows
///
/// This returns true if one of the following is true:
///
/// * The base name of the path starts with a `.`.
/// * The file attributes have the `HIDDEN` property set.
///
/// ## All other platforms
///
/// This only returns true if the base name of the path starts with a `.`.
pub(crate) fn is_hidden_entry(dent: &DirEntry) -> bool {
    #[cfg(not(windows))]
    fn imp(dent: &DirEntry) -> bool {
        is_hidden_path_only(dent.path())
    }

    #[cfg(windows)]
    fn imp(dent: &DirEntry) -> bool {
        use std::os::windows::fs::MetadataExt;
        use winapi_util::file;

        // This looks like we're doing an extra stat call, but on Windows, the
        // directory traverser reuses the metadata retrieved from each directory
        // entry and stores it on the DirEntry itself. So this is "free."
        if let Ok(md) = dent.metadata() {
            if file::is_hidden(md.file_attributes() as u64) {
                return true;
            }
        }
        is_hidden_path_only(dent.path())
    }

    imp(dent)
}

/// Returns true if and only if this path is considered to be hidden from only
/// the path itself.
///
/// This has the same behavior on all platforms.
fn is_hidden_path_only(path: &Path) -> bool {
    if let Some(name) = file_name(path) {
        name.as_encoded_bytes().starts_with(b".")
    } else {
        false
    }
}

/// Strip `prefix` from the `path` and return the remainder.
///
/// If `path` doesn't have a prefix `prefix`, then return `None`.
///
/// On Windows, ordinary `Disk`/`UNC` volume prefixes are equivalent to the
/// corresponding `VerbatimDisk`/`VerbatimUNC` spellings at this comparison
/// boundary. Disk letters are compared as `Prefix` reports them: native
/// parsing already stores `c:` and `C:` as the same disk value, so that is
/// same-volume behavior rather than extra case-folding here. The remainder
/// is a suffix of `path`, so remaining native components and verbatim bytes
/// are preserved. This does not canonicalize, reopen, or rewrite the input.
pub(crate) fn strip_prefix<'a, P: AsRef<Path> + ?Sized>(
    prefix: &'a P,
    path: &'a Path,
) -> Option<&'a Path> {
    #[cfg(unix)]
    fn imp<'a>(prefix: &'a Path, path: &'a Path) -> Option<&'a Path> {
        use std::os::unix::ffi::OsStrExt;

        let prefix = prefix.as_os_str().as_bytes();
        let path = path.as_os_str().as_bytes();
        if prefix.len() > path.len() || prefix != &path[0..prefix.len()] {
            None
        } else {
            Some(&Path::new(OsStr::from_bytes(&path[prefix.len()..])))
        }
    }

    #[cfg(not(unix))]
    fn imp<'a>(prefix: &'a Path, path: &'a Path) -> Option<&'a Path> {
        path.strip_prefix(prefix).ok().or_else(|| {
            #[cfg(windows)]
            {
                strip_equivalent_windows_volume_prefix(prefix, path)
            }
            #[cfg(not(windows))]
            {
                None
            }
        })
    }

    imp(prefix.as_ref(), path)
}

/// Strip `prefix` from `path` when they name the same Windows volume using
/// mixed ordinary/verbatim prefixes. Identical prefixes are left to
/// `Path::strip_prefix`. `DeviceNS` and other kinds are not equated; a
/// different disk letter such as `D:` is a different volume.
/// Every remaining prefix component must have one equal candidate
/// component; a longer prefix is not a prefix of a shorter candidate.
#[cfg(windows)]
fn strip_equivalent_windows_volume_prefix<'a>(
    prefix: &'a Path,
    path: &'a Path,
) -> Option<&'a Path> {
    use std::path::Component;

    let mut prefix_comps = prefix.components();
    let mut path_comps = path.components();
    match (prefix_comps.next(), path_comps.next()) {
        (Some(Component::Prefix(p)), Some(Component::Prefix(q)))
            if windows_volume_prefixes_equivalent(p.kind(), q.kind()) => {}
        _ => return None,
    }
    // `Iterator::zip` consumes one extra prefix component when the candidate
    // ends first, so leftover detection misses a root that is exactly one
    // component longer. Require a candidate component for every prefix
    // component.
    for a in prefix_comps {
        match path_comps.next() {
            Some(b) if a == b => {}
            _ => return None,
        }
    }
    Some(path_comps.as_path())
}

#[cfg(windows)]
fn windows_volume_prefixes_equivalent(
    a: std::path::Prefix<'_>,
    b: std::path::Prefix<'_>,
) -> bool {
    use std::path::Prefix::*;

    match (a, b) {
        (Disk(x), VerbatimDisk(y)) | (VerbatimDisk(x), Disk(y)) => x == y,
        (UNC(a_server, a_share), VerbatimUNC(b_server, b_share))
        | (VerbatimUNC(a_server, a_share), UNC(b_server, b_share)) => {
            a_server == b_server && a_share == b_share
        }
        _ => false,
    }
}

/// Returns true if this file path is just a file name. i.e., Its parent is
/// the empty string.
pub(crate) fn is_file_name<P: AsRef<Path>>(path: P) -> bool {
    #[cfg(unix)]
    {
        memchr::memchr(b'/', path.as_ref().as_os_str().as_encoded_bytes())
            .is_none()
    }
    #[cfg(not(unix))]
    {
        path.as_ref()
            .parent()
            .map(|p| p.as_os_str().is_empty())
            .unwrap_or(false)
    }
}

/// The final component of the path, if it is a normal file.
///
/// If the path terminates in `.`, `..`, or consists solely of a root of
/// prefix, this will return `None`.
pub(crate) fn file_name<'a, P: AsRef<Path> + ?Sized>(
    path: &'a P,
) -> Option<&'a OsStr> {
    #[cfg(unix)]
    fn imp(path: &Path) -> Option<&OsStr> {
        use std::os::unix::ffi::OsStrExt;

        use memchr::memrchr;

        let path = path.as_os_str().as_bytes();
        if path.is_empty() {
            return None;
        } else if path.len() == 1 && path[0] == b'.' {
            return None;
        } else if path.last() == Some(&b'.') {
            return None;
        } else if path.len() >= 2 && &path[path.len() - 2..] == &b".."[..] {
            return None;
        }
        let last_slash = memrchr(b'/', path).map(|i| i + 1).unwrap_or(0);
        Some(OsStr::from_bytes(&path[last_slash..]))
    }

    #[cfg(not(unix))]
    fn imp(path: &Path) -> Option<&OsStr> {
        path.file_name()
    }

    imp(path.as_ref())
}

#[cfg(all(test, windows))]
mod tests {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};

    use super::strip_prefix;

    #[test]
    fn disk_and_verbatim_disk_strip_to_native_remainder() {
        let root = Path::new(r"C:\T\room");
        let verbatim = Path::new(r"\\?\C:\T\room\note.agent-coexist.md");
        assert_eq!(
            strip_prefix(root, verbatim),
            Some(Path::new("note.agent-coexist.md"))
        );
        assert_eq!(
            strip_prefix(Path::new(r"\\?\C:\T\room"), Path::new(r"C:\T\room\note.agent-coexist.md")),
            Some(Path::new("note.agent-coexist.md"))
        );
        assert_eq!(
            strip_prefix(Path::new(r"\\?\C:\T\room"), verbatim),
            Some(Path::new("note.agent-coexist.md"))
        );
        assert_eq!(
            strip_prefix(root, Path::new(r"C:\T\room\note.agent-coexist.md")),
            Some(Path::new("note.agent-coexist.md"))
        );
        assert!(strip_prefix(root, Path::new(r"\\?\C:\T\roomx\note.agent-coexist.md")).is_none());
        assert_eq!(
            strip_prefix(root, Path::new(r"\\?\c:\T\room\note.agent-coexist.md")),
            Some(Path::new("note.agent-coexist.md"))
        );
        assert!(strip_prefix(root, Path::new(r"\\.\C:\T\room\note.agent-coexist.md")).is_none());
        assert!(strip_prefix(root, Path::new(r"\\?\D:\T\room\note.agent-coexist.md")).is_none());
    }

    #[test]
    fn unc_and_verbatim_unc_strip_to_native_remainder() {
        let root = Path::new(r"\\server\share\room");
        let verbatim =
            Path::new(r"\\?\UNC\server\share\room\note.agent-coexist.md");
        assert_eq!(
            strip_prefix(root, verbatim),
            Some(Path::new("note.agent-coexist.md"))
        );
        assert_eq!(
            strip_prefix(
                Path::new(r"\\?\UNC\server\share\room"),
                Path::new(r"\\server\share\room\note.agent-coexist.md")
            ),
            Some(Path::new("note.agent-coexist.md"))
        );
        assert!(strip_prefix(
            root,
            Path::new(r"\\?\UNC\server\share\roomx\note.agent-coexist.md")
        )
        .is_none());
        assert!(strip_prefix(
            root,
            Path::new(r"\\?\UNC\other\share\room\note.agent-coexist.md")
        )
        .is_none());
    }

    #[test]
    fn equivalent_prefix_preserves_unpaired_surrogate_remainder() {
        let root = Path::new(r"C:\T\room");
        let mut wide: Vec<u16> = r"\\?\C:\T\room\".encode_utf16().collect();
        wide.push(0xD800);
        wide.extend("note.agent-coexist.md".encode_utf16());
        let path = PathBuf::from(OsString::from_wide(&wide));
        let stripped = strip_prefix(root, &path)
            .expect("Disk and VerbatimDisk of the same volume");
        let rem: Vec<u16> = stripped.as_os_str().encode_wide().collect();
        let mut expected = vec![0xD800];
        expected.extend("note.agent-coexist.md".encode_utf16());
        assert_eq!(rem, expected);
    }

    #[test]
    fn mixed_volume_prefix_rejects_shorter_candidate() {
        assert!(strip_prefix(
            Path::new(r"C:\T\tree\hidden\work"),
            Path::new(r"\\?\C:\T\tree\hidden"),
        )
        .is_none());
        assert!(strip_prefix(
            Path::new(r"\\?\C:\T\tree\hidden\work"),
            Path::new(r"C:\T\tree\hidden"),
        )
        .is_none());
        assert_eq!(
            strip_prefix(
                Path::new(r"C:\T\tree\hidden"),
                Path::new(r"\\?\C:\T\tree\hidden"),
            ),
            Some(Path::new(""))
        );
        assert_eq!(
            strip_prefix(
                Path::new(r"C:\T\tree"),
                Path::new(r"\\?\C:\T\tree\hidden"),
            ),
            Some(Path::new("hidden"))
        );
        assert!(strip_prefix(
            Path::new(r"\\server\share\tree\hidden\work"),
            Path::new(r"\\?\UNC\server\share\tree\hidden"),
        )
        .is_none());
        assert!(strip_prefix(
            Path::new(r"\\?\UNC\server\share\tree\hidden\work"),
            Path::new(r"\\server\share\tree\hidden"),
        )
        .is_none());
        assert_eq!(
            strip_prefix(
                Path::new(r"\\server\share\tree\hidden"),
                Path::new(r"\\?\UNC\server\share\tree\hidden"),
            ),
            Some(Path::new(""))
        );
    }
}
