# 🎛️ prometheus-processor

**Post-processing** hooks for Prometheus: mux / remux, format conversion, and local-file transforms that run **after** transfer.

Not published to crates.io. Currently a deliberate empty shell so the workspace and napi graph stay stable.

## Current API

```rust
passthrough_path(path) -> Result<PathBuf>
```

Identity helper reserved for future processors. No codecs are linked.

## Constraints

- No hard dependency on external CLI converters or downloaders
- No bundling a second executable next to the `.node` addon
- When processors land, they must either be pure Rust / libraries linked into the same cdylib, or clearly documented optional system integrations that are **not** the default product path
- Local decrypt-style features (when present) apply only to files already on the user’s disk

## License

CC0-1.0 — see the repository `License.md`.
