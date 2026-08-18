# usage

Shortest product usage after verify. Full API: the `@doki-land/prometheus` package README (npm or `frontends/prometheus/readme.md` in this clone). Do not send users to unpublished internal notes.

## CLI

```bash
prometheus --version
prometheus info "<url>"
prometheus download "<url>" -o <dir>
```

Examples:

```bash
npx prometheus info "https://example.com/clip.bin"
npx prometheus download "https://example.com/clip.bin" -o ./out
```

`info` prints `MediaInfo` JSON. `download` prints `DownloadResult` JSON (`path`, `bytesWritten`, `filename`). `prometheus mcp` is the product MCP server (stub in current builds) — not Harness.

## Node API

```ts
import { version, info, download, downloadWithEvents, listTransfers } from '@doki-land/prometheus';

version(); // string, matches the native addon

const media = await info('https://example.com/clip.bin');
// { url, title, contentType, contentLength, filename, extractor }

const result = download(media.url, './out');
// { path, bytesWritten, filename }

downloadWithEvents(media.url, './out');
// result + events: started | bytes | finished | failed

listTransfers();
// [{ id: 'simple', available: true }, { id: 'native-range', available: true }]
```

`extractor` is kebab-case (for example `generic-http`, `local-file`).

## Requirements that stay true

- Node ≥ 20
- Matching `@doki-land/prometheus-<os>-<cpu>` must load
- Use only URLs / files the user is allowed to fetch

Hosts that need a logged-in session may require **user-supplied cookies**. The CLI has **no** cookies flag today; staging is env / a local file — [credentials.md](credentials.md).

Torch (`TorchRuntime`) is **not** exported from the product package. See [upgrade.md](upgrade.md) if they explicitly need platform JS / WASM.
