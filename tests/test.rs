mod fs_utils;
use std::io::Read;

use fs_utils::SkeletonFileTree;

use fake::{Fake, Faker};
use music_cache::{tests::common::*, SongTags};
use once_cell::sync::Lazy;
use tempfile::*;

#[test]
fn test_gen_file_tree() -> Result {
    let dir = tempdir()?;
    let tree: SkeletonFileTree = Faker.fake();
    tree.generate_file_structure(dir.path())?;
    Ok(())
}

static TEMP_DIR: Lazy<TempDir> = Lazy::new(|| TempDir::new().unwrap());
static DB: Lazy<sled::Db> = Lazy::new(|| sled::open(TEMP_DIR.path()).unwrap());

#[test]
fn test() -> Result {
    let tree = &*DB;
    let tags: SongTags = Faker.fake();

    let key = DB.generate_id()?.to_le_bytes();

    // let bytes = rkyv::to_bytes::<SongTags, 1024>(&tags)?;
    // // let boxed_slice: Box<[u8]> = Box::from(bytes);
    // tree.insert(key, boxed_slice)?;

    // The 1024 here has been chosen at random. I could probably make this smaller. Basically it sets the size of the vector that is serialized into.
    // It will grow if the data is larger. Therefore there is a tradeoff between speed and memory usage.
    // 1KB is probably too much as track num is u16 and then it's only title, which will be like 100 bytes max.
    // with just one insert the difference is negligible, but worth benching (also check docs there is a tool mentioned for to_bytes choosing the best size) when larger.
    // actually maybe ignore this - the size of the vector is not the size of the data, it's the size of the buffer that is allocated.
    // a tight buffer would still be faster.
    tree.insert(key, rkyv::to_bytes::<SongTags, 1024>(&tags)?.as_slice())?;

    let value: Vec<u8> = tree
        .get(key)?
        .unwrap()
        .bytes()
        .map(|x| x.unwrap())
        .collect(); // We should create a key type - ideally the unique keys that are provided by sled.
    let restored: SongTags = unsafe { rkyv::from_bytes_unchecked(&value)? };

    assert_eq!(tags, restored);
    tree.flush()?;
    Ok(())
}
