# scaffold

Create a plugin package with Harness. Only after [SKILL.md](../SKILL.md) questions for workspace and id are answered.

## When

- They want a new `@doki-land/prometheus-plugin-<id>`
- The id is kebab-case, starts with a letter, and the directory does not already exist
- **Or** they agreed in [explore.md](explore.md): the published product cannot handle this host / inspect path, and they want a plugin to start exploration

## Steps

1. Node ≥ 20. Work in the workspace root (`pnpm-workspace.yaml` + `frontends/`), or export `PROMETHEUS_WORKSPACE` to that root.

2. Install Harness as a **dev** dependency of the workspace (ask pm if unknown):

```bash
pnpm add -D @doki-land/prometheus-harness
# or: npm install -D @doki-land/prometheus-harness
```

In this clone, Harness is already a workspace package — prefer:

```bash
npx @doki-land/prometheus-harness list
```

3. Scaffold (replace `<id>` with the answered id):

```bash
npx @doki-land/prometheus-harness create <id>
```

This writes `frontends/prometheus-plugin-<id>/` with `package.json`, `tsconfig.json`, `src/index.ts`, and a README. The package name is `@doki-land/prometheus-plugin-<id>`. `package.json` must contain:

```json
{ "prometheusPlugin": { "id": "<id>" } }
```

4. If they use pnpm workspaces, `pnpm install` at the repo root so the new package is linked. Then:

```bash
pnpm --filter @doki-land/prometheus-plugin-<id> build
```

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| `plugin id must start with a letter` | Invalid id | Ask for a kebab-case id (`my-host`, not `MyHost` / `_foo`) |
| `already exists` | Directory present | Ask: reuse that package, or pick another id |
| Harness cannot find `frontends/` | Wrong cwd / `PROMETHEUS_WORKSPACE` | Ask which folder is the workspace root |
| `create` is treated as product `prometheus mcp` | Wrong package | Harness is `@doki-land/prometheus-harness`, not `prometheus mcp` |

Do not run `create` in a random app folder that is not a Prometheus workspace.

## Stop and ask

- Id or workspace still unknown
- They wanted to extend `generic-http` / `local-file` instead of a new package
- They wanted a native extractor for site coverage **without** an exploration plugin — if the published version already failed on that URL, go to [explore.md](explore.md) and ask anyway
