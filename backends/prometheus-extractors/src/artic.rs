//! Art Institute of Chicago artwork extractor.

use prometheus_types::{
    EXTRACTOR_ARTIC, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves `artic.edu/artworks/{id}` via the public AIC API + IIIF image URL.
pub struct Artic;

impl Extractor for Artic {
    fn id(&self) -> &'static str {
        EXTRACTOR_ARTIC
    }

    fn matches(&self, url: &str) -> bool {
        artwork_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = artwork_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!(
            "https://api.artic.edu/api/v1/artworks/{id}?fields=id,title,image_id,is_public_domain"
        );
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_artwork_json(&body)
    }
}

/// Parse an AIC artwork id from a page or API URL.
pub fn artwork_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(idx) = lower.find("artic.edu/artworks/") {
        return take_id(&trimmed[idx + "artic.edu/artworks/".len()..]);
    }
    if let Some(idx) = lower.find("api.artic.edu/api/v1/artworks/") {
        return take_id(&trimmed[idx + "api.artic.edu/api/v1/artworks/".len()..]);
    }
    None
}

fn take_id(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#']).next()?.trim();
    let digits = seg.split('-').next().unwrap_or(seg);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(digits.to_string())
    }
}

/// Build [`MediaInfo`] from an AIC artwork JSON body.
pub fn media_from_artwork_json(body: &str) -> Result<MediaInfo> {
    let parsed: Envelope =
        serde_json::from_str(body).map_err(|err| Error::Network(format!("artic json: {err}")))?;
    let data = parsed.data.ok_or_else(|| Error::Network("artic api returned no data".into()))?;
    let image_id = data
        .image_id
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Network("artic artwork has no image_id".into()))?;
    let url = format!("https://www.artic.edu/iiif/2/{image_id}/full/max/0/default.jpg");
    let filename = filename_from_url(&url)
        .or_else(|| data.id.map(|id| sanitize_filename(&format!("{id}.jpg"))));
    Ok(MediaInfo {
        url,
        title: data.title.filter(|s| !s.is_empty()).or_else(|| filename.clone()),
        content_type: Some("image/jpeg".into()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_ARTIC.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct Envelope {
    data: Option<Artwork>,
}

#[derive(Debug, Deserialize)]
struct Artwork {
    id: Option<u64>,
    title: Option<String>,
    image_id: Option<String>,
}
