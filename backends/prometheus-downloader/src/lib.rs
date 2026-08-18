//! Pluggable HTTP(S) transfer backends.

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn serve_head_and_get(body: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            for _ in 0..4 {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let is_head = req.starts_with("HEAD");
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"fixture.bin\"\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                if !is_head {
                    let _ = stream.write_all(body);
                }
            }
        });
        format!("http://{addr}/fixture.bin")
    }

    #[test]
    fn downloads_local_fixture() {
        let url = serve_head_and_get(b"hello-prometheus");
        thread::sleep(std::time::Duration::from_millis(20));
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("prometheus-dl-{nanos}"));
        let result = download(&url, &dir).unwrap();
        let bytes = fs::read(&result.path).unwrap();
        assert_eq!(bytes, b"hello-prometheus");
        assert_eq!(result.bytes_written, 16);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn emits_progress_events() {
        let url = serve_head_and_get(b"progress-body!!");
        thread::sleep(std::time::Duration::from_millis(20));
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("prometheus-dl-prog-{nanos}"));
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = events.clone();
        let result = download_with(&SimpleTransfer, &url, &dir, &mut |event| {
            sink.lock().unwrap().push(event);
        })
        .unwrap();
        assert_eq!(result.bytes_written, 15);
        let kinds: Vec<_> = events
            .lock()
            .unwrap()
            .iter()
            .map(|e| match e {
                ProgressEvent::Started { .. } => "started",
                ProgressEvent::Bytes { .. } => "bytes",
                ProgressEvent::Finished { .. } => "finished",
                ProgressEvent::Failed { .. } => "failed",
            })
            .collect();
        assert_eq!(kinds.first().copied(), Some("started"));
        assert_eq!(kinds.last().copied(), Some("finished"));
        assert!(kinds.iter().any(|k| *k == "bytes"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_transfer_is_simple() {
        assert_eq!(default_transfer_id(), "simple");
        assert_eq!(SimpleTransfer.id(), "simple");
    }
}
