# 🔦 @doki-land/prometheus-torch

**Torch** is an **optional upgrade** for Prometheus: a Node.js runtime for platform JavaScript and WebAssembly. Install it only when a media host requires running platform scripts (or WASM) to compute request parameters.

It is **not** a dependency of `@doki-land/prometheus`. Default installs should stay lean: product CLI + one per-OS `.node` addon. Stronger session tools (real browser / headless automation) are a further upgrade on the credential surface — also not default.

```bash
pnpm add @doki-land/prometheus-torch
# only after you need platform scripts / WASM
```

> Primary product setup is still:  
> `Install @doki-land/prometheus-skills and finish the Prometheus setup and configuration for this environment.`  
> Add Torch later when a site actually requires it.

## Why a separate package

| Default product | This upgrade |
|-----------------|--------------|
| Native extractors + transfer in one `.node` | Evaluate platform JS / WASM in-process |
| No second JS engine in Rust | `node:vm` + host `WebAssembly` |
| No Playwright by default | Does **not** embed a browser |

Torch is **not** a security sandbox. Treat untrusted platform scripts carefully (timeouts, least privilege). Do not use Torch as a substitute for a real browser session when a host requires one.

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
```

### Instantiate WebAssembly

```ts
const { instance } = await torch.instantiateWasm(bytes);
```

## Use with plugins

`PluginContext.torch` is **optional**. Authoring tools such as `@doki-land/prometheus-harness` may supply a Torch instance when testing script-oriented plugins. The product package does not.

```ts
import type { Plugin, PluginContext, MediaInfo } from '@doki-land/prometheus-plugin';

export async function inspect(url: string, ctx: PluginContext): Promise<MediaInfo> {
  if (!ctx.torch) {
    throw new Error('This plugin needs @doki-land/prometheus-torch — install that upgrade package');
  }
  const token = ctx.torch.evaluateJavascript(/* platform script source */);
  // return MediaInfo; transfer still happens in the native engine
}
```

## What Torch does **not** do

| Non-goal | Where it belongs |
|----------|------------------|
| Default product install | `@doki-land/prometheus` alone |
| High-concurrency download | Native transfer in `@doki-land/prometheus-<os>-<cpu>` |
| Credential vault | Product `createVault` / Rust credential crate |
| Headless browser / Stealth | Credential upgrade (later); not this package |

## Responsible use

Use only with content and scripts you have the right to run. Stored credentials belong to you.

## License

[CC0 1.0 Universal](../../License.md)
