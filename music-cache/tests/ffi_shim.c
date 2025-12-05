#include "music_cache.h"

#include <stdint.h>
#include <string.h>

bool ffi_expect_album_tags(db *db, const Key *album_key, const char *artist,
                          const char *title, uint16_t year) {
    AlbumTags tags = {0};

    bool result = album_tags_for_key(db, album_key, &tags);
    if (!result) {
        return result;
    }

    if (artist == NULL || tags.artist == NULL || strcmp(tags.artist, artist) != 0) {
        result = false;
    }
    else if (title == NULL || tags.title == NULL || strcmp(tags.title, title) != 0) {
        result = false;
    }
    else if (!tags.has_year || tags.year != year) {
        result = false;
    }

    free_album_tags(&tags);

    if (tags.artist != NULL || tags.title != NULL) {
        result = false;
    }

    return result;
}
