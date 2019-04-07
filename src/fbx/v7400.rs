//! FBX v7400 support.

use failure::Fallible;
use fbxcel_dom::v7400::Document;

use crate::data::Scene;

#[allow(dead_code)]
mod triangulator;

/// Loads the data from the document.
pub fn from_doc(_doc: Box<Document>) -> Fallible<Scene> {
    unimplemented!()
}
