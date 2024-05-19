use crate::methods::scan_stored_albums;
use audiotags::Tag;
use jwalk::WalkDir;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    collections::LinkedList,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::{
    song_hash_key, AlbumTags, ByteKey, HashKeyGen, Helpers, Key, Result, Song, SongTags,
    StoredAlbum,
};

fn process_tags(
    tree: &sled::Db,
    path: &Path,
    relpath: &[u8],
    song_key: &Key,
) -> Result<Option<Song>> {
    let tags = Tag::new().read_from_path(path);
    if tags.is_err() {
        return Ok(None);
    };
    let audio_tags = tags?;
    let song_tags = SongTags::read(&audio_tags);
    let album_tags = AlbumTags::read(&audio_tags);
    let song = Song::new(song_tags, relpath);
    album_upsert(tree, &album_tags, &song, song_key)?;
    Ok(Some(song))
}

// This performs an update_and_fetch inside an update_and_fetch.
// This is dangerous because sled will call update repeatedly if the underlying value changes whilst running.
// However, it's theoretically not possible because only one song is ever accessed at a time.
// It is possible to write to the same album key simultaneously but album update doesn't alter any state.
// At some point I should benchmark if the decision to do it in this absurd way is worth it over a more sensible transaction.
// For now I'm assuming it is because it lives in a very hot path of the load logic and it halves the number of lookups.
fn process_file(
    tree: Arc<sled::Db>,
    path: &Path,
    last_scan_time: &SystemTime,
    song_keys: &Arc<Mutex<HashMap<Key, Key>>>,
) -> Result<Option<(Arc<sled::Db>, std::path::PathBuf, Key)>> {
    let path_bytes = path.as_os_str().as_encoded_bytes();
    let song_key = song_hash_key(path_bytes);

    {
        // TODO unwrap here. I don't like this whole thing, want to use a proper threadsafe hashmap or trie.
        let mut guard = song_keys.lock().unwrap();
        guard.remove(&song_key);
    }

    if tree.get(&song_key)?.is_some()
        && path.metadata().and_then(|m| m.modified()).unwrap() < *last_scan_time
    {
        return Ok(None);
    }

    if let Some(ext) = path.extension() {
        // TODO Implement resilient check function equivalent
        if ext == "mp3" || ext == "flac" || ext == "m4a" {
            return Ok(Some((tree, path.to_path_buf(), song_key)));
        }
    }

    Ok(None)
}

fn add_song_to_album(bytes: &[u8], song: &Song, song_key: ByteKey) -> Result<StoredAlbum> {
    let mut album = StoredAlbum::partial_deserialize_album(bytes)?;
    let index = album
        .song_keys
        .binary_search_by(|probe| probe.0.cmp(&song.tags.track_number))
        .unwrap_or_else(|x| x);
    album
        .song_keys
        .insert(index, (song.tags.track_number, song_key));
    Ok(album)
}

fn find_remove_song_from_album(bytes: &[u8], song_key: ByteKey) -> Result<Option<StoredAlbum>> {
    let mut album = StoredAlbum::partial_deserialize_album(bytes)?;
    if album.song_keys.len() == 1 {
        return Ok(None);
    }
    album.song_keys.retain(|&(_, key)| key != song_key);
    Ok(Some(album))
}

pub fn remove_song_from_album(tree: &sled::Db, album_key: &Key, song_key: &Key) -> Result<()> {
    let byte_key = *song_key.to_byte_key();
    tree.update_and_fetch(album_key, |maybe_bytes| {
        maybe_bytes.and_then(|bytes| find_remove_song_from_album(bytes, byte_key).unwrap())
    })?;
    Ok(())
}

pub fn remove_song(tree: &sled::Db, album_key: &Key, song_key: &Key) -> Result<()> {
    remove_song_from_album(tree, album_key, song_key)?;
    tree.remove(song_key)?;
    Ok(())
}

pub fn album_upsert(
    tree: &sled::Db,
    album_tags: &AlbumTags,
    song: &Song,
    song_key: &Key,
) -> Result<()> {
    let album_key = album_tags.hash_key();
    let byte_key = *song_key.to_byte_key();
    tree.update_and_fetch(album_key, |maybe_bytes| {
        let new_album = if let Some(bytes) = maybe_bytes {
            add_song_to_album(bytes, song, byte_key).unwrap()
        } else {
            StoredAlbum::new(album_tags.clone(), (song.tags.track_number, byte_key))
        };
        Some(new_album)
    })?;
    Ok(())
}

fn apply_process_file(info: &(Arc<sled::Db>, PathBuf, Key)) -> Result<()> {
    let (tree, path, song_key) = info;
    let path_bytes = path.as_os_str().as_encoded_bytes();

    if let Some(song) = process_tags(tree, path, path_bytes, song_key)? {
        tree.insert(song_key, &song)?;
    }

    Ok(())
}

pub fn scan_library(tree: Arc<sled::Db>, dir: &Path) -> Result<()> {
    let last_scan_time = Arc::new(tree.get_last_scan_time()?);

    // TODO I don't like this but don't worry about it too much until you can start profiling. The problem is that it's blocking everything at startup and it's extremely non trivial
    let song_keys = Arc::new(Mutex::new(scan_stored_albums(&tree)?));
    let final_tree = Arc::clone(&tree);
    let final_song_keys = Arc::clone(&song_keys);

    let files_to_load = Arc::new(Mutex::new(LinkedList::new()));

    let final_files_to_load = Arc::clone(&files_to_load);

    for _ in WalkDir::new(dir).process_read_dir(move |_, _, _, children| {
        let last_scan_time = Arc::clone(&last_scan_time);
        let song_keys = Arc::clone(&song_keys);
        for dir_entry_result in children {
            let dir_entry = dir_entry_result.as_ref().unwrap();
            if dir_entry.file_type.is_file() {
                if let Some(file_to_load) = process_file(
                    Arc::clone(&tree),
                    &dir_entry.path(),
                    &last_scan_time,
                    &song_keys,
                )
                .unwrap()
                {
                    files_to_load.lock().unwrap().push_back(file_to_load);
                }
            }
        }
    }) {}

    let files_to_load_list = final_files_to_load.lock().unwrap();
    files_to_load_list
        .par_iter()
        .for_each(|file_to_load| apply_process_file(file_to_load).unwrap());

    for (album_key, song_key) in final_song_keys.lock().unwrap().iter() {
        remove_song(&final_tree, album_key, song_key)?;
    }

    final_tree.set_last_scan_time()?;
    Ok(())
}
