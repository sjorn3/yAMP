use music_cache::*;
use std::ffi::CString;

extern "C" {
    fn ffi_expect_album_tags(
        db: *mut std::ffi::c_void,
        album_key: *const Key,
        artist: *const std::os::raw::c_char,
        title: *const std::os::raw::c_char,
        year: u16,
    ) -> bool;
}

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn ffi_album_tags_round_trip() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let db = sled::open(temp_dir.path())?;

    let album_tags = AlbumTags {
        artist: Some("Codex Artist".into()),
        title: Some("FFI Album".into()),
        year: Some(2024),
    };
    let song = Song {
        tags: SongTags {
            title: Some("Intro Track".into()),
            track_number: Some(1),
        },
        relpath: b"intro.flac".to_vec(),
    };
    let album = Album {
        tags: album_tags.clone(),
        songs: vec![song],
    };

    let album_key = db.insert_metadata(&album)?;

    let artist_c = CString::new(album_tags.artist.clone().unwrap())?;
    let title_c = CString::new(album_tags.title.clone().unwrap())?;

    assert!(unsafe {
        ffi_expect_album_tags(
            &db as *const _ as *mut std::ffi::c_void,
            &album_key as *const Key,
            artist_c.as_ptr(),
            title_c.as_ptr(),
            album_tags.year.unwrap(),
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
            0,
        )
    });
}
