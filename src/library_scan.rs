use jwalk::WalkDir;
use std::path::Path;

use crate::{AlbumTags, ByteKey, HashKeyGen, Key, Result, Song, StoredAlbum};

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

// Album upsert algorithm
// 1. lookup album_tags.hash_key()
// 2. If it exists add the song to the album. (potentially albums should have their song keys as a Btreemap so I can insert songs with their track number and get them back in order, also it would help if album tags reverse pointed to the album to save lookup time.)
// 3. If it does not exist, insert a new album, with this song as the only song, and create a new album tags object.

// Every stage above that involving writes should lock the relevant keys for safety, but I believe that sled will do that already if I use the right methods at each stage.

// A big simplification of this process would be to load everything into memory during the walk and tag finding and then operate on that, performing the searches e.t.c in a big in memory data structure.
// This would potentially be faster but in general songs that are in the same album should be processed at a similar time if the folder structure is at all sensible, so we should be "lucky" with cache hits more often.
// Further, writing to disk is going to be a big bottleneck so the earlier it can be started the better.
// Also, if we imagine the UI is running in a seperate thread, it could potentially start showing music whilst it's being found, which could be kind of neat.

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

pub fn album_upsert(
    tree: &sled::Db,
    album_tags: &AlbumTags,
    song: &Song,
    song_key: Key,
) -> Result<()> {
    let album_key = album_tags.hash_key();
    let byte_key = song_key.to_byte_key();
    // Sadly update_and_fetch doesn't preserve Result types, so unwrap and hope for the best.
    tree.update_and_fetch(album_key, |maybe_bytes| {
        let new_album = if let Some(bytes) = maybe_bytes {
            add_song_to_album(bytes, song, byte_key).unwrap()
        } else {
            StoredAlbum::new(album_tags, (song.tags.track_number, byte_key))
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
                    // dir_entry.i
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
