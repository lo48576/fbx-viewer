//! FBX.

use std::path::Path;

use failure::{bail, Fallible};
use fbxcel_dom::any::AnyDocument;

use crate::data::Scene;

mod v7400;

/// Loads FBX data.
pub fn load(path: impl AsRef<Path>) -> Fallible<Scene> {
    load_impl(path.as_ref())
}

/// Loads FBX data.
fn load_impl(path: &Path) -> Fallible<Scene> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    match AnyDocument::from_seekable_reader(file)? {
        AnyDocument::V7400(doc) => v7400::from_doc(doc),
        _ => bail!("Unknown FBX DOM version"),
    }
}
