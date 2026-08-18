//! Pluggable HTTP(S) transfer backends.

#![deny(missing_docs)]

mod simple;

pub use simple::SimpleTransfer;

use std::path::{Path, PathBuf};

use prometheus_extractors::Registry;
use prometheus_types::{DownloadResult, ProgressEvent, Result, TRANSFER_SIMPLE, sanitize_filename};

/// Request handed to a [`TransferBackend`].
#[derive(Debug, Clone)]
pub struct TransferRequest {
    /// Source URL (already inspected / resolved by an extractor when using [`download`]).
    pub url: String,
    /// Destination directory (created by the caller or backend).
    pub output_dir: PathBuf,
    /// Preferred filename; backends may still uniquify collisions.
    pub filename: String,
    /// Expected content length when known from inspect.
    pub expected_length: Option<u64>,
}

/// Pluggable transfer implementation (`simple` now; `aria2` / `native-range` later).
pub trait TransferBackend: Send + Sync {
    /// Stable backend id (`kebab-case`).
    fn id(&self) -> &'static str;

    /// Download `request.url` into `request.output_dir`.
    fn transfer(
        &self,
        request: &TransferRequest,
        progress: &mut dyn FnMut(ProgressEvent),
    ) -> Result<DownloadResult>;
}

/// Download with the default [`SimpleTransfer`] backend (no progress listener).
pub fn download(url: &str, output_dir: impl AsRef<Path>) -> Result<DownloadResult> {
    download_with(&SimpleTransfer, url, output_dir, &mut |_| {})
}

/// Download using an explicit backend and progress sink.
pub fn download_with(
    backend: &dyn TransferBackend,
    url: &str,
    output_dir: impl AsRef<Path>,
    progress: &mut dyn FnMut(ProgressEvent),
) -> Result<DownloadResult> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir)?;

    let info = Registry::builtin().inspect(url)?;
    let filename = sanitize_filename(
        info.filename.as_deref().or(info.title.as_deref()).unwrap_or("download.bin"),
    );
    let request = TransferRequest {
        url: info.url,
        output_dir: output_dir.to_path_buf(),
        filename,
        expected_length: info.content_length,
    };

    match backend.transfer(&request, progress) {
        Ok(result) => Ok(result),
        Err(err) => {
            progress(ProgressEvent::Failed {
                url: request.url.clone(),
                message: err.to_string(),
                transfer: backend.id().to_string(),
            });
            Err(err)
        }
    }
}

/// Default backend id used by [`download`].
pub fn default_transfer_id() -> &'static str {
    TRANSFER_SIMPLE
}

pub(crate) fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or("download");
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{s}"))
        .unwrap_or_default();
    for i in 1..10_000 {
        let name = format!("{stem}-{i}{ext}");
        let path = dir.join(name);
        if !path.exists() {
            return path;
        }
    }
    dir.join(format!("{stem}-{}.bin", std::process::id()))
}
