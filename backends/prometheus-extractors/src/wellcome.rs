//! Wellcome Collection image / work extractor.

use prometheus_types::{
    EXTRACTOR_WELLCOME, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Wellcome Collection target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WellcomeTarget {
    /// Catalogue image id (`nwqpxugw`).
    Image(String),
    /// Catalogue work id (`zv3drmps`).
    Work(String),
    /// IIIF asset id (`B0008032`).
    IiifAsset(String),
}

/// Resolves Wellcome Collection works / images via the public catalogue + IIIF APIs.
pub struct Wellcome;

impl Extractor for Wellcome {
    fn id(&self) -> &'static str {
        EXTRACTOR_WELLCOME
    }

    fn matches(&self, url: &str) -> bool {
        wellcome_target(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let target = wellcome_target(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        match target {
            WellcomeTarget::IiifAsset(asset) => Ok(MediaInfo {
                url: iiif_full(&asset),
                title: Some(asset.clone()),
                content_type: Some("image/jpeg".into()),
                content_length: None,
                filename: Some(sanitize_filename(&format!("{asset}.jpg"))),
                extractor: EXTRACTOR_WELLCOME.to_string(),
            }),
            WellcomeTarget::Image(id) => {
                let body = fetch_json(&format!(
                    "https://api.wellcomecollection.org/catalogue/v2/images/{id}"
                ))?;
                media_from_image_json(&body)
            }
            WellcomeTarget::Work(id) => {
                let body = fetch_json(&format!(
                    "https://api.wellcomecollection.org/catalogue/v2/works/{id}?include=images"
                ))?;
                let work: WorkResponse = serde_json::from_str(&body)
                    .map_err(|err| Error::Network(format!("wellcome json: {err}")))?;
                if let Some(image_id) = work
                    .images
                    .as_ref()
                    .and_then(|list| list.first())
                    .and_then(|img| img.id.clone())
                {
                    let image_body = fetch_json(&format!(
                        "https://api.wellcomecollection.org/catalogue/v2/images/{image_id}"
                    ))?;
                    let mut info = media_from_image_json(&image_body)?;
                    if info.title.is_none() {
                        info.title = work.title;
                    }
                    return Ok(info);
                }
                media_from_work_json(&body)
            }
        }
    }
}

fn fetch_json(api: &str) -> Result<String> {
    let resp = ureq::get(api)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()
        .map_err(|err| Error::Network(err.to_string()))?;
    resp.into_string().map_err(|err| Error::Network(err.to_string()))
}

/// Parse a Wellcome Collection page, API, or IIIF URL.
pub fn wellcome_target(url: &str) -> Option<WellcomeTarget> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();

    if let Some(idx) = lower.find("iiif.wellcomecollection.org/image/") {
        let rest = &trimmed[idx + "iiif.wellcomecollection.org/image/".len()..];
        let asset = rest.split(['/', '?', '#']).next()?.trim();
        if !asset.is_empty() {
            return Some(WellcomeTarget::IiifAsset(asset.to_string()));
        }
    }

    if let Some(idx) = lower.find("api.wellcomecollection.org/catalogue/v2/images/") {
        let id =
            take_id(&trimmed[idx + "api.wellcomecollection.org/catalogue/v2/images/".len()..])?;
        return Some(WellcomeTarget::Image(id));
    }
    if let Some(idx) = lower.find("api.wellcomecollection.org/catalogue/v2/works/") {
        let id = take_id(&trimmed[idx + "api.wellcomecollection.org/catalogue/v2/works/".len()..])?;
        return Some(WellcomeTarget::Work(id));
    }

    if lower.contains("wellcomecollection.org/works/") {
        if let Some(image_id) = query_param(trimmed, "id") {
            if is_catalogue_id(&image_id) {
                return Some(WellcomeTarget::Image(image_id));
            }
        }
        if let Some(idx) = lower.find("/works/") {
            let id = take_id(&trimmed[idx + "/works/".len()..])?;
            return Some(WellcomeTarget::Work(id));
        }
    }

    None
}

fn take_id(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#']).next()?.trim();
    if is_catalogue_id(seg) { Some(seg.to_string()) } else { None }
}

fn is_catalogue_id(id: &str) -> bool {
    (6..=16).contains(&id.len()) && id.chars().all(|c| c.is_ascii_alphanumeric())
}

fn query_param(url: &str, key: &str) -> Option<String> {
    let q = url.split_once('?')?.1;
    for pair in q.split('&') {
        let (k, v) = pair.split_once('=')?;
        if k.eq_ignore_ascii_case(key) && !v.is_empty() {
            return Some(v.split('#').next().unwrap_or(v).to_string());
        }
    }
    None
}

/// Build [`MediaInfo`] from a Wellcome image JSON body.
pub fn media_from_image_json(body: &str) -> Result<MediaInfo> {
    let parsed: ImageResponse = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("wellcome json: {err}")))?;
    let info = pick_iiif(&parsed.thumbnail)
        .or_else(|| parsed.locations.as_ref().and_then(|list| list.iter().find_map(location_iiif)))
        .ok_or_else(|| Error::Network("wellcome image has no iiif location".into()))?;
    let url = normalize_iiif(&info);
    let filename = filename_from_url(&url)
        .or_else(|| parsed.id.as_deref().map(|id| sanitize_filename(&format!("{id}.jpg"))));
    let title =
        parsed.source.and_then(|s| s.title).filter(|s| !s.is_empty()).or_else(|| filename.clone());
    Ok(MediaInfo {
        url,
        title,
        content_type: Some("image/jpeg".into()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_WELLCOME.to_string(),
    })
}

/// Build [`MediaInfo`] from a Wellcome work JSON body that already embeds a thumbnail.
pub fn media_from_work_json(body: &str) -> Result<MediaInfo> {
    let parsed: WorkResponse = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("wellcome json: {err}")))?;
    let info = pick_iiif(&parsed.thumbnail)
        .ok_or_else(|| Error::Network("wellcome work has no digital images".into()))?;
    let url = normalize_iiif(&info);
    let filename = filename_from_url(&url)
        .or_else(|| parsed.id.map(|id| sanitize_filename(&format!("{id}.jpg"))));
    Ok(MediaInfo {
        url,
        title: parsed.title.filter(|s| !s.is_empty()).or_else(|| filename.clone()),
        content_type: Some("image/jpeg".into()),
        content_length: None,
        filename,
        extractor: EXTRACTOR_WELLCOME.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct ImageResponse {
    id: Option<String>,
    thumbnail: Option<Location>,
    locations: Option<Vec<Location>>,
    source: Option<Source>,
}

#[derive(Debug, Deserialize)]
struct WorkResponse {
    id: Option<String>,
    title: Option<String>,
    thumbnail: Option<Location>,
    images: Option<Vec<ImageStub>>,
}

#[derive(Debug, Deserialize)]
struct ImageStub {
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Source {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Location {
    url: Option<String>,
}

fn pick_iiif(loc: &Option<Location>) -> Option<String> {
    loc.as_ref().and_then(location_iiif)
}

fn location_iiif(loc: &Location) -> Option<String> {
    loc.url.clone().filter(|s| s.contains("iiif.wellcomecollection.org/image/"))
}

fn normalize_iiif(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.ends_with("/info.json") {
        let base = trimmed.trim_end_matches("/info.json").trim_end_matches('/');
        if let Some(asset) = base.rsplit('/').next() {
            return iiif_full(asset);
        }
    }
    if let Some(idx) = trimmed.find("/full/") {
        let head = &trimmed[..idx];
        if let Some(asset) = head.rsplit('/').next() {
            return iiif_full(asset);
        }
    }
    if !trimmed.contains("/full/") {
        if let Some(asset) = trimmed.trim_end_matches('/').rsplit('/').next() {
            if !asset.is_empty() && !asset.contains('.') {
                return iiif_full(asset);
            }
        }
    }
    trimmed.to_string()
}

fn iiif_full(asset: &str) -> String {
    format!("https://iiif.wellcomecollection.org/image/{asset}/full/max/0/default.jpg")
}
