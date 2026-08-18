//! Local `file:` URL extractor.

use std::path::PathBuf;

use prometheus_types::{EXTRACTOR_LOCAL_FILE, Error, MediaInfo, Result, sanitize_filename};

use crate::Extractor;

/// Matches `file:` URLs on the local filesystem.
pub struct LocalFile;

impl Extractor for LocalFile {
    fn id(&self) -> &'static str {
        EXTRACTOR_LOCAL_FILE
    }

    fn matches(&self, url: &str) -> bool {
        url.trim().to_ascii_lowercase().starts_with("file:")
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let url = url.trim();
        if !self.matches(url) {
            return Err(Error::UnsupportedUrl(url.to_string()));
        }
        let path = file_url_to_path(url)?;
        let meta = std::fs::metadata(&path)?;
        if !meta.is_file() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("not a file: {}", path.display()),
            )));
        }
        let filename = path.file_name().and_then(|s| s.to_str()).map(sanitize_filename);
        Ok(MediaInfo {
            url: url.to_string(),
            title: filename.clone(),
            content_type: None,
            content_length: Some(meta.len()),
            filename,
            extractor: EXTRACTOR_LOCAL_FILE.to_string(),
        })
    }
}

/// Convert a `file:` URL to a local filesystem path.
pub fn file_url_to_path(url: &str) -> Result<PathBuf> {
    let url = url.trim();
    let rest = strip_file_scheme(url).ok_or_else(|| Error::InvalidUrl(url.to_string()))?;
    let (authority, path) = split_authority_path(rest);
    if !authority.is_empty()
        && !authority.eq_ignore_ascii_case("localhost")
        && authority != "127.0.0.1"
    {
        return Err(Error::InvalidUrl(url.to_string()));
    }
    Ok(normalize_file_path(percent_decode_path(path)))
}

fn strip_file_scheme(url: &str) -> Option<&str> {
    if url.len() < 5 || !url[..5].eq_ignore_ascii_case("file:") {
        return None;
    }
    Some(&url[5..])
}

fn split_authority_path(rest: &str) -> (&str, &str) {
    if let Some(stripped) = rest.strip_prefix("//") {
        if let Some(idx) = stripped.find('/') {
            (&stripped[..idx], &stripped[idx..])
        } else if stripped.is_empty() {
            ("", "/")
        } else {
            (stripped, "/")
        }
    } else if rest.is_empty() {
        ("", "/")
    } else {
        ("", rest)
    }
}

fn percent_decode_path(input: &str) -> String {
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

fn normalize_file_path(decoded: String) -> PathBuf {
    #[cfg(windows)]
    {
        let mut s = decoded.replace('/', "\\");
        let bytes = s.as_bytes();
        if bytes.len() >= 3
            && bytes[0] == b'\\'
            && bytes[1].is_ascii_alphabetic()
            && (bytes[2] == b':' || bytes[2] == b'|')
        {
            s.remove(0);
            if s.as_bytes().get(1) == Some(&b'|') {
                s.replace_range(1..2, ":");
            }
        }
        PathBuf::from(s)
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(decoded)
    }
}
