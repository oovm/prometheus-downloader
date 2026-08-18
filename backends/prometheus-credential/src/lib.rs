//! Local credential vault.
//!
//! Encryption is not implemented; `create` only writes an empty vault header.

#![deny(missing_docs)]

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
            return Err(Error::Credential(format!("vault already exists: {}", path.display())));
        }
        fs::write(path, VAULT_MAGIC)?;
        Ok(Self { path: path.to_path_buf() })
    }

    /// Open an existing vault file and verify the magic header.
    pub fn open(path: impl AsRef<Path>, _password: &str) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path)?;
        if !bytes.starts_with(VAULT_MAGIC) {
            return Err(Error::Credential(format!("not a prometheus vault: {}", path.display())));
        }
        Ok(Self { path: path.to_path_buf() })
    }
}
