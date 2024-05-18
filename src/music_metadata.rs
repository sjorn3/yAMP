use music_cache_derive::derive_data_model;
use std::path::Path;

#[derive_data_model]
pub struct Song {
    pub tags: SongTags,
    // converting a path to a utf8 string might not be valid and there's no Archive instance for PathBuf so just store it as bytes.
    pub relpath: Vec<u8>,
}

impl Song {
    pub fn new(tags: SongTags, relpath: &[u8]) -> Song {
        Song {
            tags,
            relpath: Vec::from(relpath),
        }
    }
}

#[derive_data_model]
pub struct SongTags {
    pub title: Option<String>,
    pub track_number: Option<u16>,
}

#[cfg_attr(feature = "integration-tests", derive(Debug, PartialEq, Eq, Clone))]
pub struct Album {
    pub tags: AlbumTags,
    pub songs: Vec<Song>,
}

#[derive_data_model]
pub struct AlbumTags {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
}

pub type AudioTag = Box<dyn audiotags::AudioTag + Send + Sync>;

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
