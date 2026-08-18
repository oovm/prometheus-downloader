# 🧭 @doki-land/prometheus-skills

An **agent skill** for Prometheus setup: markdown your coding agent reads, not an installer binary.

> Ask your coding agent:  
> `Install @doki-land/prometheus-skills and finish the Prometheus setup and configuration for this environment.`

The agent should **ask** at strategy forks (CLI vs this-repo development, package manager, local vs global, …), then install `@doki-land/prometheus` plus the matching `@doki-land/prometheus-<os>-<cpu>` addon using `npm` / `pnpm` / `bun`. Torch, Harness, Playwright, and headless browsers are **not** default.

## What this package contains

| File | Role |
|------|------|
| [`SKILL.md`](./SKILL.md) | When to use, hard rules, questions to ask, dispatch |
| [`references/install.md`](./references/install.md) | Product install commands and failure handling |
| [`references/configure.md`](./references/configure.md) | Output dir, napi build, CI vs local |
| [`references/verify.md`](./references/verify.md) | `--version`, `info` / `download` smoke |
| [`references/upgrade.md`](./references/upgrade.md) | Harness / Torch only when asked |
| [`references/usage.md`](./references/usage.md) | Shortest CLI and Node API |

There is no `bin` that installs other packages. After `npm install @doki-land/prometheus-skills`, open `node_modules/@doki-land/prometheus-skills/SKILL.md`. Optionally copy `SKILL.md` and `references/` into `.cursor/skills/prometheus-skills/` (project) or `~/.cursor/skills/prometheus-skills/` (personal).

## Product packages (installed by the agent, not by this skill)

```bash
pnpm add @doki-land/prometheus
# or: npm install @doki-land/prometheus
```

Node **≥ 20**. Details: [`@doki-land/prometheus`](https://www.npmjs.com/package/@doki-land/prometheus).

## License

[CC0 1.0 Universal](../../License.md)
