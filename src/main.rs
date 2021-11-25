/* Yet Another Modbus simulator */
use clap::Parser;
mod lib;
use lib::core::config::*;
use lib::server::*;
use lib::client::*;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let config = configure(&opts).expect("failed to load the configuraion");

    if config.common.device_type == DeviceType::Server {
        start_modbus_server(config).await.unwrap();
    } else {
        start_modbus_client(config).await.unwrap();
    }
    Ok(())
}
