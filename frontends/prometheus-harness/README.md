# @doki-land/prometheus-harness

Tools for authoring Prometheus strategy plugins. No model is bundled: your coding agent calls these over MCP, or you run the CLI yourself.

```text
npx @doki-land/prometheus-harness mcp
npx @doki-land/prometheus-harness list
npx @doki-land/prometheus-harness create <id>
npx @doki-land/prometheus-harness test <id> <url>
```

Cursor MCP example (local checkout):

```json
{
  "mcpServers": {
    "prometheus-harness": {
      "command": "node",
      "args": ["frontends/prometheus-harness/bin/prometheus-harness.js", "mcp"],
      "env": { "PROMETHEUS_WORKSPACE": "." }
    }
  }
}
```

Use only with content you have the right to download. Stored credentials belong to you.
