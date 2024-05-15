use std::mem;

use music_cache_derive::taggable;

use crate::{Album, AlbumTags, Song};

#[repr(u8)]
#[taggable]
// Variant names should exactly match types they are keys for.
pub enum KeyType {
    Song,
    Album,
    AlbumTags,
}

#[repr(packed)]
pub struct Key {
    // No function is provided to convert back to a key from bytes. This is intentional.
    // Instead you should always be wiring a retrieved byte key directly back into sled to retrieve the next value.
    _tag: KeyType,
    _id: u64,
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self as *const Self as *const u8, mem::size_of::<Self>())
        }
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
    fn generate_key(&self, value: &dyn TaggableKeyType) -> Result<Key, Box<dyn std::error::Error>>;
}

impl KeyDBHelpers for sled::Db {
    fn generate_key(&self, value: &dyn TaggableKeyType) -> Result<Key, Box<dyn std::error::Error>> {
        Ok(Key {
            _tag: value.tag(),
            _id: self.generate_id()?,
        })
    }
}
