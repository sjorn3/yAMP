use std::{ffi::CString, os::raw::c_char, ptr};

use crate::{Album, AlbumTags, Key, Methods, Result};

#[repr(C)]
pub struct CAlbumTags {
    pub artist: *mut c_char,
    pub title: *mut c_char,
    pub has_year: bool,
    pub year: u16,
}

fn c_string_from_option(value: Option<String>) -> *mut c_char {
    value
        .and_then(|val| CString::new(val).ok())
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

impl From<AlbumTags> for CAlbumTags {
    fn from(tags: AlbumTags) -> Self {
        let (has_year, year) = tags
            .year
            .map(|year| (true, year))
            .unwrap_or((false, 0u16));

        CAlbumTags {
            artist: c_string_from_option(tags.artist),
            title: c_string_from_option(tags.title),
            has_year,
            year,
        }
    }
}

impl CAlbumTags {
    const fn empty() -> Self {
        CAlbumTags {
            artist: ptr::null_mut(),
            title: ptr::null_mut(),
            has_year: false,
            year: 0,
        }
    }
}

/// # Safety
/// The `db` pointer must be a valid `sled::Db` pointer, and `out_tags` must be a
/// valid, writable pointer. The caller takes ownership of any string pointers in
/// `out_tags` and should release them with `free_album_tags`.
#[no_mangle]
pub unsafe extern "C" fn album_tags_for_key(
    db: *mut sled::Db,
    album_key: *const Key,
) -> CAlbumTags {
    if db.is_null() || album_key.is_null() {
        return CAlbumTags::empty();
    }

    let key = &*album_key;
    let db_ref = &*db;

    let album: Result<Album> = db_ref.get_metadata(key);
    match album {
        Ok(album) => album.tags.into(),
        Err(_) => CAlbumTags::empty(),
    }
}

/// # Safety
/// The caller must pass a pointer previously filled by `album_tags_for_key`.
#[no_mangle]
pub unsafe extern "C" fn free_album_tags(tags: *mut CAlbumTags) {
    if tags.is_null() {
        return;
    }

    let tags = &mut *tags;
    if !tags.artist.is_null() {
        let _ = CString::from_raw(tags.artist);
        tags.artist = ptr::null_mut();
    }
    if !tags.title.is_null() {
        let _ = CString::from_raw(tags.title);
        tags.title = ptr::null_mut();
    }
}
