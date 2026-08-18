# implement

Fill in `matches` / `inspect`. Types live in `@doki-land/prometheus-plugin` (see that package README). Do not copy unpublished internal notes.

## When

- Scaffold exists under `frontends/prometheus-plugin-<id>/`

## Contract

```ts
import type { MediaInfo, Plugin, PluginContext } from '@doki-land/prometheus-plugin';

matches(url: string): boolean
inspect(url: string, ctx: PluginContext): MediaInfo | Promise<MediaInfo>
```

`MediaInfo` (camelCase wire): `url`, `title`, `contentType`, `contentLength`, `filename`, `extractor` (kebab-case id, same as the plugin id).

`PluginContext.torch` is **optional**. Thin plugins must not require it. Use it only after [torch.md](torch.md).

Export `default`, `plugin`, or named `matches` / `inspect` (Harness / loader accept these). Keep `prometheusPlugin.id` in `package.json` aligned with `plugin.id`.

Directory name must stay `prometheus-plugin-<id>` so the loader does not pick up the contract package `prometheus-plugin`.

## Steps

1. Open `src/index.ts`. Replace the template `url.includes('<id>')` stub with a precise `matches` (scheme, host, path). Do not claim every `https://` URL — `generic-http` already does that.

2. Implement `inspect` to return honest metadata for URLs they are allowed to fetch. Typical thin plugin: `HEAD`/`GET` a public JSON or file URL, fill `filename` / `contentType`. Do not download the media body into a file; the native addon does transfer.

3. Rebuild:

```bash
pnpm --filter @doki-land/prometheus-plugin-<id> build
```

Loader prefers `dist/index.js`; local work may also load `src/index.ts` depending on the workspace loader.

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| Type errors on `MediaInfo` | Wrong field names | Use camelCase from the contract package; kebab-case only for `extractor` / `id` |
| Plugin “matches everything” | Copied `generic-http` | Narrow `matches`; ask what host/path they own |
| `inspect` writes files / opens many sockets | Plugin acting like a downloader | Strip that; return `MediaInfo` only |
| Needs `ctx.torch` and it is undefined | Torch not installed | Stop; ask — see [torch.md](torch.md). Do not add Playwright instead |

## Stop and ask

- Sample URL still unknown
- `matches` would overlap `generic-http` for all HTTP(S) URLs
- They asked to “restore signatures” or unpack minified platform scripts — refuse to turn Harness into that; keep `inspect` to documented public APIs
