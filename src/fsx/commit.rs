//! In-place commit of a planned rewrite through the held, locked handle,
//! with a bounded restore of the previous bytes when it fails (§10.2.2).
//!
//! For previous bytes `old` and planned bytes `new`, with `keep` the length
//! of their common prefix and `overlap = min(old.len(), new.len())`:
//!
//! 1. growth first: `new[old.len()..]` is written at offset `old.len()`.
//!    Running out of space or into a file-size limit while the file grows
//!    therefore never touches the previous bytes, and undoing it is one
//!    `set_len(old.len())`, which frees space instead of needing it;
//! 2. `new[keep..overlap]` at offset `keep`. The common prefix (all of `old`
//!    for an append) is never rewritten;
//! 3. shrink: `set_len(new.len())`, only after the new bytes are in place;
//! 4. `sync_data`.
//!
//! A failure in 1–3 restores the previous content: `set_len(old.len())`
//! if step 1 ran, `old` over the bytes the OS reported written in step 2,
//! and the removed tail if a failed step 3 shortened the file after all. A
//! failed `write` call reports no bytes; by the regular-file contract of
//! `write(2)` / `WriteFile` it changed none, so a failure before any byte
//! is reported needs no rewrite. "Restored" means the file shows its
//! previous bytes again; like the write it undoes, it is not synced. The
//! restore is storage I/O too and can fail (a device error; no space to
//! overwrite already allocated bytes on a copy-on-write or compressing file
//! system); the file state is then reported as far as it is known, never
//! guessed. A failed step 4 undoes nothing: the new content is complete and
//! visible, and a second write and sync after a sync error could not be
//! trusted either.
//!
//! This is not crash safety: a process or machine that stops in the middle
//! of steps 1–3 still leaves previous bytes mixed with new ones.

use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};

use crate::error::WriteLeft;

/// Why [`commit`] failed, and what the file shows as a result.
#[derive(Debug)]
pub(super) enum CommitError {
    /// The file shows its previous content: it was never changed, or the
    /// restore wrote it back.
    Restored(io::Error),
    /// The file shows neither content for certain (see [`WriteLeft`]).
    Uncertain { cause: io::Error, left: WriteLeft },
}

/// Replace the content of `file` (currently `old`) with `new` in place.
pub(super) fn commit(file: &mut File, old: &[u8], new: &[u8]) -> Result<(), CommitError> {
    commit_io(file, old, new)
}

// Keep the production entry on a real File. This small interface lets the
// tests exercise partial writes and failing storage operations that a
// normal temporary directory cannot cause deterministically.
// INVARIANT: Used exclusively in internal unit tests for deterministic
// ACID rollback mathematical verification. Production code accepts only std::fs::File.
trait CommitFile: Write + Seek {
    fn set_len(&mut self, len: u64) -> io::Result<()>;
    fn len(&self) -> io::Result<u64>;
    fn sync_data(&self) -> io::Result<()>;
}

impl CommitFile for File {
    fn set_len(&mut self, len: u64) -> io::Result<()> {
        File::set_len(self, len)
    }

    fn len(&self) -> io::Result<u64> {
        Ok(self.metadata()?.len())
    }

    fn sync_data(&self) -> io::Result<()> {
        File::sync_data(self)
    }
}

fn commit_io(file: &mut impl CommitFile, old: &[u8], new: &[u8]) -> Result<(), CommitError> {
    let overlap = old.len().min(new.len());
    let keep = old[..overlap]
        .iter()
        .zip(&new[..overlap])
        .take_while(|(a, b)| a == b)
        .count();
    let mut grown = 0;
    let mut changed = 0;
    let mut shrink_tried = false;
    let steps = (|| {
        if new.len() > old.len() {
            write_at(file, old.len(), &new[old.len()..], &mut grown)?;
        }
        write_at(file, keep, &new[keep..overlap], &mut changed)?;
        if new.len() < old.len() {
            shrink_tried = true;
            file.set_len(new.len() as u64)?;
        }
        Ok::<(), io::Error>(())
    })();
    if let Err(cause) = steps {
        let undo = Undo {
            keep,
            grown: grown > 0,
            changed,
            shrink_tried,
        };
        return Err(undo.restore(file, old, cause));
    }
    file.sync_data().map_err(|cause| CommitError::Uncertain {
        cause,
        left: WriteLeft::Unconfirmed,
    })
}

/// What steps 1–3 reported doing before they failed.
struct Undo {
    keep: usize,
    /// Step 1 reported writing at least one byte past the previous end.
    grown: bool,
    /// Bytes of step 2 reported written, starting at `keep`.
    changed: usize,
    shrink_tried: bool,
}

impl Undo {
    fn restore(self, file: &mut impl CommitFile, old: &[u8], cause: io::Error) -> CommitError {
        if !self.grown && self.changed == 0 && !self.shrink_tried {
            return CommitError::Restored(cause);
        }
        // Bytes before `from` were never written: before `keep` nothing was,
        // and step 1 writes only past the previous end. A step 3 attempt
        // means step 2 completed, so its range starts at `keep` as well.
        // From `from` on, a failed restore may leave whatever the steps or
        // the restore wrote.
        let from = if self.changed > 0 || self.shrink_tried {
            self.keep
        } else {
            old.len()
        };
        let result = (|| {
            if self.grown {
                file.set_len(old.len() as u64)?;
            }
            let end = self.keep + self.changed;
            write_at(file, self.keep, &old[self.keep..end], &mut 0)?;
            if self.shrink_tried {
                // A failed truncation is expected to leave the length alone;
                // if it did not, put back what it removed.
                let len = file.len()?;
                if len < old.len() as u64 {
                    let len = len as usize;
                    write_at(file, len, &old[len..], &mut 0)?;
                }
            }
            Ok::<(), io::Error>(())
        })();
        match result {
            Ok(()) => CommitError::Restored(cause),
            Err(rollback) => CommitError::Uncertain {
                cause,
                left: WriteLeft::Mixed {
                    from: from as u64,
                    len: old.len() as u64,
                    rollback,
                },
            },
        }
    }
}

/// Write `bytes` at `offset`, counting in `written` every byte the OS
/// reports written, including those before a failing call.
fn write_at(
    file: &mut impl CommitFile,
    offset: usize,
    bytes: &[u8],
    written: &mut usize,
) -> io::Result<()> {
    if bytes.is_empty() {
        return Ok(());
    }
    file.seek(SeekFrom::Start(offset as u64))?;
    while *written < bytes.len() {
        match file.write(&bytes[*written..]) {
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(n) => *written += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::io::{Cursor, ErrorKind};

    struct FaultFile {
        bytes: Cursor<Vec<u8>>,
        writes: VecDeque<Result<usize, ErrorKind>>,
        fail_truncate: bool,
        truncate_before_error: bool,
        fail_sync: bool,
    }

    impl FaultFile {
        fn new(bytes: &[u8], writes: Vec<Result<usize, ErrorKind>>) -> Self {
            Self {
                bytes: Cursor::new(bytes.to_vec()),
                writes: writes.into(),
                fail_truncate: false,
                truncate_before_error: false,
                fail_sync: false,
            }
        }
    }

    impl Write for FaultFile {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let limit = self.writes.pop_front().unwrap_or(Ok(bytes.len()))?;
            self.bytes.write(&bytes[..bytes.len().min(limit)])
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Seek for FaultFile {
        fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(from)
        }
    }

    impl CommitFile for FaultFile {
        fn set_len(&mut self, len: u64) -> io::Result<()> {
            if self.fail_truncate {
                self.fail_truncate = false;
                if self.truncate_before_error {
                    self.bytes.get_mut().resize(len as usize, 0);
                }
                return Err(ErrorKind::Other.into());
            }
            self.bytes.get_mut().resize(len as usize, 0);
            Ok(())
        }

        fn len(&self) -> io::Result<u64> {
            Ok(self.bytes.get_ref().len() as u64)
        }

        fn sync_data(&self) -> io::Result<()> {
            if self.fail_sync {
                Err(ErrorKind::Other.into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn growth_failure_restores_existing_and_new_files() {
        for old in [b"old".as_slice(), b""] {
            let mut new = old.to_vec();
            new.extend_from_slice(b"new tail");
            let mut file = FaultFile::new(old, vec![Ok(2), Err(ErrorKind::StorageFull)]);
            assert!(matches!(
                commit_io(&mut file, old, &new),
                Err(CommitError::Restored(_))
            ));
            assert_eq!(file.bytes.get_ref(), old);
        }
    }

    #[test]
    fn overlap_failure_restores_bytes_and_removes_completed_growth() {
        let old = b"abOLD!";
        let new = b"abNEWTAIL";
        let mut file = FaultFile::new(old, vec![Ok(usize::MAX), Ok(2), Err(ErrorKind::Other)]);
        assert!(matches!(
            commit_io(&mut file, old, new),
            Err(CommitError::Restored(_))
        ));
        assert_eq!(file.bytes.get_ref(), old);
    }

    #[test]
    fn failed_restore_reports_only_the_known_unchanged_prefix() {
        let old = b"prefix-old-content";
        let new = b"prefix-new-content";
        let mut file = FaultFile::new(
            old,
            vec![Ok(2), Err(ErrorKind::Other), Err(ErrorKind::StorageFull)],
        );
        let Err(CommitError::Uncertain {
            left:
                WriteLeft::Mixed {
                    from,
                    len,
                    rollback,
                },
            ..
        }) = commit_io(&mut file, old, new)
        else {
            panic!("failed restoration must report mixed content");
        };
        assert_eq!(from, 7);
        assert_eq!(len, old.len() as u64);
        assert_eq!(rollback.kind(), ErrorKind::StorageFull);
        assert_eq!(
            &file.bytes.get_ref()[..from as usize],
            &old[..from as usize]
        );
        assert_ne!(file.bytes.get_ref(), old);
        assert_ne!(file.bytes.get_ref(), new);
    }

    #[test]
    fn failed_growth_cleanup_keeps_the_original_prefix() {
        let old = b"old";
        let mut file = FaultFile::new(old, vec![Ok(1), Err(ErrorKind::StorageFull)]);
        file.fail_truncate = true;
        let Err(CommitError::Uncertain {
            left: WriteLeft::Mixed { from, len, .. },
            ..
        }) = commit_io(&mut file, old, b"old-new")
        else {
            panic!("failed growth cleanup must report the retained tail");
        };
        assert_eq!((from, len), (3, 3));
        assert_eq!(file.bytes.get_ref(), b"old-");
    }

    #[test]
    fn failed_shrink_restores_the_tail_even_if_the_length_changed() {
        let old = b"prefix-long-content";
        let mut file = FaultFile::new(old, vec![]);
        file.fail_truncate = true;
        file.truncate_before_error = true;
        assert!(matches!(
            commit_io(&mut file, old, b"prefix-x"),
            Err(CommitError::Restored(_))
        ));
        assert_eq!(file.bytes.get_ref(), old);
    }

    #[test]
    fn sync_failure_keeps_the_complete_new_content() {
        for (old, new) in [
            (b"".as_slice(), b"new".as_slice()),
            (b"prefix-old".as_slice(), b"prefix-longer-new".as_slice()),
            (b"prefix-long-old".as_slice(), b"prefix-new".as_slice()),
        ] {
            let mut file = FaultFile::new(old, vec![]);
            file.fail_sync = true;
            assert!(matches!(
                commit_io(&mut file, old, new),
                Err(CommitError::Uncertain {
                    left: WriteLeft::Unconfirmed,
                    ..
                })
            ));
            assert_eq!(file.bytes.get_ref(), new);
        }
    }

    #[test]
    fn interrupted_and_zero_length_writes_keep_their_io_meaning() {
        let old = b"old";
        let mut file = FaultFile::new(old, vec![Err(ErrorKind::Interrupted), Ok(1)]);
        commit_io(&mut file, old, b"old-new").unwrap();
        assert_eq!(file.bytes.get_ref(), b"old-new");

        let mut file = FaultFile::new(old, vec![Ok(0)]);
        let Err(CommitError::Restored(cause)) = commit_io(&mut file, old, b"new") else {
            panic!("zero-length write must fail without changing bytes");
        };
        assert_eq!(cause.kind(), ErrorKind::WriteZero);
        assert_eq!(file.bytes.get_ref(), old);
    }
}
