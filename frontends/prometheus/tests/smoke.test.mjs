import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';

const pkgRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.join(pkgRoot, '..', '..');
const require = createRequire(import.meta.url);

function currentPlatformShort() {
    const { platform, arch } = process;
    if (platform === 'win32' && arch === 'x64') return 'win32-x64';
    if (platform === 'win32' && arch === 'arm64') return 'win32-arm64';
    if (platform === 'darwin' && arch === 'arm64') return 'darwin-arm64';
    if (platform === 'darwin' && arch === 'x64') return 'darwin-x64';
    if (platform === 'linux' && arch === 'x64') return 'linux-x64';
    if (platform === 'linux' && arch === 'arm64') return 'linux-arm64';
    return `${platform}-${arch}`;
}

function ensureNative() {
    const short = currentPlatformShort();
    const nativeDir = path.join(repoRoot, 'frontends', `prometheus-${short}`);
    const hasNode = fs.existsSync(nativeDir) && fs.readdirSync(nativeDir).some((f) => f.endsWith('.node'));
    if (!hasNode) {
        const built = spawnSync('pnpm', ['napi:build'], {
            cwd: repoRoot,
            stdio: 'inherit',
            shell: process.platform === 'win32',
        });
        assert.equal(built.status, 0, 'napi:build failed');
    }
    const after = fs.existsSync(nativeDir) && fs.readdirSync(nativeDir).some((f) => f.endsWith('.node'));
    assert.ok(after, `native binary missing under frontends/prometheus-${short}`);
}

/**
 * Serve a tiny fixture in a child process so sync N-API (ureq) does not deadlock
 * the Node event loop that would otherwise own the HTTP server.
 */
function startFixtureServer(body) {
    const script = `
const http = require('node:http');
const body = Buffer.from(${JSON.stringify(body.toString('base64'))}, 'base64');
const server = http.createServer((req, res) => {
  res.writeHead(200, {
    'Content-Type': 'application/octet-stream',
    'Content-Length': body.length,
    'Content-Disposition': 'attachment; filename="smoke.bin"',
  });
  if (req.method === 'HEAD') { res.end(); return; }
  res.end(body);
});
server.listen(0, '127.0.0.1', () => {
  const port = server.address().port;
  process.stdout.write(String(port) + '\\n');
});
`;
    return new Promise((resolve, reject) => {
        const child = spawn(process.execPath, ['-e', script], {
            stdio: ['ignore', 'pipe', 'pipe'],
        });
        let stdout = '';
        let settled = false;
        const fail = (err) => {
            if (settled) return;
            settled = true;
            try {
                child.kill();
            } catch {
                /* ignore */
            }
            reject(err);
        };
        child.stdout.on('data', (chunk) => {
            stdout += chunk.toString('utf8');
            const line = stdout.split(/\r?\n/).find((s) => /^\d+$/.test(s.trim()));
            if (!line || settled) return;
            settled = true;
            const port = Number(line.trim());
            resolve({
                url: `http://127.0.0.1:${port}/smoke.bin`,
                stop: () =>
                    new Promise((r) => {
                        child.once('exit', () => r());
                        child.kill();
                        setTimeout(r, 500);
                    }),
            });
        });
        child.stderr.on('data', (chunk) => {
            process.stderr.write(chunk);
        });
        child.on('error', fail);
        child.on('exit', (code) => {
            if (!settled) fail(new Error(`fixture server exited early (${code})`));
        });
        setTimeout(() => fail(new Error('fixture server timeout')), 10_000);
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
    const { url, stop } = await startFixtureServer(body);
    try {
        const api = await import('../dist/index.js');
        const media = await api.info(url);
        assert.equal(media.extractor, 'generic-http');
        assert.equal(media.filename, 'smoke.bin');

        const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-smoke-'));
        const result = api.download(url, dir);
        assert.equal(fs.readFileSync(result.path).toString('utf8'), 'smoke-body');
        assert.equal(result.bytesWritten, body.length);
        fs.rmSync(dir, { recursive: true, force: true });
    } finally {
        await stop();
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

test('info local-file plugin', async () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-file-'));
    const file = path.join(dir, 'clip.bin');
    fs.writeFileSync(file, 'abc');
    try {
        const api = await import('../dist/index.js');
        const media = await api.info(pathToFileURL(file).href);
        assert.equal(media.extractor, 'local-file');
        assert.equal(media.filename, 'clip.bin');
        assert.equal(media.contentLength, 3);
    } finally {
        fs.rmSync(dir, { recursive: true, force: true });
    }
});
