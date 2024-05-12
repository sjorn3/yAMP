#![allow(clippy::missing_errors_doc)]

use audiotags::Tag;
use std::path::Path;

pub struct Song {
    pub filepath: String,
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
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u16>,
    pub track_number: Option<u16>,
}

impl Song {
    pub fn from_path(path: &Path) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let tag = Tag::new().read_from_path(path)?;
        Ok(Self {
            filepath: path.to_string_lossy().to_string(),
            song_tags: SongTags {
                title: tag.title().map(ToString::to_string),
                album_artist: tag.album_artist().map(ToString::to_string),
                album: tag.album_title().map(ToString::to_string),
                year: tag.year().and_then(|y| y.try_into().ok()),
                track_number: tag.track_number(),
            },
        })
    }
}

#[cfg(any(test, feature = "integration-tests"))]
pub mod tests {
    pub mod common;
    pub use common::*;
}
