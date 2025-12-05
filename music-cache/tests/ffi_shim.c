#include "music_cache.h"

#include <stdint.h>
#include <string.h>

bool ffi_open_db_round_trip(const char *path) {
  db *handle = NULL;
  if (!open_db(path, &handle) || handle == NULL) {
    return false;
  }

  close_db(handle);
  return true;
}

bool ffi_open_db_rejects_null_path(void) {
  db *handle = NULL;
  return !open_db(NULL, &handle) && handle == NULL;
}

bool ffi_expect_album_tags(db *db, const Key *album_key,
                           const AlbumTags *expected) {
  if (db == NULL || album_key == NULL || expected == NULL) {
    return false;
  }

  AlbumTags tags = {0};

  bool result = album_tags_for_key(db, album_key, &tags);

  result &= (expected->artist == NULL && tags.artist == NULL) ||
            (expected->artist != NULL && tags.artist != NULL &&
             strcmp(tags.artist, expected->artist) == 0);

  result &= (expected->title == NULL && tags.title == NULL) ||
            (expected->title != NULL && tags.title != NULL &&
             strcmp(tags.title, expected->title) == 0);

  result &=
      (!expected->has_year && !tags.has_year) ||
      (expected->has_year && tags.has_year && tags.year == expected->year);

  free_album_tags(&tags);

  result &= tags.artist == NULL && tags.title == NULL;

  return result;
}

bool ffi_expect_song_tags(const Song *song, const Song *expected) {
  if (song == NULL || expected == NULL) {
    return false;
  }

  bool result = (expected->tags.title == NULL && song->tags.title == NULL) ||
                (expected->tags.title != NULL && song->tags.title != NULL &&
                 strcmp(song->tags.title, expected->tags.title) == 0);

  result &=
      (!expected->tags.has_track_number && !song->tags.has_track_number) ||
      (expected->tags.has_track_number && song->tags.has_track_number &&
       song->tags.track_number == expected->tags.track_number);

  result &= (expected->relpath == NULL && song->relpath == NULL) ||
            (expected->relpath != NULL && song->relpath != NULL &&
             strcmp(song->relpath, expected->relpath) == 0);

  return result;
}

bool ffi_expect_album(db *db, const Key *album_key, const Album *expected) {
  if (db == NULL || album_key == NULL || expected == NULL) {
    return false;
  }

  Album album = {0};
  bool result = album_for_key(db, album_key, &album);

  result &= ffi_expect_album_tags(db, album_key, &expected->tags);

  result &= expected->song_count == album.song_count;

  if (expected->song_count > 0 && album.song_count > 0) {
    if (expected->songs == NULL || album.songs == NULL) {
      result = false;
    } else {
      size_t songs_to_check = expected->song_count < album.song_count
                                  ? expected->song_count
                                  : album.song_count;
      for (size_t i = 0; i < songs_to_check; ++i) {
        const Song *expected_song = &expected->songs[i];
        const Song *song = &album.songs[i];

        result &= ffi_expect_song_tags(song, expected_song);
      }
    }
  } else {
    result &= expected->songs == NULL || expected->song_count == 0;
    result &= album.songs == NULL || album.song_count == 0;
  }

  free_album(&album);

  return result;
}
