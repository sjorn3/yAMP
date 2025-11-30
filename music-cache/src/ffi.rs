use std::{ffi::CString, os::raw::c_char, ptr};

use crate::{Album, AlbumTags, Key, Methods, Result, Song, SongTags};

#[repr(C)]
pub struct CAlbumTags {
    pub artist: *mut c_char,
    pub title: *mut c_char,
    pub has_year: bool,
    pub year: u16,
}

#[repr(C)]
pub struct CSongTags {
    pub title: *mut c_char,
    pub has_track_number: bool,
    pub track_number: u16,
}

#[repr(C)]
pub struct CSong {
    pub tags: CSongTags,
    pub relpath: *mut c_char,
}

#[repr(C)]
pub struct CAlbum {
    pub tags: CAlbumTags,
    pub songs: *mut CSong,
    pub song_count: usize,
}

fn c_string_from_option<T: Into<Vec<u8>>>(value: Option<T>) -> *mut c_char {
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
        Self {
            artist: ptr::null_mut(),
            title: ptr::null_mut(),
            has_year: false,
            year: 0,
        }
    }
}

impl From<SongTags> for CSongTags {
    fn from(tags: SongTags) -> Self {
        let (has_track_number, track_number) = tags
            .track_number
            .map(|track| (true, track))
            .unwrap_or((false, 0u16));

        CSongTags {
            title: c_string_from_option(tags.title),
            has_track_number,
            track_number,
        }
    }
}

impl From<Song> for CSong {
    fn from(song: Song) -> Self {
        CSong {
            tags: song.tags.into(),
            relpath: c_string_from_option(Some(song.relpath)),
        }
    }
}

impl From<Album> for CAlbum {
    fn from(album: Album) -> Self {
        let mut songs: Box<[CSong]> = album
            .songs
            .into_iter()
            .map(CSong::from)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let song_count = songs.len();
        let songs_ptr = if song_count == 0 {
            ptr::null_mut()
        } else {
            songs.as_mut_ptr()
        };
        std::mem::forget(songs);

        CAlbum {
            tags: album.tags.into(),
            songs: songs_ptr,
            song_count,
        }
    }
}

impl CAlbum {
    const fn empty() -> Self {
        CAlbum {
            tags: CAlbumTags::empty(),
            songs: ptr::null_mut(),
            song_count: 0,
        }
    }
}

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

    let album_tags: Result<AlbumTags> = db_ref.get_metadata(key);
    match album_tags {
        Ok(album_tags) => album_tags.into(),
        Err(_) => CAlbumTags::empty(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn album_for_key(db: *mut sled::Db, album_key: *const Key) -> CAlbum {
    if db.is_null() || album_key.is_null() {
        return CAlbum::empty();
    }

    let key = &*album_key;
    let db_ref = &*db;

    let album: Result<Album> = db_ref.get_metadata(key);
    match album {
        Ok(album) => album.into(),
        Err(_) => CAlbum::empty(),
    }
}

fn free_c_string(ptr: &mut *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: The pointer was allocated by CString::into_raw in this module.
        unsafe {
            let _ = CString::from_raw(*ptr);
        }
        *ptr = ptr::null_mut();
    }
}

fn free_album_tags_inner(tags: &mut CAlbumTags) {
    free_c_string(&mut tags.artist);
    free_c_string(&mut tags.title);
    tags.has_year = false;
    tags.year = 0;
}

#[no_mangle]
pub unsafe extern "C" fn free_album_tags(tags: *mut CAlbumTags) {
    if tags.is_null() {
        return;
    }

    free_album_tags_inner(&mut *tags);
}

fn free_song_tags(tags: &mut CSongTags) {
    free_c_string(&mut tags.title);
    tags.has_track_number = false;
    tags.track_number = 0;
}

fn free_song(song: &mut CSong) {
    free_song_tags(&mut song.tags);
    free_c_string(&mut song.relpath);
}

#[no_mangle]
pub unsafe extern "C" fn free_album(album: *mut CAlbum) {
    if album.is_null() {
        return;
    }

    let album = &mut *album;
    free_album_tags_inner(&mut album.tags);

    if album.songs.is_null() || album.song_count == 0 {
        album.songs = ptr::null_mut();
        album.song_count = 0;
        return;
    }

    let songs_ptr = std::ptr::slice_from_raw_parts_mut(album.songs, album.song_count);
    let mut songs_box = Box::from_raw(songs_ptr);
    for song in songs_box.iter_mut() {
        free_song(song);
    }
    album.songs = ptr::null_mut();
    album.song_count = 0;
}
