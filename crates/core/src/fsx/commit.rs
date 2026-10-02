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
    fn restore(self, file: &mut File, old: &[u8], cause: io::Error) -> CommitError {
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
                let len = file.metadata()?.len();
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
fn write_at(file: &mut File, offset: usize, bytes: &[u8], written: &mut usize) -> io::Result<()> {
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
