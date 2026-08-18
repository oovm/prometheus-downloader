# prometheus-extractors

**URL extractors** and the ordered registry used by Prometheus. An extractor answers `matches(url)` and `inspect(url) -> MediaInfo`. The first matching extractor in the registry wins.

Not published to crates.io. Consumed by `prometheus-downloader` and exposed to JavaScript through `prometheus-napi` (`info`).

## Built-in extractors

| Id | Struct | Matches | Notes |
|----|--------|---------|-------|
| `local-file` | `LocalFile` | `file:` | Local filesystem `stat` |
| `internet-archive` | `InternetArchive` | `archive.org/details/…` (also `/download/` / `/metadata/`) | Public metadata API → chooses a downloadable media file |
| `wikimedia-commons` | `WikimediaCommons` | Commons / Wikipedia / sister-project `File:` pages | MediaWiki `imageinfo` API on that host → `upload.wikimedia.org` URL |
| `peertube` | `PeerTube` | `/w/{id}`, `/videos/watch/{id}`, `/videos/embed/{id}` | Instance REST `GET /api/v1/videos/{id}` → highest-resolution `fileUrl` |
| `nasa-images` | `NasaImages` | `images.nasa.gov/details-…` (also `/details/` and `/asset/`) | Public asset API → prefers `~orig` media href |
| `met-museum` | `MetMuseum` | `metmuseum.org/art/collection/search/{id}` | Collection API `primaryImage` |
| `artic` | `Artic` | `artic.edu/artworks/{id}` | AIC API `image_id` → IIIF `full/max` JPEG |
| `cleveland-museum` | `ClevelandMuseum` | `clevelandart.org/art/{accession}` | Open-access API → print-size JPEG |
| `openverse` | `Openverse` | `openverse.org/image/{uuid}` / `/audio/{uuid}` | Catalog API work URL |
| `gutenberg` | `Gutenberg` | `gutenberg.org/ebooks/{id}` | Gutendex `formats` → prefers audio, then EPUB |
| `ccmixter` | `CcMixter` | `ccmixter.org/files/{user}/{id}` | Query API `files[].download_url` |
| `vam` | `Vam` | `collections.vam.ac.uk/item/O…` | Object API IIIF `full/max` JPEG |
| `generic-http` | `GenericHttp` | `http://` / `https://` | HEAD → Range → GET header probe; User-Agent `Prometheus/<version>` |

Registry order: **local-file → internet-archive → wikimedia-commons → peertube → nasa-images → met-museum → artic → cleveland-museum → openverse → gutenberg → ccmixter → vam → generic-http**.

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

Helpers: `filename_from_content_disposition`, `file_url_to_path`, `media_from_metadata_json`, `media_from_api_json`, `media_from_video_json`.

## Tests

Fixture-based parsing tests ship under `tests/fixtures/`. Optional live network checks run only when `PROMETHEUS_LIVE_NET=1` and soft-skip on transport errors.

## License

CC0-1.0 — see the repository `License.md`.
