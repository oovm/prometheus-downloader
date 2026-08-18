//! Node-API bindings for Prometheus.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use prometheus_extractors::Registry;
use prometheus_types::VERSION;

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

/// Resolve media metadata for a URL.
#[napi]
pub fn info(url: String) -> Result<JsMediaInfo> {
    let media = Registry::builtin()
        .inspect(&url)
        .map_err(|err| Error::from_reason(err.to_string()))?;
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

/// Create an empty credential vault file (encryption not implemented).
#[napi]
pub fn create_vault(path: String, password: String) -> Result<String> {
    let vault = prometheus_credential::Vault::create(&path, &password)
        .map_err(|err| Error::from_reason(err.to_string()))?;
    Ok(vault.path().to_string_lossy().into_owned())
}
