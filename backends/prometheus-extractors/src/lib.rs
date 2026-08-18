//! URL extractors and registry.

#![deny(missing_docs)]

mod artic;
mod ccmixter;
mod cleveland_museum;
mod generic_http;
mod gutenberg;
mod internet_archive;
mod local_file;
mod met_museum;
mod nasa_images;
mod open_library;
mod openverse;
mod peertube;
mod vam;
mod wellcome;
mod wikimedia_commons;

pub use artic::{
    Artic, artwork_id as artic_artwork_id, media_from_artwork_json as media_from_artic_json,
};
pub use ccmixter::{
    CcMixter, media_from_query_json as media_from_ccmixter_json, upload_id as ccmixter_upload_id,
};
pub use cleveland_museum::{
    ArtworkRef as ClevelandArtworkRef, ClevelandMuseum, artwork_ref as cleveland_artwork_ref,
    media_from_artwork_json as media_from_cleveland_json,
};
pub use generic_http::{GenericHttp, filename_from_content_disposition};
pub use gutenberg::{
    Gutenberg, ebook_id as gutenberg_ebook_id, media_from_book_json as media_from_gutenberg_json,
};
pub use internet_archive::{
    InternetArchive, item_id as internet_archive_item_id, media_from_metadata_json,
};
pub use local_file::{LocalFile, file_url_to_path};
pub use met_museum::{
    MetMuseum, media_from_object_json as media_from_met_json, object_id as met_object_id,
};
pub use nasa_images::{NasaImages, media_from_asset_json as media_from_nasa_json, nasa_id};
pub use open_library::{
    OpenLibrary, ResourceKind as OpenLibraryResourceKind, library_target as open_library_target,
    media_from_library_json as media_from_open_library_json,
};
pub use openverse::{
    Openverse, WorkKind as OpenverseWorkKind, media_from_work_json as media_from_openverse_json,
    work_target as openverse_work_target,
};
pub use peertube::{PeerTube, media_from_video_json, watch_target as peertube_watch_target};
pub use vam::{Vam, media_from_object_json as media_from_vam_json, object_id as vam_object_id};
pub use wellcome::{
    Wellcome, WellcomeTarget, media_from_image_json as media_from_wellcome_image_json,
    media_from_work_json as media_from_wellcome_work_json, wellcome_target,
};
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
                Box::new(PeerTube),
                Box::new(NasaImages),
                Box::new(MetMuseum),
                Box::new(Artic),
                Box::new(ClevelandMuseum),
                Box::new(Openverse),
                Box::new(Gutenberg),
                Box::new(CcMixter),
                Box::new(Vam),
                Box::new(OpenLibrary),
                Box::new(Wellcome),
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
