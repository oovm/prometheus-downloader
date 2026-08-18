//! Openverse image and audio extractor.

use prometheus_types::{
    EXTRACTOR_OPENVERSE, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Openverse work kind (`image` or `audio`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkKind {
    /// Still image.
    Image,
    /// Audio recording.
    Audio,
}

/// Origin + UUID extracted from an Openverse work URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkTarget {
    /// `image` or `audio`.
    pub kind: WorkKind,
    /// Openverse UUID.
    pub id: String,
}

/// Resolves Openverse image / audio pages via the public catalog API.
pub struct Openverse;

impl Extractor for Openverse {
    fn id(&self) -> &'static str {
        EXTRACTOR_OPENVERSE
    }

    fn matches(&self, url: &str) -> bool {
        work_target(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let target = work_target(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let kind = match target.kind {
            WorkKind::Image => "images",
            WorkKind::Audio => "audio",
        };
        let api = format!("https://api.openverse.org/v1/{kind}/{}/", target.id);
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_work_json(&body)
    }
}

/// Parse an Openverse image or audio UUID from a page or API URL.
pub fn work_target(url: &str) -> Option<WorkTarget> {
    let trimmed = url.trim();
    let rest = trimmed.strip_prefix("https://").or_else(|| trimmed.strip_prefix("http://"))?;
    let (host_raw, path_and_more) = rest.split_once('/')?;
    let host = host_raw.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    let path = path_and_more.split(['?', '#']).next().unwrap_or(path_and_more);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    match host {
        "openverse.org" => match segments.as_slice() {
            ["image", id, ..] => uuid_target(WorkKind::Image, id),
            ["audio", id, ..] => uuid_target(WorkKind::Audio, id),
            _ => None,
        },
        "api.openverse.org" => match segments.as_slice() {
            ["v1", "images", id, ..] => uuid_target(WorkKind::Image, id),
            ["v1", "audio", id, ..] => uuid_target(WorkKind::Audio, id),
            _ => None,
        },
        _ => None,
    }
}

fn uuid_target(kind: WorkKind, id: &str) -> Option<WorkTarget> {
    if looks_like_uuid(id) { Some(WorkTarget { kind, id: id.to_ascii_lowercase() }) } else { None }
}

fn looks_like_uuid(id: &str) -> bool {
    let parts: Vec<&str> = id.split('-').collect();
    parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Build [`MediaInfo`] from an Openverse work JSON body.
pub fn media_from_work_json(body: &str) -> Result<MediaInfo> {
    let parsed: WorkResponse = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("openverse json: {err}")))?;
    let url = parsed
        .url
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Network("openverse work has no media url".into()))?;
    let filename = filename_from_url(&url).or_else(|| {
        parsed.id.as_deref().map(|id| {
            let ext = parsed.filetype.as_deref().filter(|s| !s.is_empty()).unwrap_or("bin");
            sanitize_filename(&format!("{id}.{ext}"))
        })
    });
    Ok(MediaInfo {
        url,
        title: parsed.title.filter(|s| !s.is_empty()).or_else(|| filename.clone()),
        content_type: guess_content_type(parsed.filetype.as_deref(), filename.as_deref()),
        content_length: parsed.filesize,
        filename,
        extractor: EXTRACTOR_OPENVERSE.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct WorkResponse {
    id: Option<String>,
    title: Option<String>,
    url: Option<String>,
    filetype: Option<String>,
    filesize: Option<u64>,
}

fn guess_content_type(filetype: Option<&str>, filename: Option<&str>) -> Option<String> {
    let ext = filetype
        .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            filename.and_then(|name| name.rsplit_once('.')).map(|(_, ext)| ext.to_ascii_lowercase())
        })?;
    Some(match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg".into(),
        "png" => "image/png".into(),
        "gif" => "image/gif".into(),
        "webp" => "image/webp".into(),
        "mp3" => "audio/mpeg".into(),
        "ogg" | "oga" => "audio/ogg".into(),
        "wav" => "audio/wav".into(),
        "flac" => "audio/flac".into(),
        "mp4" => "video/mp4".into(),
        "webm" => "video/webm".into(),
        other => format!("application/{other}"),
    })
}
