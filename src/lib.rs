#![allow(clippy::missing_errors_doc)]

use jwalk::WalkDir;
use rkyv::{Archive, Deserialize, Serialize};
use std::path::Path;

#[cfg_attr(
    feature = "integration-tests",
    derive(Debug, fake::Dummy, PartialEq, Eq, Clone)
)]
#[derive(Archive, Serialize, Deserialize)]
pub struct SongTags {
    pub title: Option<String>,
    pub track_number: Option<u16>,
}

#[cfg_attr(
    feature = "integration-tests",
    derive(Debug, fake::Dummy, PartialEq, Eq, Clone)
)]
#[derive(Archive, Serialize, Deserialize)]
pub struct Song {
    pub tags: SongTags,
    // converting a path to a utf8 string might not be valid and there's no Archive instance for PathBuf so just store it as bytes.
    pub relpath: Vec<u8>,
}

#[cfg_attr(
    feature = "integration-tests",
    derive(Debug, fake::Dummy, PartialEq, Eq)
)]
#[derive(Archive, Serialize, Deserialize)]
pub struct AlbumTags {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
}

#[cfg_attr(
    feature = "integration-tests",
    derive(Debug, fake::Dummy, PartialEq, Eq)
)]
#[derive(Archive, Serialize, Deserialize)]
pub struct Album {
    pub tags: AlbumTags,
    pub songs: Vec<Song>,
}

type AudioTag = Box<dyn audiotags::AudioTag + Send + Sync>;

impl AlbumTags {
    pub fn read(tag: &AudioTag) -> AlbumTags {
        AlbumTags {
            artist: tag.album_artist().map(ToString::to_string),
            title: tag.album_title().map(ToString::to_string),
            year: tag.year().and_then(|y| y.try_into().ok()),
        }
    }
}

impl SongTags {
    pub fn read(tag: &AudioTag) -> SongTags {
        SongTags {
            title: tag.title().map(ToString::to_string),
            track_number: tag.track_number(),
        }
    }
}

impl Song {
    pub fn read(tag: &AudioTag, relpath: &Path) -> Song {
        Song {
            tags: SongTags::read(tag),
            relpath: relpath.to_path_buf().into_os_string().into_encoded_bytes(),
        }
    }
}

pub fn walk(dir: &Path) {
    let mut count = 0;
    let walk_dir = WalkDir::new(dir).process_read_dir(|_, _, _, children| {
        children.retain(|dir_entry_result| {
            dir_entry_result
                .as_ref()
                .map(|dir_entry: &jwalk::DirEntry<((), ())>| {
                    // dir_entry.i
                    dir_entry
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

#[cfg(any(test, feature = "integration-tests"))]
pub mod tests {
    pub mod common;
    pub use common::*;
}

pub mod db;
pub use db::*;
