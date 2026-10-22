#!/usr/bin/env node
'use strict';

// Tests for the npm wrapper. The offline group checks target resolution,
// SHA256SUMS parsing and archive extraction against fixtures built here. The
// online group, skipped with AGENT_COEXIST_SKIP_NETWORK=1, performs the real
// download the wrapper would do and runs the resulting executable.

const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const zlib = require('node:zlib');
const { execFileSync, spawnSync } = require('node:child_process');

const { extract, fromTarGz, fromZip } = require('../lib/archive.js');
const install = require('../lib/install.js');
const pkg = require('../package.json');

let passed = 0;
let skipped = 0;

function test(name, body) {
  body();
  passed++;
  process.stdout.write(`ok   ${name}\n`);
}

function skip(name, why) {
  skipped++;
  process.stdout.write(`skip ${name} (${why})\n`);
}

// ---------------------------------------------------------------- fixtures

/// A single-entry tar.gz built here, so extraction is tested without network.
function makeTarGz(name, body) {
  const header = Buffer.alloc(512);
  header.write(name, 0, 'latin1');
  header.write('000644 \0', 100, 'latin1');
  header.write('0000000 \0', 108, 'latin1');
  header.write('0000000 \0', 116, 'latin1');
  header.write(body.length.toString(8).padStart(11, '0') + ' ', 124, 'latin1');
  header.write('00000000000 ', 136, 'latin1');
  header.write('        ', 148, 'latin1'); // checksum field blank while summing
  header.write('0', 156, 'latin1');
  header.write('ustar\0' + '00', 257, 'latin1');
  let sum = 0;
  for (const b of header) sum += b;
  header.write(sum.toString(8).padStart(6, '0') + '\0 ', 148, 'latin1');

  const padded = Buffer.alloc(Math.ceil(body.length / 512) * 512);
  body.copy(padded);
  return zlib.gzipSync(Buffer.concat([header, padded, Buffer.alloc(1024)]));
}

/// A single-entry zip built here, deflated like the release workflow's.
function makeZip(name, body) {
  const nameBuf = Buffer.from(name, 'utf8');
  const deflated = zlib.deflateRawSync(body);

  const local = Buffer.alloc(30);
  local.writeUInt32LE(0x04034b50, 0);
  local.writeUInt16LE(20, 4);
  local.writeUInt16LE(8, 8); // deflate
  local.writeUInt32LE(deflated.length, 18);
  local.writeUInt32LE(body.length, 22);
  local.writeUInt16LE(nameBuf.length, 26);

  const central = Buffer.alloc(46);
  central.writeUInt32LE(0x02014b50, 0);
  central.writeUInt16LE(20, 6);
  central.writeUInt16LE(8, 10);
  central.writeUInt32LE(deflated.length, 20);
  central.writeUInt32LE(body.length, 24);
  central.writeUInt16LE(nameBuf.length, 28);
  central.writeUInt32LE(0, 42); // local header offset

  const eocd = Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50, 0);
  eocd.writeUInt16LE(1, 8);
  eocd.writeUInt16LE(1, 10);
  eocd.writeUInt32LE(central.length + nameBuf.length, 12);
  eocd.writeUInt32LE(local.length + nameBuf.length + deflated.length, 16);

  return Buffer.concat([local, nameBuf, deflated, central, nameBuf, eocd]);
}

// ------------------------------------------------------------------ offline

test('package version matches the crate version', () => {
  const manifest = fs.readFileSync(
    path.join(__dirname, '..', '..', '..', 'Cargo.toml'),
    'utf8',
  );
  const crateVersion = /^version = "([^"]+)"/m.exec(manifest)[1];
  assert.equal(
    pkg.version,
    crateVersion,
    'the wrapper downloads its own version of the release, so the two must agree',
  );
});

test('every published release target is mapped', () => {
  assert.deepEqual(Object.keys(install.TARGETS).sort(), [
    'darwin-arm64',
    'darwin-x64',
    'linux-arm64',
    'linux-x64',
    'win32-arm64',
    'win32-x64',
  ]);
  for (const archive of Object.values(install.TARGETS)) {
    assert.match(archive, /^agent-coexist-.+\.(tar\.gz|zip)$/);
  }
});

test('declared os and cpu cover exactly the mapped targets', () => {
  const platforms = new Set();
  const arches = new Set();
  for (const key of Object.keys(install.TARGETS)) {
    const [platform, arch] = key.split('-');
    platforms.add(platform);
    arches.add(arch);
  }
  assert.deepEqual([...platforms].sort(), [...pkg.os].sort());
  assert.deepEqual([...arches].sort(), [...pkg.cpu].sort());
});

test('this platform resolves to a release archive', () => {
  assert.match(install.archiveName(), /^agent-coexist-/);
});

test('SHA256SUMS parsing accepts the published binary-mode form', () => {
  const sums = Buffer.from(
    'aaaa  agent-coexist-x86_64-apple-darwin.tar.gz\n' +
      'bbbb *agent-coexist-x86_64-pc-windows-msvc.zip\n',
  );
  assert.equal(
    install.digestFor(sums, 'agent-coexist-x86_64-apple-darwin.tar.gz'),
    'aaaa',
  );
  assert.equal(
    install.digestFor(sums, 'agent-coexist-x86_64-pc-windows-msvc.zip'),
    'bbbb',
  );
  assert.throws(() => install.digestFor(sums, 'absent.tar.gz'), /no entry for absent/);
});

test('tar.gz extraction returns the named entry byte for byte', () => {
  const body = crypto.randomBytes(5000);
  const archive = makeTarGz('agent-coexist', body);
  assert.deepEqual(extract(archive, 'x.tar.gz', 'agent-coexist'), body);
  assert.equal(fromTarGz(archive, 'other'), null);
});

test('zip extraction returns the named entry byte for byte', () => {
  const body = crypto.randomBytes(5000);
  const archive = makeZip('agent-coexist.exe', body);
  assert.deepEqual(extract(archive, 'x.zip', 'agent-coexist.exe'), body);
  assert.equal(fromZip(archive, 'other'), null);
});

test('a missing entry is reported against the archive name', () => {
  const archive = makeTarGz('agent-coexist', Buffer.from('x'));
  assert.throws(
    () => extract(archive, 'the-archive.tar.gz', 'absent'),
    /the-archive\.tar\.gz does not contain 'absent'/,
  );
});

test('no wrapper source carries CRLF, which would break the shebang on Unix', () => {
  // `npx` runs bin/agent-coexist.js through its shebang on Linux and macOS.
  // With CRLF the kernel looks for an interpreter named "node\r" and fails,
  // and Windows never notices because npm generates .cmd/.ps1 shims that
  // invoke node directly. Guard the whole wrapper tree, not just the shebang.
  // Repository root: the launcher's own tree plus the root manifest that
  // the `npx github:` path reads.
  const root = path.join(__dirname, '..', '..', '..');
  const pruned = new Set(['vendor', 'node_modules', 'dist', '__pycache__']);
  const suffixes = ['.js', '.py', '.json', '.toml', '.md'];

  const offenders = [];
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (dir === root && !['packaging', 'package.json'].includes(entry.name)) continue;
      if (pruned.has(entry.name)) continue;
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(full);
        continue;
      }
      if (!suffixes.some((s) => entry.name.endsWith(s))) continue;
      if (fs.readFileSync(full).includes('\r\n')) {
        offenders.push(path.relative(root, full).split(path.sep).join('/'));
      }
    }
  };
  walk(root);
  assert.deepEqual(offenders, [], `CRLF found in: ${offenders.join(', ')}`);

  const bin = fs.readFileSync(path.join(__dirname, '..', 'bin', 'agent-coexist.js'));
  assert.equal(bin.subarray(0, 20).toString(), '#!/usr/bin/env node\n');
});

test('an unmapped platform names the supported set and the source build', () => {
  const platform = process.platform;
  Object.defineProperty(process, 'platform', { value: 'sunos', configurable: true });
  try {
    assert.throws(
      () => install.archiveName(),
      /no published binary for sunos-.*cargo install agent-coexist --locked/s,
    );
  } finally {
    Object.defineProperty(process, 'platform', { value: platform, configurable: true });
  }
});

// ------------------------------------------------------------------- online

const shim = path.join(__dirname, '..', 'bin', 'agent-coexist.js');

if (process.env.AGENT_COEXIST_SKIP_NETWORK === '1') {
  skip('real release download, verification and execution', 'AGENT_COEXIST_SKIP_NETWORK=1');
} else {
  test('the real release downloads, verifies and runs', () => {
    const vendored = install.binaryPath();
    fs.rmSync(vendored, { force: true });

    execFileSync(process.execPath, [shim, '--version'], { stdio: 'pipe' });
    assert.ok(fs.existsSync(vendored), 'the first run must leave the executable in vendor/');

    const reported = execFileSync(vendored, ['--version'], { encoding: 'utf8' }).trim();
    assert.equal(
      reported,
      `agent-coexist ${pkg.version}`,
      'the downloaded executable must be the version the wrapper declares',
    );
  });

  test('a second run reuses the vendored executable without downloading', () => {
    const run = spawnSync(process.execPath, [shim, '--version'], { stdio: 'pipe' });
    assert.equal(run.status, 0);
    assert.equal(
      run.stderr.toString(),
      '',
      'a cached run must print no fetch notice',
    );
    assert.equal(run.stdout.toString().trim(), `agent-coexist ${pkg.version}`);
  });

  test('exit codes and streams pass through unchanged', () => {
    const usage = spawnSync(process.execPath, [shim, 'not-a-command'], { stdio: 'pipe' });
    assert.equal(usage.status, 2, 'a usage error must stay exit 2 through the wrapper');
    assert.equal(usage.stdout.toString(), '', 'a usage error writes nothing to stdout');
    assert.match(usage.stderr.toString(), /error:/);

    const guide = spawnSync(process.execPath, [shim, 'guide'], { stdio: 'pipe' });
    assert.equal(guide.status, 0);
    assert.match(guide.stdout.toString(), /agent-coexist/);
  });

  test('arguments reach the executable and produce a real record', () => {
    const work = fs.mkdtempSync(path.join(require('node:os').tmpdir(), 'coexist-npm-'));
    const run = spawnSync(
      process.execPath,
      [shim, 'profile', 'alice', '--section', 'role', '-m', 'Wrapper check.'],
      { cwd: work, stdio: 'pipe', encoding: 'utf8' },
    );
    assert.equal(run.status, 0, run.stderr);
    assert.match(run.stdout, /^ok {2}profile {2}alice/);

    const written = path.join(work, 'alice.agent-coexist.md');
    assert.ok(fs.existsSync(written), 'the record must exist on disk');
    assert.match(fs.readFileSync(written, 'utf8'), /# alice\.agent-profile/);
    fs.rmSync(work, { recursive: true, force: true });
  });
}

process.stdout.write(`\n${passed} passed, ${skipped} skipped\n`);
