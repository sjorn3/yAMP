use music_cache_derive::derive_data_model;

use crate::{Album, AlbumTags, ByteKey, Key, KeyDBHelpers, KeyType, Result, Song};

pub trait Methods<T> {
    fn insert_metadata(&self, item: &T) -> Result<Key>;

    fn get_metadata(&self, key: &Key) -> Result<T>;

    fn scan(&self) -> Result<Vec<T>>;
}

impl Methods<AlbumTags> for sled::Db {
    fn insert_metadata(&self, album_tags: &AlbumTags) -> Result<Key> {
        let key = self.generate_key(album_tags).unwrap();
        let bytes = rkyv::to_bytes::<AlbumTags, 1024>(album_tags)?;
        self.insert(&key, bytes.as_slice())?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<AlbumTags> {
        let bytes = self.get(key)?.unwrap();
        Ok(unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? })
    }

    fn scan(&self) -> Result<Vec<AlbumTags>> {
        self.scan_prefix(KeyType::AlbumTags)
            .map(|album_tag| {
                album_tag
                    .map_err(|e| e.into())
                    .and_then(|(_, value)| unsafe {
                        rkyv::from_bytes_unchecked::<AlbumTags>(&value.to_vec())
                            .map_err(|e| e.into())
                    })
            })
            .collect::<Result<Vec<AlbumTags>>>()
    }
}

impl Methods<Song> for sled::Db {
    fn insert_metadata(&self, song: &Song) -> Result<Key> {
        let key = self.generate_key(song).unwrap();
        let bytes = rkyv::to_bytes::<Song, 1024>(song)?;
        self.insert(&key, bytes.as_slice())?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<Song> {
        let bytes = self.get(key)?.unwrap();
        Ok(unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? })
    }

    fn scan(&self) -> Result<Vec<Song>> {
        self.scan_prefix(KeyType::Song)
            .map(|album_tag| {
                album_tag
                    .map_err(|e| e.into())
                    .and_then(|(_, value)| unsafe {
                        rkyv::from_bytes_unchecked::<Song>(&value.to_vec()).map_err(|e| e.into())
                    })
            })
            .collect::<Result<Vec<Song>>>()
    }
}

#[derive_data_model]
struct StoredAlbum {
    tags_key: ByteKey,
    song_keys: Vec<ByteKey>,
}

impl Methods<Album> for sled::Db {
    fn insert_metadata(&self, album: &Album) -> Result<Key> {
        let key = self.generate_key(album).unwrap();

        let stored_album = StoredAlbum {
            tags_key: *self.insert_metadata(&album.tags)?.as_ref(),
            song_keys: album
                .songs
                .iter()
                .map(|song| self.insert_metadata(song).map(|key| *key.as_ref()))
                .collect::<Result<Vec<ByteKey>>>()?,
        };
        let bytes = rkyv::to_bytes::<StoredAlbum, 1024>(&stored_album)?;
        self.insert(&key, bytes.as_slice())?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<Album> {
        let bytes = self.get(key)?.unwrap();
        let stored_album: StoredAlbum = unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? };
        Ok(Album {
            tags: self.get_metadata(stored_album.tags_key.as_ref())?,
            songs: stored_album
                .song_keys
                .into_iter()
                .map(|byte_key: ByteKey| self.get_metadata(byte_key.as_ref()))
                .collect::<Result<Vec<Song>>>()?,
        })
    }

    fn scan(&self) -> Result<Vec<Album>> {
        Err("Not implemented".into())
    }
}
