---
name: prometheus-plugin
description: >-
  Guides coding agents through authoring a Prometheus plugin
  (@doki-land/prometheus-plugin-<id>) with Harness. Asks before choosing
  workspace, plugin id, sample URL, and whether Torch is needed, then scaffolds
  with prometheus-harness create, implements matches/inspect, and tests.
  Use when the user wants to write, scaffold, or test a Prometheus plugin,
  mentions @doki-land/prometheus-harness, create_plugin, or
  frontends/prometheus-plugin-*, or when the published product cannot handle
  a new host / inspect path and exploration should start with a plugin. Does not install Torch or Playwright unless
  the user explicitly needs platform JS/WASM or a real browser.
---

# Prometheus plugin authoring

This is an **agent skill** (markdown). It does not install packages for you. Read it, **ask** at forks, then run Harness / editor commands yourself.

Product setup (CLI / native) is a different skill: [../SKILL.md](../SKILL.md). Do not mix product `prometheus mcp` with Harness.

## Hard rules

1. **Ask before strategy forks.** Plugin id, workspace root, sample URL, and Torch are not guessable. Use AskQuestion (1–2 questions per turn) or ask the same options in the user's language.
2. A plugin only answers **matches** + **inspect** (`MediaInfo`). Transfer and disk IO stay in the native `@doki-land/prometheus-<os>-<cpu>` addon. Do not turn the plugin into a downloader.
3. **Tool coverage means more media hosts** in the native engine. Shipped JS plugins (`generic-http`, `local-file`) are a **contract / Harness verification** surface — not the product’s site list. If they asked for a new **site**, say that. **If the published product cannot handle this host or inspect path**, do not stop at “wait for native”: ask to **create a plugin and start exploration** ([references/explore.md](references/explore.md)). Do not silently stack plugins or add Torch.
4. **Never** default-install Playwright, Puppeteer, or a headless browser. Propose `@doki-land/prometheus-torch` only if they already said inspect must run platform JS / WASM.
5. Harness `fetch_script` is HTTP GET of text. `fetch_wasm` compiles and lists exports. Do **not** deobfuscate scripts or decompile WASM. `capture_page` and `submit_pr` are stubs — use the host `gh` for PRs.
6. Merges stay **human-reviewed**. Harness does not embed an LLM and does not auto-merge.

## Ask first (required)

Skip a question only when this conversation already answered it.

### 1. Intent

- Author / test a **plugin** in a Prometheus workspace
- They only wanted the **product CLI** → switch to [../SKILL.md](../SKILL.md) and stop this skill

If they said “add support for site X”, or `info` / `download` failed on a URL the **published** packages do not handle: follow [references/explore.md](references/explore.md). Ask to create a plugin and start exploration. Do not assume a native-only wait, and do not assume they wanted a product npm extra.

### 2. Workspace

- This Prometheus clone (directory with `pnpm-workspace.yaml` + `frontends/`)
- Another folder — set `PROMETHEUS_WORKSPACE` to that root after they confirm

### 3. Plugin id and URL

- kebab-case id starting with a letter (directory `frontends/prometheus-plugin-<id>/`, package `@doki-land/prometheus-plugin-<id>`)
- One URL they are allowed to fetch, for `matches` / `inspect` tests

Do not invent an id from a marketing name without confirming.

### 4. Torch

- Inspect is HTTP / `file:` / static JSON only → **do not** install Torch
- They explicitly need to evaluate platform JS or instantiate WASM → ask, then see [references/torch.md](references/torch.md)

After answers, follow the guides. Do not skip test.

## Guides

| Guide | When | File |
|-------|------|------|
| **scaffold** | Workspace + id known | [references/scaffold.md](references/scaffold.md) |
| **explore** | Published product cannot handle this host / inspect path | [references/explore.md](references/explore.md) |
| **implement** | After `create`, before claiming it works | [references/implement.md](references/implement.md) |
| **test** | After implement | [references/test.md](references/test.md) |
| **torch** | Only if they need platform JS / WASM | [references/torch.md](references/torch.md) |

## Cursor skill copy

Ask before copying. Copy **this folder** (`plugin/SKILL.md` + `plugin/references/`) to:

- Project: `.cursor/skills/prometheus-plugin/`
- Personal: `~/.cursor/skills/prometheus-plugin/`

## Done when

- `npx @doki-land/prometheus-harness list` shows the new id
- `test <id> <url>` reports `matches` + `inspect` success on a URL they chose
- They know downloads still go through the native addon
- Torch was not added unless they asked for JS / WASM
