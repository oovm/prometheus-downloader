# 🧩 prometheus-napi

**Node-API (napi-rs) cdylib** that exposes the Prometheus Rust engine to JavaScript. CI and `pnpm napi:build` compile this crate and copy the resulting binary into `frontends/prometheus-<os>-<cpu>/` as the sole `.node` file for that platform package.

Not published to crates.io — the npm artifact is the platform package, not this crate name.

## Exported functions (JavaScript names)

| Export | Role |
|--------|------|
| `version` | Engine / package version string |
| `info` | `Registry::builtin().inspect` → `JsMediaInfo` |
| `download` | Default `simple` transfer → `JsDownloadResult` |
| `downloadWithEvents` | Same transfer, plus collected `ProgressEvent`s |
| `listTransfers` | In-process backends linked into this addon |
| `createVault` | Credential vault create (encryption later) |

CamelCase fields on objects match the serde wire used in `prometheus-types`.

## Build

From the monorepo root:

```bash
pnpm napi:build
# → frontends/prometheus-<short>/prometheus.<triple>.node
```

Release builds for npm are produced per OS in GitHub Actions and published as `@doki-land/prometheus-<os>-<cpu>`.

## Hard constraints

- **One** `.node` per platform package
- Do not spawn or ship a second download/transcode CLI from this addon
- Keep JS bindings thin: behavior lives in `prometheus-extractors`, `prometheus-downloader`, `prometheus-credential`, …

## License

CC0-1.0 — see the repository `License.md`.
