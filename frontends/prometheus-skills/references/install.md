# install

Install the **product default path**. Do this only after [SKILL.md](../SKILL.md) questions for role, platform, package manager, and local vs global are answered.

This skill does not ship an installer. Run the commands below with the package manager the user chose.

## When

- The user asked to finish Prometheus setup, **and**
- You know they want the CLI / Node API (or this clone), **and**
- You know which OS/CPU will run it.

Do **not** install while those are unknown.

## What to install

| Include | Package |
|---------|---------|
| Yes | `@doki-land/prometheus` |
| Yes (current runtime) | `@doki-land/prometheus-<os>-<cpu>` (usually via the product `optionalDependencies`) |
| No (default) | `@doki-land/prometheus-torch` |
| No (default) | `@doki-land/prometheus-harness` |
| No | Playwright, Puppeteer, headless Chrome |

Map `process.platform`-`process.arch` to the native package:

| Runtime | Package |
|---------|---------|
| `win32-x64` | `@doki-land/prometheus-win32-x64` |
| `win32-arm64` | `@doki-land/prometheus-win32-arm64` |
| `darwin-x64` | `@doki-land/prometheus-darwin-x64` |
| `darwin-arm64` | `@doki-land/prometheus-darwin-arm64` |
| `linux-x64` | `@doki-land/prometheus-linux-x64` |
| `linux-arm64` | `@doki-land/prometheus-linux-arm64` |

If the platform is missing from that table, or the host is Alpine/musl, **stop and ask**.

## Steps

### A. End user (registry)

1. Confirm Node ≥ 20:

```bash
node -p "process.versions.node"
```

2. Install with the **answered** package manager and scope. The product package pulls the matching native via `optionalDependencies`.

```bash
# project-local
npm install @doki-land/prometheus
pnpm add @doki-land/prometheus
bun add @doki-land/prometheus

# global CLI
npm install -g @doki-land/prometheus
pnpm add -g @doki-land/prometheus
```

3. If they chose **npx only**, skip the lockfile change. Later commands are `npx prometheus …`. The native addon still has to resolve when npx fetches the product package.

4. Do not `npm install` Torch, Harness, Playwright, or extra plugins “while you are here.”

### B. This repository (local development)

Do **not** install `@doki-land/prometheus` from the registry into the clone. From the monorepo root (directory with `pnpm-workspace.yaml`):

```bash
pnpm install
pnpm napi:build
```

`pnpm napi:build` writes **this host’s** `.node` into `frontends/prometheus-<os>-<cpu>/`. Then:

```bash
pnpm --filter @doki-land/prometheus build
node frontends/prometheus/bin/prometheus.js --version
```

If they also want Harness, that is an **upgrade** — see [upgrade.md](upgrade.md). Ask; do not add it here.

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| Node version error / engines | Node < 20 | Ask which Node ≥ 20 to use; stop |
| optional native missing after install | `optionalDependencies` omitted (`--omit=optional`), wrong OS/CPU, or musl | Ask: add the platform package explicitly, switch runtime, or JS-only (CLI will fail) |
| `Cannot find module '@doki-land/prometheus'` | Product not in this cwd / PATH | Confirm local vs global and cwd; do not silently switch |
| Addon missing **in this clone** | `pnpm napi:build` not run | Ask before building; then run it on the **host OS** |
| Install pulled Torch / Playwright | Agent added extras | Remove them. Product must not depend on Torch |

To add the native package explicitly (only after asking):

```bash
npm install @doki-land/prometheus-win32-x64
# substitute the row from the table for this machine
```

## Stop and ask

- Package manager or global vs local still unknown
- This host is not the runtime (CI / other arch)
- Native still missing after install — use SKILL.md question 5
- They mention writing plugins (Harness) or running site JS/WASM (Torch) — do not fold that into this step
