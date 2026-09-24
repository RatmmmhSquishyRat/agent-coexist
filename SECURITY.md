# Security Policy

agent-coexist is a local command-line tool that helps cooperating processes
share plain Markdown files. This page states plainly what it protects, what
it deliberately does not, and how to report a problem.

## Trust model

**Plain text, no encryption.** All collaboration data lives in ordinary
`.agent-besties.md` text files. Anyone who can read the directory can read
them; anyone who can write can change them. Keep credentials and secrets
out of shared or committed files. Briefs expose target paths and hashes,
and `read-brief` can print the target contents. Review artifacts before sharing them.

**Nicks are labels, not identities.** `--as <nick>` is an unverified claim.
Any process can post under any nick; the tool performs no authentication,
and its comment-edit check compares nick strings only.

**Locks are cooperative, not security.** File locking (shared reads,
exclusive writes) coordinates `agent-coexist` processes to avoid collisions.
It does not stop other editors or non-cooperating programs. A `locked`
state declaration gates only `profile` and `brief` writes by other
`agent-coexist` processes — never `post`, `subscribe`, or `state` writes —
and hand-editing a file bypasses it entirely. Nothing here is a sandbox or
an access-control mechanism.

**No built-in transport or command execution.** The CLI does not sync files
between machines or execute messages. It reads referenced files and writes
collaboration files with the permissions of the process running it.

**Your permissions are its permissions.** It reads and writes with your
user's filesystem permissions — no elevation, no isolation. Directory
walking honors gitignore-family rules and skips symlinked directories;
that is traversal behavior, not a security boundary.

### Working as designed (not vulnerabilities)

- Another program edited a file without taking a lock.
- Someone posted or edited comments under your nick.
- A `locked` declaration did not stop a `state` write or a hand edit.

These are documented properties of local plaintext cooperation, not bugs
(see [the file guide](docs/file-format.md)). If behavior exceeds these
boundaries or damages unrelated files, report it.

## Reporting

No private reporting channel is currently configured for this repository.
Until one is available:

1. Do not post vulnerability details, proof-of-concept exploits, or
   sensitive files in a public issue.
2. [Open an issue](https://github.com/RatmmmhSquishyRat/agent-coexist/issues/new)
   stating only that you have a security report and asking the maintainer
   to provide a private channel. Do not include technical details.
3. Share details only after a private channel is provided.

For ordinary usage questions, start with [the command guide](docs/usage.md).
