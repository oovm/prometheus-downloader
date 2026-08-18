//! Open Library work / edition extractor.

use prometheus_types::{
    EXTRACTOR_OPEN_LIBRARY, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Open Library resource kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    /// `/works/OL…W`
    Work,
    /// `/books/OL…M`
    Edition,
}

/// Parsed Open Library target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryTarget {
    /// Work or edition.
    pub kind: ResourceKind,
    /// Open Library id (`OL45804W` / `OL7353617M`).
    pub id: String,
}

/// Resolves Open Library work / edition pages to a cover image URL.
pub struct OpenLibrary;

impl Extractor for OpenLibrary {
    fn id(&self) -> &'static str {
        EXTRACTOR_OPEN_LIBRARY
    }

    fn matches(&self, url: &str) -> bool {
        library_target(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let target = library_target(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let path = match target.kind {
            ResourceKind::Work => format!("works/{}", target.id),
            ResourceKind::Edition => format!("books/{}", target.id),
        };
        let api = format!("https://openlibrary.org/{path}.json");
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_library_json(&body, &target)
    }
}

/// Parse an Open Library work or edition id from a page or JSON URL.
pub fn library_target(url: &str) -> Option<LibraryTarget> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.contains("openlibrary.org/") {
        return None;
    }
    if let Some(idx) = lower.find("/works/") {
        let id = take_ol_id(&trimmed[idx + "/works/".len()..], 'W')?;
        return Some(LibraryTarget { kind: ResourceKind::Work, id });
    }
    if let Some(idx) = lower.find("/books/") {
        let id = take_ol_id(&trimmed[idx + "/books/".len()..], 'M')?;
        return Some(LibraryTarget { kind: ResourceKind::Edition, id });
    }
    None
}

fn take_ol_id(rest: &str, suffix: char) -> Option<String> {
    let seg = rest.split(['/', '?', '#', '.']).next()?.trim();
    if seg.len() < 4 {
        return None;
    }
    let upper = seg.to_ascii_uppercase();
    if !upper.starts_with("OL") {
        return None;
    }
    let digits = &upper[2..upper.len() - 1];
    let last = upper.chars().last()?;
    if last != suffix || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(upper)
}

/// Build [`MediaInfo`] from an Open Library work/edition JSON body.
pub fn media_from_library_json(body: &str, target: &LibraryTarget) -> Result<MediaInfo> {
    let parsed: LibraryDoc = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("openlibrary json: {err}")))?;
    let cover_id = pick_cover_id(&parsed).ok_or_else(|| {
        Error::Network(format!(
            "openlibrary {} `{}` has no cover id",
            kind_label(target.kind),
            target.id
        ))
    })?;
    let url = format!("https://covers.openlibrary.org/b/id/{cover_id}-L.jpg");
    let filename =
        filename_from_url(&url).or_else(|| Some(sanitize_filename(&format!("{}.jpg", target.id))));
    Ok(MediaInfo {
        url,
        title: title_of(&parsed).or_else(|| filename.clone()),
        content_type: Some("image/jpeg".into()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_OPEN_LIBRARY.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct LibraryDoc {
    title: Option<String>,
    covers: Option<Vec<i64>>,
    #[serde(rename = "cover_i")]
    cover_i: Option<i64>,
}

fn pick_cover_id(doc: &LibraryDoc) -> Option<i64> {
    if let Some(id) = doc.cover_i.filter(|n| *n > 0) {
        return Some(id);
    }
    doc.covers.as_ref()?.iter().copied().find(|n| *n > 0)
}

fn title_of(doc: &LibraryDoc) -> Option<String> {
    doc.title.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Work => "work",
        ResourceKind::Edition => "edition",
    }
}
