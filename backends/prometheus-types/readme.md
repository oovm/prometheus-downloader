# 📦 prometheus-types

Shared **types, errors, and wire shapes** for the Prometheus Rust workspace. This is the base crate: every other engine crate depends on it. Nothing here opens sockets or touches the filesystem beyond pure helpers (filename sanitization / URL path decoding).

Not published to crates.io (`publish = false`). Linked into the napi cdylib that becomes `@doki-land/prometheus-<os>-<cpu>`.

## What lives here

| Item | Purpose |
|------|---------|
| `VERSION` | Crate / product version string (`CARGO_PKG_VERSION`) |
| `EXTRACTOR_*` / `TRANSFER_*` | Stable kebab-case ids (`generic-http`, `local-file`, `simple`, …) |
| `Error` / `Result` | `thiserror`-based public errors (no `anyhow` on the public surface) |
| `MediaInfo` | Inspect metadata (serde `camelCase`) |
| `DownloadResult` | Successful download summary |
| `ProgressEvent` | Tagged union progress for transfer → napi (`kind` in kebab-case) |
| `sanitize_filename` / `filename_from_url` | Safe filename helpers |

## Wire rules

- Struct fields: `#[serde(rename_all = "camelCase")]`
- Enum tags / kinds: kebab-case (`started`, `bytes`, `finished`, `failed`)
- Do **not** add per-field `serde(rename = …)` just to satisfy old JS fixtures — update the fixtures

## Example

```rust
use prometheus_types::{MediaInfo, ProgressEvent, TRANSFER_SIMPLE, sanitize_filename};

let name = sanitize_filename("../a\\b:c"); // "_a_b_c"

let event = ProgressEvent::Started {
    url: "https://example.com/a.bin".into(),
    total_bytes: Some(12),
    transfer: TRANSFER_SIMPLE.into(),
};
```

## License

CC0-1.0 — see the repository `License.md`.
