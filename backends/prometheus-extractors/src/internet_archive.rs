//! Internet Archive item extractor (`archive.org`).

use prometheus_types::{EXTRACTOR_INTERNET_ARCHIVE, Error, MediaInfo, Result, sanitize_filename};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));
const DETAILS_MARKERS: &[&str] = &["archive.org/details/", "www.archive.org/details/"];

/// Resolves Internet Archive `/details/<id>` URLs to a downloadable file.
pub struct InternetArchive;

impl Extractor for InternetArchive {
    fn id(&self) -> &'static str {
        EXTRACTOR_INTERNET_ARCHIVE
    }

    fn matches(&self, url: &str) -> bool {
        item_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = item_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let meta_url = format!("https://archive.org/metadata/{id}");
        let resp = ureq::get(&meta_url)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_metadata_json(&body, &id)
    }
}

fn take_segment(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#']).next().unwrap_or("").trim();
    if seg.is_empty() { None } else { Some(seg.to_string()) }
}

/// Parse an IA item identifier from a details / download / metadata URL.
pub fn item_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    for marker in DETAILS_MARKERS {
        if let Some(idx) = lower.find(marker) {
            return take_segment(&trimmed[idx + marker.len()..]);
        }
    }
    if let Some(idx) = lower.find("archive.org/download/") {
        return take_segment(&trimmed[idx + "archive.org/download/".len()..]);
    }
    if let Some(idx) = lower.find("archive.org/metadata/") {
        return take_segment(&trimmed[idx + "archive.org/metadata/".len()..]);
    }
    None
}

#[derive(Debug, Deserialize)]
struct MetadataResponse {
    metadata: Option<ItemMeta>,
    files: Option<Vec<IaFile>>,
}

#[derive(Debug, Deserialize)]
struct ItemMeta {
    title: Option<String>,
    identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IaFile {
    name: String,
    format: Option<String>,
    size: Option<String>,
    source: Option<String>,
    length: Option<String>,
}

/// Build [`MediaInfo`] from an Internet Archive metadata JSON body.
pub fn media_from_metadata_json(body: &str, fallback_id: &str) -> Result<MediaInfo> {
    let parsed: MetadataResponse = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("metadata json: {err}")))?;
    let id = parsed
        .metadata
        .as_ref()
        .and_then(|m| m.identifier.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback_id.to_string());
    let title = parsed.metadata.as_ref().and_then(|m| m.title.clone());
    let files = parsed.files.unwrap_or_default();
    let chosen = pick_file(&files).ok_or_else(|| {
        Error::Network(format!("internet archive item `{id}` has no downloadable media file"))
    })?;
    let download_url = format!("https://archive.org/download/{id}/{}", chosen.name);
    let content_length = chosen.size.as_deref().and_then(|s| s.parse::<u64>().ok());
    let filename = Some(sanitize_filename(&chosen.name));
    Ok(MediaInfo {
        url: download_url,
        title: title.or_else(|| filename.clone()),
        content_type: guess_content_type(&chosen.name, chosen.format.as_deref()),
        content_length,
        filename,
        extractor: EXTRACTOR_INTERNET_ARCHIVE.to_string(),
    })
}

fn pick_file(files: &[IaFile]) -> Option<&IaFile> {
    let mut scored: Vec<(i32, &IaFile)> =
        files.iter().filter(|f| is_media_candidate(f)).map(|f| (score_file(f), f)).collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    scored.into_iter().map(|(_, f)| f).next()
}

fn is_media_candidate(file: &IaFile) -> bool {
    let name = file.name.to_ascii_lowercase();
    if name.starts_with("__ia_thumb") || name.contains(".thumbs/") {
        return false;
    }
    if name.ends_with(".xml") || name.ends_with(".torrent") || name.ends_with(".m3u") {
        return false;
    }
    let format = file.format.as_deref().unwrap_or("").to_ascii_lowercase();
    if matches!(
        format.as_str(),
        "metadata"
            | "item tile"
            | "archive bittorrent"
            | "thumbnail"
            | "png"
            | "jpeg"
            | "animated gif"
    ) {
        return false;
    }
    has_media_extension(&name)
        || format.contains("mp3")
        || format.contains("mp4")
        || format.contains("ogg")
        || format.contains("flac")
        || format.contains("wav")
        || format.contains("webm")
        || format.contains("mpeg")
        || format.contains("video")
        || format.contains("audio")
        || format.contains("cinepack")
        || format.contains("matroska")
}

fn has_media_extension(name: &str) -> bool {
    const EXTS: &[&str] = &[
        ".mp3", ".mp4", ".m4a", ".ogg", ".oga", ".ogv", ".flac", ".wav", ".webm", ".mkv", ".avi",
        ".mov", ".opus", ".aac",
    ];
    EXTS.iter().any(|ext| name.ends_with(ext))
}

fn score_file(file: &IaFile) -> i32 {
    let mut score = 0;
    if file.source.as_deref() == Some("original") {
        score += 50;
    }
    let name = file.name.to_ascii_lowercase();
    if name.ends_with(".mp3") || name.ends_with(".mp4") || name.ends_with(".webm") {
        score += 20;
    }
    if name.ends_with(".avi") || name.ends_with(".mkv") {
        score += 10;
    }
    if name.ends_with(".zip") {
        score -= 30;
    }
    if file.length.is_some() {
        score += 5;
    }
    score
}

fn guess_content_type(name: &str, format: Option<&str>) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".mp3") {
        return Some("audio/mpeg".into());
    }
    if lower.ends_with(".ogg") || lower.ends_with(".oga") {
        return Some("audio/ogg".into());
    }
    if lower.ends_with(".ogv") {
        return Some("video/ogg".into());
    }
    if lower.ends_with(".mp4") {
        return Some("video/mp4".into());
    }
    if lower.ends_with(".webm") {
        return Some("video/webm".into());
    }
    if lower.ends_with(".wav") {
        return Some("audio/wav".into());
    }
    format.map(|s| s.to_string())
}
