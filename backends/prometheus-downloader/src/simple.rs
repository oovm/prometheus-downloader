//! Single-connection HTTP(S) transfer backend.

use std::fs::File;
use std::io::{BufWriter, Read, Write};

use prometheus_types::{DownloadResult, Error, ProgressEvent, Result, TRANSFER_SIMPLE};

use crate::{TransferBackend, TransferRequest, unique_path};

/// Default User-Agent for engine HTTP requests (product identity, not browser spoofing).
pub const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

const CHUNK: usize = 64 * 1024;

/// Single-connection GET transfer (`simple`).
pub struct SimpleTransfer;

impl TransferBackend for SimpleTransfer {
    fn id(&self) -> &'static str {
        TRANSFER_SIMPLE
    }

    fn transfer(
        &self,
        request: &TransferRequest,
        progress: &mut dyn FnMut(ProgressEvent),
    ) -> Result<DownloadResult> {
        let path = unique_path(&request.output_dir, &request.filename);
        let resp = ureq::get(&request.url)
            .set("User-Agent", USER_AGENT)
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;

        let total_bytes = resp
            .header("content-length")
            .and_then(|s| s.parse::<u64>().ok())
            .or(request.expected_length);

        progress(ProgressEvent::Started {
            url: request.url.clone(),
            total_bytes,
            transfer: self.id().to_string(),
        });

        let mut reader = resp.into_reader();
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        let mut buf = [0u8; CHUNK];
        let mut bytes_written: u64 = 0;
        let mut last_emit = 0u64;

        loop {
            let n = reader.read(&mut buf).map_err(Error::from)?;
            if n == 0 {
                break;
            }
            writer.write_all(&buf[..n])?;
            bytes_written += n as u64;
            if bytes_written - last_emit >= CHUNK as u64 || total_bytes == Some(bytes_written) {
                progress(ProgressEvent::Bytes {
                    url: request.url.clone(),
                    bytes_written,
                    total_bytes,
                    transfer: self.id().to_string(),
                });
                last_emit = bytes_written;
            }
        }

        writer.into_inner().map_err(|err| Error::Io(err.into_error()))?.sync_all()?;

        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| request.filename.clone());

        progress(ProgressEvent::Finished {
            url: request.url.clone(),
            bytes_written,
            path: path.to_string_lossy().into_owned(),
            transfer: self.id().to_string(),
        });

        Ok(DownloadResult { path: path.to_string_lossy().into_owned(), bytes_written, filename })
    }
}
