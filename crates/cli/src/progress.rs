//! stderr progress for traversing commands (spec §8.4).
//!
//! Only active when stderr is a terminal. Nothing is emitted before
//! `PROGRESS_AFTER_MS` counted from the first counter callback; afterwards
//! one line is rewritten every `PROGRESS_EVERY_MS`:
//! `\rscanning <n> dirs, <m> files...` — no newline. [`Progress::finish`]
//! erases the line with blanks and a carriage return.
//!
//! The counters (`directories entered`, `managed files accounted for`,
//! including read failures) are fed by core through `Ctx::progress` /
//! `Ctx::file_read`; core never prints. Non-TTY stderr yields a `None`
//! sink and an inert handle, so a redirected stderr stays byte-clean
//! (spec §8.4.2) and stdout is never touched.

use std::io::{IsTerminal, Write};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use coexist_core::consts;
use coexist_core::ops::context::ProgressFn;

struct State {
    /// When the first counter callback arrived; `None` until then.
    started: Option<Instant>,
    /// When the line was last drawn; `None` until the first draw.
    drawn: Option<Instant>,
    /// Length, in chars, of the currently drawn line.
    drawn_len: usize,
}

fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    // The state is plain data; a panic inside one callback must not cascade
    // into later ones through poisoning.
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The caller's handle for erasing the line when traversal ends.
pub struct Progress {
    state: Arc<Mutex<State>>,
}

/// Build the progress sink for this invocation.
///
/// `active` is the CLI's per-command decision (the traversing commands:
/// `list`, `read-notification`, `check-state`, `state`, `profile`, `brief`). The sink is `None`
/// unless `active` and stderr is a TTY; the handle is always returned and
/// simply writes nothing when no line was ever drawn.
pub fn make(active: bool) -> (Option<ProgressFn>, Progress) {
    let state = Arc::new(Mutex::new(State {
        started: None,
        drawn: None,
        drawn_len: 0,
    }));
    let progress = Progress {
        state: Arc::clone(&state),
    };
    if !active || !std::io::stderr().is_terminal() {
        return (None, progress);
    }
    let sink: ProgressFn = Box::new(move |dirs, files| {
        let mut state = lock(&state);
        let now = Instant::now();
        let started = *state.started.get_or_insert(now);
        if now.duration_since(started) < Duration::from_millis(consts::PROGRESS_AFTER_MS) {
            return;
        }
        let due = match state.drawn {
            None => true,
            Some(last) => {
                now.duration_since(last) >= Duration::from_millis(consts::PROGRESS_EVERY_MS)
            }
        };
        if !due {
            return;
        }
        let line = format!("scanning {dirs} dirs, {files} files...");
        // Best-effort on stderr: a failure here must not fail the command.
        let _ = write!(std::io::stderr(), "\r{line}");
        state.drawn = Some(now);
        state.drawn_len = line.chars().count();
    });
    (Some(sink), progress)
}

impl Progress {
    /// Erase the progress line (blank it, carriage-return home). Writes
    /// nothing when nothing was ever drawn — including the whole non-TTY
    /// case (spec §8.4.1).
    pub fn finish(&self) {
        let mut state = lock(&self.state);
        if state.drawn.is_none() {
            return;
        }
        let blanks = " ".repeat(state.drawn_len);
        state.drawn = None;
        state.started = None;
        state.drawn_len = 0;
        // Best-effort on stderr: a failure here must not fail the command.
        let _ = write!(std::io::stderr(), "\r{blanks}\r");
    }
}
