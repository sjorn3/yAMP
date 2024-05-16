use fake::{
    faker::{filesystem::en::FilePath, lorem::en::Sentence, name::en::Name},
    *,
};

use crate::*;

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

impl SongTags {
    pub fn arbitrary() -> Self {
        Self {
            title: Sentence(1..10).fake(),
            track_number: (0..20).fake(),
        }
    }
}

impl Song {
    pub fn arbitrary() -> Self {
        Self {
            tags: SongTags::arbitrary(),
            relpath: FilePath().fake::<String>().into_bytes(),
        }
    }
}

impl AlbumTags {
    pub fn arbitrary() -> Self {
        Self {
            artist: Name().fake(),
            title: Sentence(1..10).fake(),
            year: (1900..3022).fake(),
        }
    }
}

impl Album {
    pub fn arbitrary() -> Self {
        Self {
            tags: AlbumTags::arbitrary(),
            songs: (0..(2..20).fake()).map(|_| Song::arbitrary()).collect(),
        }
    }
}
