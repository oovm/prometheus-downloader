use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use prometheus_credential::Vault;

#[test]
fn create_and_open_empty_vault() {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("prometheus-vault-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("vault.bin");
    let vault = Vault::create(&path, "unused").unwrap();
    assert_eq!(vault.path(), path.as_path());
    let opened = Vault::open(&path, "unused").unwrap();
    assert_eq!(opened.path(), path.as_path());
    let _ = fs::remove_dir_all(&dir);
}
