//! FBX.

use std::path::Path;

use anyhow::bail;
use fbxcel_dom::any::AnyDocument;

use crate::data::Scene;

mod v7400;

/// Loads FBX data.
pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Scene> {
    load_impl(path.as_ref())
}

/// Loads FBX data.
fn load_impl(path: &Path) -> anyhow::Result<Scene> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    match AnyDocument::from_seekable_reader(file)? {
        AnyDocument::V7400(_ver, doc) => v7400::from_doc(doc),
        _ => bail!("Unknown FBX DOM version"),
    }
}
