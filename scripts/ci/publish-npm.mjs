/**
 * GitHub Actions: publish real packages (not placeholder stubs).
 *
 * Publish flow:
 *   CI matrix builds each OS → artifact dist/<short>/prometheus.<triple>.node
 *   → publishNative publishes @doki-land/prometheus-<short> (one binary each)
 *   → publishJs publishes @doki-land/prometheus with optionalDependencies only
 *     (never embeds platform binaries in the JS package)
 *
 * - tag vX.Y.Z → version X.Y.Z
 * - Idempotent: skip if target version already on registry
 * - No NPM_TOKEN; OIDC Trusted Publisher (permissions.id-token: write)
 * - Contract: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/prometheus-downloader
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
const JS_PACKAGES = [
    { dir: 'frontends/prometheus-torch', publishName: '@doki-land/prometheus-torch' },
    { dir: 'frontends/prometheus-plugin', publishName: '@doki-land/prometheus-plugin' },
    { dir: 'frontends/prometheus-plugin-generic-http', publishName: '@doki-land/prometheus-plugin-generic-http' },
    { dir: 'frontends/prometheus-plugin-local-file', publishName: '@doki-land/prometheus-plugin-local-file' },
    { dir: 'frontends/prometheus-harness', publishName: '@doki-land/prometheus-harness' },
    { dir: 'frontends/prometheus', publishName: '@doki-land/prometheus' },
];

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

function copyTree(src, dest, filter) {
    fs.mkdirSync(dest, { recursive: true });
    for (const name of fs.readdirSync(src)) {
        if (name === 'node_modules' || name === '.git') continue;
        const from = path.join(src, name);
        const to = path.join(dest, name);
        const st = fs.statSync(from);
        if (st.isDirectory()) {
            if (filter && !filter(from, true)) continue;
            copyTree(from, to, filter);
        } else {
            if (filter && !filter(from, false)) continue;
            fs.copyFileSync(from, to);
        }
    }
}

/**
 * @param {Record<string, string>} deps
 * @param {string} version
 */
function rewriteWorkspaceDeps(deps, version) {
    if (!deps) return deps;
    /** @type {Record<string, string>} */
    const out = {};
    for (const [k, v] of Object.entries(deps)) {
        if (typeof v === 'string' && (v.startsWith('workspace:') || v === '*')) {
            out[k] = version;
        } else {
            out[k] = v;
        }
    }
    return out;
}

function rewriteDepsField(pkg, version) {
    for (const field of ['dependencies', 'optionalDependencies', 'peerDependencies']) {
        if (pkg[field]) pkg[field] = rewriteWorkspaceDeps(pkg[field], version);
    }
    return pkg;
}

function isAlreadyPublished(blob) {
    return /cannot publish over existing|EPUBLISHCONFLICT|previously published versions|version already exists|cannot publish.*same version|you cannot publish over/i.test(
        blob,
    );
}

function isAuthFailure(blob) {
    return /ENEEDAUTH|Unable to authenticate|not authorized|OIDC|trusted publisher|two-factor|need to be logged|login|identity token|do not have permission to access it|Access token expired or revoked/i.test(
        blob,
    );
}

function isMissingPackage(blob) {
    if (isAuthFailure(blob)) return false;
    return /Package not found|does not exist on the registry|cannot publish.*before creating|This package has not been created|is not in this registry/i.test(
        blob,
    );
}

function versionExists(name, version) {
    const r = run('npm', ['view', `${name}@${version}`, 'version']);
    return r.status === 0 && r.stdout === version;
}

/**
 * @param {string} stagingDir
 * @param {string} name
 * @param {string} version
 * @returns {'published'|'exists'|'auth'|'missing'|'other'}
 */
function npmPublish(stagingDir, name, version) {
    const args = ['publish', '--access', 'public'];
    console.log(`\n=== ${name}@${version} npm ${args.join(' ')} ===`);
    const r = run('npm', args, { cwd: stagingDir });
    if (r.stdout) process.stdout.write(`${r.stdout}\n`);
    if (r.stderr) process.stderr.write(`${r.stderr}\n`);
    const blob = `${r.stdout}\n${r.stderr}`;
    if (r.status === 0) return 'published';
    if (isAlreadyPublished(blob) || versionExists(name, version)) return 'exists';
    if (isAuthFailure(blob)) return 'auth';
    if (isMissingPackage(blob)) return 'missing';
    if (versionExists(name, version)) return 'exists';
    console.error(blob.slice(0, 1200));
    return 'other';
}

/**
 * @param {string} version
 * @param {string} artifactsRoot
 */
function publishNative(version, artifactsRoot) {
    let published = 0;
    let skipped = 0;
    for (const plat of NATIVE_PLATFORMS) {
        const name = `@doki-land/prometheus-${plat.short}`;
        const artDir = path.join(artifactsRoot, plat.short);
        if (!fs.existsSync(artDir)) {
            console.log(` · ${name} no artifact (${plat.short}) — skip`);
            skipped += 1;
            continue;
        }
        if (versionExists(name, version)) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }
        const stage = path.join(os.tmpdir(), `prometheus-pub-native-${plat.short}-${version}`);
        fs.rmSync(stage, { recursive: true, force: true });
        fs.mkdirSync(stage, { recursive: true });
        for (const f of fs.readdirSync(artDir)) {
            fs.copyFileSync(path.join(artDir, f), path.join(stage, f));
        }
        const want = `prometheus.${plat.triple}.node`;
        writeJson(path.join(stage, 'package.json'), {
            name,
            version,
            description: `Prometheus native N-API addon (${plat.short})`,
            license: 'CC0-1.0',
            private: false,
            os: plat.os,
            cpu: plat.cpu,
            main: want,
            files: [want, 'README.md'],
            publishConfig: { access: 'public' },
            repository: {
                type: 'git',
                url: 'git+https://github.com/oovm/prometheus-downloader.git',
            },
        });
        for (const f of fs.readdirSync(stage)) {
            if (f.endsWith('.node') && f !== want) fs.unlinkSync(path.join(stage, f));
        }
        if (!fs.existsSync(path.join(stage, want))) {
            const plain = path.join(stage, 'prometheus.node');
            if (fs.existsSync(plain)) fs.renameSync(plain, path.join(stage, want));
        }
        if (!fs.existsSync(path.join(stage, want))) {
            fail(`${name}: staged artifact missing ${want}`);
        }
        const readmeSrc = path.join(ROOT, 'frontends', `prometheus-${plat.short}`, 'README.md');
        if (fs.existsSync(readmeSrc)) {
            fs.copyFileSync(readmeSrc, path.join(stage, 'README.md'));
        } else if (!fs.existsSync(path.join(stage, 'README.md'))) {
            fail(`${name}: missing README.md (expected ${readmeSrc})`);
        }
        const outcome = npmPublish(stage, name, version);
        if (outcome === 'published') published += 1;
        else if (outcome === 'exists') {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
        } else if (outcome === 'auth') {
            fail(`OIDC/auth failed for ${name}. Add Trusted Publisher: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/prometheus-downloader`);
        } else fail(`publish failed for ${name}`);
    }
    return { published, skipped };
}

/**
 * @param {string} version
 */
function publishJs(version) {
    let published = 0;
    let skipped = 0;
    const optionalNatives = Object.fromEntries(NATIVE_PLATFORMS.map((p) => [`@doki-land/prometheus-${p.short}`, version]));

    const packages = [...JS_PACKAGES];
    const skillsDir = path.join(ROOT, 'frontends/prometheus-skills');
    if (fs.existsSync(skillsDir)) {
        packages.push({ dir: 'frontends/prometheus-skills', publishName: '@doki-land/prometheus-skills' });
    }

    for (const spec of packages) {
        const abs = path.join(ROOT, spec.dir);
        if (!fs.existsSync(abs)) {
            console.log(` · skip missing ${spec.dir}`);
            skipped += 1;
            continue;
        }
        const raw = readJson(path.join(abs, 'package.json'));
        const name = spec.publishName ?? raw.name;
        if (!name) fail(`no name for ${spec.dir}`);

        if (versionExists(name, version)) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }

        const stage = path.join(os.tmpdir(), `prometheus-pub-js-${name.replace(/[/@]/g, '-')}-${version}`);
        fs.rmSync(stage, { recursive: true, force: true });
        fs.mkdirSync(stage, { recursive: true });

        const files = Array.isArray(raw.files) && raw.files.length ? raw.files : null;
        if (files) {
            for (const f of files) {
                const from = path.join(abs, f);
                if (!fs.existsSync(from)) continue;
                const st = fs.statSync(from);
                const to = path.join(stage, f);
                if (st.isDirectory()) copyTree(from, to);
                else {
                    fs.mkdirSync(path.dirname(to), { recursive: true });
                    fs.copyFileSync(from, to);
                }
            }
            for (const extra of ['package.json', 'README.md', 'Readme.md', 'readme.md', 'License.md', 'LICENSE', 'bin']) {
                const from = path.join(abs, extra);
                if (!fs.existsSync(from)) continue;
                const to = path.join(stage, extra);
                if (fs.statSync(from).isDirectory()) copyTree(from, to);
                else fs.copyFileSync(from, to);
            }
        } else {
            copyTree(abs, stage, (p) => {
                const rel = path.relative(abs, p);
                if (rel.includes('node_modules') || rel.includes('tests') || rel.endsWith('.node')) return false;
                return true;
            });
        }

        // JS package must never ship platform binaries.
        if (name === '@doki-land/prometheus') {
            for (const f of fs.readdirSync(stage)) {
                if (f.endsWith('.node')) fs.unlinkSync(path.join(stage, f));
            }
        }

        const pkg = rewriteDepsField({ ...raw }, version);
        pkg.name = name;
        pkg.version = version;
        delete pkg.private;
        pkg.publishConfig = { ...(pkg.publishConfig ?? {}), access: 'public' };
        if (!pkg.repository) {
            pkg.repository = {
                type: 'git',
                url: 'git+https://github.com/oovm/prometheus-downloader.git',
            };
        }
        if (name === '@doki-land/prometheus') {
            pkg.optionalDependencies = { ...(pkg.optionalDependencies ?? {}), ...optionalNatives };
        }
        delete pkg.devDependencies;
        writeJson(path.join(stage, 'package.json'), pkg);

        if (
            !fs.existsSync(path.join(stage, 'README.md')) &&
            !fs.existsSync(path.join(stage, 'Readme.md')) &&
            !fs.existsSync(path.join(stage, 'readme.md'))
        ) {
            fail(`${name}: package must ship a README.md (npm best practice; no silent stub)`);
        }

        const outcome = npmPublish(stage, name, version);
        if (outcome === 'published') published += 1;
        else if (outcome === 'exists') {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
        } else if (outcome === 'auth') {
            fail(`OIDC/auth failed for ${name}. Add Trusted Publisher: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/prometheus-downloader`);
        } else if (outcome === 'missing') {
            fail(
                `${name} is not on the registry yet. Create the name first via placeholder stubs (pnpm placeholder:publish), then retry real publish.`,
            );
        } else fail(`publish failed for ${name}`);
    }
    return { published, skipped };
}

const version = resolveVersion();
if (version === '0.0.0') {
    fail('0.0.0 is reserved for placeholder stubs; use publish-placeholder.mjs');
}
console.log(`ci-publish-npm: version=${version}`);
console.log(` GITHUB_REF=${process.env.GITHUB_REF ?? '(none)'}`);
console.log(' Trusted Publisher contract: publish-npm.yml + env NPM_PUBLISH\n');

delete process.env.NODE_AUTH_TOKEN;
delete process.env.NPM_TOKEN;

const artifactsRoot = process.env.PROMETHEUS_NATIVE_ARTIFACTS || path.join(ROOT, 'dist/native-flat');

const native = publishNative(version, artifactsRoot);
const js = publishJs(version);

console.log(
    `\nci-publish-npm: done (native published=${native.published} skipped=${native.skipped}; js published=${js.published} skipped=${js.skipped})`,
);
