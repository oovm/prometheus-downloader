# upgrade (on demand)

Optional packages **after** product verify. Default setup stays `@doki-land/prometheus` + one native addon.

## When

Only when the user already asked for one of these. Do **not** propose them at the end of a normal install.

| Package | Propose when | Do not propose when |
|---------|--------------|---------------------|
| `@doki-land/prometheus-harness` | They are **writing or testing plugins** | They only want CLI / Node API downloads |
| `@doki-land/prometheus-torch` | They **explicitly** said a host must run platform JavaScript or WebAssembly | “Make downloads more reliable”, unknown errors, or you want a runtime “just in case” |
| Playwright / Puppeteer / headless Chrome | They **explicitly** asked for a real-browser session | Ever as a default, product dependency, or silent extra |

## Harness (plugin authoring)

Ask first: “Install Harness for plugin authoring in this project?” If yes:

```bash
pnpm add -D @doki-land/prometheus-harness
# or: npm install -D @doki-land/prometheus-harness
```

Then point them at the Harness package README (`@doki-land/prometheus-harness`) for `list` / `create` / `test` / `mcp`. Harness may depend on Torch so **authors** can test script-oriented plugins. That still must not be added to `@doki-land/prometheus`.

## Torch (platform JS / WASM)

Ask first, and only if they already mentioned JS/WASM: “Install `@doki-land/prometheus-torch` as an upgrade (not a product dependency)?” If yes:

```bash
pnpm add @doki-land/prometheus-torch
# or: npm install @doki-land/prometheus-torch
```

Do **not** add Torch to the product package’s `dependencies`. Read the Torch package README for `TorchRuntime`. Torch does not embed a browser.

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| Agent added Torch during product install | Violated default path | Remove it from the product install; apologize; re-verify without it |
| User wanted “better site coverage” | Coverage is more **media hosts** in the engine | Do not install plugins/Torch as a substitute; explain and ask if they actually need JS/WASM |
| Harness MCP vs product `prometheus mcp` | Different surfaces | Product MCP is the download tools (may be stubbed). Harness is plugin authoring. Ask which they meant |

## Stop and ask

- Any request that would install Playwright, Puppeteer, or a browser
- Site failures that might be missing extractors vs needing Torch — ask which they believe they need
- Adding Harness into a production app that only calls `info` / `download`
