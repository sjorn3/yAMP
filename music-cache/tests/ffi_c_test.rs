use ffi::c_string_from_option;
use music_cache::{
    tests::{common::Result, Arbitrary},
    *,
};

extern "C" {
    fn ffi_expect_album_tags(
        db: *mut std::ffi::c_void,
        album_key: *const Key,
        artist: *const std::os::raw::c_char,
        title: *const std::os::raw::c_char,
        expect_year: bool,
        year: u16,
    ) -> bool;
}

#[test]
fn ffi_album_tags_round_trip() -> Result {
    let temp_dir = tempfile::tempdir()?;
    let db = sled::open(temp_dir.path())?;

    let album = Album::arbitrary();

    let album_key = db.insert_metadata(&album)?;

    let artist_c = c_string_from_option(album.tags.artist);
    let title_c = c_string_from_option(album.tags.title);

    assert!(unsafe {
        ffi_expect_album_tags(
            &db as *const _ as *mut std::ffi::c_void,
            &album_key as *const Key,
            artist_c,
            title_c,
            album.tags.year.is_some(),
            album.tags.year.unwrap_or(0u16),
        )
    });

    Ok(())
}

#[test]
fn ffi_album_tags_rejects_invalid_args() {
    assert!(!unsafe {
        ffi_expect_album_tags(
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            false,
            0,
        )
    });
}
