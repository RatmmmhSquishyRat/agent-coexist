# Security Policy

agent-coexist is a local command-line tool that helps cooperating processes
share plain Markdown files. It assumes the agents sharing a directory are
cooperating; it does not defend against one that is not. This page states
what the tool does guarantee, what it does not, and how to report a problem.

## What the CLI does

- **Writes stay inside their target.** A command changes only the byte
  spans it targets; everything else in the file survives verbatim. The
  [file format reference](docs/file-format.md#editing-files-by-hand)
  lists each command's spans. A `state` write targets the clean entry
  lines of the sections it touches, not only the named nick's entry: a
  declare merges that status's clean lines into one line, and every
  successful declare or removal also drops expired entries, including
  other nicks', from clean lines in any section
  ([What state writes change](docs/file-format.md#what-state-writes-change)).
  A declare can also append a new status section, and a removal deletes a
  section it emptied of the named nick when only the heading and blank
  lines remain. In a state file, dirty lines, `> ` comments, fenced text,
  and other prose stay byte-for-byte. Beyond that, a command may create a new file
  (and remove it again, with any empty directories it created, if the
  write is refused) or delete a state file reduced to its heading, reader
  comments, and blank lines.
- **A failed write says what it left.** Writes happen in place, growth
  first, so running out of space while a file grows leaves its previous
  bytes untouched. If a write fails in the running process, the CLI tries
  to put the previous content back. When that fails too, the report says
  the file may hold a mix of old and new bytes; when only the final sync
  fails, it says the change is in the file but may not be stored on disk
  yet. Neither case offers a retry
  ([When a write fails partway](docs/usage.md#when-a-write-fails-partway)).
  This is not atomic and not crash-safe: a process that is killed, or a
  machine that loses power, in the middle of a write can leave the file
  mixed, and the restore itself is not synced.
- **CLI processes do not interleave.** Short-lived shared and exclusive OS
  file locks keep concurrent `agent-coexist` processes from interleaving
  reads and writes of the same file.
- **Nothing is executed or sent.** The CLI does not execute message
  content or `fix` recipes, does not sync files between machines, and
  opens no network connections of its own.
- **Your permissions, no more.** It reads referenced files and writes
  collaboration files with the permissions of the process running it. It
  neither elevates nor sandboxes.

## What it does not protect

**Plain text, no encryption.** All collaboration data lives in ordinary
`.agent-coexist.md` text files. Anyone who can read the directory can read
them; anyone who can write can change them. Keep credentials and secrets
out of shared or committed files. Briefs expose target paths and hashes,
and `read-brief` prints target contents. Review files before sharing them.

**Nicks are labels, not identities.** `--as <nick>` is an unverified claim.
Any process can post under any nick; the tool performs no authentication,
and its comment-edit check compares nick strings only.

**`locked` is a request.** A `locked` state declaration asks others not to
change a directory. The only thing the CLI enforces is that `profile` and
`brief` writes at or below that directory fail with `E-FLOCKED` until the
declaration expires, unless `--as` names the holder; without `--as`, even
the holder's own writes fail. For a write through a file
symlink, both the supplied path's directory chain and the canonical
target's parent chain are checked; there is no volume-wide hardlink
search. `post`, `subscribe`, `state`, hand edits, and every other program
are not blocked, and anyone can claim the holder's nick with `--as`.

**File locks bind the CLI, not other programs.** On Unix local
filesystems the locks are generally advisory. On Windows, `LockFileEx`
byte-range locks can also reject other handles' ordinary I/O while held.
Mapped I/O, rename-and-replace saves, and programs that ignore these locks
are not excluded, and locking on network filesystems is not guaranteed.

### Working as designed (not vulnerabilities)

- Another program edited a file without taking a lock.
- Someone posted or edited comments under your nick.
- A `locked` declaration did not stop a `post`, `subscribe`, or `state` write, or a hand edit.
- A `state` write under one nick dropped another nick's expired entry from a clean line.
- Scans skip gitignored paths or follow the traversal rules in [the command reference](docs/usage.md#scans-what-is-visited-and-filter-precedence); those rules decide what is listed, not who may read it.

Report behavior that breaks a guarantee in [What the CLI does](#what-the-cli-does),
for example a write that changes bytes outside the spans documented for
that command (such as a dirty state line, a `> ` comment, or a comment
other than the one being edited) or touches unrelated files.

## Reporting

To privately report a security vulnerability or sensitive issue:

1. Use GitHub's [Private Vulnerability Reporting](https://github.com/RatmmmhSquishyRat/agent-coexist/security/advisories/new) to submit a confidential security advisory report directly to the maintainers.
2. Do not post vulnerability details, proof-of-concept exploits, or sensitive files in public issues or pull requests.
3. Maintainers will review and respond to security advisories promptly.

For ordinary usage questions or non-security bug reports, start with `agent-coexist guide`,
[the command reference](docs/usage.md), or [open an issue](https://github.com/RatmmmhSquishyRat/agent-coexist/issues/new).
