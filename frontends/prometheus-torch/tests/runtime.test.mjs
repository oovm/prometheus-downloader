import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { TorchRuntime, version } from '../dist/index.js';

const require = createRequire(import.meta.url);
const pkgRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');

test('torch-js: version matches package.json', () => {
    const pkg = require(path.join(pkgRoot, 'package.json'));
    assert.equal(version(), pkg.version);
});

test('torch-js: evaluateJavascript in isolated context', () => {
    const torch = new TorchRuntime();
    const value = torch.evaluateJavascript('({ n: 1 + 2, flag })', {
        sandbox: { flag: 'ok' },
        filename: 'fixture.js',
    });
    assert.equal(typeof value, 'object');
    assert.ok(value);
    assert.equal(value.n, 3);
    assert.equal(value.flag, 'ok');
});

test('torch-js: sandbox does not leak into later runs', () => {
    const torch = new TorchRuntime();
    torch.evaluateJavascript('this.marker = 1', { sandbox: {} });
    assert.throws(() => torch.evaluateJavascript('marker', { timeoutMs: 1_000 }));
});

test('torch-wasm: instantiateWasm fixture add(i32,i32)', async () => {
    const torch = new TorchRuntime();
    // (module (func (export "add") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))
    const bytes = new Uint8Array([
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07,
        0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
    ]);
    const { instance } = await torch.instantiateWasm(bytes);
    const add = instance.exports.add;
    assert.equal(typeof add, 'function');
    assert.equal(add(20, 22), 42);
});
