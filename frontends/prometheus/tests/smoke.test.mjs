import assert from 'node:assert/strict';
import http from 'node:http';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

const pkgRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.join(pkgRoot, '..', '..');
const require = createRequire(import.meta.url);

function ensureNative() {
  const nodePath = path.join(pkgRoot, 'prometheus.node');
  if (!fs.existsSync(nodePath)) {
    const built = spawnSync('pnpm', ['napi:build'], {
      cwd: repoRoot,
      stdio: 'inherit',
    });
    assert.equal(built.status, 0, 'napi:build failed');
  }
  assert.ok(fs.existsSync(nodePath), 'prometheus.node missing');
}

function startServer(body) {
  return new Promise((resolve) => {
    const server = http.createServer((req, res) => {
      res.writeHead(200, {
        'Content-Type': 'application/octet-stream',
        'Content-Length': body.length,
        'Content-Disposition': 'attachment; filename="smoke.bin"',
      });
      if (req.method === 'HEAD') {
        res.end();
        return;
      }
      res.end(body);
    });
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      resolve({
        server,
        url: `http://127.0.0.1:${port}/smoke.bin`,
      });
    });
  });
}

test('version matches package.json', async () => {
  ensureNative();
  const { version } = await import('../dist/index.js');
  const pkg = require('../package.json');
  assert.equal(version(), pkg.version);
});

test('info and download over local http', async () => {
  ensureNative();
  const body = Buffer.from('smoke-body');
  const { server, url } = await startServer(body);
  try {
    const api = await import('../dist/index.js');
    const media = api.info(url);
    assert.equal(media.extractor, 'generic-http');
    assert.equal(media.filename, 'smoke.bin');

    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-smoke-'));
    const result = api.download(url, dir);
    assert.equal(fs.readFileSync(result.path).toString('utf8'), 'smoke-body');
    assert.equal(result.bytesWritten, body.length);
    fs.rmSync(dir, { recursive: true, force: true });
  } finally {
    server.close();
  }
});

test('cli --version', () => {
  ensureNative();
  const bin = path.join(pkgRoot, 'bin', 'prometheus.js');
  const out = spawnSync(process.execPath, [bin, '--version'], {
    encoding: 'utf8',
  });
  assert.equal(out.status, 0, out.stderr);
  assert.match(out.stdout.trim(), /^0\.0\.1$/);
});
