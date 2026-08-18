//! ccMixter upload extractor.

use prometheus_types::{
    EXTRACTOR_CCMIXTER, Error, MediaInfo, Result, filename_from_url, sanitize_filename,
};
use serde::Deserialize;

use crate::Extractor;

const USER_AGENT: &str = concat!("Prometheus/", env!("CARGO_PKG_VERSION"));

/// Resolves `ccmixter.org/files/{user}/{id}` via the public query API.
pub struct CcMixter;

impl Extractor for CcMixter {
    fn id(&self) -> &'static str {
        EXTRACTOR_CCMIXTER
    }

    fn matches(&self, url: &str) -> bool {
        upload_id(url).is_some()
    }

    fn inspect(&self, url: &str) -> Result<MediaInfo> {
        let id = upload_id(url).ok_or_else(|| Error::UnsupportedUrl(url.to_string()))?;
        let api = format!("https://ccmixter.org/api/query?f=json&ids={id}");
        let resp = ureq::get(&api)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| Error::Network(err.to_string()))?;
        let body = resp.into_string().map_err(|err| Error::Network(err.to_string()))?;
        media_from_query_json(&body)
    }
}

/// Parse a ccMixter upload id from a file page or query URL.
pub fn upload_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(idx) = lower.find("ccmixter.org/files/") {
        let rest = &trimmed[idx + "ccmixter.org/files/".len()..];
        let segments: Vec<&str> = rest.split(['/', '?', '#']).filter(|s| !s.is_empty()).collect();
        return match segments.as_slice() {
            [_, id, ..] => take_digits(id),
            _ => None,
        };
    }
    if lower.contains("ccmixter.org/api/query") {
        if let Some(idx) = lower.find("ids=") {
            return take_digits(&trimmed[idx + "ids=".len()..]);
        }
    }
    None
}

fn take_digits(rest: &str) -> Option<String> {
    let seg = rest.split(['/', '?', '#', '&']).next()?.trim();
    if seg.is_empty() || !seg.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(seg.to_string())
    }
}

/// Build [`MediaInfo`] from a ccMixter `api/query` JSON body.
pub fn media_from_query_json(body: &str) -> Result<MediaInfo> {
    let parsed: Vec<Upload> = serde_json::from_str(body)
        .map_err(|err| Error::Network(format!("ccmixter json: {err}")))?;
    let upload = parsed
        .into_iter()
        .next()
        .ok_or_else(|| Error::Network("ccmixter query returned no uploads".into()))?;
    let file = pick_file(upload.files.unwrap_or_default())
        .ok_or_else(|| Error::Network("ccmixter upload has no downloadable file".into()))?;
    let url = file
        .download_url
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::Network("ccmixter file has no download_url".into()))?;
    let filename = filename_from_url(&url)
        .or_else(|| file.file_name.filter(|s| !s.is_empty()).map(|s| sanitize_filename(&s)));
    Ok(MediaInfo {
        url,
        title: upload.upload_name.filter(|s| !s.is_empty()).or_else(|| filename.clone()),
        content_type: guess_content_type(filename.as_deref().unwrap_or("")),
        content_length: None,
        filename,
        extractor: EXTRACTOR_CCMIXTER.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct Upload {
    upload_name: Option<String>,
    files: Option<Vec<UploadFile>>,
}

#[derive(Debug, Deserialize)]
struct UploadFile {
    file_name: Option<String>,
    download_url: Option<String>,
    file_nicname: Option<String>,
}

fn pick_file(files: Vec<UploadFile>) -> Option<UploadFile> {
    let preferred = files.iter().position(is_preferred);
    match preferred {
        Some(idx) => files.into_iter().nth(idx),
        None => files.into_iter().next(),
    }
}

fn is_preferred(file: &UploadFile) -> bool {
    let name = file.file_name.as_deref().unwrap_or("").to_ascii_lowercase();
    let nick = file.file_nicname.as_deref().unwrap_or("").to_ascii_lowercase();
    (name.ends_with(".mp3") || nick.contains("mp3"))
        && file.download_url.as_deref().is_some_and(|s| !s.is_empty())
}

fn guess_content_type(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".mp3") {
        Some("audio/mpeg".into())
    } else if lower.ends_with(".ogg") || lower.ends_with(".oga") {
        Some("audio/ogg".into())
    } else if lower.ends_with(".flac") {
        Some("audio/flac".into())
    } else if lower.ends_with(".wav") {
        Some("audio/wav".into())
    } else {
        None
    }
}
