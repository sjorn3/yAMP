#ifndef MUSIC_CACHE_H
#define MUSIC_CACHE_H

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque database handle from Rust.
typedef struct opaque_Db db;

typedef enum KeyType {
    KeyType_Song = 0,
    KeyType_Album = 1,
    KeyType_LastScanTime = 2,
} KeyType;

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

typedef struct SongTags {
    char *title;
    bool has_track_number;
    uint16_t track_number;
} SongTags;

typedef struct Song {
    SongTags tags;
    char *relpath;
} Song;

typedef struct Album {
    AlbumTags tags;
    Song *songs;
    size_t song_count;
} Album;

typedef struct AlbumTagsWithKey {
    Key key;
    AlbumTags tags;
} AlbumTagsWithKey;

bool open_db(const char *path, db **out);

void close_db(db *db);

bool album_tags_for_key(db *db, const Key *album_key, AlbumTags *out);

bool album_for_key(db *db, const Key *album_key, Album *out);

bool scan_album_tags_sorted(db *db, AlbumTagsWithKey **out, size_t *out_len);

void free_album_tags(AlbumTags *tags);

void free_album(Album *album);

void free_album_tags_sorted(AlbumTagsWithKey *albums, size_t len);

#ifdef __cplusplus
}
#endif

#endif  // MUSIC_CACHE_H
