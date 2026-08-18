# configure

Workspace / machine setup after install. Prometheus has **no** required config file: `download` takes `-o` per invocation. Do not invent a global config format.

## When

- Product packages are installed (or this clone is built), **and**
- You have not yet claimed the environment is ready.

## What to decide (ask if unknown)

Ask **one or two** of these if they change what you do:

- Preferred output directory for downloads in this project (example: `./out`)? There is no default on disk until they pass `-o`.
- Is this **local development** or **CI**?
- In this clone: has `pnpm napi:build` already produced a `.node` for **this** OS/CPU?

## Steps

### Local end user

1. Keep the output directory as a command flag. Example after they pick a folder:

```bash
prometheus download "<url>" -o ./out
```

2. Node must stay ≥ 20 on PATH for the same machine that will run the CLI.

3. If they need to point at a specific `.node` file (unusual; ask first):

```bash
# Unix
export PROMETHEUS_NATIVE_NODE=/absolute/path/to/prometheus.<triple>.node

# Windows cmd
set PROMETHEUS_NATIVE_NODE=C:\absolute\path\to\prometheus.<triple>.node
```

Do not set this in CI unless they asked for an override.

### This repository (local development)

From the monorepo root:

```bash
pnpm install
pnpm napi:build
pnpm --filter @doki-land/prometheus build
```

`pnpm napi:build` only writes the **current host** platform package. Cross-compiling for another OS is a separate conversation — ask; do not copy the wrong `.node`.

### CI

- If CI consumes **published** npm packages: rely on `optionalDependencies` for the runner’s OS/CPU. Do not run `pnpm napi:build` unless this job is the native-build job.
- If CI **builds this clone**: run `pnpm napi:build` on that runner OS, then JS build + tests. Do not assume a Windows `.node` works on Linux.

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| CLI works on a laptop, fails in CI | Different OS/CPU; optional native skipped | Ask: install the runner’s platform package, or build native on that OS |
| `PROMETHEUS_NATIVE_NODE` set but load fails | Path wrong, or file built for another triple | Ask before unsetting; do not guess a path |
| Developer expected a config file | None exists | Explain `-o` / Node API `outputDir`; do not create one |

## Stop and ask

- Output directory would overwrite something they care about
- Job is CI but you were about to run a local-only `napi:build`
- They want credentials / vault setup — that is not this skill’s default path; confirm before touching vault files
