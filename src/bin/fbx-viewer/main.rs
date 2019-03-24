//! FBX viewer.

use log::info;

fn main() {
    env_logger::init();
    info!("version: {}", env!("CARGO_PKG_VERSION"));
}
