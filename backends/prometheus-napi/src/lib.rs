//! Node-API bindings for Prometheus.

#![deny(missing_docs)]
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use prometheus_downloader::SimpleTransfer;
use prometheus_extractors::Registry;
use prometheus_types::{ProgressEvent, VERSION};

/// Package / native addon version.
#[napi]
pub fn version() -> String {
    VERSION.to_string()
}

/// Media metadata returned to JavaScript (camelCase fields).
#[napi(object)]
pub struct JsMediaInfo {
    /// Original request URL.
    pub url: String,
    /// Human-readable title when known.
    pub title: Option<String>,
    /// HTTP Content-Type when known.
    pub content_type: Option<String>,
    /// HTTP Content-Length when known.
    pub content_length: Option<i64>,
    /// Suggested filename.
    pub filename: Option<String>,
    /// Extractor id (`kebab-case`).
    pub extractor: String,
}

/// Download summary returned to JavaScript.
#[napi(object)]
pub struct JsDownloadResult {
    /// Absolute path of the written file.
    pub path: String,
    /// Bytes written.
    pub bytes_written: i64,
    /// Filename under the output directory.
    pub filename: String,
}

/// In-process transfer backend snapshot.
#[napi(object)]
pub struct JsTransferInfo {
    /// Backend id (`kebab-case`).
    pub id: String,
    /// Whether the backend is linked into this addon.
    pub available: bool,
}

/// Structured transfer progress returned with [`download_with_events`].
#[napi(object)]
pub struct JsProgressEvent {
    /// Event kind (`started` / `bytes` / `finished` / `failed`).
    pub kind: String,
    /// Source URL.
    pub url: String,
    /// Transfer backend id.
    pub transfer: String,
    /// Bytes written so far, when this event carries a count.
    pub bytes_written: Option<i64>,
    /// Total bytes when known.
    pub total_bytes: Option<i64>,
    /// Destination path on `finished`.
    pub path: Option<String>,
    /// Error message on `failed`.
    pub message: Option<String>,
}

/// Download summary plus collected progress events.
#[napi(object)]
pub struct JsDownloadWithEvents {
    /// Absolute path of the written file.
    pub path: String,
    /// Bytes written.
    pub bytes_written: i64,
    /// Filename under the output directory.
    pub filename: String,
    /// Progress events collected during the transfer.
    pub events: Vec<JsProgressEvent>,
}

/// Resolve media metadata for a URL.
#[napi]
pub fn info(url: String) -> Result<JsMediaInfo> {
    let media =
        Registry::builtin().inspect(&url).map_err(|err| Error::from_reason(err.to_string()))?;
    Ok(JsMediaInfo {
        url: media.url,
        title: media.title,
        content_type: media.content_type,
        content_length: media.content_length.map(|n| n as i64),
        filename: media.filename,
        extractor: media.extractor,
    })
}

/// Download a URL into `output_dir`.
#[napi]
pub fn download(url: String, output_dir: String) -> Result<JsDownloadResult> {
    let result = prometheus_downloader::download(&url, &output_dir)
        .map_err(|err| Error::from_reason(err.to_string()))?;
    Ok(JsDownloadResult {
        path: result.path,
        bytes_written: result.bytes_written as i64,
        filename: result.filename,
    })
}

/// List in-process transfer backends linked into this addon.
#[napi]
pub fn list_transfers() -> Vec<JsTransferInfo> {
    prometheus_downloader::list_transfers()
        .into_iter()
        .map(|item| JsTransferInfo { id: item.id, available: item.available })
        .collect()
}

/// Download a URL into `output_dir` and return collected progress events.
#[napi]
pub fn download_with_events(url: String, output_dir: String) -> Result<JsDownloadWithEvents> {
    let (result, events) =
        prometheus_downloader::download_collecting_events(&SimpleTransfer, &url, &output_dir)
            .map_err(|err| Error::from_reason(err.to_string()))?;
    Ok(JsDownloadWithEvents {
        path: result.path,
        bytes_written: result.bytes_written as i64,
        filename: result.filename,
        events: events.into_iter().map(js_progress_event).collect(),
    })
}

fn js_progress_event(event: ProgressEvent) -> JsProgressEvent {
    match event {
        ProgressEvent::Started { url, total_bytes, transfer } => JsProgressEvent {
            kind: "started".into(),
            url,
            transfer,
            bytes_written: None,
            total_bytes: total_bytes.map(|n| n as i64),
            path: None,
            message: None,
        },
        ProgressEvent::Bytes { url, bytes_written, total_bytes, transfer } => JsProgressEvent {
            kind: "bytes".into(),
            url,
            transfer,
            bytes_written: Some(bytes_written as i64),
            total_bytes: total_bytes.map(|n| n as i64),
            path: None,
            message: None,
        },
        ProgressEvent::Finished { url, bytes_written, path, transfer } => JsProgressEvent {
            kind: "finished".into(),
            url,
            transfer,
            bytes_written: Some(bytes_written as i64),
            total_bytes: None,
            path: Some(path),
            message: None,
        },
        ProgressEvent::Failed { url, message, transfer } => JsProgressEvent {
            kind: "failed".into(),
            url,
            transfer,
            bytes_written: None,
            total_bytes: None,
            path: None,
            message: Some(message),
        },
    }
}

/// Create an empty credential vault file (encryption not implemented).
#[napi]
pub fn create_vault(path: String, password: String) -> Result<String> {
    let vault = prometheus_credential::Vault::create(&path, &password)
        .map_err(|err| Error::from_reason(err.to_string()))?;
    Ok(vault.path().to_string_lossy().into_owned())
}
