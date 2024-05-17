use std::{
    hash::{DefaultHasher, Hasher},
    mem,
};

use music_cache_derive::{derive_data_model, taggable};
use zerocopy::AsBytes;

use crate::{Album, AlbumTags, Result, Song};

#[repr(u8)]
#[taggable(Song, Album, AlbumTags)]
#[derive_data_model]
// Variant names should exactly match types they are keys for.
pub enum KeyType {
    Song,
    Album,
    LastScanTime,
}

#[repr(packed)]
pub struct Key {
    _tag: KeyType,
    _id: u64,
}

pub type ByteKey = [u8; mem::size_of::<Key>()];

impl Key {
    pub fn to_byte_key(&self) -> &ByteKey {
        unsafe { std::mem::transmute(self) }
    }

    pub fn from_byte_key(byte_key: &ByteKey) -> &Key {
        unsafe { std::mem::transmute(byte_key) }
    }
}

impl AsRef<Key> for ByteKey {
    fn as_ref(&self) -> &Key {
        unsafe { &*(self as *const Self as *const Key) }
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self as *const Self as *const u8, mem::size_of::<Self>())
        }
    }
}

impl AsRef<Key> for [u8] {
    fn as_ref(&self) -> &Key {
        unsafe { &*(self as *const Self as *const Key) }
    }
}

impl AsRef<[u8]> for KeyType {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self as *const Self as *const u8, mem::size_of::<Self>())
        }
    }
}

pub trait KeyDBHelpers {
    fn generate_key(&self, value: &dyn TaggableKeyType) -> Result<Key>;
}

impl KeyDBHelpers for sled::Db {
    fn generate_key(&self, value: &dyn TaggableKeyType) -> Result<Key> {
        Ok(Key {
            _tag: value.tag(),
            _id: self.generate_id()?,
        })
    }
}

pub fn hash_key(key_type: KeyType, hasher: impl Hasher) -> Key {
    let hash = hasher.finish();
    Key {
        _tag: key_type,
        _id: hash,
    }
}

pub trait HashKeyGen {
    fn hash_key(&self) -> Key;
}

pub fn song_hash_key(relpath: &[u8]) -> Key {
    let mut hasher = DefaultHasher::new();
    hasher.write(relpath);
    hash_key(KeyType::Song, hasher)
}

impl HashKeyGen for Song {
    fn hash_key(&self) -> Key {
        song_hash_key(&self.relpath)
    }
}

impl HashKeyGen for AlbumTags {
    fn hash_key(&self) -> Key {
        let mut hasher = DefaultHasher::new();

        hasher.maybe_write(&self.artist);
        hasher.maybe_write(&self.title);
        hasher.maybe_write_u16(&self.year);

        hash_key(KeyType::Album, hasher)
    }
}

trait HashMaybeWrite {
    fn maybe_write(&mut self, val: &Option<String>);
    fn maybe_write_u16(&mut self, val: &Option<u16>);
}

impl HashMaybeWrite for DefaultHasher {
    fn maybe_write(&mut self, value: &Option<String>) {
        if let Some(string) = value {
            self.write(string.as_bytes());
        }
    }

    fn maybe_write_u16(&mut self, val: &Option<u16>) {
        if let Some(val) = val {
            self.write(val.as_bytes());
        }
    }
}
