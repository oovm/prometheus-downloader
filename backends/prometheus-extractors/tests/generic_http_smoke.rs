use prometheus_extractors::{Extractor, GenericHttp, filename_from_content_disposition};

#[test]
fn matches_http_urls() {
    let g = GenericHttp;
    assert!(g.matches("https://example.com/a.bin"));
    assert!(!g.matches("ftp://example.com/a.bin"));
}

#[test]
fn parses_content_disposition() {
    assert_eq!(
        filename_from_content_disposition(
            "attachment; filename=\"clip.bin\"; filename*=UTF-8''nice%20clip.bin"
        ),
        Some("nice clip.bin".to_string())
    );
}
