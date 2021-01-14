//! FBX viewer.

use fbx_viewer::CliOpt;
use log::info;
use structopt::StructOpt;

pub mod vulkan;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    info!("version: {}", env!("CARGO_PKG_VERSION"));

    let opt = CliOpt::from_args();
    vulkan::main(opt).expect("Vulkan mode failed");
}
