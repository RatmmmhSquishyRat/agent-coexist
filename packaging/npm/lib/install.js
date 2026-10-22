'use strict';

// Resolve, fetch and verify the released agent-coexist executable for the
// running platform. The npm package carries no binary: it downloads the
// archive the Rust release workflow published for its own version, checks it
// against that release's SHA256SUMS, and keeps the executable under vendor/.
//
// Nothing here runs at install time. The bin shim calls ensureBinary() on
// first use, so the package works under `npx`, under `--ignore-scripts`, and
// in sandboxes that forbid install hooks.

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const crypto = require('node:crypto');

const { extract, LauncherError } = require('./archive.js');

const { version } = require('../package.json');
const REPO = 'RatmmmhSquishyRat/agent-coexist';
const BASE = `https://github.com/${REPO}/releases/download/v${version}`;

/// Release archive naming, keyed by `${process.platform}-${process.arch}`.
/// Linux uses the static musl builds, which run on glibc systems too.
const TARGETS = {
  'darwin-arm64': 'agent-coexist-aarch64-apple-darwin.tar.gz',
  'darwin-x64': 'agent-coexist-x86_64-apple-darwin.tar.gz',
  'linux-arm64': 'agent-coexist-aarch64-unknown-linux-musl.tar.gz',
  'linux-x64': 'agent-coexist-x86_64-unknown-linux-musl.tar.gz',
  'win32-arm64': 'agent-coexist-aarch64-pc-windows-msvc.zip',
  'win32-x64': 'agent-coexist-x86_64-pc-windows-msvc.zip',
};

const EXE = process.platform === 'win32' ? 'agent-coexist.exe' : 'agent-coexist';
const VENDOR = path.join(__dirname, '..', 'vendor');

/// Where the executable for this platform is kept once fetched.
function binaryPath() {
  return path.join(VENDOR, EXE);
}

/// The release archive for the running platform.
function archiveName() {
  const key = `${process.platform}-${process.arch}`;
  const name = TARGETS[key];
  if (name === undefined) {
    const supported = Object.keys(TARGETS).sort().join(', ');
    throw new LauncherError(
      `agent-coexist has no published binary for ${key}.\n` +
        `Published targets: ${supported}.\n` +
        `Build it from source instead: cargo install agent-coexist --locked`,
    );
  }
  return name;
}

async function get(url) {
  const response = await fetch(url, { redirect: 'follow' });
  if (!response.ok) {
    throw new LauncherError(`GET ${url} failed: ${response.status} ${response.statusText}`);
  }
  return Buffer.from(await response.arrayBuffer());
}

/// The expected digest of `name` from this release's SHA256SUMS.
function digestFor(sums, name) {
  for (const line of sums.toString('utf8').split('\n')) {
    const [digest, entry] = line.trim().split(/ +\*?/);
    if (entry === name) return digest;
  }
  throw new LauncherError(`SHA256SUMS for v${version} has no entry for ${name}`);
}

/// Download the archive, verify it against the release's SHA256SUMS, and
/// write the executable to vendor/. Returns its path.
async function download() {
  const name = archiveName();
  process.stderr.write(`agent-coexist: fetching ${name} (v${version})\n`);

  const [archive, sums] = await Promise.all([
    get(`${BASE}/${name}`),
    get(`${BASE}/SHA256SUMS`),
  ]);

  const expected = digestFor(sums, name);
  const actual = crypto.createHash('sha256').update(archive).digest('hex');
  if (actual !== expected) {
    throw new LauncherError(
      `checksum mismatch for ${name}\n  expected ${expected}\n  actual   ${actual}\n` +
        `The download does not match the release's SHA256SUMS. Nothing was installed.`,
    );
  }

  const bytes = extract(archive, name, EXE);
  fs.mkdirSync(VENDOR, { recursive: true });

  // Write to a unique temporary name and rename into place, so concurrent
  // first runs cannot observe a partially written executable.
  const staged = path.join(VENDOR, `.${EXE}.${process.pid}`);
  fs.writeFileSync(staged, bytes, { mode: 0o755 });
  fs.renameSync(staged, binaryPath());

  return binaryPath();
}

/// The executable's path, downloading it if this is the first run.
async function ensureBinary() {
  const bin = binaryPath();
  if (fs.existsSync(bin)) return bin;
  return download();
}

module.exports = {
  ensureBinary, binaryPath, archiveName, digestFor,
  version, TARGETS, EXE, LauncherError,
};
