//! FBX viewer.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::cli_opt::CliOpt;

mod cli_opt;
pub mod data;
pub mod fbx;
