import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);

export type MediaInfo = {
    url: string;
    title?: string | null;
    contentType?: string | null;
    contentLength?: number | null;
    filename?: string | null;
    extractor: string;
};

export type DownloadResult = {
    path: string;
    bytesWritten: number;
    filename: string;
};

type NativeAddon = {
    version: () => string;
    info: (url: string) => {
        url: string;
        title?: string | null;
        contentType?: string | null;
        contentLength?: number | null;
        filename?: string | null;
        extractor: string;
    };
    download: (
        url: string,
        outputDir: string,
    ) => {
        path: string;
        bytesWritten: number;
        filename: string;
    };
    createVault: (path: string, password: string) => string;
};

let cached: NativeAddon | undefined;

function platformIds() {
    const { platform, arch } = process;
    let triple = `${platform}-${arch}`;
    if (platform === 'win32' && arch === 'x64') triple = 'win32-x64-msvc';
    else if (platform === 'win32' && arch === 'arm64') triple = 'win32-arm64-msvc';
    else if (platform === 'darwin' && arch === 'arm64') triple = 'darwin-arm64';
    else if (platform === 'darwin' && arch === 'x64') triple = 'darwin-x64';
    else if (platform === 'linux' && arch === 'x64') triple = 'linux-x64-gnu';
    else if (platform === 'linux' && arch === 'arm64') triple = 'linux-arm64-gnu';
    const short =
        triple === 'win32-x64-msvc'
            ? 'win32-x64'
            : triple === 'win32-arm64-msvc'
              ? 'win32-arm64'
              : triple === 'linux-x64-gnu'
                ? 'linux-x64'
                : triple === 'linux-arm64-gnu'
                  ? 'linux-arm64'
                  : triple;
    return { triple, short };
}

/** Load the platform-specific N-API addon via optionalDependency packages. */
export function loadNative(): NativeAddon {
    if (cached) return cached;

    const envPath = (typeof process.env.PROMETHEUS_NATIVE_NODE === 'string' && process.env.PROMETHEUS_NATIVE_NODE.trim()) || '';
    if (envPath) {
        cached = require(path.resolve(envPath)) as NativeAddon;
        return cached;
    }

    const { triple, short } = platformIds();
    const name = `@doki-land/prometheus-${short}`;
    const pkgRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
    /** @type {string[]} */
    const candidates = [];
    try {
        const resolved = require.resolve(`${name}/package.json`);
        const dir = path.dirname(resolved);
        candidates.push(path.join(dir, `prometheus.${triple}.node`));
    } catch {
        /* optional */
    }
    candidates.push(
        path.join(pkgRoot, 'node_modules', name, `prometheus.${triple}.node`),
        path.join(pkgRoot, '..', `prometheus-${short}`, `prometheus.${triple}.node`),
    );

    for (const candidate of candidates) {
        if (fs.existsSync(candidate)) {
            cached = require(candidate) as NativeAddon;
            return cached;
        }
    }

    throw new Error(`prometheus native addon not found for ${name}. Run: pnpm napi:build (writes frontends/prometheus-${short}/)`);
}
