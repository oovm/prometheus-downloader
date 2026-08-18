import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export type MediaInfo = {
    url: string;
    title?: string | null;
    contentType?: string | null;
    contentLength?: number | null;
    filename?: string | null;
    extractor: string;
};

export type StrategyContext = {
    torch: {
        evaluateJavascript: (source: string, options?: Record<string, unknown>) => unknown;
        instantiateWasm: (bytes: BufferSource, imports?: WebAssembly.Imports) => Promise<WebAssembly.WebAssemblyInstantiatedSource>;
    };
};

export type StrategyPlugin = {
    id: string;
    matches: (url: string) => boolean;
    inspect: (url: string, ctx: StrategyContext) => MediaInfo | Promise<MediaInfo>;
};

export type LoadedStrategy = {
    id: string;
    dir: string;
    packageName: string;
    plugin: StrategyPlugin;
};

function readJson(file: string): Record<string, unknown> | null {
    try {
        return JSON.parse(fs.readFileSync(file, 'utf8')) as Record<string, unknown>;
    } catch {
        return null;
    }
}

function strategyIdFromPkg(pkg: Record<string, unknown>, dirName: string): string | null {
    const mark = pkg.prometheusStrategy;
    if (mark && typeof mark === 'object' && mark !== null && typeof (mark as { id?: unknown }).id === 'string') {
        return (mark as { id: string }).id;
    }
    const name = typeof pkg.name === 'string' ? pkg.name : '';
    const m = name.match(/prometheus-strategy-([a-z0-9-]+)$/);
    if (m) return m[1];
    if (dirName.startsWith('prometheus-strategy-')) return dirName.slice('prometheus-strategy-'.length);
    return null;
}

export function discoverStrategyDirs(workspaceRoot: string): string[] {
    const frontends = path.join(workspaceRoot, 'frontends');
    if (!fs.existsSync(frontends)) return [];
    /** @type {string[]} */
    const out = [];
    for (const name of fs.readdirSync(frontends)) {
        if (!name.startsWith('prometheus-strategy-')) continue;
        const dir = path.join(frontends, name);
        if (!fs.statSync(dir).isDirectory()) continue;
        const pkg = readJson(path.join(dir, 'package.json'));
        if (!pkg) continue;
        out.push(dir);
    }
    return out.sort();
}

async function importPlugin(dir: string): Promise<StrategyPlugin> {
    const pkg = readJson(path.join(dir, 'package.json')) ?? {};
    const candidates = ['dist/index.js', 'src/index.ts', 'index.js'];
    let modPath: string | null = null;
    for (const rel of candidates) {
        const p = path.join(dir, rel);
        if (fs.existsSync(p)) {
            modPath = p;
            break;
        }
    }
    if (!modPath) {
        throw new Error(`strategy module missing under ${dir}`);
    }
    const imported = (await import(pathToFileURL(modPath).href)) as {
        default?: StrategyPlugin;
        plugin?: StrategyPlugin;
        id?: string;
        matches?: StrategyPlugin['matches'];
        inspect?: StrategyPlugin['inspect'];
    };
    const plugin = imported.default ?? imported.plugin ?? (imported as StrategyPlugin);
    if (typeof plugin.matches !== 'function' || typeof plugin.inspect !== 'function') {
        throw new Error(`invalid strategy exports in ${dir}`);
    }
    const id = plugin.id || strategyIdFromPkg(pkg, path.basename(dir));
    if (!id) throw new Error(`strategy id missing in ${dir}`);
    return { id, matches: plugin.matches, inspect: plugin.inspect };
}

export async function loadStrategies(workspaceRoot: string): Promise<LoadedStrategy[]> {
    const dirs = discoverStrategyDirs(workspaceRoot);
    /** @type {LoadedStrategy[]} */
    const loaded = [];
    for (const dir of dirs) {
        const pkg = readJson(path.join(dir, 'package.json')) ?? {};
        const plugin = await importPlugin(dir);
        loaded.push({
            id: plugin.id,
            dir,
            packageName: typeof pkg.name === 'string' ? pkg.name : path.basename(dir),
            plugin,
        });
    }
    return loaded;
}

export function resolveStrategy(loaded: LoadedStrategy[], url: string): LoadedStrategy | undefined {
    return loaded.find((item) => item.plugin.matches(url));
}
