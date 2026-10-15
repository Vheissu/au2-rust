// Compiles crates/wasm to WebAssembly and generates the JS glue and
// TypeScript declarations in web/wasm/.
//
//   node scripts/build-wasm.mjs          optimised release build
//   node scripts/build-wasm.mjs --dev    faster debug build
//
// Two steps, no wasm-pack:
//   1. cargo builds a .wasm for the wasm32-unknown-unknown target.
//   2. wasm-bindgen reads the metadata the #[wasm_bindgen] macros embedded
//      in that file and writes an ES module that loads it.
//
// The CLI in step 2 must be the exact version of the wasm-bindgen crate in
// Cargo.lock, because the embedded metadata format is not stable across
// versions. This script checks that before building.

import { spawn, execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../..', import.meta.url));
const outDir = fileURLToPath(new URL('../wasm', import.meta.url));
const crate = 'au2-rust-wasm';

/** @param {{ dev?: boolean }} [options] */
export async function buildWasm({ dev = false } = {}) {
  checkBindgenVersion();

  const profile = dev ? 'debug' : 'release';
  await run('cargo', [
    'build',
    '--package', crate,
    '--target', 'wasm32-unknown-unknown',
    ...(dev ? [] : ['--release']),
  ]);

  const wasm = join(root, 'target', 'wasm32-unknown-unknown', profile, `${crate.replaceAll('-', '_')}.wasm`);
  await run('wasm-bindgen', [
    wasm,
    '--target', 'web',
    '--out-dir', outDir,
    '--out-name', 'core',
    ...(dev ? ['--debug', '--keep-debug'] : []),
  ]);
}

function checkBindgenVersion() {
  const lock = readFileSync(join(root, 'Cargo.lock'), 'utf8');
  const expected = lock.match(/name = "wasm-bindgen"\nversion = "([^"]+)"/)?.[1];
  if (!expected) {
    throw new Error('wasm-bindgen is missing from Cargo.lock. Run `cargo fetch` first.');
  }

  let installed;
  try {
    installed = execFileSync('wasm-bindgen', ['--version'], { encoding: 'utf8' }).trim().split(' ')[1];
  } catch {
    installed = undefined;
  }

  if (installed !== expected) {
    const found = installed ? `found ${installed}` : 'it is not installed';
    throw new Error(
      `wasm-bindgen CLI ${expected} is required (${found}). Install it with:\n\n` +
      `  cargo install wasm-bindgen-cli --version ${expected} --locked\n`,
    );
  }
}

/**
 * @param {string} command
 * @param {string[]} args
 * @returns {Promise<void>}
 */
function run(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: root, stdio: 'inherit' });
    child.on('error', reject);
    child.on('exit', code =>
      code === 0 ? resolve() : reject(new Error(`${command} exited with code ${code}`)),
    );
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  buildWasm({ dev: process.argv.includes('--dev') }).catch(error => {
    console.error(`\n${error.message}`);
    process.exitCode = 1;
  });
}
