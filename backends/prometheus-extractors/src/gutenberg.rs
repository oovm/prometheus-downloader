//! Project Gutenberg extractor via the public Gutendex catalog.

use prometheus_types::{
    EXTRACTOR_GUTENBERG, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves `gutenberg.org/ebooks/{id}` via Gutendex `formats`.
pub struct Gutenberg;

impl Extractor for Gutenberg {
    fn id(&self) -> &'static str {
        EXTRACTOR_GUTENBERG
    }

    fn matches(&self, url: &str) -> bool {
        ebook_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = ebook_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!("https://gutendex.com/books/{id}");
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_book_json(&body)
    }
}

/// Parse a Gutenberg ebook id from a book page, files URL, or Gutendex URL.
pub fn ebook_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(idx) = lower.find("gutenberg.org/ebooks/") {
        return take_digits(&trimmed[idx + "gutenberg.org/ebooks/".len()..]);
    }
    if let Some(idx) = lower.find("gutendex.com/books/") {
        return take_digits(&trimmed[idx + "gutendex.com/books/".len()..]);
    }
    if let Some(idx) = lower.find("gutenberg.org/files/") {
        return take_digits(&trimmed[idx + "gutenberg.org/files/".len()..]);
    }
    None
}

fn take_digits(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#', '.']).next()?.trim();
    if seg.is_empty() || !seg.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(seg.to_string())
    }
}

/// Build [`MediaInfo`] from a Gutendex book JSON body.
pub fn media_from_book_json(body: &str) -> Result<MediaInfo> {
    let parsed: BookResponse = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("gutenberg json: {err}")))?;
    let (mime, url) = pick_format(&parsed.formats)
        .ok_or_else(|| Error::Network("gutenberg book has no downloadable format".into()))?;
    let filename = filename_from_url(&url).or_else(|| {
        parsed.id.map(|id| {
            let ext = extension_for(mime);
            sanitize_filename(&format!("{id}.{ext}"))
        })
    });
    Ok(MediaInfo {
        url,
        title: parsed.title.filter(|s| !s.is_empty()).or_else(|| filename.clone()),
        content_type: Some(mime.to_string()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_GUTENBERG.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct BookResponse {
    id: Option<u64>,
    title: Option<String>,
    formats: Option<std::collections::HashMap<String, String>>,
}

fn pick_format(
    formats: &Option<std::collections::HashMap<String, String>>,
) -> Option<(&str, String)> {
    let formats = formats.as_ref()?;
    let mut scored: Vec<(i32, &str, String)> = formats
        .iter()
        .filter(|(mime, url)| !url.is_empty() && keep_mime(mime))
        .map(|(mime, url)| (score_mime(mime), mime.as_str(), url.clone()))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    scored.into_iter().find(|(score, _, _)| *score > 0).map(|(_, mime, url)| (mime, url))
}

fn keep_mime(mime: &str) -> bool {
    let lower = mime.to_ascii_lowercase();
    if lower.contains("rdf+xml") || lower.contains("octet-stream") || lower.contains("x-mobipocket")
    {
        return false;
    }
    if lower.contains("zip") && !lower.contains("epub") {
        return false;
    }
    true
}

fn score_mime(mime: &str) -> i32 {
    let lower = mime.to_ascii_lowercase();
    if lower.starts_with("audio/mpeg") || lower == "audio/mp3" {
        100
    } else if lower.starts_with("audio/") {
        80
    } else if lower.contains("epub") {
        50
    } else if lower.starts_with("text/plain") && lower.contains("charset=utf-8") {
        35
    } else if lower.starts_with("text/plain") {
        25
    } else if lower.starts_with("image/jpeg") {
        15
    } else if lower.starts_with("text/html") {
        5
    } else {
        0
    }
}

fn extension_for(mime: &str) -> &'static str {
    let lower = mime.to_ascii_lowercase();
    if lower.starts_with("audio/mpeg") {
        "mp3"
    } else if lower.starts_with("audio/") {
        "audio"
    } else if lower.starts_with("image/") {
        "jpg"
    } else if lower.starts_with("application/epub") {
        "epub"
    } else if lower.starts_with("text/plain") {
        "txt"
    } else {
        "bin"
    }
}
