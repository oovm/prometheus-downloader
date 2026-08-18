//! Local credential vault.
//!
//! Encryption is not implemented; `create` only writes an empty vault header.

use std::fs;
use std::path::{Path, PathBuf};

use prometheus_types::{Error, Result};

/// Magic header for an empty vault file.
pub const VAULT_MAGIC: &[u8] = b"PROMETHEUS_VAULT_v0\n";

/// On-disk credential vault handle.
#[derive(Debug, Clone)]
pub struct Vault {
    path: PathBuf,
}

impl Vault {
    /// Path of the vault file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create an empty vault file at `path`.
    ///
    /// `password` is accepted for API stability but is not used yet.
    pub fn create(path: impl AsRef<Path>, _password: &str) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        if path.exists() {
            return Err(Error::Credential(format!(
                "vault already exists: {}",
                path.display()
            )));
        }
        fs::write(path, VAULT_MAGIC)?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Open an existing vault file and verify the magic header.
    pub fn open(path: impl AsRef<Path>, _password: &str) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path)?;
        if !bytes.starts_with(VAULT_MAGIC) {
            return Err(Error::Credential(format!(
                "not a prometheus vault: {}",
                path.display()
            )));
        }
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn create_and_open_empty_vault() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("prometheus-vault-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.bin");
        let vault = Vault::create(&path, "unused").unwrap();
        assert_eq!(vault.path(), path.as_path());
        let opened = Vault::open(&path, "unused").unwrap();
        assert_eq!(opened.path(), path.as_path());
        let _ = fs::remove_dir_all(&dir);
    }
}
