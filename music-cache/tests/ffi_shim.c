#include "music_cache.h"

#include <stdint.h>
#include <string.h>

bool ffi_expect_album_tags(db *db, const Key *album_key, const char *artist,
                           const char *title, bool expect_year, uint16_t year) {
  AlbumTags tags = {0};

  bool result = album_tags_for_key(db, album_key, &tags);

  result &= (artist == NULL && tags.artist == NULL) ||
            (artist != NULL && tags.artist != NULL &&
             strcmp(tags.artist, artist) == 0);

  result &=
      (title == NULL && tags.title == NULL) ||
      (title != NULL && tags.title != NULL && strcmp(tags.title, title) == 0);

  result &= (!expect_year && !tags.has_year) ||
            (expect_year && tags.has_year && tags.year == year);

  free_album_tags(&tags);

  result &= tags.artist == NULL && tags.title == NULL;

  return result;
}
