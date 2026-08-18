//! URL extractors and registry.

#![deny(missing_docs)]

mod generic_http;

pub use generic_http::{GenericHttp, filename_from_content_disposition};

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
        Self { extractors: vec![Box::new(GenericHttp)] }
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
