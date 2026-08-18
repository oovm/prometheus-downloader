//! URL extractors and registry.

#![deny(missing_docs)]

mod generic_http;
mod internet_archive;
mod local_file;
mod wikimedia_commons;

pub use generic_http::{GenericHttp, filename_from_content_disposition};
pub use internet_archive::{
    InternetArchive, item_id as internet_archive_item_id, media_from_metadata_json,
};
pub use local_file::{LocalFile, file_url_to_path};
pub use wikimedia_commons::{
    WikimediaCommons, file_title as wikimedia_file_title, media_from_api_json,
};

use prometheus_types::{Error, MediaInfo, Result};

/// Resolves media metadata from a URL.
pub trait Extractor: Send + Sync {
    /// Stable extractor id (`kebab-case`).
    fn id(&self) -> &'static str;
    /// Whether this extractor claims the URL.
    fn matches(&self, url: &str) -> bool;
    /// Inspect the URL and return metadata.
    fn inspect(&self, url: &str) -> Result<MediaInfo>;
}

/// Ordered extractor registry. First match wins.
pub struct Registry {
    extractors: Vec<Box<dyn Extractor>>,
}

impl Registry {
    /// Built-in extractors shipped with this crate.
    pub fn builtin() -> Self {
        Self {
            extractors: vec![
                Box::new(LocalFile),
                Box::new(InternetArchive),
                Box::new(WikimediaCommons),
                Box::new(GenericHttp),
            ],
        }
    }

    /// Inspect a URL with the first matching extractor.
    pub fn inspect(&self, url: &str) -> Result<MediaInfo> {
        for extractor in &self.extractors {
            if extractor.matches(url) {
                return extractor.inspect(url);
            }
        }
        Err(Error::UnsupportedUrl(url.to_string()))
    }
}
