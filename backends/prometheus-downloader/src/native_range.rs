//! In-process multi-connection HTTP Range transfer backend.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use prometheus_types::{DownloadResult, Error, ProgressEvent, Result, TRANSFER_NATIVE_RANGE};
use ripget::{DownloadOptions, ProgressReporter};

use crate::simple::{USER_AGENT, copy_local_file};
use crate::{TransferBackend, TransferRequest, unique_path};

const RANGE_THREADS: usize = 4;

/// Multi-connection Range transfer (`native-range`).
///
/// Linked into the same napi cdylib as [`crate::SimpleTransfer`]. Does not spawn a
/// sidecar process. Default product download still uses `simple`.
pub struct NativeRangeTransfer;

impl TransferBackend for NativeRangeTransfer {
    fn id(&self) -> &'static str {
        TRANSFER_NATIVE_RANGE
    }

    fn transfer(
        &self,
        request: &TransferRequest,
        progress: &mut dyn FnMut(ProgressEvent),
    ) -> Result<DownloadResult> {
        if request.url.trim().to_ascii_lowercase().starts_with("file:") {
            return copy_local_file(self.id(), request, progress);
        }

        let path = unique_path(&request.output_dir, &request.filename);
        let reporter = Arc::new(RangeProgress {
            url: request.url.clone(),
            transfer: self.id().to_string(),
            total: AtomicU64::new(request.expected_length.unwrap_or(0)),
            downloaded: AtomicU64::new(0),
            events: Mutex::new(Vec::new()),
        });

        let options = DownloadOptions::new()
            .user_agent(USER_AGENT)
            .threads(RANGE_THREADS)
            .progress(reporter.clone());

        let report = tokio_rt()
            .block_on(ripget::download_url_with_options(&request.url, &path, options))
            .map_err(|err| Error::Network(err.to_string()))?;

        let bytes_written = report.bytes;
        replay_progress(&reporter, progress, &request.url, self.id(), bytes_written, &path);

        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| request.filename.clone());

        Ok(DownloadResult { path: path.to_string_lossy().into_owned(), bytes_written, filename })
    }
}

fn tokio_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("prometheus-range")
            .build()
            .expect("tokio runtime for native-range")
    })
}

struct RangeProgress {
    url: String,
    transfer: String,
    total: AtomicU64,
    downloaded: AtomicU64,
    events: Mutex<Vec<ProgressEvent>>,
}

impl ProgressReporter for RangeProgress {
    fn init(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
        if let Ok(mut events) = self.events.lock() {
            events.push(ProgressEvent::Started {
                url: self.url.clone(),
                total_bytes: Some(total),
                transfer: self.transfer.clone(),
            });
        }
    }

    fn add(&self, delta: u64) {
        let bytes_written = self.downloaded.fetch_add(delta, Ordering::Relaxed) + delta;
        let total_bytes = {
            let total = self.total.load(Ordering::Relaxed);
            if total == 0 { None } else { Some(total) }
        };
        if let Ok(mut events) = self.events.lock() {
            events.push(ProgressEvent::Bytes {
                url: self.url.clone(),
                bytes_written,
                total_bytes,
                transfer: self.transfer.clone(),
            });
        }
    }
}

fn replay_progress(
    reporter: &RangeProgress,
    progress: &mut dyn FnMut(ProgressEvent),
    url: &str,
    transfer: &str,
    bytes_written: u64,
    path: &Path,
) {
    let mut events = reporter.events.lock().map(|g| g.clone()).unwrap_or_default();
    if !events.iter().any(|e| matches!(e, ProgressEvent::Started { .. })) {
        progress(ProgressEvent::Started {
            url: url.to_string(),
            total_bytes: Some(bytes_written),
            transfer: transfer.to_string(),
        });
    }
    for event in events.drain(..) {
        progress(event);
    }
    progress(ProgressEvent::Finished {
        url: url.to_string(),
        bytes_written,
        path: path.to_string_lossy().into_owned(),
        transfer: transfer.to_string(),
    });
}
