# 🛠️ @doki-land/prometheus-harness

**Harness** is the authoring toolkit for Prometheus plugins. It exposes the same capabilities as a small CLI and as an **MCP stdio server** so your coding agent (Cursor, Claude Desktop, and similar) can list, scaffold, and test `@doki-land/prometheus-plugin-*` packages.

Harness **does not** embed a language model. Your assistant stays on your side of the MCP socket; this package only provides tools.

This is **not** the product download MCP. End-user download tools live under `prometheus mcp` on `@doki-land/prometheus` (stubbed until the engine surface is ready). Use Harness when you are writing or verifying plugins.

## Install

```bash
pnpm add -D @doki-land/prometheus-harness
# or run via npx without a local install
```

Requires **Node.js ≥ 20**. Point `PROMETHEUS_WORKSPACE` at the monorepo root (the directory that contains `pnpm-workspace.yaml` and `frontends/`), or run from inside that tree so discovery can walk upward.

## CLI

```bash
npx @doki-land/prometheus-harness mcp
npx @doki-land/prometheus-harness list
npx @doki-land/prometheus-harness create <id>
npx @doki-land/prometheus-harness test <id> <url>
npx @doki-land/prometheus-harness --help
```

Examples:

```bash
export PROMETHEUS_WORKSPACE=/path/to/prometheus-downloader

npx @doki-land/prometheus-harness list
npx @doki-land/prometheus-harness create my-site
npx @doki-land/prometheus-harness test generic-http "https://example.com/a.bin"
npx @doki-land/prometheus-harness test local-file "file:///tmp/clip.bin"
```

`create` writes `frontends/prometheus-plugin-<id>/` with `package.json`, `tsconfig.json`, `src/index.ts`, and a README stub. Ids must be kebab-case and start with a letter.

## MCP server

```bash
npx @doki-land/prometheus-harness mcp
```

Cursor / local checkout example:

```json
{
  "mcpServers": {
    "prometheus-harness": {
      "command": "node",
      "args": ["frontends/prometheus-harness/bin/prometheus-harness.js", "mcp"],
      "env": { "PROMETHEUS_WORKSPACE": "." }
    }
  }
}
```

Published install example:

```json
{
  "mcpServers": {
    "prometheus-harness": {
      "command": "npx",
      "args": ["-y", "@doki-land/prometheus-harness", "mcp"],
      "env": { "PROMETHEUS_WORKSPACE": "/absolute/path/to/workspace" }
    }
  }
}
```

### Tools (current)

| Tool | Behavior |
|------|----------|
| `list_plugins` | Scan `frontends/prometheus-plugin-*` |
| `create_plugin` | Scaffold a new plugin package |
| `get_plugin_template` | Return template file bodies |
| `test_plugin` | Load plugin; run `matches` / `inspect` on a URL |
| `diff_plugin` | `git diff` for that package |
| `fetch_script` | HTTP GET text (**no deobfuscation**) |
| `fetch_wasm` | GET bytes, `WebAssembly.compile`, list exports (**no decompile**) |
| `capture_page` | Stub (no Playwright hard dependency) |
| `submit_pr` | Stub (use host `gh` from your assistant) |

Harness will not register tools that turn the public MCP into a deobfuscation or platform-protection manual.

## How it fits the product

```text
Your coding agent
      │ MCP stdio
      ▼
@doki-land/prometheus-harness
      │ reads / writes workspace
      ▼
frontends/prometheus-plugin-<id>/
      │ npm publish (when ready)
      ▼
@doki-land/prometheus  →  torch + plugins  →  napi transfer / disk
```

After a plugin returns `MediaInfo`, downloads still go through the **single** platform `.node` addon. Harness never becomes a second download engine.

## Programmatic use

```ts
import { /* package exports used by the CLI */ } from '@doki-land/prometheus-harness';
```

Prefer the CLI / MCP for day-to-day work; the package’s public surface is oriented around those entrypoints and the shared helpers they call (`listPlugins`, `createPlugin`, `testPlugin`, …).

## Responsible use

Use only with content you have the right to download. Stored credentials belong to you. Keep plugin merges human-reviewed.

## License

[CC0 1.0 Universal](../../License.md)
