//! PeerTube watch-page extractor (public REST API).

use prometheus_types::{
    EXTRACTOR_PEERTUBE, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves PeerTube `/w/` and `/videos/watch/` URLs via `/api/v1/videos/{id}`.
pub struct PeerTube;

impl Extractor for PeerTube {
    fn id(&self) -> &'static str {
        EXTRACTOR_PEERTUBE
    }

    fn matches(&self, url: &str) -> bool {
        watch_target(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let target = watch_target(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!("{}/api/v1/videos/{}", target.origin, target.id);
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_video_json(&body)
    }
}

/// Origin + video id extracted from a PeerTube watch URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchTarget {
    /// `https://host` with no trailing slash.
    pub origin: String,
    /// UUID or short UUID path segment.
    pub id: String,
}

/// Parse a PeerTube watch / embed URL.
pub fn watch_target(url: &str) -> Option<WatchTarget> {
    let trimmed = url.trim();
    let (scheme, rest) = split_scheme(trimmed)?;
    let (host, path) = rest.split_once('/')?;
    if host.is_empty() || !is_peertube_candidate_host(host) {
        return None;
    }
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let id = video_id_from_path(path)?;
    Some(WatchTarget { origin: format!("{scheme}://{host}"), id })
}

fn is_peertube_candidate_host(host: &str) -> bool {
    let lowered = host.to_ascii_lowercase();
    let host = lowered.strip_prefix("www.").unwrap_or(&lowered);
    let blocked = [
        "archive.org",
        "youtube.com",
        "youtu.be",
        "vimeo.com",
        "dailymotion.com",
        "github.com",
        "gitlab.com",
        "wikipedia.org",
        "wikimedia.org",
        "imgur.com",
        "flickr.com",
        "ted.com",
        "bandcamp.com",
    ];
    if blocked.iter().any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}"))) {
        return false;
    }
    true
}

fn split_scheme(url: &str) -> Option<(&str, &str)> {
    let lower = url.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("https://") {
        let _ = rest;
        Some(("https", url.get(8..)?))
    } else if let Some(rest) = lower.strip_prefix("http://") {
        let _ = rest;
        Some(("http", url.get(7..)?))
    } else {
        None
    }
}

fn video_id_from_path(path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let (id, short_watch) = match segments.as_slice() {
        ["w", id, ..] => (*id, true),
        ["videos", "watch", id, ..] | ["videos", "embed", id, ..] => (*id, false),
        _ => return None,
    };
    if is_video_id(id) && (!short_watch || looks_like_uuid(id) || id.len() >= 16) {
        Some(id.to_string())
    } else {
        None
    }
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

fn is_video_id(id: &str) -> bool {
    if id.len() < 4 || id.len() > 64 {
        return false;
    }
    let lower = id.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "trending"
            | "overview"
            | "local"
            | "recent"
            | "most-liked"
            | "about"
            | "login"
            | "signup"
            | "search"
            | "accounts"
            | "videos"
    ) {
        return false;
    }
    id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Build [`MediaInfo`] from a PeerTube `GET /api/v1/videos/{id}` JSON body.
pub fn media_from_video_json(body: &str) -> Result<MediaInfo> {
    let parsed: VideoResponse = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("peertube json: {err}")))?;
    let chosen = pick_file(&parsed)
        .ok_or_else(|| Error::Network("peertube video has no downloadable fileUrl".into()))?;
    let filename = filename_from_url(&chosen.url)
        .or_else(|| parsed.name.as_deref().map(|n| sanitize_filename(&format!("{n}.mp4"))));
    let content_type = guess_content_type(&chosen.url);
    Ok(MediaInfo {
        url: chosen.url,
        title: parsed.name.clone().or_else(|| filename.clone()),
        content_type,
        content_length: chosen.size,
        filename,
        extractor: EXTRACTOR_PEERTUBE.to_string(),
    })
}

#[derive(Clone)]
struct ChosenFile {
    url: String,
    size: Option<u64>,
    height: i64,
}

#[derive(Debug, Deserialize)]
struct VideoResponse {
    name: Option<String>,
    files: Option<Vec<VideoFile>>,
    #[serde(rename = "streamingPlaylists")]
    streaming_playlists: Option<Vec<StreamingPlaylist>>,
}

#[derive(Debug, Deserialize)]
struct StreamingPlaylist {
    files: Option<Vec<VideoFile>>,
}

#[derive(Debug, Deserialize)]
struct VideoFile {
    #[serde(rename = "fileUrl")]
    file_url: Option<String>,
    #[serde(rename = "fileDownloadUrl")]
    file_download_url: Option<String>,
    size: Option<u64>,
    resolution: Option<Resolution>,
}

#[derive(Debug, Deserialize)]
struct Resolution {
    id: Option<i64>,
}

fn pick_file(video: &VideoResponse) -> Option<ChosenFile> {
    let mut candidates = Vec::new();
    collect_files(video.files.as_deref(), &mut candidates);
    if let Some(lists) = video.streaming_playlists.as_deref() {
        for list in lists {
            collect_files(list.files.as_deref(), &mut candidates);
        }
    }
    candidates.sort_by(|a, b| {
        b.height.cmp(&a.height).then(b.size.unwrap_or(0).cmp(&a.size.unwrap_or(0)))
    });
    let best_video = candidates.iter().find(|c| c.height > 0).cloned();
    best_video.or_else(|| candidates.into_iter().next())
}

fn collect_files(files: Option<&[VideoFile]>, out: &mut Vec<ChosenFile>) {
    let Some(files) = files else { return };
    for file in files {
        let url = file
            .file_url
            .as_deref()
            .or(file.file_download_url.as_deref())
            .filter(|s| !s.is_empty());
        let Some(url) = url else { continue };
        out.push(ChosenFile {
            url: url.to_string(),
            size: file.size,
            height: file.resolution.as_ref().and_then(|r| r.id).unwrap_or(0),
        });
    }
}

fn guess_content_type(url: &str) -> Option<String> {
    let lower = url.to_ascii_lowercase();
    if lower.contains(".webm") {
        Some("video/webm".into())
    } else if lower.contains(".mp4") {
        Some("video/mp4".into())
    } else {
        None
    }
}
