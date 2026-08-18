//! Victoria and Albert Museum collection extractor.

use prometheus_types::{
    EXTRACTOR_VAM, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves V&A collection item pages via the public object API.
pub struct Vam;

impl Extractor for Vam {
    fn id(&self) -> &'static str {
        EXTRACTOR_VAM
    }

    fn matches(&self, url: &str) -> bool {
        object_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = object_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!("https://api.vam.ac.uk/v2/object/{id}");
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_object_json(&body)
    }
}

/// Parse a V&A system number (`O…`) from a collection or API URL.
pub fn object_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(idx) = lower.find("collections.vam.ac.uk/item/") {
        return take_id(&trimmed[idx + "collections.vam.ac.uk/item/".len()..]);
    }
    if let Some(idx) = lower.find("api.vam.ac.uk/v2/object/") {
        return take_id(&trimmed[idx + "api.vam.ac.uk/v2/object/".len()..]);
    }
    None
}

fn take_id(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#']).next()?.trim();
    if seg.len() < 2 {
        return None;
    }
    let (first, rest_id) = seg.split_at(1);
    if !first.eq_ignore_ascii_case("O") || !rest_id.chars().all(|c| c.is_ascii_alphanumeric()) {
        None
    } else {
        Some(format!("O{rest_id}"))
    }
}

/// Build [`MediaInfo`] from a V&A object JSON body.
pub fn media_from_object_json(body: &str) -> Result<MediaInfo> {
    let parsed: Envelope =
        serde_json::from_str(body).map_err(|err| Error::Network(format!("vam json: {err}")))?;
    let iiif = parsed
        .meta
        .images
        .and_then(|img| img.iiif_image)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Network("vam object has no iiif image".into()))?;
    let url = iiif_full(&iiif);
    let filename = filename_from_url(&url)
        .or_else(|| parsed.record.system_number.map(|id| sanitize_filename(&format!("{id}.jpg"))));
    let title = parsed
        .record
        .titles
        .unwrap_or_default()
        .into_iter()
        .find_map(|t| t.title.filter(|s| !s.is_empty()));
    Ok(MediaInfo {
        url,
        title: title.or_else(|| filename.clone()),
        content_type: Some("image/jpeg".into()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_VAM.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct Envelope {
    meta: Meta,
    #[serde(default)]
    record: Record,
}

#[derive(Debug, Deserialize)]
struct Meta {
    images: Option<Images>,
}

#[derive(Debug, Deserialize)]
struct Images {
    #[serde(rename = "_iiif_image")]
    iiif_image: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Record {
    #[serde(rename = "systemNumber")]
    system_number: Option<String>,
    titles: Option<Vec<Title>>,
}

#[derive(Debug, Deserialize)]
struct Title {
    title: Option<String>,
}

fn iiif_full(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.contains("/full/") {
        format!("{trimmed}.jpg")
    } else {
        format!("{trimmed}/full/max/0/default.jpg")
    }
}
