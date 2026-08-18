use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use prometheus_downloader::{
    SimpleTransfer, TransferBackend, default_transfer_id, download, download_collecting_events,
    download_with, list_transfers,
};
use prometheus_types::ProgressEvent;

fn file_url(path: &std::path::Path) -> String {
    let mut path = path.to_string_lossy().replace('\\', "/");
    if !path.starts_with('/') {
        path.insert(0, '/');
    }
    format!("file://{path}")
}

fn serve_head_and_get(body: &'static [u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        for _ in 0..4 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let is_head = req.starts_with("HEAD");
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"fixture.bin\"\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            if !is_head {
                let _ = stream.write_all(body);
            }
        }
    });
    format!("http://{addr}/fixture.bin")
}

#[test]
fn downloads_local_fixture() {
    let url = serve_head_and_get(b"hello-prometheus");
    thread::sleep(std::time::Duration::from_millis(20));
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("prometheus-dl-{nanos}"));
    let result = download(&url, &dir).unwrap();
    let bytes = fs::read(&result.path).unwrap();
    assert_eq!(bytes, b"hello-prometheus");
    assert_eq!(result.bytes_written, 16);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn copies_local_file_url() {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let src_dir = std::env::temp_dir().join(format!("prometheus-src-{nanos}"));
    fs::create_dir_all(&src_dir).unwrap();
    let src = src_dir.join("clip.bin");
    fs::write(&src, b"abc").unwrap();
    let url = file_url(&src);
    let dir = std::env::temp_dir().join(format!("prometheus-dl-file-{nanos}"));
    let result = download(&url, &dir).unwrap();
    assert_eq!(fs::read(&result.path).unwrap(), b"abc");
    assert_eq!(result.bytes_written, 3);
    assert_eq!(result.filename, "clip.bin");
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&src_dir);
}

#[test]
fn emits_progress_events() {
    let url = serve_head_and_get(b"progress-body!!");
    thread::sleep(std::time::Duration::from_millis(20));
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("prometheus-dl-prog-{nanos}"));
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    let result = download_with(&SimpleTransfer, &url, &dir, &mut |event| {
        sink.lock().unwrap().push(event);
    })
    .unwrap();
    assert_eq!(result.bytes_written, 15);
    let kinds: Vec<_> = events
        .lock()
        .unwrap()
        .iter()
        .map(|e| match e {
            ProgressEvent::Started { .. } => "started",
            ProgressEvent::Bytes { .. } => "bytes",
            ProgressEvent::Finished { .. } => "finished",
            ProgressEvent::Failed { .. } => "failed",
        })
        .collect();
    assert_eq!(kinds.first().copied(), Some("started"));
    assert_eq!(kinds.last().copied(), Some("finished"));
    assert!(kinds.iter().any(|k| *k == "bytes"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn collects_progress_events() {
    let url = serve_head_and_get(b"collect-me");
    thread::sleep(std::time::Duration::from_millis(20));
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("prometheus-dl-collect-{nanos}"));
    let (result, events) = download_collecting_events(&SimpleTransfer, &url, &dir).unwrap();
    assert_eq!(result.bytes_written, 10);
    assert!(matches!(events.first(), Some(ProgressEvent::Started { .. })));
    assert!(matches!(events.last(), Some(ProgressEvent::Finished { .. })));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn default_transfer_is_simple() {
    assert_eq!(default_transfer_id(), "simple");
    assert_eq!(SimpleTransfer.id(), "simple");
}

#[test]
fn lists_in_process_simple_only() {
    let listed = list_transfers();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "simple");
    assert!(listed[0].available);
}
