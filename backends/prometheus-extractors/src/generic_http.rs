//! Direct HTTP(S) file URL extractor.

use std::io::Read;

use prometheus_types::{
    EXTRACTOR_GENERIC_HTTP, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Matches `http://` and `https://` URLs.
pub struct GenericHttp;

impl Extractor for GenericHttp {
    fn id(&self) -> &'static str {
        EXTRACTOR_GENERIC_HTTP
    }

    fn matches(&self, url: &str) -> bool {
        let lower = url.trim().to_ascii_lowercase();
        lower.starts_with("http://") || lower.starts_with("https://")
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let url = url.trim();
        if !self.matches(url) {
            return Err(Error::UnsupportedUrl(url.to_string()));
        }

        let mut content_type = None;
        let mut content_length = None;
        let mut filename = filename_from_url(url);

        match probe_headers(url) {
            Ok(meta) => {
                content_type = meta.content_type;
                content_length = meta.content_length;
                if let Some(name) = meta.filename {
                    filename = Some(name);
                }
            }
            Err(err) => {
                // Keep URL-derived metadata when the probe fails after a DNS/TLS error
                // only if we already have a filename; otherwise surface the error.
                if filename.is_none() {
                    return Err(err);
                }
            }
        }

        let title = filename.clone();
        Ok(MediaInfo {
            url: url.to_string(),
            title,
            content_type,
            content_length,
            filename,
            extractor: EXTRACTOR_GENERIC_HTTP.to_string(),
        })
    }
}

struct ProbeMeta {
    content_type: Option<String>,
    content_length: Option<u64>,
    filename: Option<String>,
}

fn probe_headers(url: &str) -> Result<ProbeMeta> {
    match ureq::head(url).set("User-Agent", USER_AGENT).call() {
        Ok(resp) => return Ok(meta_from_response(&resp)),
        Err(ureq::Error::Status(_, _)) => {
            // Some hosts reject HEAD; fall through to Range / GET.
        }
        Err(err) => return Err(Error::Network(err.to_string())),
    }

    match ureq::get(url).set("User-Agent", USER_AGENT).set("Range", "bytes=0-0").call() {
        Ok(resp) => {
            let meta = meta_from_response(&resp);
            drain_probe_body(resp);
            return Ok(meta);
        }
        Err(ureq::Error::Status(_, _)) => {}
        Err(err) => return Err(Error::Network(err.to_string())),
    }

    let resp = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|err| Error::Network(err.to_string()))?;
    let meta = meta_from_response(&resp);
    drain_probe_body(resp);
    Ok(meta)
}

fn drain_probe_body(resp: ureq::Response) {
    let mut reader = resp.into_reader();
    let mut buf = [0u8; 1];
    let _ = reader.read(&mut buf);
}

fn meta_from_response(resp: &ureq::Response) -> ProbeMeta {
    let content_type = resp.header("content-type").map(|s| s.to_string());
    let mut content_length = resp.header("content-length").and_then(|s| s.parse::<u64>().ok());
    if let Some(range) = resp.header("content-range") {
        if let Some(total) = content_range_total(range) {
            content_length = Some(total);
        }
    }
    let filename = resp.header("content-disposition").and_then(filename_from_content_disposition);
    ProbeMeta { content_type, content_length, filename }
}

fn content_range_total(header: &str) -> Option<u64> {
    // bytes 0-0/1234
    let total = header.rsplit('/').next()?;
    if total == "*" {
        return None;
    }
    total.parse().ok()
}

/// Parse a download filename from an HTTP `Content-Disposition` header.
pub fn filename_from_content_disposition(header: &str) -> Option<String> {
    // Prefer filename*=UTF-8''... then filename="..."
    for part in header.split(';') {
        let part = part.trim();
        let lower = part.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("filename*=") {
            let _ = rest;
            if let Some(eq) = part.find('=') {
                let value = part[eq + 1..].trim().trim_matches('"');
                if let Some(decoded) = decode_rfc5987(value) {
                    return Some(sanitize_filename(&decoded));
                }
            }
        }
    }
    for part in header.split(';') {
        let part = part.trim();
        let lower = part.to_ascii_lowercase();
        if lower.starts_with("filename=") && !lower.starts_with("filename*=") {
            if let Some(eq) = part.find('=') {
                let value = part[eq + 1..].trim().trim_matches('"');
                if !value.is_empty() {
                    return Some(sanitize_filename(value));
                }
            }
        }
    }
    None
}

fn decode_rfc5987(value: &str) -> Option<String> {
    // charset'lang'%xx%xx
    let mut parts = value.splitn(3, '\'');
    let _charset = parts.next()?;
    let _lang = parts.next()?;
    let encoded = parts.next()?;
    let mut bytes = Vec::new();
    let raw = encoded.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%' && i + 2 < raw.len() {
            if let (Some(h), Some(l)) = (from_hex(raw[i + 1]), from_hex(raw[i + 2])) {
                bytes.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        bytes.push(raw[i]);
        i += 1;
    }
    String::from_utf8(bytes).ok()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
