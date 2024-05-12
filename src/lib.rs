// There are many tags on a music file, but we only care about:
//   - Track Title
//   - Album Artist
//   - Year
//   - Track Number
//   - Album Title

// Album Art can come later for now.

// use fake::Dummy;
// #[derive(Debug, Clone, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Debug, fake::Dummy))]
pub struct SongTags {
    pub title: String,
    pub album_artist: String,
    pub album: String,
    pub year: u32,
    pub track_number: u32,
}
