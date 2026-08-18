use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use prometheus_extractors::{
    Extractor, GenericHttp, LocalFile, Registry, filename_from_content_disposition,
};

fn file_url(path: &std::path::Path) -> String {
    let mut path = path.to_string_lossy().replace('\\', "/");
    if !path.starts_with('/') {
        path.insert(0, '/');
    }
    format!("file://{path}")
}

fn serve_head_405_then_range(body: &'static [u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        for _ in 0..6 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let first = req.lines().next().unwrap_or("");
            let is_head = first.starts_with("HEAD");
            let wants_range = req.to_ascii_lowercase().contains("range:");
            if is_head {
                let _ = stream.write_all(
                    b"HTTP/1.1 405 Method Not Allowed\r\nAllow: GET\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
            } else if wants_range {
                let header = format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-0/{}\r\nContent-Length: 1\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"range.bin\"\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                if !body.is_empty() {
                    let _ = stream.write_all(&body[..1]);
                }
            } else {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(body);
            }
        }
    });
    format!("http://{addr}/range.bin")
}

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

#[test]
fn inspects_when_head_is_rejected() {
    let url = serve_head_405_then_range(b"0123456789abcdef");
    thread::sleep(std::time::Duration::from_millis(20));
    let info = GenericHttp.inspect(&url).unwrap();
    assert_eq!(info.extractor, "generic-http");
    assert_eq!(info.filename.as_deref(), Some("range.bin"));
    assert_eq!(info.content_length, Some(16));
}

#[test]
fn inspects_local_file_url() {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("prometheus-ex-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("clip.bin");
    fs::write(&file, b"abc").unwrap();
    let url = file_url(&file);
    let info = LocalFile.inspect(&url).unwrap();
    assert_eq!(info.extractor, "local-file");
    assert_eq!(info.filename.as_deref(), Some("clip.bin"));
    assert_eq!(info.content_length, Some(3));
    let via_registry = Registry::builtin().inspect(&url).unwrap();
    assert_eq!(via_registry.extractor, "local-file");
    let _ = fs::remove_dir_all(&dir);
}
