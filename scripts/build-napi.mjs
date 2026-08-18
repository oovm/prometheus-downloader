/**
 * Build `prometheus-napi` and copy the `.node` into `frontends/prometheus/`.
 *
 * Usage: node scripts/build-napi.mjs [--release]
 */
import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const release = process.argv.includes('--release');
const profile = release ? 'release' : 'debug';

const cargoArgs = ['build', '--manifest-path', path.join(root, 'Cargo.toml'), '-p', 'prometheus-napi'];
if (release) cargoArgs.push('--release');

console.log(`cargo ${cargoArgs.join(' ')}`);
const build = spawnSync('cargo', cargoArgs, { cwd: root, stdio: 'inherit' });
if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

const targetDir = path.join(root, 'target', profile);
const stem = 'prometheus_napi';
/** @type {string[]} */
const candidates = [];
if (process.platform === 'win32') {
  candidates.push(`${stem}.dll`, `${stem}.node`);
} else if (process.platform === 'darwin') {
  candidates.push(`lib${stem}.dylib`, `${stem}.dylib`, `lib${stem}.so`, `${stem}.so`, `${stem}.node`);
} else {
  candidates.push(`lib${stem}.so`, `${stem}.so`, `${stem}.node`);
}

let artifact = null;
for (const name of candidates) {
  const p = path.join(targetDir, name);
  if (existsSync(p)) {
    artifact = p;
    break;
  }
}

if (!artifact) {
  const deps = path.join(targetDir, 'deps');
  if (existsSync(deps)) {
    for (const name of readdirSync(deps)) {
      const base = name.replace(/^lib/, '');
      if (
        (name === stem ||
          name.startsWith(`${stem}.`) ||
          base.startsWith(`${stem}.`) ||
          name.startsWith(`lib${stem}.`)) &&
        (name.endsWith('.dll') || name.endsWith('.so') || name.endsWith('.dylib') || name.endsWith('.node'))
      ) {
        artifact = path.join(deps, name);
        break;
      }
    }
  }
}

if (!artifact) {
  console.error(`Could not find ${stem} under ${targetDir}`);
  process.exit(1);
}

const outDir = path.join(root, 'frontends', 'prometheus');
mkdirSync(outDir, { recursive: true });
const outFile = path.join(outDir, 'prometheus.node');
copyFileSync(artifact, outFile);
console.log(`wrote ${outFile}`);
