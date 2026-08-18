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
    for (const name of ['install', 'configure', 'verify', 'upgrade', 'usage', 'credentials']) {
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
    assert.match(upgrade, /explore.md/);
    assert.match(upgrade, /credentials.md/);
    const verify = read('references/verify.md');
    assert.match(verify, /explore.md/);
    assert.match(verify, /Create a new plugin|create a plugin/i);
    assert.match(verify, /credentials.md/);
});

test('credentials guide is user-supplied cookies only', () => {
    const skill = read('SKILL.md');
    assert.match(skill, /references\/credentials.md/);
    assert.match(skill, /user-supplied cookies/i);
    assert.match(skill, /IDE built-in browser/);
    const credentials = read('references/credentials.md');
    assert.match(credentials, /user-supplied cookies/i);
    assert.match(credentials, /PROMETHEUS_COOKIES_FILE/);
    assert.match(credentials, /PROMETHEUS_COOKIES/);
    assert.match(credentials, /Simple Browser|IDE built-in browser/);
    assert.match(credentials, /encryption is not implemented/i);
    assert.match(credentials, /Netscape/);
    assert.match(credentials, /AskQuestion/);
    assert.match(credentials, /does \*\*not\*\* consume these values today|does not consume/i);
});

test('plugin authoring skill is markdown and asks first', () => {
    const text = read('plugin/SKILL.md');
    assert.match(text, /^---\r?\nname: prometheus-plugin\r?\n/);
    assert.match(text, /AskQuestion/);
    assert.match(text, /prometheus-harness create/);
    assert.match(text, /matches/);
    assert.doesNotMatch(text, /install-product|detect\.mjs/);
    assert.ok(text.split(/\r?\n/).length < 500);
    for (const name of ['scaffold', 'implement', 'test', 'torch', 'explore']) {
        const body = read(`plugin/references/${name}.md`);
        assert.ok(body.length > 80, `${name}.md too short`);
        assert.match(body, /Stop and ask/i);
    }
    const explore = read('plugin/references/explore.md');
    assert.match(explore, /Create a new plugin/);
    assert.match(explore, /npx @doki-land\/prometheus-harness create/);
    assert.match(explore, /credentials.md/);
    const scaffold = read('plugin/references/scaffold.md');
    assert.match(scaffold, /npx @doki-land\/prometheus-harness create/);
});
