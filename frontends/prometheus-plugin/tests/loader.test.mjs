import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { loadPlugins, resolvePlugin } from '../dist/index.js';

const repoRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..');

test('plugin-load: discovers generic-http', async () => {
    const loaded = await loadPlugins(repoRoot);
    const ids = loaded.map((item) => item.id);
    assert.ok(ids.includes('generic-http'));
    assert.ok(ids.includes('local-file'));
    const hit = resolvePlugin(loaded, 'https://example.com/a.bin');
    assert.equal(hit?.id, 'generic-http');
    const local = resolvePlugin(loaded, 'file:///tmp/a.bin');
    assert.equal(local?.id, 'local-file');
});
