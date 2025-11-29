#include "music_cache.h"

#include <stdint.h>
#include <string.h>

// Returns 0 on success, otherwise an error code indicating what differed.
int ffi_expect_album_tags(sled_Db *db, const Key *album_key, const char *artist,
                          const char *title, uint16_t year) {
    AlbumTags tags = album_tags_for_key(db, album_key);

    int code = 0;

    if (artist == NULL || tags.artist == NULL || strcmp(tags.artist, artist) != 0) {
        code = 1;
    }
    else if (title == NULL || tags.title == NULL || strcmp(tags.title, title) != 0) {
        code = 2;
    }
    else if (!tags.has_year || tags.year != year) {
        code = 3;
    }

    free_album_tags(&tags);
    if (tags.artist != NULL || tags.title != NULL) {
        code = 4;
    }

    return code;
}
