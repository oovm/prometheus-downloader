# upgrade (on demand)

Optional packages **after** product verify. Default setup stays `@doki-land/prometheus` + one native addon.

## When

Only when the user already asked for one of these. Do **not** propose them at the end of a normal install.

| Package | Propose when | Do not propose when |
|---------|--------------|---------------------|
| `@doki-land/prometheus-harness` | They are **writing or testing plugins**, **or** the published product cannot handle a new host / inspect path and they agreed to explore | They only want CLI / Node API on URLs the current release already handles |
| `@doki-land/prometheus-torch` | They **explicitly** said a host must run platform JavaScript or WebAssembly | “Make downloads more reliable”, unknown errors, or you want a runtime “just in case” |
| Playwright / Puppeteer / headless Chrome | They **explicitly** asked for a real-browser session | Ever as a default, product dependency, silent extra, or a way to harvest cookies |

User-supplied cookies (IDE browser or paste) are **not** an upgrade package. See [credentials.md](credentials.md). Do not install a browser stack to obtain cookies.

## Harness (plugin authoring)

Ask first: “Install Harness for plugin authoring in this project?” If yes:

```bash
pnpm add -D @doki-land/prometheus-harness
# or: npm install -D @doki-land/prometheus-harness
```

Then follow the plugin authoring skill: [plugin/SKILL.md](../plugin/SKILL.md) (scaffold, implement, test). The Harness package README (`@doki-land/prometheus-harness`) has CLI / MCP flags. Harness may depend on Torch so **authors** can test script-oriented plugins. That still must not be added to `@doki-land/prometheus`.

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
| User wanted a host the **published** version cannot inspect | Old release has no extractor / plugin for that path | Ask to create a plugin and start exploration ([plugin/references/explore.md](../plugin/references/explore.md)). Do not add Torch/Playwright as a substitute; do not pretend extra npm plugins are product site coverage |
| Host needs a login session | Public API is not enough; they have not supplied cookies | Follow [credentials.md](credentials.md). Do not install Playwright to fetch cookies |
| Harness MCP vs product `prometheus mcp` | Different surfaces | Product MCP is the download tools (may be stubbed). Harness is plugin authoring. Ask which they meant |

## Stop and ask

- Any request that would install Playwright, Puppeteer, or a browser
- Site failures: missing native extractor vs **exploration plugin** vs Torch — if the published version cannot handle the path, ask to create a plugin ([plugin/references/explore.md](../plugin/references/explore.md))
- Login wall on a session host — [credentials.md](credentials.md), not Playwright
- Adding Harness into a production app that only calls `info` / `download`
