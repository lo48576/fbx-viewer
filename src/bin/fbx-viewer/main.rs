//! FBX viewer.

use std::env;

use fbx_viewer::CliOpt;
use log::info;
use structopt::StructOpt;

pub mod vulkan;

fn main() {
    setup_logger();
    info!("version: {}", env!("CARGO_PKG_VERSION"));

    let opt = CliOpt::from_args();
    vulkan::main(opt).expect("Vulkan mode failed");
}

fn setup_logger() {
    /// Default log level.
    const DEFAULT_LOG_LEVEL: &str = "debug";
    /// Envvar name for the logger.
    const LOG_VAR: &str = "RUST_LOG";

    let underscored_name = env!("CARGO_PKG_NAME").replace('-', "_");
    let defval = format!("{}={}", underscored_name, DEFAULT_LOG_LEVEL);

    let newval = match env::var(LOG_VAR) {
        Ok(v) => format!("{},{}", defval, v),
        Err(_) => defval,
    };
    env::set_var(LOG_VAR, &newval);
    env_logger::init();
}
