# explore (published product cannot handle this path)

When the **already published** product (`@doki-land/prometheus` + its native extractors + bundled plugins on the registry) cannot inspect or download a URL the user is allowed to fetch, **stop and ask** to start exploration with a **new plugin**. Do not only say “wait for a native extractor”, and do not silently add Torch or a browser.

Long-term **host coverage** still belongs in the native engine. A plugin here is an **authoring / exploration** unit (Harness), not a substitute for stacking npm packages as the product’s site list.

## When

- `prometheus info` / `download` (or `harness test` on `generic-http` / `local-file`) fails or returns the wrong extractor for a **new host or inspect path**
- The installed version is a **published** npm release (or an older workspace tag), and that cut has no extractor / plugin for this path
- They asked to handle a URL that existing `@doki-land/prometheus-plugin-*` packages do not claim

Do **not** use this guide for: native `.node` missing, Node < 20, network 404 on a URL that `generic-http` should handle, or “make downloads faster”.

## Ask first (required)

One or two questions, then stop until they answer:

1. The published product cannot handle this URL/host. **Create a new plugin** in the Prometheus workspace to start exploration (`npx @doki-land/prometheus-harness create <id>`)?
2. If yes: kebab-case **id** and one **sample URL** they are allowed to fetch.

If they say no: leave the product as-is; explain that shipping that host in the engine is a native-extractor change. Do not install Torch or Playwright as a consolation.

If they say yes: continue with [scaffold.md](scaffold.md) → [implement.md](implement.md) → [test.md](test.md). Propose Torch only via [torch.md](torch.md) when they already need platform JS / WASM.

## Steps (after they agree)

1. Confirm workspace (`pnpm-workspace.yaml` + `frontends/`) and `PROMETHEUS_WORKSPACE` if needed.
2. `npx @doki-land/prometheus-harness create <id>`
3. Narrow `matches` to **this** host/path (do not claim all `https://`).
4. Implement `inspect` against public APIs / files they may fetch. Plugin does not download bytes to disk.
5. `npx @doki-land/prometheus-harness test <id> "<url>"`

Keep PRs human-reviewed. Do not publish the exploration plugin as if it were product-wide site coverage.

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| Agent skipped asking and created a plugin | Violated ask-first | Stop; confirm they want the package; delete only if they say so |
| Agent told them only “coverage is native” and stopped | Missed this guide | Ask the create-plugin question above |
| `generic-http` already matches the URL | Direct file; not a new inspect path | Do not scaffold; debug network / `-o` instead |
| They wanted YouTube / Bilibili “support” as an npm install | Product coverage ≠ plugin pile | Ask: exploration plugin in **this clone**, or wait for a native extractor? Do not `npm i` extra site plugins into the product |

## Stop and ask

- Published vs this-clone: if they are on registry-only CLI with no workspace, ask whether to clone/open this repo before `create`
- Sample URL missing or they cannot fetch it
- Torch / Playwright suggested to “make the old version work” — refuse unless they explicitly need JS/WASM or a browser session
