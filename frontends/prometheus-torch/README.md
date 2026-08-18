# 🔦 @doki-land/prometheus-torch

**Torch** is Prometheus’s Node.js runtime for platform JavaScript and WebAssembly. When a site or media host ships a script (or a WASM module) that must run to produce request parameters, Torch evaluates it **in the current Node process** — the same process that already hosts the N-API engine.

You normally **do not** install Torch by itself. `@doki-land/prometheus` depends on it. Prefer:

> Install `@doki-land/prometheus-skills` and finish the Prometheus setup and configuration for this environment.

## Why a separate package

The product CLI should stay thin. Script evaluation is a distinct concern:

- Isolates the `vm` / `WebAssembly` surface behind a small, versioned API
- Lets plugin authors receive a `torch` handle in `PluginContext` without re-implementing context setup
- Keeps the Rust engine free of a second JavaScript interpreter

Torch is **not** a security sandbox. Treat untrusted platform scripts accordingly (timeouts, least privilege, future process isolation if needed). Do not use Torch as a substitute for a real browser session when a host requires one.

## Install

```bash
pnpm add @doki-land/prometheus-torch
# usually pulled in by: pnpm add @doki-land/prometheus
```

Requires **Node.js ≥ 20**.

## API

```ts
import { TorchRuntime, version } from '@doki-land/prometheus-torch';

version(): string

class TorchRuntime {
  evaluateJavascript(
    source: string,
    options?: {
      filename?: string;
      timeoutMs?: number;          // default 5000
      sandbox?: Record<string, unknown>;
    },
  ): unknown;

  instantiateWasm(
    bytes: BufferSource,
    imports?: WebAssembly.Imports,
  ): Promise<WebAssembly.WebAssemblyInstantiatedSource>;
}
```

### Evaluate JavaScript

```ts
const torch = new TorchRuntime();

const sum = torch.evaluateJavascript('1 + 2'); // 3

const withSandbox = torch.evaluateJavascript('greeting + "!"', {
  filename: 'snippet.js',
  timeoutMs: 2000,
  sandbox: { greeting: 'hello' },
});
```

Each call builds a fresh `vm` context from the provided sandbox. Values do not leak across calls unless you put them in the sandbox yourself.

Empty source throws `TypeError`. Timeouts throw from `vm.Script#runInContext`.

### Instantiate WebAssembly

```ts
import fs from 'node:fs';

const bytes = fs.readFileSync('./fixture.wasm');
const { instance, module } = await torch.instantiateWasm(bytes);
// instance.exports… depend on the module
```

This is the host `WebAssembly.instantiate` API. Torch does not decompile modules, rewrite imports, or patch browser globals.

## What Torch does **not** do

| Non-goal | Where it belongs instead |
|----------|---------------------------|
| High-concurrency HTTP download | Native transfer inside `@doki-land/prometheus-<os>-<cpu>` |
| Credential vault storage | `createVault` / Rust credential crate via the product package |
| Spawning a second JS engine (Deno, QuickJS, …) | Frozen out — use this Node process |
| Shipping exploit / signature-recovery cookbooks | Out of scope for the public package |

## Use with plugins

Plugin `inspect` receives a context object whose `torch` field matches the TorchRuntime methods:

```ts
import type { Plugin, PluginContext, MediaInfo } from '@doki-land/prometheus-plugin';

export async function inspect(url: string, ctx: PluginContext): Promise<MediaInfo> {
  const token = ctx.torch.evaluateJavascript(/* platform script source */);
  // return MediaInfo; transfer still happens in the native engine
}
```

Scaffold and test plugins with [`@doki-land/prometheus-harness`](https://www.npmjs.com/package/@doki-land/prometheus-harness).

## Versioning

Tracks the Prometheus workspace line (`0.0.x` while APIs are unstable). `version()` returns the package.json version string.

## Responsible use

Use only with content and scripts you have the right to run. Stored credentials belong to you.

## License

[CC0 1.0 Universal](../../License.md)
