#![allow(clippy::missing_errors_doc)]

use audiotags::Tag;
use std::path::{Path, PathBuf};

pub struct Song {
    // I don't think this will be resiliant to moving the library dir.
    // The real solution here is to have the user specify library location
    // and then store the relative path.
    pub filepath: PathBuf,
    pub song_tags: SongTags,
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
pub struct SongTags {
    pub title: Option<String>,
    pub track_number: Option<u16>,
    pub album: Album,
}

#[cfg_attr(
    feature = "integration-tests",
    derive(Debug, fake::Dummy, PartialEq, Eq)
)]
pub struct Album {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
}

impl Song {
    pub fn from_path(path: &Path) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let tag = Tag::new().read_from_path(path)?;
        Ok(Self {
            filepath: path.to_path_buf(),
            song_tags: SongTags {
                title: tag.title().map(ToString::to_string),
                track_number: tag.track_number(),
                album: Album {
                    artist: tag.album_artist().map(ToString::to_string),
                    title: tag.album_title().map(ToString::to_string),
                    year: tag.year().and_then(|y| y.try_into().ok()),
                },
            },
        })
    }
}

#[cfg(any(test, feature = "integration-tests"))]
pub mod tests {
    pub mod common;
    pub use common::*;
}
