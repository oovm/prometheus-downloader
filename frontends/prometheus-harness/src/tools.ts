import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { loadStrategies } from '@doki-land/prometheus-plugin';
import { TorchRuntime } from '@doki-land/prometheus-torch';

export function workspaceRoot(): string {
    const env = process.env.PROMETHEUS_WORKSPACE?.trim();
    if (env) return path.resolve(env);
    let dir = process.cwd();
    for (;;) {
        if (fs.existsSync(path.join(dir, 'pnpm-workspace.yaml')) && fs.existsSync(path.join(dir, 'frontends'))) {
            return dir;
        }
        const parent = path.dirname(dir);
        if (parent === dir) return process.cwd();
        dir = parent;
    }
}

export function sanitizeStrategyId(raw: string): string {
    const id = raw
        .trim()
        .toLowerCase()
        .replace(/_/g, '-')
        .replace(/[^a-z0-9-]/g, '');
    if (!id || !/^[a-z][a-z0-9-]*$/.test(id)) {
        throw new Error('platform id must start with a letter and use kebab-case');
    }
    return id;
}

function strategyTemplate(id: string): { files: Record<string, string> } {
    const pkg = {
        name: `@doki-land/prometheus-strategy-${id}`,
        version: '0.0.1',
        description: `Prometheus strategy plugin (${id})`,
        license: 'CC0-1.0',
        type: 'module',
        main: './dist/index.js',
        types: './dist/index.d.ts',
        prometheusStrategy: { id },
        exports: { '.': { types: './dist/index.d.ts', default: './dist/index.js' } },
        files: ['dist', 'README.md'],
        scripts: { build: 'tsc -p tsconfig.json' },
        devDependencies: {
            '@doki-land/prometheus-plugin': 'workspace:*',
            '@types/node': '^22.15.0',
            typescript: '^5.8.3',
        },
        engines: { node: '>=20' },
    };
    const tsconfig = {
        compilerOptions: {
            target: 'ES2022',
            module: 'NodeNext',
            moduleResolution: 'NodeNext',
            declaration: true,
            outDir: 'dist',
            rootDir: 'src',
            strict: true,
            skipLibCheck: true,
            esModuleInterop: true,
            forceConsistentCasingInFileNames: true,
            types: ['node'],
        },
        include: ['src/**/*.ts'],
    };
    const index = `import type { MediaInfo, StrategyContext, StrategyPlugin } from '@doki-land/prometheus-plugin';

const ID = '${id}';

export function matches(url: string): boolean {
    return url.includes('${id}');
}

export async function inspect(url: string, _ctx: StrategyContext): Promise<MediaInfo> {
    return {
        url,
        title: null,
        contentType: null,
        contentLength: null,
        filename: null,
        extractor: ID,
    };
}

export const plugin: StrategyPlugin = { id: ID, matches, inspect };
export default plugin;
`;
    return {
        files: {
            'package.json': `${JSON.stringify(pkg, null, 4)}\n`,
            'tsconfig.json': `${JSON.stringify(tsconfig, null, 4)}\n`,
            'README.md': `# @doki-land/prometheus-strategy-${id}\n\nStrategy plugin for ${id}.\n`,
            'src/index.ts': index,
        },
    };
}

export async function listStrategies() {
    const root = workspaceRoot();
    const loaded = await loadStrategies(root);
    return {
        workspace: root,
        strategies: loaded.map((item) => ({ id: item.id, packageName: item.packageName, dir: item.dir })),
    };
}

export function createStrategy(platform: string) {
    const id = sanitizeStrategyId(platform);
    const root = workspaceRoot();
    const dir = path.join(root, 'frontends', `prometheus-strategy-${id}`);
    if (fs.existsSync(dir)) {
        throw new Error(`already exists: ${dir}`);
    }
    const { files } = strategyTemplate(id);
    for (const [rel, body] of Object.entries(files)) {
        const dest = path.join(dir, rel);
        fs.mkdirSync(path.dirname(dest), { recursive: true });
        fs.writeFileSync(dest, body);
    }
    return { path: dir, files: Object.keys(files) };
}

export function getStrategyTemplate(platform: string) {
    const id = sanitizeStrategyId(platform);
    const { files } = strategyTemplate(id);
    return { id, files };
}

export async function testStrategy(platform: string, url: string) {
    const id = sanitizeStrategyId(platform);
    const root = workspaceRoot();
    const loaded = await loadStrategies(root);
    const found = loaded.find((item) => item.id === id);
    if (!found) {
        return { success: false, logs: [], error: `strategy not found: ${id}` };
    }
    const logs = [`loaded ${found.packageName} from ${found.dir}`];
    const matched = found.plugin.matches(url);
    logs.push(`matches=${matched}`);
    if (!matched) {
        return { success: false, logs, error: 'url did not match strategy' };
    }
    const torch = new TorchRuntime();
    const media = await found.plugin.inspect(url, { torch });
    logs.push(`extractor=${media.extractor}`);
    return { success: true, logs, media };
}

export function diffStrategy(platform: string) {
    const id = sanitizeStrategyId(platform);
    const root = workspaceRoot();
    const rel = path.join('frontends', `prometheus-strategy-${id}`);
    const r = spawnSync('git', ['diff', '--', rel.replaceAll('\\', '/')], {
        cwd: root,
        encoding: 'utf8',
        shell: process.platform === 'win32',
    });
    return {
        platform: id,
        changedFiles: rel,
        diff: r.stdout || r.stderr || '',
        status: r.status,
    };
}

export async function fetchScript(url: string) {
    const res = await fetch(url, { redirect: 'follow' });
    const code = await res.text();
    return {
        url,
        status: res.status,
        contentType: res.headers.get('content-type'),
        bytes: Buffer.byteLength(code),
        code,
    };
}

export async function fetchWasm(url: string) {
    const res = await fetch(url, { redirect: 'follow' });
    const buf = new Uint8Array(await res.arrayBuffer());
    const module = await WebAssembly.compile(buf);
    const exports = WebAssembly.Module.exports(module).map((item) => ({ name: item.name, kind: item.kind }));
    return { url, status: res.status, bytes: buf.byteLength, exports };
}

export function capturePage(_url: string) {
    return {
        unimplemented: true,
        error: 'capture_page is not available in this build',
    };
}

export function submitPr() {
    return {
        unimplemented: true,
        error: 'submit_pr is not available; open a pull request from the host git client',
    };
}

export const TOOL_DEFS = [
    {
        name: 'list_strategies',
        description: 'List strategy plugins in the Prometheus workspace',
        inputSchema: { type: 'object', properties: {}, additionalProperties: false },
    },
    {
        name: 'create_strategy',
        description: 'Write a new strategy plugin scaffold under frontends/',
        inputSchema: {
            type: 'object',
            properties: { platform: { type: 'string' } },
            required: ['platform'],
            additionalProperties: false,
        },
    },
    {
        name: 'get_strategy_template',
        description: 'Return the standard strategy plugin template files',
        inputSchema: {
            type: 'object',
            properties: { platform: { type: 'string' } },
            required: ['platform'],
            additionalProperties: false,
        },
    },
    {
        name: 'test_strategy',
        description: 'Load a strategy plugin and run matches/inspect against a URL',
        inputSchema: {
            type: 'object',
            properties: { platform: { type: 'string' }, url: { type: 'string' } },
            required: ['platform', 'url'],
            additionalProperties: false,
        },
    },
    {
        name: 'diff_strategy',
        description: 'Show git diff for a strategy plugin directory',
        inputSchema: {
            type: 'object',
            properties: { platform: { type: 'string' } },
            required: ['platform'],
            additionalProperties: false,
        },
    },
    {
        name: 'fetch_script',
        description: 'HTTP GET a text resource (no transform)',
        inputSchema: {
            type: 'object',
            properties: { url: { type: 'string' } },
            required: ['url'],
            additionalProperties: false,
        },
    },
    {
        name: 'fetch_wasm',
        description: 'HTTP GET a WebAssembly module and list its exports',
        inputSchema: {
            type: 'object',
            properties: { url: { type: 'string' } },
            required: ['url'],
            additionalProperties: false,
        },
    },
    {
        name: 'capture_page',
        description: 'Capture a page session (not available in this build)',
        inputSchema: {
            type: 'object',
            properties: { url: { type: 'string' } },
            required: ['url'],
            additionalProperties: false,
        },
    },
    {
        name: 'submit_pr',
        description: 'Open a GitHub pull request (not available in this build)',
        inputSchema: { type: 'object', properties: { message: { type: 'string' } }, additionalProperties: false },
    },
];

export async function callTool(name: string, args: Record<string, unknown>): Promise<unknown> {
    switch (name) {
        case 'list_strategies':
            return listStrategies();
        case 'create_strategy':
            return createStrategy(String(args.platform ?? ''));
        case 'get_strategy_template':
            return getStrategyTemplate(String(args.platform ?? ''));
        case 'test_strategy':
            return testStrategy(String(args.platform ?? ''), String(args.url ?? ''));
        case 'diff_strategy':
            return diffStrategy(String(args.platform ?? ''));
        case 'fetch_script':
            return fetchScript(String(args.url ?? ''));
        case 'fetch_wasm':
            return fetchWasm(String(args.url ?? ''));
        case 'capture_page':
            return capturePage(String(args.url ?? ''));
        case 'submit_pr':
            return submitPr();
        default:
            throw new Error(`unknown tool: ${name}`);
    }
}
