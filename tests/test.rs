use music_cache::*;

use fake::{Fake, Faker};
use id3::{Tag as ID3Tag, TagLike, Version};
use std::fs::File;
use std::path::Path;
use tempfile::*;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_create_dummy_mp3() -> Result {
    let dir = tempdir()?;

    let temp_path = dir.path().join("music.mp3");

    let song_tags: SongTags = Faker.fake();

    write_tags_to_path(&temp_path, &song_tags)?;
    check_tags_from_path(&temp_path, &song_tags)?;

    Ok(())
}

// id3 is here is because it allows writing to an empty file. audiotags does not.
// would otherwise need to keep a dummy mp3 file and constantly copy it around.
fn write_tags_to_path(path: &Path, song: &SongTags) -> Result {
    File::create(path)?;

    let mut tag = ID3Tag::new();

    if let Some(album) = &song.album {
        tag.set_album(album);
    }

    if let Some(title) = &song.title {
        tag.set_title(title);
    }

    if let Some(album_artist) = &song.album_artist {
        tag.set_album_artist(album_artist);
    }

    if let Some(year) = &song.year {
        tag.set_year(i32::from(*year));
    }

    if let Some(track_number) = &song.track_number {
        tag.set_track(u32::from(*track_number));
    }

    tag.write_to_path(path, Version::Id3v24)?;
    Ok(())
}

fn check_tags_from_path(path: &Path, song: &SongTags) -> Result {
    assert_eq!(&SongTags::from_path(path)?, song);
    Ok(())
}
