//! FBX viewer.

use clap::Parser;
use fbx_viewer::CliOpt;
use log::info;

pub mod vulkan;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    info!("version: {}", env!("CARGO_PKG_VERSION"));

    let opt = CliOpt::parse();
    vulkan::main(opt).expect("Vulkan mode failed");
}
