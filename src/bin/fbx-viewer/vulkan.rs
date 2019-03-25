//! Vulkan version.

use failure::Fallible;
use log::info;

use self::setup::setup;

mod setup;

pub fn main() -> Fallible<()> {
    info!("Vulkan mode");

    let (_device, _queue) = setup()?;

    Ok(())
}
