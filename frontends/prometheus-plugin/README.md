# 🔌 @doki-land/prometheus-plugin

Contract types and workspace **loader** for Prometheus plugins. A plugin is a small npm package (`@doki-land/prometheus-plugin-<id>`) that answers two questions: “does this URL belong to me?” and “what metadata does it resolve to?”

Transfer, Range downloads, and file IO stay in the **native** addon. Plugins do not open high-concurrency sockets and do not replace `@doki-land/prometheus`.

## Install

```bash
pnpm add @doki-land/prometheus-plugin
```

Authors usually also depend on `@doki-land/prometheus-torch` (or receive `torch` through `PluginContext` from the product / harness).

Requires **Node.js ≥ 20**.

## Plugin contract

```ts
import type { MediaInfo, Plugin, PluginContext } from '@doki-land/prometheus-plugin';

export type MediaInfo = {
  url: string;
  title?: string | null;
  contentType?: string | null;
  contentLength?: number | null;
  filename?: string | null;
  extractor: string; // kebab-case plugin / extractor id
};

export type PluginContext = {
  torch: {
    evaluateJavascript: (source: string, options?: Record<string, unknown>) => unknown;
    instantiateWasm: (
      bytes: BufferSource,
      imports?: WebAssembly.Imports,
    ) => Promise<WebAssembly.WebAssemblyInstantiatedSource>;
  };
};

export type Plugin = {
  id: string;
  matches: (url: string) => boolean;
  inspect: (url: string, ctx: PluginContext) => MediaInfo | Promise<MediaInfo>;
};
```

Mark the package so discovery works:

```json
{
  "name": "@doki-land/prometheus-plugin-example",
  "prometheusPlugin": { "id": "example" }
}
```

Directory names under a monorepo `frontends/` tree must look like `prometheus-plugin-<id>` so the loader does not confuse them with this contract package (`prometheus-plugin`).

Export either `default`, `plugin`, or a module that itself has `matches` / `inspect`.

## Loader API

```ts
import {
  discoverPluginDirs,
  loadPlugins,
  resolvePlugin,
} from '@doki-land/prometheus-plugin';
import type { LoadedPlugin } from '@doki-land/prometheus-plugin';

discoverPluginDirs(workspaceRoot: string): string[]
loadPlugins(workspaceRoot: string): Promise<LoadedPlugin[]>
resolvePlugin(loaded: LoadedPlugin[], url: string): LoadedPlugin | undefined
```

`loadPlugins` looks under `<workspaceRoot>/frontends/prometheus-plugin-*`, reads each `package.json`, and imports `dist/index.js` (or `src/index.ts` / `index.js` during local work).

`LoadedPlugin` includes `{ id, dir, packageName, plugin }`.

## Shipped example plugins

| Package | `id` | Matches |
|---------|------|---------|
| `@doki-land/prometheus-plugin-generic-http` | `generic-http` | `http://` / `https://` |
| `@doki-land/prometheus-plugin-local-file` | `local-file` | `file:` |

These are **verification / contract** surfaces for Harness and the product bundle. Broad media coverage and transfer performance belong in the Rust extractors + transfer backends inside the `.node` addon — do not grow capability by stacking dozens of npm plugins.

## Authoring workflow

1. Scaffold with [`@doki-land/prometheus-harness`](https://www.npmjs.com/package/@doki-land/prometheus-harness):  
   `npx @doki-land/prometheus-harness create <id>`
2. Implement `matches` / `inspect` (use `ctx.torch` when a platform script is required).
3. `npx @doki-land/prometheus-harness test <id> <url>`
4. Keep PRs human-reviewed; Harness does not bundle an LLM and does not auto-merge.

## Design notes

- **First match wins** when a host iterates plugins in order (product currently tries `local-file` then `generic-http` before native fallback).
- **Wire names** follow the engine: camelCase fields, kebab-case ids.
- Plugins should return honest `MediaInfo`. They must not pretend to be a download engine.

## Responsible use

Use only with content you have the right to download. Stored credentials belong to you.

## License

[CC0 1.0 Universal](../../License.md)
