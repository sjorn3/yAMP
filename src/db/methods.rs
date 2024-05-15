use music_cache_derive::derive_data_model;

use crate::{Album, AlbumTags, ByteKey, Key, KeyDBHelpers, KeyType, Result, Song};

pub trait Methods<T> {
    fn insert_metadata(&self, item: &T) -> Result<Key>;

    fn get_metadata(&self, key: &Key) -> Result<T>;
}

impl Methods<AlbumTags> for sled::Db {
    fn insert_metadata(&self, album_tags: &AlbumTags) -> Result<Key> {
        let key = self.generate_key(album_tags)?;
        let bytes = rkyv::to_bytes::<AlbumTags, 1024>(album_tags)?;
        self.insert(&key, bytes.as_slice())?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<AlbumTags> {
        let bytes = self
            .get(key)?
            .ok_or("Could not find album tags key in db")?;
        Ok(unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? })
    }
}

impl Methods<Song> for sled::Db {
    fn insert_metadata(&self, song: &Song) -> Result<Key> {
        let key = self.generate_key(song)?;
        let bytes = rkyv::to_bytes::<Song, 1024>(song)?;
        self.insert(&key, bytes.as_slice())?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<Song> {
        let bytes = self.get(key)?.ok_or("Could not find song tags key in db")?;
        Ok(unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? })
    }
}

#[derive_data_model]
pub struct StoredAlbum {
    tags_key: ByteKey,
    song_keys: Vec<ByteKey>,
}

fn retrieve_album(tree: &sled::Db, stored_album: StoredAlbum) -> Result<Album> {
    Ok(Album {
        tags: tree.get_metadata(stored_album.tags_key.as_ref())?,
        songs: stored_album
            .song_keys
            .iter()
            .map(|byte_key: &ByteKey| tree.get_metadata(byte_key.as_ref()))
            .collect::<Result<Vec<Song>>>()?,
    })
}

impl Methods<Album> for sled::Db {
    fn insert_metadata(&self, album: &Album) -> Result<Key> {
        let key = self.generate_key(album)?;

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
        let bytes = self.get(key)?.ok_or("Could not find album key in db")?;
        let stored_album: StoredAlbum = unsafe { rkyv::from_bytes_unchecked(&bytes.to_vec())? };
        retrieve_album(self, stored_album)
    }
}

pub trait Scan {
    fn scan_albums(&self) -> impl Iterator<Item = Result<Album>>;
    fn scan_songs(&self) -> impl Iterator<Item = Result<Song>>;
    fn scan_album_tags(&self) -> impl Iterator<Item = Result<AlbumTags>>;
}

impl Scan for sled::Db {
    fn scan_albums(&self) -> impl Iterator<Item = Result<Album>> {
        self.scan_prefix(KeyType::Album).map(|album_tag| {
            album_tag
                .map_err(|e| e.into())
                .and_then(|(_, value)| unsafe {
                    rkyv::from_bytes_unchecked::<StoredAlbum>(&value.to_vec()).map_err(|e| e.into())
                })
                .and_then(|stored_album| retrieve_album(self, stored_album))
        })
    }

    fn scan_songs(&self) -> impl Iterator<Item = Result<Song>> {
        self.scan_prefix(KeyType::Song).map(|bytes| {
            bytes.map_err(|e| e.into()).and_then(|(_, value)| unsafe {
                rkyv::from_bytes_unchecked::<Song>(&value.to_vec()).map_err(|e| e.into())
            })
        })
    }

    fn scan_album_tags(&self) -> impl Iterator<Item = Result<AlbumTags>> {
        self.scan_prefix(KeyType::AlbumTags).map(|bytes| {
            bytes.map_err(|e| e.into()).and_then(|(_, value)| unsafe {
                rkyv::from_bytes_unchecked::<AlbumTags>(&value.to_vec()).map_err(|e| e.into())
            })
        })
    }
}
