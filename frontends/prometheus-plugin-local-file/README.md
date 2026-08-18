# 📁 @doki-land/prometheus-plugin-local-file

Prometheus plugin for local **`file:`** URLs. It reads metadata from the filesystem (`stat`) so agents and CLIs can treat a path on disk the same way they treat a remote media URL at the `info` layer.

Copying / download into an output directory is still performed by the native engine when you call `download` (the addon understands `file:` as well). This plugin exists so the **JavaScript plugin contract** and Harness can verify local-file behavior without going through napi first.

## Install

```bash
pnpm add @doki-land/prometheus-plugin-local-file
```

Usually installed transitively with `@doki-land/prometheus`. Marked as:

```json
{ "prometheusPlugin": { "id": "local-file" } }
```

## Behavior

| Method | Behavior |
|--------|----------|
| `matches(url)` | `true` when the trimmed URL lowercases to a `file:` scheme |
| `inspect(url, ctx)` | `fileURLToPath` → `fs.statSync`; requires a regular file |

Returned fields:

| Field | Source |
|-------|--------|
| `url` | Original `file:` URL |
| `title` / `filename` | Basename of the path |
| `contentLength` | `stat.size` |
| `contentType` | `null` (no MIME sniffing in this thin plugin) |
| `extractor` | `"local-file"` |

Throws if the path is missing or not a file. `ctx.torch` is unused.

## Example

```ts
import { pathToFileURL } from 'node:url';
import plugin from '@doki-land/prometheus-plugin-local-file';
import { TorchRuntime } from '@doki-land/prometheus-torch';

const url = pathToFileURL('/tmp/clip.bin').href;
plugin.matches(url); // true

const media = await plugin.inspect(url, { torch: new TorchRuntime() });
// media.extractor === 'local-file'
// media.contentLength === file size
```

Product package (plugin runs before native fallback):

```ts
import { info } from '@doki-land/prometheus';
import { pathToFileURL } from 'node:url';

const media = await info(pathToFileURL('./clip.bin').href);
```

## Testing with Harness

```bash
npx @doki-land/prometheus-harness test local-file "file:///absolute/path/to/clip.bin"
```

## Responsible use

Use only with files you have the right to read. Local decrypt features elsewhere in Prometheus (when present) are likewise limited to files already on your disk.

## License

[CC0 1.0 Universal](../../License.md)
