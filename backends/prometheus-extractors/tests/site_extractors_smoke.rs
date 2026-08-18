use std::fs;
use std::path::PathBuf;

use prometheus_extractors::{
    Extractor, InternetArchive, PeerTube, WikimediaCommons, internet_archive_item_id,
    media_from_api_json, media_from_metadata_json, media_from_video_json, peertube_watch_target,
    wikimedia_file_title,
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
    assert_eq!(
        wikimedia_file_title("https://en.wikipedia.org/wiki/File:Example.jpg").as_deref(),
        Some("File:Example.jpg")
    );
    assert!(
        WikimediaCommons.matches("https://en.wikipedia.org/w/index.php?title=File:Example.jpg")
    );
    assert!(
        !WikimediaCommons
            .matches("https://upload.wikimedia.org/wikipedia/commons/c/c8/Example.ogg")
    );
    assert!(!WikimediaCommons.matches("https://example.com/?title=File:Example.jpg"));
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
fn parses_peertube_watch_target() {
    let watch = peertube_watch_target(
        "https://peertube2.cpy.re/videos/watch/7bc04dcc-1bde-4350-99a2-8d67fc1534e5",
    )
    .unwrap();
    assert_eq!(watch.origin, "https://peertube2.cpy.re");
    assert_eq!(watch.id, "7bc04dcc-1bde-4350-99a2-8d67fc1534e5");
    assert!(PeerTube.matches("https://peertube.cpy.re/w/ghjnHKBEA5fD3iJFp9asjz"));
    assert!(
        PeerTube
            .matches("https://peertube2.cpy.re/videos/embed/7bc04dcc-1bde-4350-99a2-8d67fc1534e5")
    );
    assert!(!PeerTube.matches("https://en.wikipedia.org/wiki/File:Example.jpg"));
    assert!(!PeerTube.matches("https://www.youtube.com/videos/watch/dQw4w9WgXcQ"));
    assert!(!PeerTube.matches("https://example.com/w/about"));
}

#[test]
fn media_from_peertube_fixture() {
    let body = fixture("peertube_elephants_dream.json");
    let info = media_from_video_json(&body).unwrap();
    assert_eq!(info.extractor, "peertube");
    assert_eq!(info.title.as_deref(), Some("Elephants Dream"));
    assert!(info.url.contains("1080-fragmented.mp4"));
    assert_eq!(info.content_length, Some(6305624));
    assert_eq!(info.content_type.as_deref(), Some("video/mp4"));
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

#[test]
fn live_wikipedia_file_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match WikimediaCommons.inspect("https://en.wikipedia.org/wiki/File:Example.jpg") {
        Ok(info) => {
            assert_eq!(info.extractor, "wikimedia-commons");
            assert!(info.url.contains("upload.wikimedia.org"));
        }
        Err(err) => eprintln!("skip live wikipedia-file ({err})"),
    }
}

#[test]
fn live_peertube_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match PeerTube
        .inspect("https://peertube2.cpy.re/videos/watch/7bc04dcc-1bde-4350-99a2-8d67fc1534e5")
    {
        Ok(info) => {
            assert_eq!(info.extractor, "peertube");
            assert!(info.url.contains("peertube2.cpy.re"));
        }
        Err(err) => eprintln!("skip live peertube ({err})"),
    }
}
