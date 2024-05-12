use music_cache::*;

use fake::{Dummy, Fake, Faker};
use id3::{Tag, TagLike, Version};
use std::fs::File;
use tempfile::*;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_create_dummy_mp3() -> Result {
    let dir = tempdir()?;

    let temp_file = dir.path().join("music.mp3");
    File::create(&temp_file)?;

    let mut tag = Tag::new();
    tag.set_album("Album Title");

    tag.write_to_path(&temp_file, Version::Id3v24)?;

    let tag_result = Tag::read_from_path(temp_file)?;
    assert_eq!(tag_result.album().ok_or("No Album Tag")?, "Album Title");

    Ok(())
}

#[test]
fn test_dummy() -> Result {
    let music: SongTags = Faker.fake();
    println!("{:?}", music);

    Ok(())
}
