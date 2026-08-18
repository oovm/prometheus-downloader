# 🧭 @doki-land/prometheus-skills

Agent **skills** (markdown) for Prometheus. This package is not an installer binary.

| Skill | Prompt / when |
|-------|----------------|
| Setup | `Install @doki-land/prometheus-skills and finish the Prometheus setup and configuration for this environment.` |
| Plugin | User wants to write, scaffold, or test a `@doki-land/prometheus-plugin-<id>` package; **or** the published product cannot handle a new host / inspect path |

The agent should **ask** at strategy forks, then run `npm` / `pnpm` / `bun` / Harness itself. Torch, Playwright, and headless browsers are **not** default. Cookies are **user-supplied** (IDE browser or paste); the agent must not read other apps’ cookie databases.

## What this package contains

| File | Role |
|------|------|
| [`SKILL.md`](./SKILL.md) | Product setup: ask, install `@doki-land/prometheus` + native addon, verify |
| [`references/`](./references/) | Setup guides: install, configure, verify, credentials (user-supplied cookies), upgrade, usage |
| [`plugin/SKILL.md`](./plugin/SKILL.md) | Plugin authoring: ask, Harness `create` / `test`, contract |
| [`plugin/references/`](./plugin/references/) | Plugin guides: explore, scaffold, implement, test, torch |

After `npm install @doki-land/prometheus-skills`, read those `SKILL.md` files. Optionally copy:

- Setup → `.cursor/skills/prometheus-skills/` (`SKILL.md` + `references/`)
- Plugin → `.cursor/skills/prometheus-plugin/` (`plugin/SKILL.md` + `plugin/references/` as that folder’s `SKILL.md` + `references/`)

Ask which destinations before copying.

## Product vs plugins

```bash
pnpm add @doki-land/prometheus
```

Plugins are a separate authoring path (`@doki-land/prometheus-harness`). Coverage of media **hosts** belongs in the native engine, not a pile of npm plugins.

Node **≥ 20**. Details: [`@doki-land/prometheus`](https://www.npmjs.com/package/@doki-land/prometheus), [`@doki-land/prometheus-plugin`](https://www.npmjs.com/package/@doki-land/prometheus-plugin), [`@doki-land/prometheus-harness`](https://www.npmjs.com/package/@doki-land/prometheus-harness).

## License

[CC0 1.0 Universal](../../License.md)
