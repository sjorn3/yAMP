#ifndef MUSIC_CACHE_H
#define MUSIC_CACHE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque sled database handle from Rust.
typedef struct sled_Db sled_Db;

typedef enum KeyType {
    KeyType_Song = 0,
    KeyType_Album = 1,
    KeyType_LastScanTime = 2,
} KeyType;

// Matches Rust's #[repr(packed)] Key layout.
#pragma pack(push, 1)
typedef struct Key {
    KeyType _tag;
    uint64_t _id;
} Key;
#pragma pack(pop)

typedef struct AlbumTags {
    char *artist;
    char *title;
    bool has_year;
    uint16_t year;
} AlbumTags;

// Returns album tags for the provided key. Caller owns returned strings and
// must release them with free_album_tags.
AlbumTags album_tags_for_key(sled_Db *db, const Key *album_key);

// Releases memory allocated inside AlbumTags.
void free_album_tags(AlbumTags *tags);

#ifdef __cplusplus
}
#endif

#endif  // MUSIC_CACHE_H
