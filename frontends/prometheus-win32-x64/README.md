# 🟩 @doki-land/prometheus-win32-x64

Optional **native N-API addon** for `@doki-land/prometheus` on **Windows x64**.

This package exists so the main JavaScript package stays portable: `@doki-land/prometheus` never ships every OS binary inside one tarball. Instead, npm/pnpm selects this package through `optionalDependencies` when you install on win32/x64.

## What is inside

| Artifact | Role |
|----------|------|
| `prometheus.win32-x64-msvc.node` | The **only** native binary in this package |

That single `.node` file is the Prometheus engine for this platform: URL extractors, HTTP(S) transfer, local `file:` copy, progress events, and credential vault hooks. There is **no** second executable, no download CLI sidecar, and no PATH tool wrapper.

## Install

You usually do **not** install this package directly:

```bash
pnpm add @doki-land/prometheus
```

npm will attempt to install `@doki-land/prometheus-win32-x64` automatically on matching machines. Direct install is fine for debugging:

```bash
pnpm add @doki-land/prometheus-win32-x64
```

### Platform constraints

```json
{ "os": ["win32"], "cpu": ["x64"] }
```

Triple / filename: `win32-x64-msvc` → `prometheus.win32-x64-msvc.node`.

## How the product loads it

`@doki-land/prometheus` resolves candidates in order roughly like:

1. `PROMETHEUS_NATIVE_NODE` env override (absolute path to a `.node`)
2. `require.resolve('@doki-land/prometheus-win32-x64/package.json')` sibling binary
3. Workspace / monorepo `frontends/prometheus-win32-x64/` after `pnpm napi:build`

If nothing loads, run a local build from the monorepo:

```bash
pnpm napi:build
```

## Versioning & publishing

Built and published **per platform** from CI (`publish-npm` workflow). Version tracks the product line (`0.0.x` while unstable). Do not copy foreign-platform `.node` files into this folder.

## Responsible use

Use only with content you have the right to download. Stored credentials belong to you.

## License

[CC0 1.0 Universal](../../License.md)
