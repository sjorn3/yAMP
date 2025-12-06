use music_cache::{
    tests::{common::Result, Arbitrary},
    *,
};
use std::ffi::CString;

extern "C" {
    fn ffi_open_db_round_trip(path: *const std::os::raw::c_char) -> bool;
    fn ffi_open_db_rejects_null_path() -> bool;

    fn ffi_expect_album(
        db: *mut std::ffi::c_void,
        album_key: *const Key,
        expected: *const ffi::CAlbum,
    ) -> bool;
    fn ffi_expect_album_tags(
        db: *mut std::ffi::c_void,
        album_key: *const Key,
        expected: *const ffi::CAlbumTags,
    ) -> bool;
    fn ffi_expect_scan_album_tags_sorted(db: *mut std::ffi::c_void, expected_len: usize) -> bool;
}

#[test]
fn ffi_open_db_opens_database() -> Result {
    let temp_dir = tempfile::tempdir()?;
    let path = CString::new(temp_dir.path().to_str().expect("temp path is valid utf-8"))?;

    assert!(unsafe { ffi_open_db_round_trip(path.as_ptr()) });

    Ok(())
}

#[test]
fn ffi_open_db_rejects_null_path_via_shim() {
    assert!(unsafe { ffi_open_db_rejects_null_path() });
}

#[test]
fn ffi_album_tags_round_trip() -> Result {
    let temp_dir = tempfile::tempdir()?;
    let db = sled::open(temp_dir.path())?;

    let album = Album::arbitrary();
    let mut expected: ffi::CAlbumTags = album.tags.clone().into();

    assert!(unsafe {
        ffi_expect_album_tags(
            &db as *const _ as *mut std::ffi::c_void,
            &db.insert_metadata(&album)? as *const Key,
            &expected as *const ffi::CAlbumTags,
        )
    });

    unsafe { free_album_tags(&mut expected as *mut ffi::CAlbumTags) };

    Ok(())
}

#[test]
fn ffi_album_tags_rejects_invalid_args() {
    assert!(!unsafe {
        ffi_expect_album_tags(std::ptr::null_mut(), std::ptr::null(), std::ptr::null())
    });
}

#[test]
fn ffi_scan_album_tags_sorted_round_trip() -> Result {
    let temp_dir = tempfile::tempdir()?;
    let db = sled::open(temp_dir.path())?;

    for _ in 0..10 {
        db.insert_metadata(&Album::arbitrary())?;
    }

    let expected_len = db.scan_album_tags_sorted()?.len();

    assert!(unsafe {
        ffi_expect_scan_album_tags_sorted(&db as *const _ as *mut std::ffi::c_void, expected_len)
    });

    Ok(())
}

#[test]
fn ffi_album_round_trip() -> Result {
    let temp_dir = tempfile::tempdir()?;
    let db = sled::open(temp_dir.path())?;

    let album = Album::arbitrary();
    let album_key = db.insert_metadata(&album)?;
    let mut expected: ffi::CAlbum = album.into();

    assert!(unsafe {
        ffi_expect_album(
            &db as *const _ as *mut std::ffi::c_void,
            &album_key as *const Key,
            &expected as *const ffi::CAlbum,
        )
    });

    unsafe { free_album(&mut expected as *mut ffi::CAlbum) };

    Ok(())
}
