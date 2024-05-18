use music_cache::{
    tests::{common::Result, Arbitrary},
    *,
};
use std::{path::Path, sync::Arc};
use tempfile::*;

mod fs_utils;
use fs_utils::SkeletonFileTree;

use rand::prelude::*;

#[ignore]
#[allow(dead_code)]
fn test_my_library() -> Result {
    let db_dir = tempdir()?;
    let tree = Arc::new(sled::open(db_dir.path())?);
    let dir = Path::new(r"librarypath");

    scan_library(Arc::clone(&tree), dir)?;

    let albums = tree
        .scan_albums()
        .collect::<music_cache::Result<Vec<Album>>>()?;

    for album in albums.iter() {
        println!(
            "{} by {} - {}",
            album.tags.title.clone().unwrap(),
            album.tags.artist.clone().unwrap_or("Unknown".to_string()),
            album.tags.year.unwrap_or(0),
        );
        let tags: Vec<String> = album
            .songs
            .iter()
            .map(|song| song.tags.title.clone().unwrap())
            .collect();
        println!("Songs: {:#?}", tags);
    }

    let songs = tree
        .scan_songs()
        .collect::<music_cache::Result<Vec<Song>>>()?;

    println!("Total Songs: {:?}", songs.len());

    Ok(())
}

#[test]
fn test_gen_file_tree() -> Result {
    // TODO we should add some junk files inside here too.
    // And potentially images since album art is sometimes in the dir.
    let dir = tempdir()?;

    let tree: SkeletonFileTree = SkeletonFileTree {
        dirs: vec![
            SkeletonFileTree {
                dirs: vec![SkeletonFileTree {
                    dirs: vec![],
                    files: 10,
                }],
                files: 0,
            },
            SkeletonFileTree {
                dirs: vec![SkeletonFileTree {
                    dirs: vec![],
                    files: 10,
                }],
                files: 10,
            },
        ],
        files: 0,
    };
    let all_tags = tree.generate_file_structure(dir.path())?;

    let db_dir = tempdir()?;
    let tree = Arc::new(sled::open(db_dir.path())?);

    scan_library(Arc::clone(&tree), dir.path())?;

    for (album_tags, song) in all_tags {
        let restored_song = tree.get_song_from_path(&song.relpath)?()?;
        assert!(restored_song.tags == song.tags);
        let restored_album: Album = tree.get_metadata(&album_tags.hash_key())?;
        assert!(restored_album.songs.contains(&restored_song));
        assert!(restored_album.tags == album_tags);
    }

    Ok(())
}

#[test]
fn test_db_round_trip() -> Result {
    let dir = TempDir::new().unwrap();
    let tree = sled::open(dir.path()).unwrap();
    let tags: Song = Song::arbitrary();
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
fn test_db_retrieve_song_by_path() -> Result {
    let dir = TempDir::new().unwrap();
    let tree = sled::open(dir.path()).unwrap();
    let tags = Song::arbitrary();
    let tags2 = Song::arbitrary();
    tree.insert_metadata(&tags)?;
    tree.insert_metadata(&tags2)?;
    let restored = tree.get_song_from_path(&tags.relpath)?()?;
    let restored2 = tree.get_song_from_path(&tags2.relpath)?()?;
    assert_eq!(tags, restored);
    assert_eq!(tags2, restored2);
    Ok(())
}

#[test]
fn test_db_scan_songs() -> Result {
    let dir = TempDir::new().unwrap();
    let tree = sled::open(dir.path()).unwrap();
    let song = Song::arbitrary();

    tree.insert_metadata(&song)?;

    for restored_song in tree.scan_songs() {
        assert_eq!(restored_song?, song);
    }
    Ok(())
}

#[test]
fn test_db_scan_albums() -> Result {
    let dir = TempDir::new().unwrap();
    let tree = sled::open(dir.path()).unwrap();
    let album = Album::arbitrary();
    tree.insert_metadata(&album)?;

    for restored_album in tree.scan_albums() {
        assert_eq!(restored_album?, album);
    }
    Ok(())
}

#[test]
fn test_scan_time() -> Result {
    let dir = TempDir::new()?;
    let tree = sled::open(dir.path())?;
    tree.set_last_scan_time()?;
    let time = tree.get_last_scan_time()?;
    tree.set_last_scan_time()?;
    let new_time = tree.get_last_scan_time()?;
    assert!(new_time > time);

    Ok(())
}

#[test]
fn test_album_upsert() -> Result {
    let dir = TempDir::new()?;
    let tree = sled::open(dir.path())?;
    let album_tags = AlbumTags::arbitrary();
    let mut songs: Vec<Song> = (0..10).map(|_| Song::arbitrary()).collect();
    for (i, song) in songs.iter_mut().enumerate() {
        song.tags.track_number = Some(i as u16);
    }

    let mut unordered_songs: Vec<(usize, &mut Song)> = songs.iter_mut().enumerate().collect();

    let mut rng = thread_rng();
    unordered_songs.shuffle(&mut rng);

    for (_, song) in unordered_songs {
        let song_key = tree.insert_metadata(song)?;
        album_upsert(&tree, &album_tags, song, &song_key)?;
    }

    let restored_album: Album = tree.get_metadata(&album_tags.hash_key())?;
    for (i, song) in songs.iter().enumerate() {
        assert_eq!(restored_album.songs[i], *song);
    }
    Ok(())
}

#[test]
fn test_remove_song_from_album() -> Result {
    let dir = TempDir::new()?;
    let tree = sled::open(dir.path())?;
    let album_tags = AlbumTags::arbitrary();
    let album_key = album_tags.hash_key();
    let songs: Vec<Song> = (0..10).map(|_| Song::arbitrary()).collect();

    let song_keys: Vec<Key> = songs
        .iter()
        .map(|song| {
            let key = tree.insert_metadata(song)?;
            album_upsert(&tree, &album_tags, song, &key)?;
            Ok(key)
        })
        .collect::<music_cache::Result<Vec<Key>>>()?;

    assert!(tree.get(&album_key)?.is_some());

    let mut shuffled: Vec<(&Key, &Song)> = song_keys.iter().zip(songs.iter()).collect();
    let mut rng = thread_rng();
    shuffled.shuffle(&mut rng);

    for (key, song) in shuffled {
        let album: Album = tree.get_metadata(&album_key)?;
        assert!(album.songs.contains(song));

        remove_song_from_album(&tree, &album_key, key)?;

        let result_album: music_cache::Result<Album> = tree.get_metadata(&album_key);
        if let Ok(album) = result_album {
            assert!(!album.songs.contains(song))
        };
        assert!(tree.get(key)?.is_some());
    }

    assert!(tree.get(album_key)?.is_none());

    Ok(())
}
