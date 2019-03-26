//! Vulkan version.

use failure::Fallible;
use fbx_viewer::{fbx, CliOpt};
use log::info;

use self::setup::setup;

mod setup;

pub fn main(_opt: CliOpt) -> Fallible<()> {
    info!("Vulkan mode");

    let (_device, _queue) = setup()?;

    Ok(())
}
