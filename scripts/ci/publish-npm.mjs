/**
 * GitHub Actions: publish real packages (not placeholder stubs).
 *
 * - tag vX.Y.Z → version X.Y.Z
 * - Idempotent: skip if target version already on registry
 * - No NPM_TOKEN; OIDC Trusted Publisher (permissions.id-token: write)
 * - Contract: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/prometheus-downloader
 *
 * Until feature-ready, keep workspace packages on 0.0.x and do not cut release tags.
 * Placeholders on the registry stay at 0.0.0 (see publish-placeholder.mjs).
 */

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

const NATIVE_PLATFORMS = [
    { short: 'win32-x64', triple: 'win32-x64-msvc', os: ['win32'], cpu: ['x64'] },
    { short: 'win32-arm64', triple: 'win32-arm64-msvc', os: ['win32'], cpu: ['arm64'] },
    { short: 'darwin-x64', triple: 'darwin-x64', os: ['darwin'], cpu: ['x64'] },
    { short: 'darwin-arm64', triple: 'darwin-arm64', os: ['darwin'], cpu: ['arm64'] },
    { short: 'linux-x64', triple: 'linux-x64-gnu', os: ['linux'], cpu: ['x64'] },
    { short: 'linux-arm64', triple: 'linux-arm64-gnu', os: ['linux'], cpu: ['arm64'] },
];

/** @type {{ dir: string, publishName?: string }[]} */
const JS_PACKAGES = [{ dir: 'frontends/prometheus', publishName: '@doki-land/prometheus' }];

function fail(msg) {
    console.error(`ci-publish-npm: ${msg}`);
    process.exit(1);
}

function run(cmd, args, opts = {}) {
    const r = spawnSync(cmd, args, {
        cwd: opts.cwd ?? ROOT,
        encoding: 'utf8',
        shell: process.platform === 'win32',
        env: opts.env ?? process.env,
        stdio: opts.stdio ?? 'pipe',
    });
    return {
        status: r.status ?? 1,
        stdout: String(r.stdout ?? '').trim(),
        stderr: String(r.stderr ?? '').trim(),
    };
}

function resolveVersion() {
    const fromArg = process.argv.find((a) => a.startsWith('--version='))?.slice('--version='.length);
    if (fromArg) return fromArg.replace(/^v/, '');
    const ref = process.env.GITHUB_REF ?? '';
    const m = ref.match(/^refs\/tags\/(?:placeholder-)?v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/);
    if (m) return m[1];
    fail('need --version=X.Y.Z or GITHUB_REF=refs/tags/vX.Y.Z');
}

function readJson(p) {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function writeJson(p, obj) {
    fs.writeFileSync(p, `${JSON.stringify(obj, null, 2)}\n`);
}

function viewVersion(name) {
    const r = run('npm', ['view', name, 'version'], { stdio: 'pipe' });
    if (r.status === 0 && r.stdout) return r.stdout;
    return null;
}

function publishDir(dir, name) {
    const args = ['publish'];
    if (name.startsWith('@')) args.push('--access', 'public');
    console.log(`=== ${name}  npm ${args.join(' ')} ===`);
    const r = run('npm', args, { cwd: dir, stdio: 'inherit' });
    if (r.status !== 0) fail(`publish failed: ${name}`);
}

function stageMainPackage(version, outRoot) {
    const src = path.join(ROOT, 'frontends/prometheus');
    const dest = path.join(outRoot, 'prometheus');
    fs.cpSync(src, dest, {
        recursive: true,
        filter: (p) => !p.includes(`${path.sep}node_modules${path.sep}`) && !p.endsWith('.tsbuildinfo'),
    });
    const pkgPath = path.join(dest, 'package.json');
    const pkg = readJson(pkgPath);
    pkg.name = '@doki-land/prometheus';
    pkg.version = version;
    pkg.private = false;
    pkg.optionalDependencies = Object.fromEntries(
        NATIVE_PLATFORMS.map((p) => [`@doki-land/prometheus-${p.short}`, version]),
    );
    writeJson(pkgPath, pkg);
    return dest;
}

function stageNativePackage(version, platform, artifactsRoot, outRoot) {
    const name = `@doki-land/prometheus-${platform.short}`;
    const dest = path.join(outRoot, `prometheus-${platform.short}`);
    fs.mkdirSync(dest, { recursive: true });
    const binaryName = `prometheus.${platform.triple}.node`;
    const candidates = [
        path.join(artifactsRoot, platform.short, binaryName),
        path.join(artifactsRoot, platform.short, 'prometheus.node'),
        path.join(ROOT, 'frontends/prometheus', binaryName),
        path.join(ROOT, 'frontends/prometheus', 'prometheus.node'),
    ];
    const src = candidates.find((p) => fs.existsSync(p));
    if (!src) {
        fail(`missing native binary for ${platform.short}; looked under ${artifactsRoot}`);
    }
    fs.copyFileSync(src, path.join(dest, binaryName));
    writeJson(path.join(dest, 'package.json'), {
        name,
        version,
        description: `Prometheus native N-API addon (${platform.short})`,
        license: 'CC0-1.0',
        os: platform.os,
        cpu: platform.cpu,
        main: binaryName,
        files: [binaryName, 'README.md'],
    });
    fs.writeFileSync(
        path.join(dest, 'README.md'),
        `# ${name}\n\nOptional native addon for @doki-land/prometheus.\n`,
    );
    return { name, dest };
}

function main() {
    const version = resolveVersion();
    if (version === '0.0.0') {
        fail('0.0.0 is reserved for placeholder stubs; use publish-placeholder.mjs');
    }
    console.log(`ci-publish-npm: version ${version}`);

    const artifactsRoot = process.env.PROMETHEUS_NATIVE_ARTIFACTS || path.join(ROOT, 'dist/native-flat');
    const stage = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-publish-'));

    for (const platform of NATIVE_PLATFORMS) {
        const name = `@doki-land/prometheus-${platform.short}`;
        const existing = viewVersion(name);
        if (existing === version) {
            console.log(`  skip ${name}@${version} (already published)`);
            continue;
        }
        const staged = stageNativePackage(version, platform, artifactsRoot, stage);
        publishDir(staged.dest, staged.name);
    }

    for (const entry of JS_PACKAGES) {
        const name = entry.publishName ?? readJson(path.join(ROOT, entry.dir, 'package.json')).name;
        const existing = viewVersion(name);
        if (existing === version) {
            console.log(`  skip ${name}@${version} (already published)`);
            continue;
        }
        const dest = stageMainPackage(version, stage);
        publishDir(dest, name);
    }

    console.log('ci-publish-npm: done');
}

main();
