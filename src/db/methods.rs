use music_cache_derive::derive_data_model;
use rkyv::Deserialize;
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

impl Methods<Song> for sled::Db {
    fn insert_metadata(&self, song: &Song) -> Result<Key> {
        let key = song.hash_key();
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
    tags: AlbumTags,
    song_keys: Vec<ByteKey>,
}

fn retrieve_album(tree: &sled::Db, stored_album: &ArchivedStoredAlbum) -> Result<Album> {
    Ok(Album {
        tags: stored_album.tags.deserialize(&mut rkyv::Infallible)?,
        songs: stored_album
            .song_keys
            .iter()
            .map(|byte_key: &ByteKey| tree.get_metadata(byte_key.as_ref()))
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
                .map(|song| self.insert_metadata(song).map(|key| *key.as_ref()))
                .collect::<Result<Vec<ByteKey>>>()?,
        };
        let bytes = rkyv::to_bytes::<StoredAlbum, 1024>(&stored_album)?;
        self.insert(&key, bytes.as_slice())?;
        Ok(key)
    }

    fn get_metadata(&self, key: &Key) -> Result<Album> {
        let bytes = self.get(key)?.ok_or("Could not find album key in db")?;
        let aligned = &bytes.to_vec();
        let stored_album = unsafe { rkyv::archived_root::<StoredAlbum>(aligned) };
        retrieve_album(self, stored_album)
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
                .and_then(|(_, bytes)| unsafe {
                    let aligned = &bytes.to_vec();
                    let stored_album = rkyv::archived_root::<StoredAlbum>(aligned);
                    retrieve_album(self, stored_album)
                })
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
