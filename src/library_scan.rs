use audiotags::Error::UnknownFileExtension;
use audiotags::Tag;
use jwalk::WalkDir;
use std::{path::Path, time::SystemTime};

use crate::{
    song_hash_key, AlbumTags, ByteKey, HashKeyGen, Key, Result, Song, SongTags, StoredAlbum,
};

// Algorithm
// 1. Walk the directory
// 2. For each file attempt to read the tags (audiotags will early exit if it doesn't know the extension)
//    1. Also, load all KeyType::Songs into memory in a set with quick deletion, and remove all that are found in the walk. Remove any remaining keys from the db at the end.
// 3. If the tags are read successfully
//    1. Check if the file path exists as a key in the db.
//    2. If it does exist, check the modification time of the file, overwrite the songtags in the db if newer than global last scan time.
//       If the albumtags have changed, remove the reference to this song from the album and perform an album upsert.
//       If that was the last song in the album, remove the album and album tags object (also any album art.. other references e.t.c)
//    3. If it does not exist, add the new song tags and perform an album upsert.
//    4. Elsewise, if the file does exist in the db and the modification time is not newer, do nothing.
// 4. Once complete for all files, update global last scan time. As such, if a panic occurs during the scan, it will restart on the next run. Progress will be saved although work will be repeated anyway.

// full_update will typically just be checking when the file has been modified and perform various functions if so, but it can also be set true for all files to force a full library reset.
// This would be neater if both songs and albums had a pointer to their album tags. However,

// At this point, we've check the song metadata and established whether or not the album should be updated.
// We also will already know the new key and have the album upsert alg, so this is literally just
// inserting song tags and performing album upsert. As such not sure this is even that helpful.
// fn handle_song_path(song_path: &Path, update: bool, tags: AudioTags) {}

// modification_date = get the file modification date
// maybe_song_tags = get the hash key for the file path and maybe fetch from the db
// if None or (Some(song_tags) and modification_date > global last scan date) -> maybe calculate audio tags -> if it's a valid song insert tags and perform album upsert
// else do Nothing.
// There's a somewhat subtle question here in that we could calculate the audio tags inside a update_and_fetch transaction and save doing a second lookup.
// Since there can only ever be one of each file, this won't lock anything, and really behaviour will be exactly the same but we won't be able to report errors.
// should probably check if rust has something like a try catch to catch panics in a higher scope.
// Should absolutely prioritize the "Nothing" state and make that as fast as possible - this will be what you're doing most of the time. Therefore it's worth trying to do no IO or more than one lookup in this case.

fn process_tags(
    tree: &sled::Db,
    path: &Path,
    relpath: &[u8],
    song_key: &Key,
) -> Result<Option<Song>> {
    let tags = Tag::new().read_from_path(path);
    if let Err(UnknownFileExtension(_)) = tags {
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
fn process_file(tree: &sled::Db, path: &Path, last_scan_time: &SystemTime) -> Result<()> {
    let path_bytes = path.as_os_str().as_encoded_bytes();
    let song_key = song_hash_key(path_bytes);

    tree.update_and_fetch(&song_key, |maybe_bytes| {
        if maybe_bytes.is_some()
            && path.metadata().and_then(|m| m.modified()).unwrap() < *last_scan_time
        {
            return maybe_bytes.map(|bytes| bytes.into());
        }

        process_tags(tree, path, path_bytes, &song_key)
            .unwrap()
            .map(|song| song.serialize().unwrap().into_boxed_slice())
    })?;
    Ok(())
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

pub fn remove_song_from_album(
    tree: &sled::Db,
    album_tags: &AlbumTags,
    song_key: &Key,
) -> Result<()> {
    let album_key = album_tags.hash_key();
    let byte_key = *song_key.to_byte_key();
    tree.update_and_fetch(album_key, |maybe_bytes| {
        maybe_bytes.and_then(|bytes| find_remove_song_from_album(bytes, byte_key).unwrap())
    })?;
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

pub fn walk(dir: &Path) {
    let mut count = 0;
    let walk_dir = WalkDir::new(dir).process_read_dir(|_, _, _, children| {
        children.retain(|dir_entry_result| {
            dir_entry_result
                .as_ref()
                .map(|dir_entry: &jwalk::DirEntry<((), ())>| {
                    // dir_entry.metadata().modified
                    dir_entry.file_type.is_file()
                        && dir_entry
                            .file_name
                            .to_str()
                            .map(|s| s.ends_with(".mp3"))
                            .unwrap_or(false)
                })
                .unwrap_or(false)
        });
    });

    for entry in walk_dir {
        let entry = entry.unwrap();
        count += 1;
        println!("{}", entry.path().display());
    }
    println!("Count: {}", count);
}
