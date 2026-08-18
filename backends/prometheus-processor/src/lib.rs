//! Media post-processing (mux, remux, local file decrypt).
//!
//! No processors are registered yet.

use prometheus_types::Result;

/// Placeholder identity hook reserved for future post-processors.
pub fn passthrough_path(path: impl AsRef<std::path::Path>) -> Result<std::path::PathBuf> {
    Ok(path.as_ref().to_path_buf())
}
