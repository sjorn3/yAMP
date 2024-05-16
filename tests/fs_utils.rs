#![allow(clippy::missing_errors_doc)]

use std::{fs::File, path::Path};

use audiotags::Tag;
use fake::{Dummy, Faker};
use id3::{Tag as ID3Tag, TagLike, Version};
use music_cache::tests::common::*;
use music_cache::{AlbumTags, SongTags};
use tempfile::tempdir;

#[test]
fn test_round_trip_dummy_mp3() -> Result {
    let dir = tempdir()?;

    let temp_path = dir.path().join("music.mp3");

    let album = AlbumTags::arbitrary();
    let song_tags = SongTags::arbitrary();

    write_tags_to_path(&temp_path, &album, &song_tags)?;
    check_tags_from_path(&temp_path, &album, &song_tags)?;

    Ok(())
}

#[derive(Debug)]
pub struct SkeletonFileTree {
    dirs: Vec<SkeletonFileTree>,
    files: u8,
}

impl Dummy<Faker> for SkeletonFileTree {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        fn gen_tree<R: rand::Rng + ?Sized>(rng: &mut R, depth: u8) -> SkeletonFileTree {
            let next_depth = if depth <= 1 {
                0
            } else {
                rng.gen_range(1..=depth)
            };
            SkeletonFileTree {
                dirs: if next_depth == 0 {
                    Vec::new()
                } else {
                    (0..rng.gen_range(0..4))
                        .map(|_| gen_tree(rng, next_depth))
                        .collect()
                },
                files: rng.gen_range(0..3),
            }
        }
        gen_tree(rng, 3)
    }
}

impl SkeletonFileTree {
    pub fn generate_file_structure(&self, path: &Path) -> Result {
        for (i, dir) in self.dirs.iter().enumerate() {
            let new_path = path.join(i.to_string());
            std::fs::create_dir(&new_path)?;
            dir.generate_file_structure(&new_path)?;
        }
        let album_tags = AlbumTags::arbitrary();
        for file in 0..self.files {
            let new_path = path.join(file.to_string() + ".mp3");
            let song_tags = SongTags::arbitrary();
            write_tags_to_path(&new_path, &album_tags, &song_tags)?;
        }
        Ok(())
    }
}

// id3 is here is because it allows writing to an empty file. audiotags does not.
// would otherwise need to keep a dummy mp3 file and constantly copy it around.
fn write_tags_to_path(path: &Path, album_tags: &AlbumTags, song_tags: &SongTags) -> Result {
    File::create(path)?;

    let mut tag = ID3Tag::new();

    if let Some(album) = &album_tags.title {
        tag.set_album(album);
    }

    if let Some(title) = &song_tags.title {
        tag.set_title(title);
    }

    if let Some(album_artist) = &album_tags.artist {
        tag.set_album_artist(album_artist);
    }

    if let Some(year) = &album_tags.year {
        tag.set_year(i32::from(*year));
    }

    if let Some(track_number) = &song_tags.track_number {
        tag.set_track(u32::from(*track_number));
    }

    tag.write_to_path(path, Version::Id3v24)?;
    Ok(())
}

fn check_tags_from_path(
    path: &Path,
    expected_album: &AlbumTags,
    expected_song: &SongTags,
) -> Result {
    let tag = Tag::new().read_from_path(path)?;
    let album = AlbumTags::read(&tag);
    let song = SongTags::read(&tag);
    assert_eq!(*expected_song, song);
    assert_eq!(*expected_album, album);
    Ok(())
}
