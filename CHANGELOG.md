# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning is
[SemVer](https://semver.org/spec/2.0.0.html).

## [Unreleased]

### Added

- Public documentation set: README, a usage guide (`docs/usage.md`), a
  file-format guide (`docs/file-format.md`), and a cover image
  (`docs/assets/cover.webp`).
- Contribution guide, security policy, and a bug-report issue form.
- MIT license for code and authored documentation; the supplied cover
  image is not part of the MIT grant.

### Fixed

- Creating a post, profile, brief, or subscription refuses an invalid
  file-name stem (`E-NAME`) instead of writing an unusable heading.
  Existing files whose heading already parses stay editable when the
  stem and heading disagree.
- Removing a nick from a state file refuses the whole selected removal
  (`E-STATE-DIRTY`) when that nick is still declared on a mixed line.
  The printed retry keeps the complete original argv, including
  `--status`, so after the line is made parseable only that selected
  status is removed. Handwritten text, fences, and leftover fragments
  stay in place; a missing nick does not rewrite the file.
- A successful state declaration or removal still drops expired entries
  on clean lines. Read commands never perform that rewrite.
- Incomplete lock and state scans stay visible as `skipped` and
  `partial` rows. Non-comment stray state text (`W-STRAY`) is `partial`
  through that same scan; valid declarations remain, and omitted bytes
  never invent a lock. Identified `locked` holders still block `profile`
  and `brief` writes; an incomplete scan without such a holder warns and
  allows the write.
- `check-state` treats a fatal ignore-source error like `list` and
  `read-notification`: first line `error`, `reason  cannot scan <path>:
  <source>`, exit 1, no overview or `skipped  unreadable` stand-in.
  Ordinary unreadable directories stay skipped warnings. After a
  successful `state` write, the same source error is a `warn` with a
  `scan  <path>  <cause>` row, exit 0; the committed bytes stay, and
  there is no `fix` row and no rollback.
- Type-mismatch recovery follows the file's actual heading. Heading and
  name-mismatch recovery never suggest copying an invalid H1 built from
  a bad file stem.
- A refused creating write deletes only an empty file and empty
  directories this invocation created and still owns. Pre-existing,
  replaced, contested, or already-written objects are left alone.
  Concurrent replacements are not atomic.
- `check` reports each expiry and in-file lock-conflict diagnostic
  once, from the selected declarations. `W-STATE-CONFLICT` pairs are
  expanded for that explicit check and for a wait report that hits,
  including the snapshot from just before a deletion; ordinary parse
  and scans do not build them. A complete check still lists every pair
  and may grow with the number of overlapping locks.
  `read-notification` includes ancestor state files and does not
  double-count the starting directory's state file.
- Isolated `CR` at end of file is kept as data. Appends insert a space
  before `LF` so that `CR` is not turned into `CRLF` and does not
  activate previously opaque headings or properties. Missing-body and
  missing-property inserts (`subscribe`, `profile`, `brief`, `post`)
  reuse a following whitespace-only line that already ends in `LF`; an
  unterminated whitespace-only last line keeps those bytes, then `LF`,
  then the new block.
- `@alice-@bob` is two mentions (`alice-` and `bob`) under the existing
  preceding-character rule. A first legal post sequence greater than 1
  is the existing non-blocking gap warning; comments are not
  renumbered. Duplicate numbers remain blocking.
- Title and diagnostic columns fold display text (control characters
  become U+FFFD) without rewriting stored heading identity or body
  bytes. A file-name stem in diagnostic prose uses that same folded
  display; it is not a native-path escape. Usage errors, including
  argument-parser failures, emit one stderr line with the useful error
  and hint retained (CR/LF become spaces at emission); stored text is
  not rewritten; stdout stays empty; exit 2. Explicit `--help` and
  `--version` remain stdout, exit 0. Human title diagnostics stay
  sanitized. A stored heading that already contains control characters
  is recovered by editing that heading by hand to a valid, distinct
  title, then running `check` again; recovery does not emit raw control
  characters or a command aimed at a U+FFFD sibling. A profile section
  whose title is outside the existing section grammar gets manual
  heading-and-fenced-body advice, not a `--section` command the CLI
  would reject. An invalid stored status name is repaired the same way:
  rename the heading by hand to a valid, distinct status, then run
  `check` again; recovery does not emit a `--status` command the CLI
  would reject or a normalized sibling of the stored name. Literal/prose
  `fix` lines (file-content headers, manual heading instructions) are
  not POSIX-quoted command operands; command-form retries still use the
  existing POSIX encoding of native operands. CLI flags still reject
  control-bearing titles. A runtime title or nick that looks like
  `<file>` or `<nick>` is the actual value; only a genuine missing path
  or author still prints the static reader marker.
- Ignore and git-config sources are loaded through a pinned, locally
  patched `ignore` 0.4.33 so scans open those sources as regular files,
  match native paths, and surface load errors. Unix matcher prefix
  stripping requires a component boundary; raw `./` remainder spelling
  is preserved. On Windows, ordinary and verbatim volume prefixes
  compare as native Rust `Prefix` values (`c:` and `C:` are the same
  disk; `C:\` and `\\?\C:\` are the same volume). Prefix stripping
  consumes every remaining root component, so a shorter candidate
  cannot be treated as under that root. Ordinary and verbatim spellings
  of the same volume agree on global directory excludes. Directory
  components are not given extra case-folding, and I/O paths are not
  rewritten. Upstream glob grammar, precedence, and licenses are
  unchanged.
- In a `core.excludesFile` path read from git config, only a recognized
  leading `~` home component expands, at a component boundary and using
  the home directory's native bytes. Interior or later tildes —
  including Windows short native names such as `RUNNER~1` — and
  `~user` forms stay literal.
- Scan trees keep native source-path identity: distinct native directories
  stay distinct nodes when display names fold together, and a root whose
  name contains spaces still relativizes correctly. `check-state`
  descendant group paths are native paths relative to the requested DIR
  before that folding. Folded display is output only. On Windows,
  drive-relative display stays `C:notes` (never `C:/notes`); equivalent
  ordinary and verbatim absolute volume paths are shown cwd-relative
  using native Prefix identity, with no directory case-folding or extra
  I/O.
- Functional-lock and related source warnings keep source order.
- An explicit scan `DIR` that is a directory symlink, junction, or `.git`
  directory is entered as a directory. Discovered descendant links,
  junctions, and `.git` directories stay pruned.
- `--target` and `--regex` reject raw CR/LF without trimming other
  whitespace. `--remove` cannot be combined with `--target`; the
  combination is refused before any mutation. `--regex ''` deletes every
  regex property line on the selected entry, including handwritten
  duplicates. Generated recovery keeps native path operands (including
  the managed suffix when present, internal path whitespace, and
  Windows verbatim spelling) separately from folded display. Those
  words are encoded for a POSIX shell consumer: a nonempty word of
  ASCII letters, digits, or `_@%+,./-` is left as-is; every other word
  (empty, braces, `=`, quotes, `$`, non-ASCII, and the rest) is
  single-quoted. This preserves operands for POSIX consumers; it is not
  a user quoting API, and cmd.exe/PowerShell still need adjustment.
  Positional native operands that start with `-` are written as `./…`;
  option values use `--key=<quoted-value>`. That encoding applies to
  command-form recovery. Literal/prose `fix` lines remain file content
  or hand-edit instructions.
- Ignore source selection follows Git worktree, separate-gitdir, and
  explicit global-config override layout: relative gitfile and commondir
  paths resolve from the gitfile or gitdir, ordinary `.git` directories
  honor `commondir` the same way gitfiles do, a missing optional
  commondir does not drop that gitdir's exclude rules, and an explicit
  global config replaces the default files instead of falling through.
  Gitfile and commondir files are read as whole descriptors; only
  trailing CR/LF are trimmed, so an internal LF stays in the native
  path. Descriptor-safe reads, relative bases, optional missing
  sources, load errors, and ignore grammar are unchanged.
- `wait` and `wait-post` treat a parent that exists but is not a
  directory as a path-class error. `wait-state` still reports
  `E-NOT-DIR`. A wait report includes a brief description once when
  the file is created, deleted, or changes type. Snapshot pairing uses
  first-wins indexes of raw identities while file-order vectors remain
  the report. That removes avoidable identity search on filtered
  non-hits; it does not change filters, diffs, baselines, or required
  conflict-pair output, and it is not a universal latency promise.
- `profile` and `brief` use the existing TTY progress sink during
  ancestor lock scans.
- File-parsed leap seconds print as `:60` in `YYYY-MM-DDTHH:MM:SSZ`.
  `AGENT_COEXIST_NOW` still rejects `:60`.
- Empty scalar columns in result first lines, wait ready lines, and
  Fields keys/values are shown as `/` so the column stays present.
  This is display only: stored identity and file bytes are unchanged,
  and ordinary nonempty text is unchanged. A stored value that is
  itself `/` prints the same way; there is no extra encoding.
  Verbatim `| ` text blocks are not slash-substituted.
- A wait that refuses an oversize present file (`E-TOO-LARGE`) uses
  the same check-shaped form as `check`: a `file` row, then the
  diagnostic, and no `type` row. That applies both before the ready
  line and if the file later grows past 16 MiB after readiness.
  Wait does not invent a type from an earlier snapshot.
- Removing a nick with `--status` when that nick has no entry under
  the selected status still refuses the whole removal
  (`E-STATE-NOTFOUND`) and writes nothing. The reason names the
  supplied status. The `fix` is a read-only `check-state` of the
  directory and keeps that selection; it does not drop `--status` or
  delete another declaration. Copied inspection leaves live lock
  bytes unchanged.
- Unmatched `--include` globs on `list`, `read-notification`, and
  `check-state` hint rows are whitespace-folded and
  control-sanitized for display only. Matching still uses the glob
  as typed; a raw double-space pattern stays distinct from a
  single-space sibling.
- Shared parsing no longer repeats prefix searches. Ordered
  identities and diagnostics are unchanged. Observed Linux
  profile-update stdout and filtered wait cancellation on the
  generated inputs used for this change stayed the same; this is
  not a universal timing claim.
- State declaration and type-mismatch recovery print a
  directory-level `state` command only when the inspected file is
  that directory's selected `state.agent-besties.md`. A misnamed
  or otherwise unselected state-content file is repaired by hand
  on the inspected file; those recipes do not mutate a sibling
  reserved file. Rename advice inspects both files and never
  overwrites a separate existing state file. Duplicate,
  inverted-range, expired, same-file conflict, and type-mismatch
  cases on an unselected file share that gate. The selected
  reserved file still uses the existing recipes.
- Profile heading repair offers a copyable `## <name>` example
  only when that name is unused among existing headings and other
  heading repairs in the same report. Occupied earlier, later, or
  simultaneous colliding names stay manual distinct-name guidance.
  A unique Unicode example remains copyable.
- `check` of a post lists `authors` in first-occurrence order. A
  `post` write lists mentions and reply targets in body order,
  with missing `@#n` still written as `(not found)`. `--keyword`
  remains a case-insensitive substring used as typed, without
  whitespace folding. `check-state` and conflict notifications
  keep native file order; a conflict names the deepest overlapping
  own lock, last at equal depth. Required per-file conflict rows
  stay. These are identity and report contracts, not a universal
  timing claim.
- A generated same-file lock-conflict `fix` includes
  `--status=locked`, so copying it removes only that lock.
  An explicit `--remove` without `--status` still spans
  every status that nick holds.
- Wait-state recovery recipes are gated once at the final
  report against the currently selected reserved file, not
  retained from the ready line. A `removed` warning from the
  deleted baseline can remain; it does not print a mutating
  command against a later selected file.
- File-symlink `profile`/`brief` writes honor lock chains on
  the supplied parent and the canonical referent parent.
  Own `--as`, expiry, and the incomplete-scan fallback are
  unchanged. Failed canonicalization keeps the supplied
  chain. This is not a volume-wide hardlink search, and
  concurrent replacements are not atomic.
- Copied multi-header post repairs in one report reserve
  distinct unused sequence numbers. Later exhaustion stays
  a manual heading repair. Bodies and already-legal comments
  are unchanged.
- State declaration merging and status-section cleanup keep
  original reports and file bytes. These are identity and byte
  contracts, not a universal timing claim.
- Recovery commands no longer lose native operand identity. Diagnostic
  operands and the original argv stay native until emission: on Unix,
  bytes that are not valid UTF-8 are emitted as ASCII-octal `printf`
  command substitutions inside the same word, so a generated `fix` can
  no longer address a U+FFFD replacement sibling, while ordinary UTF-8
  operands keep the existing readable quoting. A recipe whose operand
  or original argv word contains NUL, CR, or LF, or whose native data
  cannot be represented in the documented POSIX UTF-8 syntax (such as
  unpaired UTF-16 surrogates on Windows), is printed as whole-recipe
  manual recovery instead of a lossy executable lookalike.
  Human-readable display remains separately lossy, and
  cmd.exe/PowerShell remain out of scope.

## [0.1.0] - 2026-09-22

Initial release.

### Added

- Eleven base commands over four local Markdown collaboration file types
  (`.agent-besties.md`): `list`, `check`, `profile`, `post`, `subscribe`,
  `brief`, `state`, `read-notification`, `lookfor-comment`, `read-brief`,
  `check-state`.
- Raw-byte parsing with splice-based writes: unknown hand-written content,
  original line endings and untouched bytes are preserved; only the ranges a
  command targets change.
- Cooperative file locking between `agent-coexist` processes (shared reads,
  exclusive writes, bounded waits with fast-fail/queue budgets), directory
  walking with gitignore-family rules, brief-target SHA-256 verification
  with fresh/shifted/stale/unchecked states, and directory-state conflict
  detection with functional locks for `profile`/`brief` writes.
- Three optional wait commands (`wait`, `wait-post`, `wait-state`) that poll
  a single collaboration file until its first change: without a projection
  filter any change ends the wait (including byte-only format changes),
  while `wait-post --as <nick>` / `wait-state --status <name>` end it only
  on changes qualifying for that nick or status; each result reports the
  change with `bytes` and `semantic` rows. Ctrl-C cancels with exit 130.
- Structured first-line/status output with two-space column rows,
  diagnostics with `why`/`fix`, stderr-only progress, and exit codes
  0/1/2/3 (+130 for cancelled waits).
