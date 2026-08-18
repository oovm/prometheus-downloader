# test

Prove `matches` / `inspect` on a URL the user chose.

## When

- `src/index.ts` is implemented (or they asked to verify an existing plugin)
- You know a URL they are allowed to fetch

## Steps

From the workspace root:

```bash
npx @doki-land/prometheus-harness list
npx @doki-land/prometheus-harness test <id> "<url>"
```

Examples (bundled verification plugins, not new site coverage):

```bash
npx @doki-land/prometheus-harness test generic-http "https://example.com/a.bin"
npx @doki-land/prometheus-harness test local-file "file:///tmp/clip.bin"
```

Optional: `npx @doki-land/prometheus-harness diff <id>` to review the package git diff before a human PR.

MCP (same tools, no LLM inside Harness):

```bash
npx @doki-land/prometheus-harness mcp
```

Configure the host with `PROMETHEUS_WORKSPACE` pointing at the workspace root. Product `prometheus mcp` is **not** this server.

## Failure

| Symptom | Likely cause | What to do |
|---------|--------------|------------|
| Plugin not in `list` | Wrong workspace; package.json missing `prometheusPlugin`; dir not `prometheus-plugin-<id>` | Ask; confirm `PROMETHEUS_WORKSPACE` |
| `matches` false | `matches` too strict, or wrong URL | Ask for the real URL; do not broaden to all HTTPS |
| `inspect` throws / empty `MediaInfo` | Network, URL, or incomplete implement | Ask for a URL they can fetch; do not install Torch unless they said JS/WASM is required |
| `capture_page` does nothing useful | Stub; no Playwright dependency | Do not add Playwright unless they explicitly asked for a browser session |
| Want a GitHub PR | `submit_pr` is a stub | Use host `gh`; keep human review |

## Stop and ask

- Test URL missing or they do not have the right to fetch it
- Failure might be “needs Torch” vs “bad matcher” — ask which they believe
- They want to publish to npm — confirm versioning (`0.0.x` until ready) and human review; do not publish from this skill by default
