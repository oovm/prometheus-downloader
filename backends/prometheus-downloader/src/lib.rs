//! Single-connection HTTP(S) downloader.

use std::fs::{self, File};
use std::io::{copy, BufWriter};
use std::path::{Path, PathBuf};

use prometheus_extractors::Registry;
use prometheus_types::{sanitize_filename, DownloadResult, Error, Result};

/// Download `url` into `output_dir` using the built-in extractor registry.
pub fn download(url: &str, output_dir: impl AsRef<Path>) -> Result<DownloadResult> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let info = Registry::builtin().inspect(url)?;
    let filename = sanitize_filename(
        info.filename
            .as_deref()
            .or(info.title.as_deref())
            .unwrap_or("download.bin"),
    );
    let path = unique_path(output_dir, &filename);

    let resp = ureq::get(url)
        .call()
        .map_err(|err| Error::Network(err.to_string()))?;

    let mut reader = resp.into_reader();
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    let bytes_written = copy(&mut reader, &mut writer)?;
    writer
        .into_inner()
        .map_err(|err| Error::Io(err.into_error()))?
        .sync_all()?;

    Ok(DownloadResult {
        path: path.to_string_lossy().into_owned(),
        bytes_written,
        filename: path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or(filename),
    })
}

fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{s}"))
        .unwrap_or_default();
    for i in 1..10_000 {
        let name = format!("{stem}-{i}{ext}");
        let path = dir.join(name);
        if !path.exists() {
            return path;
        }
    }
    dir.join(format!("{stem}-{}.bin", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn serve_head_and_get(body: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            // inspect() may HEAD; download() then GET.
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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("prometheus-dl-{nanos}"));
        let result = download(&url, &dir).unwrap();
        let bytes = fs::read(&result.path).unwrap();
        assert_eq!(bytes, b"hello-prometheus");
        assert_eq!(result.bytes_written, 16);
        let _ = fs::remove_dir_all(&dir);
    }
}
