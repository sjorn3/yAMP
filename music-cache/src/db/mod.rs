mod key;
pub use key::*;

#[allow(clippy::unnecessary_to_owned)]
// Unfortunately sled doesn't guarantee alignment, so push the value into a vector to ensure it's aligned, adding a copy.
// I could maybe think about forking sled to have alignment. But that's a pretty big undertaking and I don't know what the tradeoffs are.
pub mod methods;
pub use methods::*;
