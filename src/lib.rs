#![allow(clippy::missing_errors_doc)]

use audiotags::Tag;
use rkyv::{Archive, Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub struct Song<'a> {
    // I don't think this will be resiliant to moving the library dir.
    // The real solution here is to have the user specify library location
    // and then store the relative path.
    pub filepath: PathBuf,
    pub tags: SongTags,
    pub album: &'a Album,
}

// There are many tags on a music file, but we only care about:
//   - Track Title
//   - Album Artist
//   - Year
//   - Track Number
//   - Album Title

// Album Art can come later for now.
#[cfg_attr(
    feature = "integration-tests",
    derive(Debug, fake::Dummy, PartialEq, Eq)
)]
#[derive(Archive, Serialize, Deserialize)]
pub struct SongTags {
    pub title: Option<String>,
    pub track_number: Option<u16>,
}

#[cfg_attr(
    feature = "integration-tests",
    derive(Debug, fake::Dummy, PartialEq, Eq)
)]
#[derive(Archive, Serialize, Deserialize)]
pub struct Album {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
}

type AudioTag = Box<dyn audiotags::AudioTag + Send + Sync>;

impl Album {
    pub fn read(tag: &AudioTag) -> Album {
        Album {
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

impl<'a> Song<'a> {
    pub fn from_path(
        path: &Path,
        album: &'a Album,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let tag = Tag::new().read_from_path(path)?;
        Ok(Self {
            filepath: path.to_path_buf(),
            tags: SongTags::read(&tag),
            album,
        })
    }
}

#[cfg(any(test, feature = "integration-tests"))]
pub mod tests {
    pub mod common;
    pub use common::*;
}
