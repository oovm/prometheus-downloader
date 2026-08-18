use std::fs;
use std::path::PathBuf;

use prometheus_extractors::{
    Artic, CcMixter, ClevelandArtworkRef, ClevelandMuseum, Extractor, Gutenberg, InternetArchive,
    MetMuseum, NasaImages, OpenLibrary, OpenLibraryResourceKind, Openverse, OpenverseWorkKind,
    PeerTube, Vam, Wellcome, WellcomeTarget, WikimediaCommons, artic_artwork_id,
    ccmixter_upload_id, cleveland_artwork_ref, gutenberg_ebook_id, internet_archive_item_id,
    media_from_api_json, media_from_artic_json, media_from_ccmixter_json,
    media_from_cleveland_json, media_from_gutenberg_json, media_from_met_json,
    media_from_metadata_json, media_from_nasa_json, media_from_open_library_json,
    media_from_openverse_json, media_from_vam_json, media_from_video_json,
    media_from_wellcome_image_json, met_object_id, nasa_id, open_library_target,
    openverse_work_target, peertube_watch_target, vam_object_id, wellcome_target,
    wikimedia_file_title,
};

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    fs::read_to_string(path).expect("fixture")
}

#[test]
fn parses_internet_archive_item_id() {
    assert_eq!(
        internet_archive_item_id("https://archive.org/details/testmp3testfile").as_deref(),
        Some("testmp3testfile")
    );
    assert_eq!(
        internet_archive_item_id("https://archive.org/download/testmp3testfile/mpthreetest.mp3")
            .as_deref(),
        Some("testmp3testfile")
    );
    assert!(InternetArchive.matches("https://archive.org/details/BigBuckBunny_328"));
    assert!(!InternetArchive.matches("https://example.com/details/x"));
}

#[test]
fn media_from_internet_archive_fixture() {
    let body = fixture("internet_archive_testmp3.json");
    let info = media_from_metadata_json(&body, "testmp3testfile").unwrap();
    assert_eq!(info.extractor, "internet-archive");
    assert_eq!(info.filename.as_deref(), Some("mpthreetest.mp3"));
    assert_eq!(info.content_length, Some(198658));
    assert_eq!(info.url, "https://archive.org/download/testmp3testfile/mpthreetest.mp3");
    assert_eq!(info.title.as_deref(), Some("mp3 test file"));
}

#[test]
fn parses_wikimedia_file_title() {
    assert_eq!(
        wikimedia_file_title("https://commons.wikimedia.org/wiki/File:Example.ogg").as_deref(),
        Some("File:Example.ogg")
    );
    assert_eq!(
        wikimedia_file_title("https://commons.wikimedia.org/wiki/File:Big_Buck_Bunny_Trailer.webm")
            .as_deref(),
        Some("File:Big Buck Bunny Trailer.webm")
    );
    assert!(WikimediaCommons.matches("https://commons.wikimedia.org/wiki/File:Example.ogg"));
    assert_eq!(
        wikimedia_file_title("https://en.wikipedia.org/wiki/File:Example.jpg").as_deref(),
        Some("File:Example.jpg")
    );
    assert!(
        WikimediaCommons.matches("https://en.wikipedia.org/w/index.php?title=File:Example.jpg")
    );
    assert!(
        !WikimediaCommons
            .matches("https://upload.wikimedia.org/wikipedia/commons/c/c8/Example.ogg")
    );
    assert!(!WikimediaCommons.matches("https://example.com/?title=File:Example.jpg"));
}

#[test]
fn media_from_wikimedia_fixture() {
    let body = fixture("wikimedia_example_ogg.json");
    let info = media_from_api_json(&body).unwrap();
    assert_eq!(info.extractor, "wikimedia-commons");
    assert_eq!(info.filename.as_deref(), Some("Example.ogg"));
    assert_eq!(info.content_length, Some(104793));
    assert_eq!(info.content_type.as_deref(), Some("application/ogg"));
    assert_eq!(info.url, "https://upload.wikimedia.org/wikipedia/commons/c/c8/Example.ogg");
}

#[test]
fn parses_peertube_watch_target() {
    let watch = peertube_watch_target(
        "https://peertube2.cpy.re/videos/watch/7bc04dcc-1bde-4350-99a2-8d67fc1534e5",
    )
    .unwrap();
    assert_eq!(watch.origin, "https://peertube2.cpy.re");
    assert_eq!(watch.id, "7bc04dcc-1bde-4350-99a2-8d67fc1534e5");
    assert!(PeerTube.matches("https://peertube.cpy.re/w/ghjnHKBEA5fD3iJFp9asjz"));
    assert!(
        PeerTube
            .matches("https://peertube2.cpy.re/videos/embed/7bc04dcc-1bde-4350-99a2-8d67fc1534e5")
    );
    assert!(!PeerTube.matches("https://en.wikipedia.org/wiki/File:Example.jpg"));
    assert!(!PeerTube.matches("https://www.youtube.com/videos/watch/dQw4w9WgXcQ"));
    assert!(!PeerTube.matches("https://example.com/w/about"));
}

#[test]
fn media_from_peertube_fixture() {
    let body = fixture("peertube_elephants_dream.json");
    let info = media_from_video_json(&body).unwrap();
    assert_eq!(info.extractor, "peertube");
    assert_eq!(info.title.as_deref(), Some("Elephants Dream"));
    assert!(info.url.contains("1080-fragmented.mp4"));
    assert_eq!(info.content_length, Some(6305624));
    assert_eq!(info.content_type.as_deref(), Some("video/mp4"));
}

#[test]
fn live_internet_archive_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match InternetArchive.inspect("https://archive.org/details/testmp3testfile") {
        Ok(info) => {
            assert_eq!(info.extractor, "internet-archive");
            assert!(info.url.contains("archive.org/download/"));
        }
        Err(err) => eprintln!("skip live internet-archive ({err})"),
    }
}

#[test]
fn live_wikimedia_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match WikimediaCommons.inspect("https://commons.wikimedia.org/wiki/File:Example.ogg") {
        Ok(info) => {
            assert_eq!(info.extractor, "wikimedia-commons");
            assert!(info.url.contains("upload.wikimedia.org"));
        }
        Err(err) => eprintln!("skip live wikimedia-commons ({err})"),
    }
}

#[test]
fn live_wikipedia_file_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match WikimediaCommons.inspect("https://en.wikipedia.org/wiki/File:Example.jpg") {
        Ok(info) => {
            assert_eq!(info.extractor, "wikimedia-commons");
            assert!(info.url.contains("upload.wikimedia.org"));
        }
        Err(err) => eprintln!("skip live wikipedia-file ({err})"),
    }
}

#[test]
fn live_peertube_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match PeerTube
        .inspect("https://peertube2.cpy.re/videos/watch/7bc04dcc-1bde-4350-99a2-8d67fc1534e5")
    {
        Ok(info) => {
            assert_eq!(info.extractor, "peertube");
            assert!(info.url.contains("peertube2.cpy.re"));
        }
        Err(err) => eprintln!("skip live peertube ({err})"),
    }
}

#[test]
fn parses_nasa_images_id() {
    assert_eq!(
        nasa_id("https://images.nasa.gov/details-as11-40-5874").as_deref(),
        Some("as11-40-5874")
    );
    assert_eq!(
        nasa_id("https://images-api.nasa.gov/asset/as11-40-5874").as_deref(),
        Some("as11-40-5874")
    );
    assert!(NasaImages.matches("https://images.nasa.gov/details/as11-40-5874"));
    assert!(!NasaImages.matches("https://www.nasa.gov/news"));
}

#[test]
fn media_from_nasa_fixture() {
    let body = fixture("nasa_as11_40_5874.json");
    let info = media_from_nasa_json(&body, "as11-40-5874").unwrap();
    assert_eq!(info.extractor, "nasa-images");
    assert_eq!(info.url, "https://images-assets.nasa.gov/image/as11-40-5874/as11-40-5874~orig.jpg");
    assert_eq!(info.content_type.as_deref(), Some("image/jpeg"));
}

#[test]
fn parses_met_object_id() {
    assert_eq!(
        met_object_id("https://www.metmuseum.org/art/collection/search/436535").as_deref(),
        Some("436535")
    );
    assert!(
        MetMuseum
            .matches("https://collectionapi.metmuseum.org/public/collection/v1/objects/436535")
    );
    assert!(!MetMuseum.matches("https://www.metmuseum.org/visit"));
}

#[test]
fn media_from_met_fixture() {
    let body = fixture("met_wheat_field.json");
    let info = media_from_met_json(&body).unwrap();
    assert_eq!(info.extractor, "met-museum");
    assert_eq!(info.title.as_deref(), Some("Wheat Field with Cypresses"));
    assert!(info.url.contains("images.metmuseum.org"));
}

#[test]
fn parses_artic_artwork_id() {
    assert_eq!(
        artic_artwork_id("https://www.artic.edu/artworks/129884-starry-night-and-the-astronauts")
            .as_deref(),
        Some("129884")
    );
    assert!(Artic.matches("https://api.artic.edu/api/v1/artworks/129884"));
    assert!(!Artic.matches("https://www.artic.edu/iiif/2/abc/full/max/0/default.jpg"));
}

#[test]
fn media_from_artic_fixture() {
    let body = fixture("artic_129884.json");
    let info = media_from_artic_json(&body).unwrap();
    assert_eq!(info.extractor, "artic");
    assert_eq!(info.title.as_deref(), Some("Starry Night and the Astronauts"));
    assert!(info.url.contains("e966799b-97ee-1cc6-bd2f-a94b4b8bb8f9"));
    assert!(info.url.ends_with("/full/max/0/default.jpg"));
}

#[test]
fn parses_cleveland_artwork_ref() {
    assert_eq!(
        cleveland_artwork_ref("https://www.clevelandart.org/art/1915.534"),
        Some(ClevelandArtworkRef::Accession("1915.534".into()))
    );
    assert_eq!(
        cleveland_artwork_ref("https://openaccess-api.clevelandart.org/api/artworks/94979"),
        Some(ClevelandArtworkRef::Id("94979".into()))
    );
    assert!(ClevelandMuseum.matches("https://www.clevelandart.org/art/1915.534"));
    assert!(
        !ClevelandMuseum
            .matches("https://openaccess-cdn.clevelandart.org/1915.534/1915.534_web.jpg")
    );
}

#[test]
fn media_from_cleveland_fixture() {
    let body = fixture("cleveland_1915_534.json");
    let info = media_from_cleveland_json(&body).unwrap();
    assert_eq!(info.extractor, "cleveland-museum");
    assert_eq!(info.title.as_deref(), Some("Nathaniel Hurd"));
    assert!(info.url.contains("1915.534_print.jpg"));
}

#[test]
fn live_nasa_images_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match NasaImages.inspect("https://images.nasa.gov/details-as11-40-5874") {
        Ok(info) => {
            assert_eq!(info.extractor, "nasa-images");
            assert!(info.url.contains("images-assets.nasa.gov"));
        }
        Err(err) => eprintln!("skip live nasa-images ({err})"),
    }
}

#[test]
fn live_met_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match MetMuseum.inspect("https://www.metmuseum.org/art/collection/search/436535") {
        Ok(info) => {
            assert_eq!(info.extractor, "met-museum");
            assert!(info.url.contains("images.metmuseum.org"));
        }
        Err(err) => eprintln!("skip live met-museum ({err})"),
    }
}

#[test]
fn live_artic_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match Artic.inspect("https://www.artic.edu/artworks/129884") {
        Ok(info) => {
            assert_eq!(info.extractor, "artic");
            assert!(info.url.contains("artic.edu/iiif/"));
        }
        Err(err) => eprintln!("skip live artic ({err})"),
    }
}

#[test]
fn live_cleveland_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match ClevelandMuseum.inspect("https://www.clevelandart.org/art/1915.534") {
        Ok(info) => {
            assert_eq!(info.extractor, "cleveland-museum");
            assert!(info.url.contains("openaccess-cdn.clevelandart.org"));
        }
        Err(err) => eprintln!("skip live cleveland-museum ({err})"),
    }
}

#[test]
fn parses_openverse_work_target() {
    let image = openverse_work_target(
        "https://openverse.org/image/79725d88-81f9-41e8-8f6c-b7dc0bbfcbea/the-moon-tonight",
    )
    .unwrap();
    assert_eq!(image.kind, OpenverseWorkKind::Image);
    assert_eq!(image.id, "79725d88-81f9-41e8-8f6c-b7dc0bbfcbea");
    assert!(
        Openverse
            .matches("https://api.openverse.org/v1/audio/2c51e4eb-dbe7-468c-ba6c-d1c85a1b71ae/")
    );
    assert!(!Openverse.matches("https://openverse.org/search/?q=moon"));
}

#[test]
fn media_from_openverse_image_fixture() {
    let body = fixture("openverse_moon.json");
    let info = media_from_openverse_json(&body).unwrap();
    assert_eq!(info.extractor, "openverse");
    assert_eq!(info.title.as_deref(), Some("The Moon tonight"));
    assert!(info.url.contains("staticflickr.com"));
}

#[test]
fn media_from_openverse_audio_fixture() {
    let body = fixture("openverse_piano.json");
    let info = media_from_openverse_json(&body).unwrap();
    assert_eq!(info.extractor, "openverse");
    assert_eq!(info.content_type.as_deref(), Some("audio/mpeg"));
    assert_eq!(info.content_length, Some(227209));
}

#[test]
fn parses_gutenberg_ebook_id() {
    assert_eq!(gutenberg_ebook_id("https://www.gutenberg.org/ebooks/11").as_deref(), Some("11"));
    assert_eq!(gutenberg_ebook_id("https://gutendex.com/books/11").as_deref(), Some("11"));
    assert!(Gutenberg.matches("https://www.gutenberg.org/files/11/11-0.txt"));
    assert!(!Gutenberg.matches("https://www.gutenberg.org/browse/scores/top"));
}

#[test]
fn media_from_gutenberg_fixture() {
    let body = fixture("gutenberg_alice.json");
    let info = media_from_gutenberg_json(&body).unwrap();
    assert_eq!(info.extractor, "gutenberg");
    assert_eq!(info.title.as_deref(), Some("Alice's Adventures in Wonderland"));
    assert!(info.url.contains(".epub"));
}

#[test]
fn parses_ccmixter_upload_id() {
    assert_eq!(
        ccmixter_upload_id("https://ccmixter.org/files/grapes/16626").as_deref(),
        Some("16626")
    );
    assert!(CcMixter.matches("https://ccmixter.org/api/query?f=json&ids=16626"));
    assert!(!CcMixter.matches("https://ccmixter.org/view/media/remix"));
}

#[test]
fn media_from_ccmixter_fixture() {
    let body = fixture("ccmixter_16626.json");
    let info = media_from_ccmixter_json(&body).unwrap();
    assert_eq!(info.extractor, "ccmixter");
    assert_eq!(info.title.as_deref(), Some("I dunno"));
    assert_eq!(info.url, "https://ccmixter.org/content/grapes/grapes_-_I_dunno.mp3");
    assert_eq!(info.content_type.as_deref(), Some("audio/mpeg"));
}

#[test]
fn live_openverse_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match Openverse.inspect("https://openverse.org/image/79725d88-81f9-41e8-8f6c-b7dc0bbfcbea") {
        Ok(info) => {
            assert_eq!(info.extractor, "openverse");
            assert!(!info.url.is_empty());
        }
        Err(err) => eprintln!("skip live openverse ({err})"),
    }
}

#[test]
fn live_gutenberg_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match Gutenberg.inspect("https://www.gutenberg.org/ebooks/11") {
        Ok(info) => {
            assert_eq!(info.extractor, "gutenberg");
            assert!(info.url.contains("gutenberg.org"));
        }
        Err(err) => eprintln!("skip live gutenberg ({err})"),
    }
}

#[test]
fn live_ccmixter_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match CcMixter.inspect("https://ccmixter.org/files/grapes/16626") {
        Ok(info) => {
            assert_eq!(info.extractor, "ccmixter");
            assert!(info.url.contains("ccmixter.org/content/"));
        }
        Err(err) => eprintln!("skip live ccmixter ({err})"),
    }
}

#[test]
fn parses_vam_object_id() {
    assert_eq!(
        vam_object_id("https://collections.vam.ac.uk/item/O12511/window-frame/").as_deref(),
        Some("O12511")
    );
    assert!(Vam.matches("https://api.vam.ac.uk/v2/object/O12511"));
    assert!(!Vam.matches("https://collections.vam.ac.uk/search/"));
}

#[test]
fn media_from_vam_fixture() {
    let body = fixture("vam_o12511.json");
    let info = media_from_vam_json(&body).unwrap();
    assert_eq!(info.extractor, "vam");
    assert_eq!(info.title.as_deref(), Some("Window frame"));
    assert_eq!(
        info.url,
        "https://framemark.vam.ac.uk/collections/2007BM5784/full/max/0/default.jpg"
    );
}

#[test]
fn live_vam_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match Vam.inspect("https://collections.vam.ac.uk/item/O12511/") {
        Ok(info) => {
            assert_eq!(info.extractor, "vam");
            assert!(info.url.contains("framemark.vam.ac.uk"));
        }
        Err(err) => eprintln!("skip live vam ({err})"),
    }
}

#[test]
fn parses_open_library_target() {
    let work =
        open_library_target("https://openlibrary.org/works/OL45804W/Fantastic_Mr_Fox").unwrap();
    assert_eq!(work.kind, OpenLibraryResourceKind::Work);
    assert_eq!(work.id, "OL45804W");
    assert!(OpenLibrary.matches("https://openlibrary.org/books/OL7353617M.json"));
    assert!(!OpenLibrary.matches("https://openlibrary.org/authors/OL34184A"));
}

#[test]
fn media_from_open_library_fixture() {
    let body = fixture("openlibrary_fantastic_mr_fox.json");
    let target = open_library_target("https://openlibrary.org/works/OL45804W").unwrap();
    let info = media_from_open_library_json(&body, &target).unwrap();
    assert_eq!(info.extractor, "open-library");
    assert_eq!(info.title.as_deref(), Some("Fantastic Mr Fox"));
    assert_eq!(info.url, "https://covers.openlibrary.org/b/id/6498519-L.jpg");
}

#[test]
fn parses_wellcome_target() {
    assert_eq!(
        wellcome_target("https://wellcomecollection.org/works/zv3drmps/images?id=nwqpxugw"),
        Some(WellcomeTarget::Image("nwqpxugw".into()))
    );
    assert_eq!(
        wellcome_target("https://wellcomecollection.org/works/zv3drmps"),
        Some(WellcomeTarget::Work("zv3drmps".into()))
    );
    assert!(Wellcome.matches("https://iiif.wellcomecollection.org/image/B0008032/info.json"));
    assert!(!Wellcome.matches("https://wellcomecollection.org/search"));
}

#[test]
fn media_from_wellcome_image_fixture() {
    let body = fixture("wellcome_nwqpxugw.json");
    let info = media_from_wellcome_image_json(&body).unwrap();
    assert_eq!(info.extractor, "wellcome");
    assert_eq!(info.title.as_deref(), Some("Aurelia aurita the moon Jellyfish"));
    assert_eq!(
        info.url,
        "https://iiif.wellcomecollection.org/image/B0008032/full/max/0/default.jpg"
    );
}

#[test]
fn live_open_library_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match OpenLibrary.inspect("https://openlibrary.org/works/OL45804W") {
        Ok(info) => {
            assert_eq!(info.extractor, "open-library");
            assert!(info.url.contains("covers.openlibrary.org"));
        }
        Err(err) => eprintln!("skip live open-library ({err})"),
    }
}

#[test]
fn live_wellcome_inspect_optional() {
    if std::env::var_os("PROMETHEUS_LIVE_NET").is_none() {
        return;
    }
    match Wellcome.inspect("https://wellcomecollection.org/works/zv3drmps/images?id=nwqpxugw") {
        Ok(info) => {
            assert_eq!(info.extractor, "wellcome");
            assert!(info.url.contains("iiif.wellcomecollection.org"));
        }
        Err(err) => eprintln!("skip live wellcome ({err})"),
    }
}
