/* Yet Another Modbus Simulator */
mod config;
mod types;
mod data;
mod util;
mod client;
mod server;
use clap::Parser;

use crate::{
    types::*,
    config::*,
    client::*,
    server::*,
};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let mut config = configure(&opts).expect("failed to load the configuraion");

    config.verbose_mode = opts.verbose_mode;
    if config.common.device_type == DeviceType::Server {
        if let Err(e) = start_modbus_server(config).await {
            println!("exit with error: {}", e);
        }
    } else {
        if let Err(e) = start_modbus_client(config).await {
            println!("exit with error: {}", e);
        }
    }
    Ok(())
}
