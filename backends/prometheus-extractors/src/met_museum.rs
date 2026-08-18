//! Metropolitan Museum of Art collection extractor.

use prometheus_types::{
    EXTRACTOR_MET_MUSEUM, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves Met collection object pages via the public collection API.
pub struct MetMuseum;

impl Extractor for MetMuseum {
    fn id(&self) -> &'static str {
        EXTRACTOR_MET_MUSEUM
    }

    fn matches(&self, url: &str) -> bool {
        object_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = object_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!("https://collectionapi.metmuseum.org/public/collection/v1/objects/{id}");
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_object_json(&body)
    }
}

/// Parse a Met object id from a collection page or API URL.
pub fn object_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(idx) = lower.find("metmuseum.org/art/collection/search/") {
        return take_digits(&trimmed[idx + "metmuseum.org/art/collection/search/".len()..]);
    }
    if let Some(idx) = lower.find("collectionapi.metmuseum.org/public/collection/v1/objects/") {
        return take_digits(
            &trimmed[idx + "collectionapi.metmuseum.org/public/collection/v1/objects/".len()..],
        );
    }
    None
}

fn take_digits(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#']).next()?.trim();
    if seg.is_empty() || !seg.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(seg.to_string())
    }
}

/// Build [`MediaInfo`] from a Met collection object JSON body.
pub fn media_from_object_json(body: &str) -> Result<MediaInfo> {
    let parsed: ObjectResponse =
        serde_json::from_str(body).map_err(|err| Error::Network(format!("met json: {err}")))?;
    let url = nonempty(parsed.primary_image)
        .or_else(|| nonempty(parsed.primary_image_small))
        .ok_or_else(|| Error::Network("met object has no primaryImage".into()))?;
    let filename = filename_from_url(&url)
        .or_else(|| parsed.object_id.map(|id| sanitize_filename(&format!("{id}.jpg"))));
    Ok(MediaInfo {
        url,
        title: nonempty(parsed.title).or_else(|| filename.clone()),
        content_type: Some("image/jpeg".into()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_MET_MUSEUM.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct ObjectResponse {
    #[serde(rename = "objectID")]
    object_id: Option<u64>,
    title: Option<String>,
    #[serde(rename = "primaryImage")]
    primary_image: Option<String>,
    #[serde(rename = "primaryImageSmall")]
    primary_image_small: Option<String>,
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.is_empty())
}
