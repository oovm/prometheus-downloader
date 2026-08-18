import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';
import { createPlugin, fetchScript, fetchWasm, listPlugins, testPlugin, workspaceRoot } from '../dist/tools.js';

test('harness workspace root', () => {
    const root = workspaceRoot();
    assert.ok(fs.existsSync(path.join(root, 'pnpm-workspace.yaml')));
});

test('harness-create writes scaffold', () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-harness-'));
    fs.writeFileSync(path.join(tmp, 'pnpm-workspace.yaml'), "packages:\n  - 'frontends/*'\n");
    fs.mkdirSync(path.join(tmp, 'frontends'));
    const prev = process.env.PROMETHEUS_WORKSPACE;
    process.env.PROMETHEUS_WORKSPACE = tmp;
    try {
        const result = createPlugin('demo-site');
        assert.ok(fs.existsSync(path.join(result.path, 'src/index.ts')));
        assert.ok(fs.existsSync(path.join(result.path, 'package.json')));
    } finally {
        if (prev === undefined) delete process.env.PROMETHEUS_WORKSPACE;
        else process.env.PROMETHEUS_WORKSPACE = prev;
        fs.rmSync(tmp, { recursive: true, force: true });
    }
});

test('plugin-load via list_plugins includes generic-http', async () => {
    const listed = await listPlugins();
    assert.ok(listed.plugins.some((item) => item.id === 'generic-http'));
    assert.ok(listed.plugins.some((item) => item.id === 'local-file'));
});

function serve(body, headers) {
    return new Promise((resolve) => {
        const server = http.createServer((req, res) => {
            res.writeHead(200, headers);
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
                url: `http://127.0.0.1:${port}/clip.bin`,
                stop: () =>
                    new Promise((done) => {
                        server.close(() => done());
                    }),
            });
        });
    });
}

test('harness-test generic-http inspect', async () => {
    const body = Buffer.from('hello-plugin');
    const { url, stop } = await serve(body, {
        'Content-Type': 'application/octet-stream',
        'Content-Length': String(body.length),
        'Content-Disposition': 'attachment; filename="clip.bin"',
    });
    try {
        const result = await testPlugin('generic-http', url);
        assert.equal(result.success, true);
        assert.equal(result.media?.extractor, 'generic-http');
        assert.equal(result.media?.filename, 'clip.bin');
    } finally {
        await stop();
    }
});

test('fetch_wasm lists add export', async () => {
    const bytes = new Uint8Array([
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07,
        0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
    ]);
    const { url, stop } = await serve(Buffer.from(bytes), {
        'Content-Type': 'application/wasm',
        'Content-Length': String(bytes.length),
    });
    try {
        const result = await fetchWasm(url);
        assert.ok(result.exports.some((item) => item.name === 'add'));
    } finally {
        await stop();
    }
});

test('harness-test local-file inspect', async () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-local-'));
    const file = path.join(dir, 'note.bin');
    fs.writeFileSync(file, 'xyz');
    try {
        const result = await testPlugin('local-file', pathToFileURL(file).href);
        assert.equal(result.success, true);
        assert.equal(result.media?.extractor, 'local-file');
        assert.equal(result.media?.filename, 'note.bin');
        assert.equal(result.media?.contentLength, 3);
    } finally {
        fs.rmSync(dir, { recursive: true, force: true });
    }
});

test('fetch_script returns text body', async () => {
    const body = 'export const n = 1;\n';
    const { url, stop } = await serve(body, {
        'Content-Type': 'text/javascript',
        'Content-Length': String(Buffer.byteLength(body)),
    });
    try {
        const result = await fetchScript(url);
        assert.equal(result.status, 200);
        assert.equal(result.code, body);
    } finally {
        await stop();
    }
});
