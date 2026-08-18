//! Shared types and errors for Prometheus.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Crate / package version string shared with the Node surface.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Extractor id for direct HTTP(S) file URLs.
pub const EXTRACTOR_GENERIC_HTTP: &str = "generic-http";

/// Transfer backend id for the single-connection HTTP client.
pub const TRANSFER_SIMPLE: &str = "simple";

/// Errors returned by engine crates.
#[derive(Debug, Error)]
pub enum Error {
    /// The URL is not supported by any registered extractor.
    #[error("unsupported URL: {0}")]
    UnsupportedUrl(String),
    /// The URL could not be parsed.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    /// Network or HTTP failure.
    #[error("network: {0}")]
    Network(String),
    /// Local filesystem failure.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// Credential vault failure.
    #[error("credential: {0}")]
    Credential(String),
}

/// Result alias for engine crates.
pub type Result<T> = std::result::Result<T, Error>;

/// Media metadata resolved from a URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    /// Original request URL.
    pub url: String,
    /// Human-readable title when known.
    pub title: Option<String>,
    /// HTTP `Content-Type` when known.
    pub content_type: Option<String>,
    /// HTTP `Content-Length` when known.
    pub content_length: Option<u64>,
    /// Suggested filename for download.
    pub filename: Option<String>,
    /// Extractor id (`kebab-case`), e.g. `generic-http`.
    pub extractor: String,
}

/// Successful download summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    /// Absolute path of the written file.
    pub path: String,
    /// Bytes written to disk.
    pub bytes_written: u64,
    /// Filename component under the output directory.
    pub filename: String,
}

/// Structured download progress for Transfer backends → napi / MCP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum ProgressEvent {
    /// Transfer started; `totalBytes` is known when the server advertises length.
    Started { url: String, total_bytes: Option<u64>, transfer: String },
    /// Bytes written so far.
    Bytes { url: String, bytes_written: u64, total_bytes: Option<u64>, transfer: String },
    /// Transfer finished successfully.
    Finished { url: String, bytes_written: u64, path: String, transfer: String },
    /// Transfer failed after start (or during setup when `bytesWritten` is zero).
    Failed { url: String, message: String, transfer: String },
}

/// Strip path separators and reserved characters from a download filename.
pub fn sanitize_filename(name: &str) -> String {
    let trimmed = name.trim().trim_matches(|c| c == '/' || c == '\\');
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if matches!(ch, '/' | '\\' | '\0' | '<' | '>' | ':' | '"' | '|' | '?' | '*') {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    let out = out.trim().trim_start_matches('.');
    if out.is_empty() { "download.bin".to_string() } else { out.to_string() }
}

/// Derive a filename from a URL path segment.
pub fn filename_from_url(url: &str) -> Option<String> {
    let without_query = url.split(['?', '#']).next().unwrap_or(url);
    let segment = without_query.rsplit('/').next().unwrap_or("");
    if segment.is_empty() {
        return None;
    }
    let decoded = percent_decode(segment);
    let name = sanitize_filename(&decoded);
    if name == "download.bin" { None } else { Some(name) }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_rejects_path_parts() {
        assert_eq!(sanitize_filename("../a\\b:c"), "_a_b_c");
        assert_eq!(sanitize_filename("   "), "download.bin");
    }

    #[test]
    fn filename_from_simple_url() {
        assert_eq!(
            filename_from_url("https://example.com/files/demo%20clip.bin?x=1"),
            Some("demo clip.bin".to_string())
        );
    }

    #[test]
    fn progress_event_wire_kind() {
        let event = ProgressEvent::Started {
            url: "https://example.com/a.bin".into(),
            total_bytes: Some(12),
            transfer: TRANSFER_SIMPLE.into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["kind"], "started");
        assert_eq!(json["totalBytes"], 12);
        assert_eq!(json["transfer"], "simple");
    }
}
