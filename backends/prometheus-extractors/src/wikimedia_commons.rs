//! Wikimedia Commons file-page extractor.

use prometheus_types::{EXTRACTOR_WIKIMEDIA_COMMONS, Error, MediaInfo, Result, sanitize_filename};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves Commons `File:` pages to the upload.wikimedia.org media URL.
pub struct WikimediaCommons;

impl Extractor for WikimediaCommons {
    fn id(&self) -> &'static str {
        EXTRACTOR_WIKIMEDIA_COMMONS
    }

    fn matches(&self, url: &str) -> bool {
        file_page(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let page = file_page(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!(
            "https://{}/w/api.php?action=query&titles={}&prop=imageinfo&iiprop=url|size|mime&format=json",
            page.host,
            percent_encode(&page.title)
        );
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_api_json(&body)
    }
}

/// Host + `File:` title extracted from a Commons / Wikipedia file URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePage {
    /// Hostname used for `/w/api.php` (no scheme).
    pub host: String,
    /// Canonical `File:…` title.
    pub title: String,
}

/// Extract a `File:…` title from a Commons file URL.
pub fn file_title(url: &str) -> Option<String> {
    file_page(url).map(|page| page.title)
}

/// Parse Commons or Wikipedia `File:` page URLs.
pub fn file_page(url: &str) -> Option<FilePage> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("upload.wikimedia.org/") {
        return None;
    }

    let host = url_host(trimmed)?;
    if !is_mediawiki_file_host(&host) {
        return None;
    }

    if let Some(idx) = lower.find("/wiki/") {
        let rest = &trimmed[idx + "/wiki/".len()..];
        let segment = rest.split(['#', '?']).next().unwrap_or(rest);
        let title = normalize_file_title(segment)?;
        return Some(FilePage { host, title });
    }

    if let Some(idx) = lower.find("title=") {
        let rest = &trimmed[idx + "title=".len()..];
        let segment = rest.split(['#', '&']).next().unwrap_or(rest);
        let title = normalize_file_title(segment)?;
        return Some(FilePage { host, title });
    }

    None
}

fn url_host(url: &str) -> Option<String> {
    let rest =
        url.trim().strip_prefix("https://").or_else(|| url.trim().strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next()?.trim();
    if host.is_empty() { None } else { Some(host.to_ascii_lowercase()) }
}

fn is_mediawiki_file_host(host: &str) -> bool {
    let host = host.strip_prefix("www.").unwrap_or(host);
    host == "commons.wikimedia.org"
        || host == "wikipedia.org"
        || host.ends_with(".wikipedia.org")
        || host.ends_with(".wikimedia.org")
        || host.ends_with(".wiktionary.org")
        || host.ends_with(".wikibooks.org")
        || host.ends_with(".wikisource.org")
        || host.ends_with(".wikinews.org")
        || host.ends_with(".wikiversity.org")
        || host.ends_with(".wikivoyage.org")
        || host.ends_with(".wikiquote.org")
        || host.ends_with(".mediawiki.org")
}

fn normalize_file_title(segment: &str) -> Option<String> {
    let decoded = percent_decode(segment.replace('+', " "));
    if !decoded.to_ascii_lowercase().starts_with("file:") {
        return None;
    }
    Some(decoded.replace('_', " "))
}

/// Build [`MediaInfo`] from a Commons `action=query&prop=imageinfo` JSON body.
pub fn media_from_api_json(body: &str) -> Result<MediaInfo> {
    let parsed: ApiResponse =
        serde_json::from_str(body).map_err(|err| Error::Network(format!("commons json: {err}")))?;
    let page = parsed
        .query
        .pages
        .into_values()
        .next()
        .ok_or_else(|| Error::Network("commons api returned no pages".into()))?;
    if page.missing.is_some() {
        return Err(Error::Network(format!(
            "commons file missing: {}",
            page.title.unwrap_or_default()
        )));
    }
    let info = page
        .imageinfo
        .and_then(|list| list.into_iter().next())
        .ok_or_else(|| Error::Network("commons api returned no imageinfo".into()))?;
    let raw_url = strip_utm(&info.url);
    let filename =
        raw_url.rsplit('/').next().map(|s| s.split('?').next().unwrap_or(s)).map(sanitize_filename);
    let title = page.title.map(|t| t.trim_start_matches("File:").replace('_', " "));
    Ok(MediaInfo {
        url: raw_url,
        title: title.or_else(|| filename.clone()),
        content_type: info.mime,
        content_length: info.size,
        filename,
        extractor: EXTRACTOR_WIKIMEDIA_COMMONS.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    query: Query,
}

#[derive(Debug, Deserialize)]
struct Query {
    pages: std::collections::HashMap<String, Page>,
}

#[derive(Debug, Deserialize)]
struct Page {
    title: Option<String>,
    missing: Option<serde_json::Value>,
    imageinfo: Option<Vec<ImageInfo>>,
}

#[derive(Debug, Deserialize)]
struct ImageInfo {
    url: String,
    size: Option<u64>,
    mime: Option<String>,
}

fn strip_utm(url: &str) -> String {
    match url.split_once('?') {
        Some((base, _)) if base.contains("upload.wikimedia.org") => base.to_string(),
        _ => url.to_string(),
    }
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.replace(' ', "_").as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b':' => {
                out.push(*b as char)
            }
            byte => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn percent_decode(input: String) -> String {
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
