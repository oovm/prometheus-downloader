# torch (on demand)

Optional runtime so a plugin can evaluate platform JavaScript or instantiate WebAssembly **in process**. Not a product dependency. Not a browser.

## When

Only if the user **already said** `inspect` must run platform JS or WASM. Do not suggest Torch to fix random `inspect` failures or to “add more sites”.

## Steps

Ask: “Install `@doki-land/prometheus-torch` for this plugin (upgrade, not a product dependency)?” If no, stop.

If yes, in the **plugin** package (or the workspace, as they prefer — ask):

```bash
pnpm add @doki-land/prometheus-torch
# or: npm install @doki-land/prometheus-torch
```

Harness already depends on Torch so `test` can supply `ctx.torch` when you run:

```bash
npx @doki-land/prometheus-harness test <id> "<url>"
```

In `inspect`, guard the optional context:

```ts
if (!ctx.torch) {
    throw new Error('This plugin needs @doki-land/prometheus-torch');
}
const token = ctx.torch.evaluateJavascript(/* source they have the right to run */);
```

Do **not** add Torch to `@doki-land/prometheus` `dependencies`. Read the Torch package README for `TorchRuntime` options (`timeoutMs`, sandbox).

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| `ctx.torch` missing during `test` | Not running via Harness, or old Harness | Use `prometheus-harness test`; ask before adding a product dependency |
| They asked for Playwright / Chrome | Different upgrade | Stop. Browser sessions are not this guide. Do not add them by default |
| Timeout / untrusted script | Torch is not a security sandbox | Tighten timeout; do not disable limits silently |

## Stop and ask

- They have not confirmed JS / WASM is required
- They want a headless browser instead of Torch
- They want Torch as a dependency of the product CLI package — refuse
