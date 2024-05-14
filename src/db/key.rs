use std::mem;

use crate::{Album, AlbumTags, Song};

#[repr(u8)]
#[cfg_attr(
    any(test, feature = "integration-tests"),
    derive(zerocopy::AsBytes, zerocopy::Unaligned)
)]
// Add new fields at the end to not break existing keys.
pub enum KeyType {
    Song,
    Album,
    AlbumTags,
}

pub trait Taggable {
    fn tag(&self) -> KeyType;
}

impl Taggable for Song {
    fn tag(&self) -> KeyType {
        KeyType::Song
    }
}

impl Taggable for AlbumTags {
    fn tag(&self) -> KeyType {
        KeyType::AlbumTags
    }
}

impl Taggable for Album {
    fn tag(&self) -> KeyType {
        KeyType::Album
    }
}

#[repr(packed)]
#[cfg_attr(
    feature = "integration-tests",
    // Get compile time errors if these key types can't be safely converted to bytes whilst developing, but remove the extra dependency for release.
    // Therefore use .as_ref() not .to_bytes().
    derive(zerocopy::AsBytes, zerocopy::Unaligned)
)]
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

pub trait KeyDBHelpers<E> {
    fn generate_key(&self, value: &dyn Taggable) -> Result<Key, E>;
}

impl KeyDBHelpers<Box<dyn std::error::Error>> for sled::Db {
    fn generate_key(&self, value: &dyn Taggable) -> Result<Key, Box<dyn std::error::Error>> {
        Ok(Key {
            _tag: value.tag(),
            _id: self.generate_id()?.to_le(), // Little Endian is default for most systems, but should improve portability if we are explicit.
        })
    }
}
