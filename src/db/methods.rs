use music_cache_derive::derive_data_model;
use rkyv::{AlignedVec, Deserialize};
use sled::IVec;
use std::time::SystemTime;

use crate::*;

// If I wanted to get rid of rkyv entirely - I could probably insert all the strings in a given struct into the sled db and then just store the keys into packed structs.
// This might need some kind of benching though because it's not obvious which would be faster. I.e. one requires a bunch of lookups and the other requires copy to archive type and the alignment copy.
// I get the feeling that the rkyv approach is probably faster, but using less memory would also be nice.
// Can also get some small savings probably e.g. artists are likely repeated across many albums.

pub trait Methods<T> {
    fn insert_metadata(&self, item: &T) -> Result<Key>;

    fn get_metadata(&self, key: &Key) -> Result<T>;
}

impl Song {
    pub fn serialize(&self) -> Result<AlignedVec> {
        Ok(rkyv::to_bytes::<Song, 1024>(self)?)
    }
}

impl Methods<Song> for sled::Db {
    fn insert_metadata(&self, song: &Song) -> Result<Key> {
        let key = song.hash_key();
        self.insert(&key, song)?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<Song> {
        let bytes = self.get(key)?.ok_or("Could not find song tags key in db")?;
        Ok(unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? })
    }
}

#[derive_data_model]
pub struct StoredAlbum {
    tags: AlbumTags,
    pub song_keys: Vec<(Option<u16>, ByteKey)>, // TODO Maybe Benchmark, but albums are small so maintaining sort this way seems best.
}

impl From<StoredAlbum> for IVec {
    fn from(album: StoredAlbum) -> Self {
        sled::IVec::from(album.serialize().unwrap().as_ref())
    }
}

impl From<&Song> for IVec {
    fn from(song: &Song) -> Self {
        sled::IVec::from(song.serialize().unwrap().as_ref())
    }
}

impl StoredAlbum {
    pub fn new(tags: AlbumTags, first_track: (Option<u16>, ByteKey)) -> Self {
        Self {
            tags,
            song_keys: vec![first_track],
        }
    }

    pub fn partial_deserialize_album(bytes: &[u8]) -> Result<StoredAlbum> {
        let aligned = &bytes.to_vec();
        Ok(unsafe { rkyv::from_bytes_unchecked(aligned)? })
    }

    pub fn serialize(&self) -> Result<AlignedVec> {
        Ok(rkyv::to_bytes::<StoredAlbum, 1024>(self)?)
    }
}

fn deserialize_album(tree: &sled::Db, bytes: &[u8]) -> Result<Album> {
    let aligned = &bytes.to_vec();
    let stored_album = unsafe { rkyv::archived_root::<StoredAlbum>(aligned) };
    Ok(Album {
        tags: stored_album.tags.deserialize(&mut rkyv::Infallible)?,
        songs: stored_album
            .song_keys
            .iter()
            .map(|(_, byte_key)| tree.get_metadata(byte_key.as_ref()))
            .collect::<Result<Vec<Song>>>()?,
    })
}

impl Methods<Album> for sled::Db {
    fn insert_metadata(&self, album: &Album) -> Result<Key> {
        let key = album.tags.hash_key();

        let stored_album = StoredAlbum {
            tags: album.tags.clone(),
            song_keys: album
                .songs
                .iter()
                .map(|song| {
                    self.insert_metadata(song)
                        .map(|key| (song.tags.track_number, *key.to_byte_key()))
                })
                .collect::<Result<Vec<(Option<u16>, ByteKey)>>>()?,
        };
        self.insert(&key, stored_album)?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<Album> {
        let bytes = self.get(key)?.ok_or("Could not find album key in db")?;
        deserialize_album(self, bytes.as_ref())
    }
}

pub trait Helpers {
    fn scan_albums(&self) -> impl Iterator<Item = Result<Album>>;
    fn scan_songs(&self) -> impl Iterator<Item = Result<Song>>;
    fn get_song_from_path(&self, relpath: &[u8]) -> Result<Lazy<'_, Song>>;
    fn set_last_scan_time(&self) -> Result<()>;
    fn get_last_scan_time(&self) -> Result<SystemTime>;
}

impl Helpers for sled::Db {
    fn scan_albums(&self) -> impl Iterator<Item = Result<Album>> {
        self.scan_prefix(KeyType::Album).map(|album_tag| {
            album_tag
                .map_err(|e| e.into())
                .and_then(|(_, bytes)| deserialize_album(self, bytes.as_ref()))
        })
    }

    fn scan_songs(&self) -> impl Iterator<Item = Result<Song>> {
        self.scan_prefix(KeyType::Song).map(|bytes| {
            bytes.map_err(|e| e.into()).and_then(|(_, value)| unsafe {
                rkyv::from_bytes_unchecked::<Song>(&value.to_vec()).map_err(|e| e.into())
            })
        })
    }

    fn get_song_from_path(&self, relpath: &[u8]) -> Result<Lazy<'_, Song>> {
        let key = song_hash_key(relpath);
        Ok(Box::new(move || {
            let bytes = self.get(key)?.ok_or("Could not find song tags key in db")?;
            Ok(unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? })
        }))
    }

    fn set_last_scan_time(&self) -> Result<()> {
        let last_scan_time = SystemTime::now();
        let bytes: [u8; std::mem::size_of::<SystemTime>()] =
            unsafe { std::mem::transmute(last_scan_time) };
        self.insert(KeyType::LastScanTime, &bytes)?;
        Ok(())
    }

    fn get_last_scan_time(&self) -> Result<SystemTime> {
        let bytes = self
            .get(KeyType::LastScanTime)?
            .ok_or("Could not find last scan time in db")?
            .to_vec();
        Ok(unsafe { *(bytes.as_ptr() as *const SystemTime) })
    }
}
