#![allow(clippy::missing_errors_doc)]

use std::error::Error;

#[cfg(any(test, feature = "integration-tests"))]
pub mod tests {
    pub mod common;
    pub use common::*;
}

pub mod db;
pub use db::*;

pub mod music_metadata;
pub use music_metadata::*;

pub mod ffi;
pub use ffi::*;

pub mod library_scan;
pub use library_scan::*;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub type Lazy<'a, T> = Box<dyn FnOnce() -> Result<T> + 'a>;
