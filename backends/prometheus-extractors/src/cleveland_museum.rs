//! Cleveland Museum of Art open-access extractor.

use prometheus_types::{
    EXTRACTOR_CLEVELAND_MUSEUM, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves Cleveland collection pages via the public open-access API.
pub struct ClevelandMuseum;

impl Extractor for ClevelandMuseum {
    fn id(&self) -> &'static str {
        EXTRACTOR_CLEVELAND_MUSEUM
    }

    fn matches(&self, url: &str) -> bool {
        artwork_ref(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let target = artwork_ref(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = match &target {
            ArtworkRef::Id(id) => {
                format!("https://openaccess-api.clevelandart.org/api/artworks/{id}")
            }
            ArtworkRef::Accession(acc) => format!(
                "https://openaccess-api.clevelandart.org/api/artworks/?accession_number={}",
                percent_encode(acc)
            ),
        };
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_artwork_json(&body)
    }
}

/// Numeric API id or accession number taken from a Cleveland URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtworkRef {
    /// Open-access API numeric id.
    Id(String),
    /// Accession number as shown on `/art/{accession}`.
    Accession(String),
}

/// Parse a Cleveland artwork reference from a page or API URL.
pub fn artwork_ref(url: &str) -> Option<ArtworkRef> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(idx) = lower.find("openaccess-api.clevelandart.org/api/artworks/") {
        let rest = &trimmed[idx + "openaccess-api.clevelandart.org/api/artworks/".len()..];
        if rest.starts_with('?') {
            return accession_from_query(rest);
        }
        let seg = rest.split(['/', '?', '#']).next()?.trim();
        if !seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit()) {
            return Some(ArtworkRef::Id(seg.to_string()));
        }
        return None;
    }
    if let Some(idx) = lower.find("clevelandart.org/art/") {
        let rest = &trimmed[idx + "clevelandart.org/art/".len()..];
        let seg = rest.split(['/', '?', '#']).next()?.trim();
        if seg.is_empty() || seg.eq_ignore_ascii_case("artworks") {
            return None;
        }
        return Some(ArtworkRef::Accession(percent_decode(seg.replace('+', " "))));
    }
    None
}

fn accession_from_query(query: &str) -> Option<ArtworkRef> {
    let q = query.trim_start_matches('?');
    for pair in q.split('&') {
        let (key, value) = pair.split_once('=')?;
        if key.eq_ignore_ascii_case("accession_number") && !value.is_empty() {
            return Some(ArtworkRef::Accession(percent_decode(value.replace('+', " "))));
        }
    }
    None
}

/// Build [`MediaInfo`] from a Cleveland open-access JSON body.
pub fn media_from_artwork_json(body: &str) -> Result<MediaInfo> {
    let parsed: Envelope = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("cleveland json: {err}")))?;
    let artwork = parsed
        .into_artwork()
        .ok_or_else(|| Error::Network("cleveland api returned no artwork".into()))?;
    let url = pick_image(&artwork.images)
        .ok_or_else(|| Error::Network("cleveland artwork has no image url".into()))?;
    let filename = filename_from_url(&url).or_else(|| {
        artwork.accession_number.as_deref().map(|acc| sanitize_filename(&format!("{acc}.jpg")))
    });
    Ok(MediaInfo {
        url,
        title: artwork.title.filter(|s| !s.is_empty()).or_else(|| filename.clone()),
        content_type: Some("image/jpeg".into()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_CLEVELAND_MUSEUM.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct Envelope {
    data: ClevelandData,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ClevelandData {
    One(Artwork),
    Many(Vec<Artwork>),
}

impl Envelope {
    fn into_artwork(self) -> Option<Artwork> {
        match self.data {
            ClevelandData::One(art) => Some(art),
            ClevelandData::Many(list) => list.into_iter().next(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Artwork {
    title: Option<String>,
    accession_number: Option<String>,
    images: Option<Images>,
}

#[derive(Debug, Deserialize)]
struct Images {
    print: Option<SizedImage>,
    full: Option<SizedImage>,
    web: Option<SizedImage>,
}

#[derive(Debug, Deserialize)]
struct SizedImage {
    url: Option<String>,
}

fn pick_image(images: &Option<Images>) -> Option<String> {
    let images = images.as_ref()?;
    for slot in [&images.print, &images.full, &images.web] {
        if let Some(url) = slot.as_ref().and_then(|img| img.url.clone()).filter(|s| !s.is_empty()) {
            return Some(url);
        }
    }
    None
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(*b as char),
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
