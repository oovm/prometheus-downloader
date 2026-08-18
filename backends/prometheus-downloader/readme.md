# ⬇️ prometheus-downloader

**In-process** HTTP(S) transfer backends for Prometheus. Downloads run inside the same napi cdylib as the rest of the engine — one `.node` per platform, no sidecar CLI, no PATH download tool.

Not published to crates.io.

## Surfaces

```rust
pub trait TransferBackend: Send + Sync {
    fn id(&self) -> &'static str;
    fn transfer(
        &self,
        request: &TransferRequest,
        progress: &mut dyn FnMut(ProgressEvent),
    ) -> Result<DownloadResult>;
}

download(url, output_dir) -> DownloadResult
download_with(backend, url, output_dir, progress) -> DownloadResult
download_collecting_events(backend, url, output_dir) -> (DownloadResult, Vec<ProgressEvent>)
list_transfers() -> Vec<TransferInfo>
default_transfer_id() -> &'static str  // "simple"
```

`download` inspects via `Registry::builtin()`, sanitizes a filename, then calls the backend.

## Current backend: `simple`

`SimpleTransfer` (`TRANSFER_SIMPLE`):

- HTTP(S): single-connection GET with `User-Agent: Prometheus/<version>`, chunked write, progress callbacks
- `file:`: copy from the local path into the output directory with the same progress shape

`list_transfers()` returns only backends **linked into** this crate (today: `simple` with `available: true`).

## Progress

`ProgressEvent` variants (`started` / `bytes` / `finished` / `failed`) are defined in `prometheus-types` and forwarded to napi as `downloadWithEvents`.

## Hard constraints

- Do **not** spawn external downloaders or bundle a second executable as a transfer implementation
- Future multi-connection Range / resume backends must stay **in-process** (linked into the same cdylib)
- Filename collisions get uniquified under the output directory (`name-1.ext`, …)

## License

CC0-1.0 — see the repository `License.md`.
