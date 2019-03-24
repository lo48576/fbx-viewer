//! FBX viewer.

use log::info;

pub mod vulkan;

fn main() {
    env_logger::init();
    info!("version: {}", env!("CARGO_PKG_VERSION"));

    vulkan::main()
}
