mod fs_utils;
use fs_utils::SkeletonFileTree;

use fake::{Fake, Faker};
use music_cache::tests::common::*;
use tempfile::*;

#[test]
fn test_gen_file_tree() -> Result {
    let dir = tempdir()?;
    let tree: SkeletonFileTree = Faker.fake();
    tree.generate_file_structure(dir.path())?;
    Ok(())
}
