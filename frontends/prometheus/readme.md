# 🔥 @doki-land/prometheus

The **product package** for Prometheus: a media acquisition CLI and Node API. Install this (or let `@doki-land/prometheus-skills` install it for you) when you want to resolve media metadata and download files you are allowed to fetch.

```bash
pnpm add @doki-land/prometheus
# npm install @doki-land/prometheus
# yarn add @doki-land/prometheus
```

> Prefer agent-guided setup?  
> `Install @doki-land/prometheus-skills and finish the Prometheus setup and configuration for this environment.`

## What you get

- **CLI** (`prometheus`) for version, `info`, and `download`
- **Typed Node API** wrapping a per-platform **N-API** addon
- **Optional** platform packages (`@doki-land/prometheus-win32-x64`, `darwin-arm64`, …) pulled in via `optionalDependencies`
- Bundled thin plugins for direct HTTP(S) and `file:` inspect, plus `@doki-land/prometheus-torch` for platform scripts / WASM

This package is **JavaScript only**. It never embeds every platform’s `.node` binary. On install, npm/pnpm selects the matching `@doki-land/prometheus-<os>-<cpu>` optional dependency. That platform package contains **exactly one** `.node` file — the whole engine lives there.

## Requirements

| Requirement | Notes |
|-------------|--------|
| Node.js | `>= 20` |
| Native addon | Matching `@doki-land/prometheus-<os>-<cpu>` must be present (optionalDependency or local `pnpm napi:build`) |
| Network / disk | For the URLs and paths you choose to inspect or download |

Supported optional natives today: `win32-x64`, `win32-arm64`, `darwin-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64`.

## CLI

```bash
prometheus --version
prometheus info <url>
prometheus download <url> -o <dir>
prometheus mcp    # product MCP — stub in current builds
```

Examples:

```bash
prometheus info "https://cdn.example.com/demo.mp4"
prometheus download "https://cdn.example.com/demo.mp4" -o ./downloads
prometheus info "file:///C:/Users/you/Videos/clip.bin"
```

`info` prints JSON (`MediaInfo`). `download` prints a JSON `DownloadResult` (`path`, `bytesWritten`, `filename`).

## Node API

```ts
import {
  version,
  info,
  download,
  downloadWithEvents,
  listTransfers,
  createVault,
  bundledPlugins,
} from '@doki-land/prometheus';
import type { MediaInfo, DownloadResult, ProgressEvent, TransferInfo } from '@doki-land/prometheus';

version(): string

info(url: string): Promise<MediaInfo>
download(url: string, outputDir: string): DownloadResult
downloadWithEvents(url: string, outputDir: string): DownloadResult & { events: ProgressEvent[] }
listTransfers(): TransferInfo[]
createVault(vaultPath: string, password: string): string
bundledPlugins(): Plugin[]
```

### `MediaInfo`

CamelCase wire shape shared with the native engine:

| Field | Type | Meaning |
|-------|------|---------|
| `url` | `string` | Request URL |
| `title` | `string \| null` | Human title when known |
| `contentType` | `string \| null` | HTTP `Content-Type` when known |
| `contentLength` | `number \| null` | Length in bytes when known |
| `filename` | `string \| null` | Suggested filename |
| `extractor` | `string` | kebab-case id, e.g. `generic-http`, `local-file` |

### Progress events

`downloadWithEvents` collects structured events from the **in-process** transfer backend (no sidecar process):

| `kind` | When |
|--------|------|
| `started` | Transfer begins (`totalBytes` when advertised) |
| `bytes` | Chunk progress |
| `finished` | Success (`path` set) |
| `failed` | Error after start (or during setup) |

`listTransfers()` currently returns `[{ id: 'simple', available: true }]`. Only backends **linked into** the `.node` addon are listed.

### How `info` chooses a path

1. Try bundled plugins (`local-file`, then `generic-http`) via Torch context.
2. Fall back to the native extractor registry inside the addon.

Download always goes through the native transfer layer.

## Local development in this monorepo

```bash
pnpm install
pnpm napi:build   # builds the .node for your host into frontends/prometheus-<short>/
pnpm --filter @doki-land/prometheus build
pnpm --filter @doki-land/prometheus test
```

Override the loaded binary for debugging:

```bash
set PROMETHEUS_NATIVE_NODE=E:\path\to\prometheus.win32-x64-msvc.node
```

## Related packages

| Package | When you need it |
|---------|------------------|
| [`@doki-land/prometheus-torch`](https://www.npmjs.com/package/@doki-land/prometheus-torch) | Direct access to JS / WASM evaluation (pulled in automatically) |
| [`@doki-land/prometheus-plugin`](https://www.npmjs.com/package/@doki-land/prometheus-plugin) | Authoring plugin types / loading workspace plugins |
| [`@doki-land/prometheus-harness`](https://www.npmjs.com/package/@doki-land/prometheus-harness) | Scaffold and test plugins from your coding agent |
| `@doki-land/prometheus-<os>-<cpu>` | Platform binary (usually installed for you) |

## Versioning

| Version | Meaning |
|---------|---------|
| `0.0.0` | npm name-hold stub |
| `0.0.x` | Development workspace / pre-release APIs |
| `≥ 0.1.0` | Intended first “real” tagged releases |

## Responsible use

Use only with content you have the right to download. Stored credentials belong to you.

## License

[CC0 1.0 Universal](../../License.md)
