# 🔥 Prometheus

**Prometheus** is a media acquisition toolkit for creators, coding agents, and local automation. It resolves media metadata, downloads files you are allowed to fetch, and keeps credentials on your machine. The public surface is Node-first: a CLI, a typed JavaScript API, and (later) MCP tools. Heavy lifting lives in a single per-platform native addon.

This repository is the monorepo behind the `@doki-land/prometheus*` packages on npm.

> **Primary setup path:** ask your coding agent  
> `Install @doki-land/prometheus-skills and finish the Prometheus setup and configuration for this environment.`  
> The skill installs the product packages and walks remaining configuration. You should not have to memorize a long list of one-off CLI scripts.

## Why this shape

Most download tooling either dumps everything into one giant script host, or ships a pile of side binaries next to Node. Prometheus takes a different cut:

| Layer | Package / artifact | Job |
|-------|--------------------|-----|
| Product entry | `@doki-land/prometheus` | CLI + Node API users actually call |
| Script / WASM host | `@doki-land/prometheus-torch` | Evaluate platform JavaScript and instantiate WASM **in this Node process** |
| Plugin contract | `@doki-land/prometheus-plugin` | Load `@doki-land/prometheus-plugin-*` modules for inspect / match |
| Plugin authoring | `@doki-land/prometheus-harness` | MCP + CLI for scaffolding and testing plugins (**no LLM bundled**) |
| Native engine | `@doki-land/prometheus-<os>-<cpu>` | **One** `.node` file per platform: extractors, transfer, vault hooks |

Transfer, HTTP probing, local `file:` copy, and progress events all run **inside that one `.node`**. Prometheus does not ship a second download CLI, does not spawn PATH tools as the transfer implementation, and does not pack multi-platform binaries into the main JS package.

## Status

Workspace packages currently ship as **`0.0.1`** (development). Registry name-hold stubs use **`0.0.0`**. Treat APIs as unstable until a `≥0.1.0` release is tagged.

What works today on the happy path:

- `prometheus --version`
- `info` / `download` for direct HTTP(S) file URLs (native engine)
- `downloadWithEvents` for structured progress (`started` / `bytes` / `finished` / `failed`)
- `file:` metadata via the bundled local-file plugin (and native extractor fallback)
- Torch `evaluateJavascript` / `instantiateWasm`
- Harness plugin list / create / test / fetch helpers

What is intentionally thin or stubbed: product MCP server, multi-connection Range transfer, real vault encryption, and **broad media-site coverage** (more hosts still to land in Rust extractors).

## Quick start (after install)

```bash
pnpm add @doki-land/prometheus
# or: npm install @doki-land/prometheus
```

```bash
npx prometheus --version
npx prometheus info "https://example.com/clip.bin"
npx prometheus download "https://example.com/clip.bin" -o ./out
```

```ts
import { version, info, download, downloadWithEvents, listTransfers } from '@doki-land/prometheus';

console.log(version());

const media = await info('https://example.com/clip.bin');
// { url, title, contentType, contentLength, filename, extractor }

const result = download(media.url, './out');
// { path, bytesWritten, filename }

const withEvents = downloadWithEvents(media.url, './out');
// { path, bytesWritten, filename, events: ProgressEvent[] }

console.log(listTransfers());
// [{ id: 'simple', available: true }]
```

Node **≥ 20**. The matching platform optional dependency must resolve so the `.node` addon can load. In this monorepo, run `pnpm napi:build` on your host OS first.

## Monorepo layout

```text
prometheus/
  backends/          Rust crates (publish = false; linked into the napi cdylib)
  frontends/         npm packages (@doki-land/prometheus*)
  scripts/           napi build + publish helpers
  .github/workflows/ CI on master/dev; npm publish on v* tags
```

| Frontend package | Role |
|------------------|------|
| [`@doki-land/prometheus`](./frontends/prometheus) | CLI / Node API |
| [`@doki-land/prometheus-torch`](./frontends/prometheus-torch) | JS + WASM runtime |
| [`@doki-land/prometheus-plugin`](./frontends/prometheus-plugin) | Plugin types + workspace loader |
| [`@doki-land/prometheus-harness`](./frontends/prometheus-harness) | Plugin authoring MCP / CLI |
| [`@doki-land/prometheus-plugin-generic-http`](./frontends/prometheus-plugin-generic-http) | Direct HTTP(S) inspect plugin |
| [`@doki-land/prometheus-plugin-local-file`](./frontends/prometheus-plugin-local-file) | `file:` inspect plugin |
| `@doki-land/prometheus-<os>-<cpu>` | Per-platform native addon only |

Rust crates under `backends/` are **not** published to crates.io. They exist to build the napi cdylib cleanly (`prometheus-types`, `extractors`, `downloader`, `credential`, `processor`, `scheduler`, `napi`).

## Development

```bash
pnpm install
pnpm napi:build          # writes frontends/prometheus-<short>/prometheus.<triple>.node
pnpm build               # native + JS packages
cargo test --workspace   # Rust
pnpm test                # Node tests
pnpm fmt                 # Biome + rustfmt
pnpm fmt:check
```

CI runs format checks, `cargo test --workspace`, `pnpm build`, and `pnpm test` on `master` and `dev`.

## Responsible use

Use Prometheus only with content you have the right to download. Stored credentials belong to you; the tool does not claim platform copyrights on your behalf. Local decrypt / remux features (when they land) are for files already on your disk.

## License

[CC0 1.0 Universal](./License.md) — public domain dedication. Attribution is appreciated but not required.
