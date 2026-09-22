//! Compile-time constants, transcribed from spec §12 (normative constants table).

/// Managed-file name suffix (§12, per §2.4).
pub const SUFFIX: &str = ".agent-besties.md";

/// Default duration of a `locked` declaration: 2h in seconds (§12, per §6.3.1).
pub const DEFAULT_FOR_LOCKED: i64 = 7200; // 2h in seconds

/// Default duration of non-`locked` state declarations: 8h in seconds (§12, per §6.3.1).
pub const DEFAULT_FOR_OTHER: i64 = 28800; // 8h

/// Window in which a nick's own valid state entry is flagged `expiring` (until-now at or below this): 30m in seconds (§12, per §10.6.3).
pub const EXPIRING_WINDOW: i64 = 1800; // 30m

/// Exclusive-lock queueing cap for post/subscribe writes: 5000 ms (§12, per §10.2).
pub const POST_WAIT_MS: u64 = 5000;

/// Fast-fail poll cap for exclusive locks on profile/brief/state writes: 200 ms (§12, per §10.2).
pub const FASTFAIL_GRACE_MS: u64 = 200;

/// Shared-lock poll cap for single-file reads (check, lookfor-comment, read-brief): 1000 ms (§12, per §10.2).
pub const READ_WAIT_MS: u64 = 1000;

/// Shared-lock poll cap for multi-file scan reads (list, read-notification, check-state): 100 ms (§12, per §10.2).
pub const SCAN_READ_WAIT_MS: u64 = 100;

/// Lock polling interval: 20 ms (§12, per §10.2).
pub const LOCK_POLL_MS: u64 = 20;

/// Reopen retry cap when a locked file's identity changed on the write path: 3 (§12, per §10.2.6).
pub const OPEN_RETRIES: usize = 3;

/// Managed-file byte cap enforced on the write path and during scans: 16 MiB (§12, per §10.1.7, §10.2.2).
pub const MAX_MANAGED_FILE_BYTES: usize = 16 * 1024 * 1024;

/// In-memory target-read cap (regex and rendering only; streaming hash is uncapped): 64 MiB (§12, per §10.4.2, §10.10.3).
pub const MAX_TARGET_BYTES: usize = 64 * 1024 * 1024;

/// Excerpt truncation bound (beyond it: 77 chars + `...`): 80 (§12, per §8.1.8).
pub const EXCERPT_MAX: usize = 80;

/// Key column width in diagnostic lines: 12 (§12, per §8.1.5).
pub const KEY_WIDTH: usize = 12;

/// Type column position in `list` output: 30 (§12, per §8.2 list).
pub const TREE_TYPE_COL: usize = 30;

/// Kind column width in `read-notification`: 10 (§12, per §8.2 read-notification).
pub const KIND_WIDTH: usize = 10;

/// Delay before scan progress lines may start on a TTY stderr: 500 ms (§12, per §8.4).
pub const PROGRESS_AFTER_MS: u64 = 500;

/// Scan progress line rewrite interval: 200 ms (§12, per §8.4).
pub const PROGRESS_EVERY_MS: u64 = 200;
