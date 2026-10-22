#!/usr/bin/env node
'use strict';

// Thin launcher: make sure the released executable is present, then hand the
// whole invocation to it. Arguments, stdio and the exit code pass straight
// through, because agent-coexist's exit codes carry meaning (0 ok or warn,
// 1 error, 2 usage, 3 lock refusal, 130 cancelled) and its reports are read
// from stdout and stderr.

const { spawnSync } = require('node:child_process');
const { ensureBinary, LauncherError } = require('../lib/install.js');

async function main() {
  const bin = await ensureBinary();
  const run = spawnSync(bin, process.argv.slice(2), { stdio: 'inherit' });

  // Ctrl-C reaches the child directly through the inherited console; the
  // child reports it as exit 130. A signal seen here instead means the
  // child was killed without reporting, so report it the way a shell does.
  if (run.signal !== null) {
    process.exit(128 + (require('node:os').constants.signals[run.signal] ?? 0));
  }
  process.exit(run.status);
}

main().catch((error) => {
  process.stderr.write(`agent-coexist: ${error.message}\n`);
  process.exit(1);
});
