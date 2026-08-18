use std::fs;
use std::path::PathBuf;

use prometheus_extractors::{
    Extractor, InternetArchive, WikimediaCommons, internet_archive_item_id, media_from_api_json,
    media_from_metadata_json, wikimedia_file_title,
};

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    fs::read_to_string(path).expect("fixture")
}

#[test]
fn parses_internet_archive_item_id() {
    assert_eq!(
        internet_archive_item_id("https://archive.org/details/testmp3testfile").as_deref(),
        Some("testmp3testfile")
    );
    assert_eq!(
        internet_archive_item_id("https://archive.org/download/testmp3testfile/mpthreetest.mp3")
            .as_deref(),
        Some("testmp3testfile")
    );
    assert!(InternetArchive.matches("https://archive.org/details/BigBuckBunny_328"));
    assert!(!InternetArchive.matches("https://example.com/details/x"));
}

#[test]
fn media_from_internet_archive_fixture() {
    let body = fixture("internet_archive_testmp3.json");
    let info = media_from_metadata_json(&body, "testmp3testfile").unwrap();
    assert_eq!(info.extractor, "internet-archive");
    assert_eq!(info.filename.as_deref(), Some("mpthreetest.mp3"));
    assert_eq!(info.content_length, Some(198658));
    assert_eq!(info.url, "https://archive.org/download/testmp3testfile/mpthreetest.mp3");
    assert_eq!(info.title.as_deref(), Some("mp3 test file"));
}

#[test]
fn parses_wikimedia_file_title() {
    assert_eq!(
        wikimedia_file_title("https://commons.wikimedia.org/wiki/File:Example.ogg").as_deref(),
        Some("File:Example.ogg")
    );
    assert_eq!(
        wikimedia_file_title("https://commons.wikimedia.org/wiki/File:Big_Buck_Bunny_Trailer.webm")
            .as_deref(),
        Some("File:Big Buck Bunny Trailer.webm")
    );
    assert!(WikimediaCommons.matches("https://commons.wikimedia.org/wiki/File:Example.ogg"));
    assert!(
        !WikimediaCommons
            .matches("https://upload.wikimedia.org/wikipedia/commons/c/c8/Example.ogg")
    );
}

#[test]
fn media_from_wikimedia_fixture() {
    let body = fixture("wikimedia_example_ogg.json");
    let info = media_from_api_json(&body).unwrap();
    assert_eq!(info.extractor, "wikimedia-commons");
    assert_eq!(info.filename.as_deref(), Some("Example.ogg"));
    assert_eq!(info.content_length, Some(104793));
    assert_eq!(info.content_type.as_deref(), Some("application/ogg"));
    assert_eq!(info.url, "https://upload.wikimedia.org/wikipedia/commons/c/c8/Example.ogg");
}

#[test]
fn live_internet_archive_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match InternetArchive.inspect("https://archive.org/details/testmp3testfile") {
        Ok(info) => {
            assert_eq!(info.extractor, "internet-archive");
            assert!(info.url.contains("archive.org/download/"));
        }
        Err(err) => eprintln!("skip live internet-archive ({err})"),
    }
}

#[test]
fn live_wikimedia_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match WikimediaCommons.inspect("https://commons.wikimedia.org/wiki/File:Example.ogg") {
        Ok(info) => {
            assert_eq!(info.extractor, "wikimedia-commons");
            assert!(info.url.contains("upload.wikimedia.org"));
        }
        Err(err) => eprintln!("skip live wikimedia-commons ({err})"),
    }
}
