use fake::{
    faker::{lorem::en::Sentence, name::en::Name},
    *,
};

use crate::*;

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub trait Arbitrary {
    fn arbitrary() -> Self;
}

impl Arbitrary for SongTags {
    fn arbitrary() -> Self {
        Self {
            title: Sentence(1..10).fake(),
            track_number: (0..20).fake(),
        }
    }
}

impl Arbitrary for Song {
    fn arbitrary() -> Self {
        Self {
            tags: SongTags::arbitrary(),
            relpath: (0..16).map(|_| Faker.fake::<u8>()).collect(),
        }
    }
}

impl Arbitrary for AlbumTags {
    fn arbitrary() -> Self {
        Self {
            artist: Name().fake(),
            title: Sentence(1..10).fake(),
            year: (1900..3022).fake(),
        }
    }
}

impl Arbitrary for Album {
    fn arbitrary() -> Self {
        Self {
            tags: AlbumTags::arbitrary(),
            songs: (0..(2..20).fake()).map(|_| Song::arbitrary()).collect(),
        }
    }
}
