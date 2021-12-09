use crate::{data::*, types::*};
use anyhow::{self, Context};
use clap::Parser;
use serde::Deserialize;
use std::{fs, net::SocketAddr, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(version = "0.1", author = "Justin Huang <justin.y.huang@live.com>")]
pub struct Opts {
    /// Sets the simulator type: Client or Server
    #[clap(arg_enum, short, long, required_unless_present("config-file"))]
    device_type: Option<DeviceType>,
    // TODO: complete the list of manual options
    /// Sets the configuration file to load
    #[clap(short, long, required_unless_present_all(&["device-type"]))]
    config_file: Option<String>,

    /// Verbose mode
    #[clap(short, long)]
    pub verbose_mode: bool,
}

#[derive(Debug, Deserialize)]
pub struct ModbusRequest {
    /// description of the request
    pub description: String,
    /// the function code
    pub function_code: FunctionCode,
    /// the start address of the register/coil to access
    pub access_start_address: u16,
    /// the number of registers/coils to access
    pub access_quantity: u16,
    /// the values to write
    pub new_values: Option<Vec<String>>,
    /// repeat times (0xFFFF to repeat indefinitely)
    pub repeat_times: Option<u16>,
    /// delay before request, in 100 ms
    pub delay: Option<u64>,
    /// type of the data in the request
    pub data_type: Option<DataType>,
}

#[derive(Debug, Deserialize)]
pub struct ModbusCommonConfig {
    /// the modbus protocol type
    pub protocol_type: ProtocolType,
    /// the device type (client/server)
    pub device_type: DeviceType,
    /// the serial port when using Modbus RTU
    pub device_port: Option<String>,
    /// the id of the client/server
    pub device_id: u8,
    /// the socket address when using Modbus TCP
    pub device_ip_address: Option<SocketAddr>,
    /// the baudrate when using Modbus RTU
    pub baudrate: Option<u32>,
}

pub const REPEAT_TIME_INDEFINITE: u16 = 0xFFFF;

#[derive(Debug, Deserialize)]
pub struct ModbusClientRequest {
    /// requested server ID for Modbus RTU
    pub server_id: Option<u8>,
    /// requested server address for Modbus TCP
    pub server_address: Option<SocketAddr>,
    /// repeat times (0xFFFF to repeat indefinitely)
    pub repeat_times: Option<u16>,
    /// request files
    pub request_files: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct ModbusClientConfig {
    /// requests send by the client
    pub requests: Vec<ModbusClientRequest>,
    /// the register database
    pub register_data: Option<ModbusRegisterDatabase>,
}

#[derive(Debug, Deserialize)]
pub struct ModbusServerConfig {
    /// the register and coil database
    pub register_data: ModbusRegisterDatabase,
    pub coil_data: ModbusCoilDatabase,
}

impl ModbusServerConfig {
    pub fn get_db(self) -> (ModbusRegisterDatabase, ModbusCoilDatabase) {
        (self.register_data, self.coil_data)
    }
}

#[derive(Debug, Deserialize)]
pub struct ModbusDeviceConfig {
    /// common configuration for all Modbus devices
    pub common: ModbusCommonConfig,
    /// configuration for Modbus client devices
    pub client: Option<ModbusClientConfig>,
    /// configuration for Modbus server devices
    pub server: Option<ModbusServerConfig>,
    /* internal application options */
    #[serde(default)]
    pub verbose_mode: bool,
}

fn parse_config_str(config_str: &str) -> anyhow::Result<ModbusDeviceConfig> {
    serde_yaml::from_str(&config_str).with_context(|| format!("failed to parse the config string"))
}

pub fn configure(opts: &Opts) -> anyhow::Result<ModbusDeviceConfig> {
    if let Some(config_file) = &opts.config_file {
        match fs::read_to_string(config_file) {
            Ok(config_str) => parse_config_str(&config_str),
            Err(e) => Err(e.into()),
        }
    } else {
        todo!()
    }
}
