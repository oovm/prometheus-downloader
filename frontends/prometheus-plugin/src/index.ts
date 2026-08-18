import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const PLUGIN_DIR_PREFIX = 'prometheus-plugin-';

export type MediaInfo = {
    url: string;
    title?: string | null;
    contentType?: string | null;
    contentLength?: number | null;
    filename?: string | null;
    extractor: string;
};

export type PluginContext = {
    /** Present only when `@doki-land/prometheus-torch` (or a compatible host) is installed. */
    torch?: {
        evaluateJavascript: (source: string, options?: Record<string, unknown>) => unknown;
        instantiateWasm: (bytes: BufferSource, imports?: WebAssembly.Imports) => Promise<WebAssembly.WebAssemblyInstantiatedSource>;
    };
};

export type Plugin = {
    id: string;
    matches: (url: string) => boolean;
    inspect: (url: string, ctx: PluginContext) => MediaInfo | Promise<MediaInfo>;
};

export type LoadedPlugin = {
    id: string;
    dir: string;
    packageName: string;
    plugin: Plugin;
};

function readJson(file: string): Record<string, unknown> | null {
    try {
        return JSON.parse(fs.readFileSync(file, 'utf8')) as Record<string, unknown>;
    } catch {
        return null;
    }
}

function pluginIdFromPkg(pkg: Record<string, unknown>, dirName: string): string | null {
    const mark = pkg.prometheusPlugin;
    if (mark && typeof mark === 'object' && mark !== null && typeof (mark as { id?: unknown }).id === 'string') {
        return (mark as { id: string }).id;
    }
    const name = typeof pkg.name === 'string' ? pkg.name : '';
    const m = name.match(/prometheus-plugin-([a-z0-9-]+)$/);
    if (m) return m[1];
    if (dirName.startsWith(PLUGIN_DIR_PREFIX)) return dirName.slice(PLUGIN_DIR_PREFIX.length);
    return null;
}

export function discoverPluginDirs(workspaceRoot: string): string[] {
    const frontends = path.join(workspaceRoot, 'frontends');
    if (!fs.existsSync(frontends)) return [];
    const out = [];
    for (const name of fs.readdirSync(frontends)) {
        if (!name.startsWith(PLUGIN_DIR_PREFIX)) continue;
        const dir = path.join(frontends, name);
        if (!fs.statSync(dir).isDirectory()) continue;
        const pkg = readJson(path.join(dir, 'package.json'));
        if (!pkg) continue;
        out.push(dir);
    }
    return out.sort();
}

async function importPlugin(dir: string): Promise<Plugin> {
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
        throw new Error(`plugin module missing under ${dir}`);
    }
    const imported = (await import(pathToFileURL(modPath).href)) as {
        default?: Plugin;
        plugin?: Plugin;
        id?: string;
        matches?: Plugin['matches'];
        inspect?: Plugin['inspect'];
    };
    const plugin = imported.default ?? imported.plugin ?? (imported as Plugin);
    if (typeof plugin.matches !== 'function' || typeof plugin.inspect !== 'function') {
        throw new Error(`invalid plugin exports in ${dir}`);
    }
    const id = plugin.id || pluginIdFromPkg(pkg, path.basename(dir));
    if (!id) throw new Error(`plugin id missing in ${dir}`);
    return { id, matches: plugin.matches, inspect: plugin.inspect };
}

export async function loadPlugins(workspaceRoot: string): Promise<LoadedPlugin[]> {
    const dirs = discoverPluginDirs(workspaceRoot);
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

export function resolvePlugin(loaded: LoadedPlugin[], url: string): LoadedPlugin | undefined {
    return loaded.find((item) => item.plugin.matches(url));
}
