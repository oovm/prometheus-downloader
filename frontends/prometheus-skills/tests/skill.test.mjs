import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const pkgRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');

function read(rel) {
    return fs.readFileSync(path.join(pkgRoot, rel), 'utf8');
}

test('package is a docs skill, not an installer', () => {
    const pkg = JSON.parse(read('package.json'));
    assert.equal(pkg.name, '@doki-land/prometheus-skills');
    assert.equal(pkg.version, '0.0.1');
    assert.equal(pkg.license, 'CC0-1.0');
    assert.equal(pkg.bin, undefined);
    assert.equal(pkg.dependencies, undefined);
    assert.ok(!fs.existsSync(path.join(pkgRoot, 'scripts')));
    assert.ok(!fs.existsSync(path.join(pkgRoot, 'bin')));
});

test('SKILL.md matches Cursor skill frontmatter', () => {
    const text = read('SKILL.md');
    assert.match(text, /^---\r?\nname: prometheus-skills\r?\n/);
    assert.match(text, /description:/);
    assert.match(text, /Install @doki-land\/prometheus-skills and finish the Prometheus setup and configuration for this environment/);
    assert.match(text, /AskQuestion/);
    assert.match(text, /@doki-land\/prometheus-torch/);
    assert.doesNotMatch(text, /install-product|detect\.mjs|check-native/);
    assert.ok(text.split(/\r?\n/).length < 500, 'SKILL.md should stay under 500 lines');
});

test('reference guides exist and tell the agent to ask', () => {
    for (const name of ['install', 'configure', 'verify', 'upgrade', 'usage']) {
        const body = read(`references/${name}.md`);
        assert.ok(body.length > 80, `${name}.md too short`);
        if (name !== 'usage') {
            assert.match(body, /Stop and ask|ask first|Ask /i);
        }
    }
    const install = read('references/install.md');
    assert.match(install, /npm install @doki-land\/prometheus/);
    assert.match(install, /pnpm napi:build/);
    assert.doesNotMatch(install, /install-product/);
    const upgrade = read('references/upgrade.md');
    assert.match(upgrade, /Do \*\*not\*\* propose them at the end of a normal install|not default/i);
    assert.match(upgrade, /Playwright/);
});
