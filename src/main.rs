/* Yet Another Modbus Simulator */
use clap::Parser;
use yams_core::{
    config::*,
    types::*};
use yams_server::*;
use yams_client::*;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let config = configure(&opts).expect("failed to load the configuraion");

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
