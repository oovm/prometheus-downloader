//! Direct HTTP(S) file URL extractor.

use prometheus_types::{
    filename_from_url, sanitize_filename, Error, MediaInfo, Result, EXTRACTOR_GENERIC_HTTP,
};

use crate::Extractor;

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

        match ureq::head(url).call() {
            Ok(resp) => {
                content_type = resp.header("content-type").map(|s| s.to_string());
                content_length = resp
                    .header("content-length")
                    .and_then(|s| s.parse::<u64>().ok());
                if let Some(raw) = resp.header("content-disposition") {
                    if let Some(name) = filename_from_content_disposition(raw) {
                        filename = Some(name);
                    }
                }
            }
            Err(ureq::Error::Status(code, resp)) => {
                // Some hosts reject HEAD; still return URL-derived metadata.
                let _ = (code, resp);
            }
            Err(err) => {
                // Keep URL-derived metadata when the probe fails after a DNS/TLS error
                // only if we already have a filename; otherwise surface the error.
                if filename.is_none() {
                    return Err(Error::Network(err.to_string()));
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

fn filename_from_content_disposition(header: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_http_urls() {
        let g = GenericHttp;
        assert!(g.matches("https://example.com/a.bin"));
        assert!(!g.matches("ftp://example.com/a.bin"));
    }

    #[test]
    fn parses_content_disposition() {
        assert_eq!(
            filename_from_content_disposition(
                "attachment; filename=\"clip.bin\"; filename*=UTF-8''nice%20clip.bin"
            ),
            Some("nice clip.bin".to_string())
        );
    }
}
