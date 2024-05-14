mod fs_utils;

use fs_utils::SkeletonFileTree;

use fake::{Fake, Faker};
use music_cache::{tests::common::*, *};
use once_cell::sync::Lazy;
use tempfile::*;

#[test]
fn test_gen_file_tree() -> Result {
    // TODO we should add some junk files inside here too.
    // And potentially images since album art is sometimes in the dir.
    let dir = tempdir()?;
    let tree: SkeletonFileTree = Faker.fake();
    tree.generate_file_structure(dir.path())?;
    Ok(())
}

static TEMP_DIR: Lazy<TempDir> = Lazy::new(|| TempDir::new().unwrap());
static DB: Lazy<sled::Db> = Lazy::new(|| sled::open(TEMP_DIR.path()).unwrap());

#[test]
fn test_db_round_trip() -> Result {
    let tree = &*DB;
    let tags: SongTags = Faker.fake();
    let key = tree.generate_key(&tags)?;

    {
        let archived_tags: SongTags = tags.clone();
        let bytes = rkyv::to_bytes::<SongTags, 1024>(&archived_tags)?;
        tree.insert(&key, bytes.as_slice())?;
    }

    let value = tree.get(&key)?.unwrap();
    #[allow(clippy::unnecessary_to_owned)]
    // Unfortunately sled doesn't guarantee alignment, so push the value into a vector to ensure it's aligned, adding a copy.
    // If I'm doing this anyway, rkyv has some bytecheck features that might be worth looking into.
    // the bytecheck stuff doesn't fix the alignment issue.

    // I could maybe think about forking sled to have alignment. But that's a pretty big undertaking and I don't know what the tradeoffs are.
    let restored: SongTags = unsafe { rkyv::from_bytes_unchecked(&value.to_vec())? };
    assert_eq!(tags, restored);
    Ok(())
}

#[test]
fn test_db_scan_prefix() -> Result {
    let dir = TempDir::new().unwrap();
    let tree = sled::open(dir.path()).unwrap();
    let tags: SongTags = Faker.fake();

    let key = tree.generate_key(&tags)?;

    let first_value = rkyv::to_bytes::<SongTags, 1024>(&tags)?;
    tree.insert(key, first_value.as_ref())?;

    for x in tree.scan_prefix(KeyType::SongTags) {
        let (key, value) = x?;
        #[allow(clippy::unnecessary_to_owned)]
        let restore: SongTags = unsafe { rkyv::from_bytes_unchecked(&value.to_vec())? };
        assert_eq!(restore, tags);
        assert_eq!(key.to_vec(), key.as_ref().to_vec());
    }
    Ok(())
}

#[test]
fn test_walk_dir() -> Result {
    let dir = tempdir()?;
    let tree: SkeletonFileTree = Faker.fake();
    tree.generate_file_structure(dir.path())?;
    music_cache::walk(dir.path());
    Ok(())
}
