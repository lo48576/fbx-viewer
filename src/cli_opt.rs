//! CLI options.

use std::path::PathBuf;

use clap::Parser;

/// CLI options.
#[derive(Debug, Parser)]
pub struct CliOpt {
    /// FBX file
    pub fbx_path: PathBuf,
}
