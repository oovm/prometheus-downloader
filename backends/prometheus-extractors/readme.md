# prometheus-extractors

**URL extractors** and the ordered registry used by Prometheus. An extractor answers `matches(url)` and `inspect(url) -> MediaInfo`. The first matching extractor in the registry wins.

Not published to crates.io. Consumed by `prometheus-downloader` and exposed to JavaScript through `prometheus-napi` (`info`).

## Built-in extractors

| Id | Struct | Matches | Notes |
|----|--------|---------|-------|
| `local-file` | `LocalFile` | `file:` | Local filesystem `stat` |
| `internet-archive` | `InternetArchive` | `archive.org/details/…` (also `/download/` / `/metadata/`) | Public metadata API → chooses a downloadable media file |
| `wikimedia-commons` | `WikimediaCommons` | `commons.wikimedia.org/wiki/File:…` | MediaWiki `imageinfo` API → `upload.wikimedia.org` URL |
| `generic-http` | `GenericHttp` | `http://` / `https://` | HEAD → Range → GET header probe; User-Agent `Prometheus/<version>` |

Registry order: **local-file → internet-archive → wikimedia-commons → generic-http**.

```rust
use prometheus_extractors::Registry;

let media = Registry::builtin().inspect(url)?;
```

## Contract

```rust
pub trait Extractor: Send + Sync {
    fn id(&self) -> &'static str;
    fn matches(&self, url: &str) -> bool;
    fn inspect(&self, url: &str) -> Result<MediaInfo>;
}
```

**Tool coverage means more media sites / hosts**, added here—not by stacking npm plugins. Transfer backends are a separate axis.

Helpers: `filename_from_content_disposition`, `file_url_to_path`, `media_from_metadata_json`, `media_from_api_json`.

## Tests

Fixture-based parsing tests ship under `tests/fixtures/`. Optional live network checks run only when `PROMETHEUS_LIVE_NET=1` and soft-skip on transport errors.

## License

CC0-1.0 — see the repository `License.md`.
