# Security Policy

agent-coexist is a local command-line tool that helps cooperating processes
share plain Markdown files. This page states what it protects, what it
deliberately does not, and how to report a problem.

## Trust model

**Plain text, no encryption.** All collaboration data lives in ordinary
`.agent-besties.md` text files. Anyone who can read the directory can read
them; anyone who can write can change them. Keep credentials and secrets
out of shared or committed files. Briefs expose target paths and hashes,
and `read-brief` prints target contents. Review files before sharing them.

**Nicks are labels, not identities.** `--as <nick>` is an unverified claim.
Any process can post under any nick; the tool performs no authentication,
and its comment-edit check compares nick strings only.

**`locked` is a request.** A `locked` state declaration asks others not to
change a directory. The only thing the CLI enforces is that other nicks'
`profile` and `brief` writes at or below that directory fail with
`E-FLOCKED` until the declaration expires. For a write through a file
symlink, both the supplied path's directory chain and the canonical
target's parent chain are checked; there is no volume-wide hardlink
search. `post`, `subscribe`, `state`, hand edits, and every other program
are not blocked, and anyone can claim the holder's nick with `--as`.

**File locks coordinate the CLI, not other programs.** Short-lived shared
and exclusive OS file locks keep concurrent `agent-coexist` processes from
interleaving reads and writes. On Unix local filesystems they are generally
advisory. On Windows, `LockFileEx` byte-range locks can also reject other
handles' ordinary I/O while held. Mapped I/O, rename-and-replace saves, and
programs that ignore these locks are not excluded, and locking on network
filesystems is not guaranteed.

**No transport, no command execution.** The CLI does not sync files between
machines or execute message content or `fix` recipes. It reads referenced
files and writes collaboration files with the permissions of the process
running it: no elevation, no sandbox.

### Working as designed (not vulnerabilities)

- Another program edited a file without taking a lock.
- Someone posted or edited comments under your nick.
- A `locked` declaration did not stop a `post`, `subscribe`, or `state` write, or a hand edit.
- Scans skip gitignored paths or follow the traversal rules in [the command reference](docs/usage.md#scans-what-is-visited-and-filter-precedence); those rules decide what is listed, not who may read it.

If behavior goes beyond these boundaries, for example a write that changes
bytes outside the span a command targets or touches unrelated files, report
it.

## Reporting

No private reporting channel is currently configured for this repository.
Until one is available:

1. Do not post vulnerability details, proof-of-concept exploits, or
   sensitive files in a public issue.
2. [Open an issue](https://github.com/RatmmmhSquishyRat/agent-coexist/issues/new)
   stating only that you have a security report and asking the maintainer
   to provide a private channel. Do not include technical details.
3. Share details only after a private channel is provided.

For ordinary usage questions, start with `agent-coexist guide` and
[the command reference](docs/usage.md).
