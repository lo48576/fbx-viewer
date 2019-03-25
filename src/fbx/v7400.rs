//! FBX v7400 support.

use failure::Fallible;
use fbxcel_dom::v7400::Document;

/// Loads the data from the document.
pub fn from_doc(_doc: Box<Document>) -> Fallible<()> {
    unimplemented!()
}
