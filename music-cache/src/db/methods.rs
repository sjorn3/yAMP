use music_cache_derive::derive_data_model;
use sled::IVec;
use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use crate::*;

pub trait Methods<T> {
    fn insert_metadata(&self, item: &T) -> Result<Key>;

    fn get_metadata(&self, key: &Key) -> Result<T>;
}

impl Song {
    pub fn serialize(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn deserialize(bytes: IVec) -> Result<Song> {
        Ok(bitcode::decode(bytes.as_ref())?)
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
        Song::deserialize(bytes)
    }
}

#[derive_data_model]
pub struct StoredAlbum {
    tags: AlbumTags,
    pub song_keys: Vec<(Option<u16>, ByteKey)>, // TODO Maybe Benchmark, but albums are small so maintaining sort this way seems best.
}

impl From<StoredAlbum> for IVec {
    fn from(album: StoredAlbum) -> Self {
        album.serialize().into()
    }
}

impl From<&Song> for IVec {
    fn from(song: &Song) -> Self {
        song.serialize().into()
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
        Ok(bitcode::decode(bytes)?)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bitcode::encode(self)
    }
}

fn deserialize_album(tree: &sled::Db, bytes: &[u8]) -> Result<Album> {
    let stored_album = StoredAlbum::partial_deserialize_album(bytes)?;
    Ok(Album {
        tags: stored_album.tags,
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

pub type AlbumKeyBySongKey = HashMap<Key, Key>;

impl Methods<AlbumTags> for sled::Db {
    fn insert_metadata(&self, album_tags: &AlbumTags) -> Result<Key> {
        // Not recommended to use this method.
        let empty_album = Album {
            tags: album_tags.clone(),
            songs: Vec::new(),
        };
        self.insert_metadata(&empty_album)
    }

    fn get_metadata(&self, key: &Key) -> Result<AlbumTags> {
        let bytes = self.get(key)?.ok_or("Could not find album key in db")?;
        Ok(StoredAlbum::partial_deserialize_album(bytes.as_ref())?.tags)
    }
}

pub fn scan_stored_albums(tree: &sled::Db) -> Result<AlbumKeyBySongKey> {
    tree.scan_prefix(KeyType::Album)
        .flat_map(|e| {
            e.map(|(album_key, bytes)| {
                StoredAlbum::partial_deserialize_album(bytes.as_ref()).map(|album| {
                    let album_key: &Key = album_key.into();
                    (album_key.clone(), album)
                })
            })
        })
        .try_fold(AlbumKeyBySongKey::new(), |mut map, value| {
            value.map(|(key, stored_album)| {
                for (_, song_key) in stored_album.song_keys {
                    map.insert(Key::from_byte_key_owned(song_key), key.clone());
                }
                map
            })
        })
}

pub trait Helpers {
    fn scan_albums(&self) -> impl Iterator<Item = Result<Album>>;
    fn scan_songs(&self) -> impl Iterator<Item = Result<Song>>;
    fn scan_album_tags_sorted(&self) -> Result<Vec<(Key, AlbumTags)>>;
    fn get_song_from_path(&self, relpath: &[u8]) -> Result<Lazy<'_, Song>>;
    fn set_last_scan_time(&self) -> Result<()>;
    fn get_last_scan_time(&self) -> Result<SystemTime>;
    fn scan_album_song_keys(&self) -> Result<HashSet<(Key, Key)>>;
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
            bytes
                .map_err(|e| e.into())
                .and_then(|(_, bytes)| Song::deserialize(bytes))
        })
    }

    fn scan_album_tags_sorted(&self) -> Result<Vec<(Key, AlbumTags)>> {
        let mut albums: Vec<(Key, AlbumTags)> = self
            .scan_prefix(KeyType::Album)
            .map(|entry| {
                entry.map_err(|e| e.into()).and_then(|(album_key, bytes)| {
                    let album_key: &Key = (&album_key).into();
                    let tags = StoredAlbum::partial_deserialize_album(bytes.as_ref())?.tags;
                    Ok((Key::from_byte_key_owned(*album_key.to_byte_key()), tags))
                })
            })
            .collect::<Result<_>>()?;

        albums.sort_by(|(key_a, tags_a), (key_b, tags_b)| {
            tags_a
                .artist
                .as_ref()
                .cmp(&tags_b.artist.as_ref())
                .then_with(|| tags_a.year.cmp(&tags_b.year))
                .then_with(|| key_a.to_byte_key().cmp(key_b.to_byte_key()))
        });

        Ok(albums)
    }

    // get all album_key, song_key pairs in a hash set
    fn scan_album_song_keys(&self) -> Result<HashSet<(Key, Key)>> {
        // scan_stored_albums(&self).collect()
        unimplemented!()
    }

    fn get_song_from_path(&self, relpath: &[u8]) -> Result<Lazy<'_, Song>> {
        let key = song_hash_key(relpath);
        Ok(Box::new(move || self.get_metadata(&key)))
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
            .update_and_fetch(KeyType::LastScanTime, |maybe_bytes| match maybe_bytes {
                Some(bytes) => Some(bytes.into()),
                None => Some(
                    (unsafe {
                        std::mem::transmute::<SystemTime, [u8; std::mem::size_of::<SystemTime>()]>(
                            SystemTime::UNIX_EPOCH,
                        )
                    })
                    .to_vec(),
                ),
            })?
            .ok_or("Impossible; Could not find last scan time in db")?
            .to_vec();
        Ok(unsafe { *(bytes.as_ptr() as *const SystemTime) })
    }
}
