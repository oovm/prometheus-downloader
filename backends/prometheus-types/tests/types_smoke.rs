use prometheus_types::{ProgressEvent, TRANSFER_SIMPLE, filename_from_url, sanitize_filename};

#[test]
fn sanitize_rejects_path_parts() {
    assert_eq!(sanitize_filename("../a\\b:c"), "_a_b_c");
    assert_eq!(sanitize_filename("   "), "download.bin");
}

#[test]
fn filename_from_simple_url() {
    assert_eq!(
        filename_from_url("https://example.com/files/demo%20clip.bin?x=1"),
        Some("demo clip.bin".to_string())
    );
}

#[test]
fn progress_event_wire_kind() {
    let event = ProgressEvent::Started {
        url: "https://example.com/a.bin".into(),
        total_bytes: Some(12),
        transfer: TRANSFER_SIMPLE.into(),
    };
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["kind"], "started");
    assert_eq!(json["totalBytes"], 12);
    assert_eq!(json["transfer"], "simple");
}
