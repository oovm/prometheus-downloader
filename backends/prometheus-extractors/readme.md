# 🧭 prometheus-extractors

**URL extractors** and the ordered registry used by Prometheus. An extractor answers `matches(url)` and `inspect(url) -> MediaInfo`. The first matching extractor in the registry wins.

Not published to crates.io. Consumed by `prometheus-downloader` and exposed to JavaScript through `prometheus-napi` (`info`).

## Built-in extractors

| Id | Struct | Matches | Notes |
|----|--------|---------|-------|
| `generic-http` | `GenericHttp` | `http://` / `https://` | Probes metadata with `HEAD` → `Range: bytes=0-0` → GET headers; User-Agent `Prometheus/<version>` |
| `local-file` | `LocalFile` | `file:` | Local filesystem `stat`; path parsing for Windows and POSIX `file:` forms |

Registry construction:

```rust
use prometheus_extractors::Registry;

let media = Registry::builtin().inspect(url)?;
```

Order: **`local-file` then `generic-http`**, so `file:` never falls through to HTTP.

## Contract

```rust
pub trait Extractor: Send + Sync {
    fn id(&self) -> &'static str;
    fn matches(&self, url: &str) -> bool;
    fn inspect(&self, url: &str) -> Result<MediaInfo>;
}
```

Helpers re-exported for tests and callers:

- `filename_from_content_disposition`
- `file_url_to_path`

## Design notes

- Extractors resolve **metadata**, not bytes. Transfer belongs in `prometheus-downloader`.
- Site-specific adapters should stay focused: return honest `MediaInfo` (and later request parameters), without turning this crate into a dump of unrelated host logic.
- npm `@doki-land/prometheus-plugin-*` packages mirror a subset of this surface for Harness verification; the engine registry remains authoritative for native `info` / `download`.

## License

CC0-1.0 — see the repository `License.md`.
