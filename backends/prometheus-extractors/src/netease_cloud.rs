//! NetEase Cloud Music song page extractor (public free tracks only).

use prometheus_types::{
    EXTRACTOR_NETEASE_CLOUD, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));
const REFERER: &str = "https://music.163.com/";

/// Resolves `music.163.com` song pages when a public direct media URL is available.
pub struct NeteaseCloud;

impl Extractor for NeteaseCloud {
    fn id(&self) -> &'static str {
        EXTRACTOR_NETEASE_CLOUD
    }

    fn matches(&self, url: &str) -> bool {
        song_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = song_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let detail = fetch_json(&format!("https://music.163.com/api/song/detail/?ids=%5B{id}%5D"))?;
        let player = fetch_json(&format!(
            "https://music.163.com/api/song/enhance/player/url?id={id}&ids=%5B{id}%5D&br=128000"
        ))?;
        media_from_detail_and_player_json(&detail, &player)
    }
}

fn fetch_json(api: &str) -> Result<String> {
    let resp = ureq::get(api)
        .set("User-Agent", USER_AGENT)
        .set("Referer", REFERER)
        .set("Accept", "application/json")
        .call()
        .map_err(|err| Error::Network(err.to_string()))?;
    resp.into_string().map_err(|err| Error::Network(err.to_string()))
}

/// Parse a NetEase Cloud Music song id from common page URL shapes.
///
/// Accepts `…/song?id=`, `…/m/song?id=`, and hash-router forms (`#/song?id=`, `#m/song?id=`).
pub fn song_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !is_netease_host(&lower) {
        return None;
    }

    // Fold hash-router prefixes into a plain path so `/#/song?id=` looks like `/song?id=`.
    let normalized =
        trimmed.replace("/#/", "/").replace("/#m/", "/m/").replace("#/", "/").replace("#m/", "/m/");
    let norm_lower = normalized.to_ascii_lowercase();

    let is_song_page = norm_lower.contains("/song?")
        || norm_lower.contains("/m/song?")
        || norm_lower.contains("/song&")
        || norm_lower.contains("/m/song&");
    if !is_song_page {
        return None;
    }

    query_param(&normalized, "id").and_then(|id| take_digits(&id))
}

fn is_netease_host(lower_url: &str) -> bool {
    lower_url.contains("://music.163.com")
        || lower_url.contains("://y.music.163.com")
        || lower_url.starts_with("music.163.com")
        || lower_url.starts_with("y.music.163.com")
}

fn query_param(url: &str, key: &str) -> Option<String> {
    let q = url.split_once('?')?.1;
    for pair in q.split('&') {
        let pair = pair.split('#').next().unwrap_or(pair);
        let (k, v) = pair.split_once('=')?;
        if k.eq_ignore_ascii_case(key) && !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

fn take_digits(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#', '&', '.']).next()?.trim();
    if seg.is_empty() || !seg.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(seg.to_string())
    }
}

/// Build [`MediaInfo`] from song detail + player URL JSON bodies.
pub fn media_from_detail_and_player_json(
    detail_body: &str,
    player_body: &str,
) -> Result<MediaInfo> {
    let detail: DetailResponse = serde_json::from_str(detail_body)
        .map_err(|err| Error::Network(format!("netease detail json: {err}")))?;
    let player: PlayerResponse = serde_json::from_str(player_body)
        .map_err(|err| Error::Network(format!("netease player json: {err}")))?;

    let song = detail
        .songs
        .into_iter()
        .next()
        .ok_or_else(|| Error::Network("netease song not found".into()))?;
    let song_id = song.id.unwrap_or(0);
    let title = song.name.filter(|s| !s.is_empty());
    let detail_fee = song.fee.unwrap_or(-1);

    if detail_fee != 0 {
        return Err(Error::Network(
            "netease song has no public free download URL (membership or paid track)".into(),
        ));
    }

    let entry = player
        .data
        .into_iter()
        .next()
        .ok_or_else(|| Error::Network("netease player returned no media entry".into()))?;

    if entry.fee.unwrap_or(detail_fee) != 0 {
        return Err(Error::Network(
            "netease song has no public free download URL (membership or paid track)".into(),
        ));
    }

    let raw_url = entry
        .url
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Network("netease song has no public media URL".into()))?;

    if !(raw_url.starts_with("http://") || raw_url.starts_with("https://")) {
        return Err(Error::Network("netease song has no public media URL".into()));
    }

    let url = prefer_https(raw_url);
    let ext = entry.type_.filter(|s| !s.is_empty()).unwrap_or_else(|| "mp3".into());
    let filename = filename_from_url(&url).or_else(|| {
        let base = title
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("netease-{song_id}"));
        Some(sanitize_filename(&format!("{base}.{ext}")))
    });

    Ok(MediaInfo {
        url,
        title: title.clone().or_else(|| filename.clone()),
        content_type: guess_content_type(&ext),
        content_length: entry.size.filter(|&n| n > 0),
        filename,
        extractor: EXTRACTOR_NETEASE_CLOUD.to_string(),
    })
}

fn prefer_https(url: String) -> String {
    if let Some(rest) = url.strip_prefix("http://") { format!("https://{rest}") } else { url }
}

fn guess_content_type(ext: &str) -> Option<String> {
    match ext.to_ascii_lowercase().as_str() {
        "mp3" => Some("audio/mpeg".into()),
        "flac" => Some("audio/flac".into()),
        "aac" | "m4a" => Some("audio/mp4".into()),
        "ogg" | "oga" => Some("audio/ogg".into()),
        "wav" => Some("audio/wav".into()),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct DetailResponse {
    songs: Vec<SongDetail>,
}

#[derive(Debug, Deserialize)]
struct SongDetail {
    id: Option<u64>,
    name: Option<String>,
    fee: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PlayerResponse {
    data: Vec<PlayerEntry>,
}

#[derive(Debug, Deserialize)]
struct PlayerEntry {
    url: Option<String>,
    fee: Option<i64>,
    size: Option<u64>,
    #[serde(rename = "type")]
    type_: Option<String>,
}
