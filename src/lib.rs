#![allow(clippy::missing_errors_doc)]

use jwalk::WalkDir;
use std::path::Path;

pub fn walk(dir: &Path) {
    let mut count = 0;
    let walk_dir = WalkDir::new(dir).process_read_dir(|_, _, _, children| {
        children.retain(|dir_entry_result| {
            dir_entry_result
                .as_ref()
                .map(|dir_entry: &jwalk::DirEntry<((), ())>| {
                    // dir_entry.i
                    dir_entry
                        .file_name
                        .to_str()
                        .map(|s| s.ends_with(".mp3"))
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        });
    });

    for entry in walk_dir {
        let entry = entry.unwrap();
        count += 1;
        println!("{}", entry.path().display());
    }
    println!("Count: {}", count);
}

#[cfg(any(test, feature = "integration-tests"))]
pub mod tests {
    pub mod common;
    pub use common::*;
}

pub mod db;
pub use db::*;

pub mod music_metadata;
pub use music_metadata::*;
