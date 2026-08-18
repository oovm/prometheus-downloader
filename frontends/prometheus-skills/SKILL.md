---
name: prometheus-skills
description: >-
  Guides coding agents through Prometheus setup for the current environment.
  Asks the user before choosing a strategy, then the agent installs
  @doki-land/prometheus plus the matching @doki-land/prometheus-<os>-<cpu>
  native addon, and walks configure/verify. Use when the user says to install @doki-land/prometheus-skills,
  finish the Prometheus setup and configuration for this environment, or needs
  the Prometheus CLI or Node API installed. Does not install Torch, Playwright,
  or a headless browser unless the user explicitly asks.
---

# Prometheus setup

This package is an **agent skill** (markdown). It does not install other packages. You (the coding agent) read these files, **ask** at strategy forks, then run `npm` / `pnpm` / `bun` yourself.

Primary user prompt:

```text
Install @doki-land/prometheus-skills and finish the Prometheus setup and configuration for this environment.
```

## Hard rules

1. **Ask before strategy forks.** Do not guess package manager, global vs local, product vs Harness, this-machine vs CI, or how to fix a missing native addon. Use AskQuestion when available (1–2 questions per turn). Otherwise ask the same options in the user's language.
2. **Product default path only** until the user says otherwise: `@doki-land/prometheus` + current-platform `@doki-land/prometheus-<os>-<cpu>`.
3. **Never** add `@doki-land/prometheus-torch`, Playwright, Puppeteer, or a headless browser as a product dependency. Do not install them by default. Propose Torch **only** if the user already said a host needs to run platform JS / WASM.
4. Run the install/verify commands in this skill yourself. Do not wrap them in a custom installer from this package.
5. Coverage is **more media hosts** in the engine, not a pile of plugins.

## Bootstrap

If you cannot see this file yet, add the skill package (docs only — not the product CLI):

```bash
npm install @doki-land/prometheus-skills
# or: pnpm add @doki-land/prometheus-skills
# or: npm install -g @doki-land/prometheus-skills
```

Then **read** `node_modules/@doki-land/prometheus-skills/SKILL.md` (or the matching path under a global prefix). Ask before copying into Cursor:

- Project skill: `.cursor/skills/prometheus-skills/` (copy `SKILL.md` + `references/`)
- Personal skill: `~/.cursor/skills/prometheus-skills/`

Do not invent a postinstall hook.

## Ask first (required)

Skip a question only when the user already answered it in this conversation. Otherwise **stop and ask**. Suggested order, **one or two items per turn**:

### 1. Role

How will they use Prometheus?

- End user: CLI and/or Node API from the registry
- This repository: local development (`pnpm install` + `pnpm napi:build`)

### 2. Install scope

What should be installed?

- Product default only (`@doki-land/prometheus` + current platform native)
- Product + Harness (they are **writing plugins**)

Do **not** offer Torch on this question. Mention Torch later only if they already said the site must run JS / WASM.

### 3. Runtime platform

Is `process.platform` / `process.arch` the machine that will **run** Prometheus?

- Yes, this machine
- No: CI image, another OS/CPU, or cross-compile (stop; do not install this host's native package into a foreign runtime)

Confirm with:

```bash
node -p "process.versions.node + ' ' + process.platform + '-' + process.arch"
```

Need **Node ≥ 20**. Supported natives: `win32-x64`, `win32-arm64`, `darwin-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64` (gnu). Alpine/musl: stop and ask.

### 4. Package manager and location

- npm / pnpm / bun / not sure (check PATH, then confirm)
- Project-local dependency / global CLI / npx only (no lockfile change)

### 5. Native-load failure (only if verify fails)

- Install the matching `@doki-land/prometheus-<os>-<cpu>` explicitly
- This clone: run `pnpm napi:build`
- Leave the JS package for now (tell them `info` / `download` will fail until the addon loads)

After answers, follow the matching guide. Do not skip verify.

## Guides

| Guide | When | File |
|-------|------|------|
| **install** | After role / pm / scope / platform are known | [references/install.md](references/install.md) |
| **configure** | After install, before treating the CLI as ready | [references/configure.md](references/configure.md) |
| **verify** | After configure; required before claiming success | [references/verify.md](references/verify.md) |
| **upgrade** | Only if they asked for plugins or platform JS/WASM | [references/upgrade.md](references/upgrade.md) |
| **usage** | After verify, or when they only want commands | [references/usage.md](references/usage.md) |

## Done when

- Node ≥ 20
- `prometheus --version` prints a version
- Native addon for this OS/CPU loads
- User knows `info` / `download` and that Torch is optional

If anything in that list fails, stop and ask — do not widen the install to Torch or a browser.
