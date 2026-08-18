//! NASA Images and Video Library extractor.

use prometheus_types::{
    EXTRACTOR_NASA_IMAGES, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves `images.nasa.gov` detail pages via the public asset API.
pub struct NasaImages;

impl Extractor for NasaImages {
    fn id(&self) -> &'static str {
        EXTRACTOR_NASA_IMAGES
    }

    fn matches(&self, url: &str) -> bool {
        nasa_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = nasa_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!("https://images-api.nasa.gov/asset/{id}");
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_asset_json(&body, &id)
    }
}

/// Parse a NASA Images nasa_id from a details or asset URL.
pub fn nasa_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    let rest = trimmed.strip_prefix("https://").or_else(|| trimmed.strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next()?.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    if host != "images.nasa.gov" && host != "images-api.nasa.gov" {
        return None;
    }
    if let Some(idx) = lower.find("/details-") {
        return take_id(&trimmed[idx + "/details-".len()..]);
    }
    if let Some(idx) = lower.find("/details/") {
        return take_id(&trimmed[idx + "/details/".len()..]);
    }
    if let Some(idx) = lower.find("/asset/") {
        return take_id(&trimmed[idx + "/asset/".len()..]);
    }
    None
}

fn take_id(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#']).next()?.trim();
    if seg.is_empty() || seg.len() > 128 { None } else { Some(seg.to_string()) }
}

/// Build [`MediaInfo`] from a NASA `GET /asset/{id}` JSON body.
pub fn media_from_asset_json(body: &str, fallback_id: &str) -> Result<MediaInfo> {
    let parsed: AssetResponse =
        serde_json::from_str(body).map_err(|err| Error::Network(format!("nasa json: {err}")))?;
    let items = parsed.collection.items.unwrap_or_default();
    let url = pick_href(&items).ok_or_else(|| {
        Error::Network(format!("nasa asset `{fallback_id}` has no downloadable media href"))
    })?;
    let filename =
        filename_from_url(&url).or_else(|| Some(sanitize_filename(&format!("{fallback_id}.jpg"))));
    Ok(MediaInfo {
        url,
        title: filename.clone(),
        content_type: guess_content_type(filename.as_deref().unwrap_or("")),
        content_length: None,
        filename,
        extractor: EXTRACTOR_NASA_IMAGES.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct AssetResponse {
    collection: Collection,
}

#[derive(Debug, Deserialize)]
struct Collection {
    items: Option<Vec<AssetItem>>,
}

#[derive(Debug, Deserialize)]
struct AssetItem {
    href: Option<String>,
}

fn pick_href(items: &[AssetItem]) -> Option<String> {
    let mut scored: Vec<(i32, String)> = items
        .iter()
        .filter_map(|item| item.href.as_deref())
        .filter(|href| is_media_href(href))
        .map(|href| {
            let https = to_https(href);
            (score_href(&https), https)
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, href)| href).next()
}

fn is_media_href(href: &str) -> bool {
    let lower = href.to_ascii_lowercase();
    if lower.ends_with("metadata.json") || lower.ends_with(".srt") || lower.ends_with(".vtt") {
        return false;
    }
    lower.contains("~orig.")
        || lower.contains("~large.")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".tif")
        || lower.ends_with(".tiff")
        || lower.ends_with(".mp4")
        || lower.ends_with(".mov")
        || lower.ends_with(".webm")
}

fn score_href(href: &str) -> i32 {
    let lower = href.to_ascii_lowercase();
    if lower.contains("~thumb") || lower.contains("~small") {
        return -20;
    }
    if lower.contains("~orig") {
        return 100;
    }
    if lower.ends_with(".mp4") || lower.ends_with(".mov") || lower.ends_with(".webm") {
        return 80;
    }
    if lower.contains("~large") {
        return 60;
    }
    if lower.contains("~medium") {
        return 20;
    }
    10
}

fn to_https(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("http://") {
        format!("https://{rest}")
    } else {
        url.to_string()
    }
}

fn guess_content_type(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".png") {
        Some("image/png".into())
    } else if lower.ends_with(".tif") || lower.ends_with(".tiff") {
        Some("image/tiff".into())
    } else if lower.ends_with(".mp4") {
        Some("video/mp4".into())
    } else if lower.ends_with(".webm") {
        Some("video/webm".into())
    } else if lower.ends_with(".mov") {
        Some("video/quicktime".into())
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg".into())
    } else {
        None
    }
}
