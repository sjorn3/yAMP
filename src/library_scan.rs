use jwalk::WalkDir;
use std::path::Path;

// Algorithm
// 1. Walk the directory
// 2. For each file attempt to read the tags (audiotags will early exit if it doesn't know the extension)
// 3. If the tags are read successfully
//    1. Check if the file path exists as a key in the db.
//    2. If it does exist, check the modification time of the file, overwrite the songtags in the db if newer.
//       If the albumtags are newer, remove the reference to this song from the album and perform an album upsert.
//    3. If it does not exist, add the new song tags and perform an album upsert.
//    4. Elsewise, if the file does exist in the db and the modification time is not newer, do nothing.

// Album upsert algorithm
// 1. Search for an album tags object with the matching tags. (This might be faster with tantivy if that's integrated)
// 2. If it exists, find the album and add the song to the album. (potentially albums should have their song keys as a Btreemap so I can insert songs with their track number and get them back in order, also it would help if album tags reverse pointed to the album to save lookup time.)
// 3. If it does not exist, insert a new album, with this song as the only song.

// Every stage above involving writes should lock the relevant keys for safety, but I believe that sled will do that already if I use the right methods at each stage.

// A big simplification of this process would be to load everything into memory during the walk and tag finding and then operate on that, performing the searches e.t.c in a big in memory data structure.
// This would potentially be faster but in general songs that are in the same album should be processed at a similar time if the folder structure is at all sensible, so we should be "lucky" with cache hits more often.
// Further, writing to disk is going to be a big bottleneck so the earlier it can be started the better.
// Also, if we imagine the UI is running in a seperate thread, it could potentially start showing music whilst it's being found, which could be kind of neat.

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
