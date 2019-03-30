//! CLI options.

use std::path::PathBuf;

use structopt::StructOpt;

/// CLI options.
#[derive(Debug, StructOpt)]
pub struct CliOpt {
    /// FBX file
    #[structopt(parse(from_os_str))]
    pub fbx_path: PathBuf,
}
