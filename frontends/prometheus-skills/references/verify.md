# verify

Prove the product CLI and native addon work. Required before telling the user setup is done.

## When

- Install (and configure) finished, **or**
- Native load failed and you need a clear error to ask about.

## Steps

1. Version (must print a version string):

```bash
prometheus --version
# or: npx prometheus --version
# this clone: node frontends/prometheus/bin/prometheus.js --version
```

2. Direct-URL smoke. Use a URL they are **allowed** to fetch (a file they host, or `https://example.com/…` only if they accept it may 404). Do not pick random media-site URLs.

```bash
prometheus info "https://example.com/clip.bin"
prometheus download "https://example.com/clip.bin" -o ./out
```

`info` prints JSON (`url`, `extractor`, …). `download` prints JSON (`path`, `bytesWritten`, `filename`). Direct HTTP files use extractor `generic-http`.

3. Optional Node API check (project-local install):

```js
import { version, info } from '@doki-land/prometheus';
console.log(version());
```

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| `prometheus: command not found` | Global bin not on PATH, or they used project-local | Ask: `npx prometheus`, or add a local script; do not silently `npm i -g` |
| `native addon not found` / cannot load `.node` | `optionalDependencies` not resolved; wrong platform package; this clone without `pnpm napi:build` | **Stop.** Ask SKILL.md question 5 |
| Version works, `info`/`download` throw | Addon loaded but network/URL invalid | Ask for a URL they can fetch; do not widen to Torch |
| `info` 404 / extractor error on a media host | No extractor for that host yet, or not a direct file | Explain coverage is **hosts in the engine**; do not install Torch or a browser unless they explicitly need JS/WASM |
| Works locally, fails in CI | Runner OS/CPU differs; optional native omitted | Ask before adding `--omit=optional` workarounds |

Typical native-missing message looks like: run `pnpm napi:build` (writes `frontends/prometheus-<os>-<cpu>/`). That hint is for **this clone**. On a registry install, add the matching `@doki-land/prometheus-<os>-<cpu>` package instead — after asking.

## Stop and ask

Native did not load — pick one, do not guess:

- Install `@doki-land/prometheus-<os>-<cpu>` explicitly
- This clone: `pnpm napi:build` on the host OS
- Leave JS installed; `info` / `download` will fail until the addon is present

Do not “fix” verify by installing Torch, Playwright, or a headless browser.
