# 🌐 @doki-land/prometheus-plugin-generic-http

Prometheus plugin for **direct** `http://` and `https://` file URLs. It implements the `@doki-land/prometheus-plugin` contract: `matches` + `inspect`. Actual bytes on disk are still written by the native transfer backend inside the platform `.node` addon.

This package is a **thin verification surface** for Harness and the product bundle. Prefer growing HTTP coverage and transfer behavior in the Rust engine, not by cloning this plugin per host.

## Install

```bash
pnpm add @doki-land/prometheus-plugin-generic-http
```

Usually installed transitively with `@doki-land/prometheus`. Marked as:

```json
{ "prometheusPlugin": { "id": "generic-http" } }
```

## Behavior

| Method | Behavior |
|--------|----------|
| `matches(url)` | `true` when the trimmed URL starts with `http://` or `https://` |
| `inspect(url, ctx)` | Best-effort `HEAD` via `fetch`; fills `contentType`, `contentLength`, `filename` from headers / URL path |

Returned `MediaInfo.extractor` is always `"generic-http"`.

Filename resolution prefers `Content-Disposition` (`filename*` then `filename`), then the last path segment of the URL.

If `HEAD` fails, inspect still returns URL-derived metadata when possible (the **native** extractor additionally probes with `Range` / GET headers for hosts that reject `HEAD`).

`ctx.torch` is available but unused for plain direct files.

## Example

```ts
import plugin from '@doki-land/prometheus-plugin-generic-http';
import { TorchRuntime } from '@doki-land/prometheus-torch';

plugin.matches('https://cdn.example.com/a.bin'); // true

const media = await plugin.inspect('https://cdn.example.com/a.bin', {
  torch: new TorchRuntime(),
});
// media.extractor === 'generic-http'
```

From the product package:

```ts
import { info } from '@doki-land/prometheus';

const media = await info('https://cdn.example.com/a.bin');
```

## Testing with Harness

```bash
npx @doki-land/prometheus-harness test generic-http "https://example.com/file.bin"
```

## Responsible use

Use only with content you have the right to download.

## License

[CC0 1.0 Universal](../../License.md)
