mod fs_utils;

use fs_utils::SkeletonFileTree;

use fake::{Fake, Faker};
use music_cache::{tests::common::Result, *};
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
    let tags: Song = Faker.fake();
    let key = {
        // Archived tags will be freed before restoring, verifying that no pointers are stored.
        let archived_tags = tags.clone();
        tree.insert_metadata(&archived_tags)?
    };
    let restored: Song = tree.get_metadata(&key)?;
    assert_eq!(tags, restored);
    Ok(())
}

#[test]
fn test_db_scan_songs() -> Result {
    let dir = TempDir::new().unwrap();
    let tree = sled::open(dir.path()).unwrap();
    let tags: Song = Faker.fake();

    tree.insert_metadata(&tags)?;

    let songs = tree.scan_songs();

    for song in songs {
        assert_eq!(song?, tags);
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
