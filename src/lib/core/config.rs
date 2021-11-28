use clap::{Parser, ArgEnum};
use serde::Deserialize;
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    fs};
use crate::lib::core::util::*;

#[derive(ArgEnum, Clone, PartialEq, Debug, Deserialize)]
pub enum DeviceType {
    Client,
    Server,
}

#[derive(ArgEnum, Clone, PartialEq, Debug, Deserialize)]
pub enum ProtocolType {
    RTU,
    TCP,
}

#[derive(ArgEnum, Clone, PartialEq, Debug, Deserialize)]
pub enum DataType {
    Float32,
    Float64,
    Uint32,
    Uint64,
    Int32,
    Int64,
}

#[derive(ArgEnum, Clone, PartialEq, Debug, Deserialize)]
pub enum FunctionCode {
    ReadCoils,
    ReadDiscreteInputs,
    ReadHoldingRegisters,
    ReadInputRegisters,
    WriteSingleCoil,
    WriteSingleRegister,
    ReadExceptionStatus,
    Diagnostics
}

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
    pub new_values: Option<Vec<u16>>,
    /// repeat times (0xFFFF to repeat indefinitely)
    pub repeat_times: Option<u16>,
    /// delay before request, in 100 ms
    pub delay: Option<u64>,
    /// type of the data in the request
    pub data_type: DataType,
}

#[derive(Debug, Deserialize)]
pub struct ModbusCommonConfig {
    /// the modbus protocol type
    pub protocol_type: ProtocolType,
    /// the device type (client/server)
    pub device_type: DeviceType,
    /// the serial port when using Modbus RTU
    pub device_port: Option<String>,
    /// the address of the client/server
    pub device_address: Option<u8>,
    /// the socket address when using Modbus TCP
    pub device_ip_address: Option<SocketAddr>,
    /// the baudrate when using Modbus RTU
    pub baudrate: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModbusRegisterData {
    /// the data description
    pub data_description: String,
    /// data type
    pub data_type: DataType,
    /// data value in a string
    pub data_value: String,
}

impl ModbusRegisterData {
    pub fn write_to_be_u16(&self, it: &mut std::slice::IterMut<u16>) -> usize
    {
        let mut registers_written = 0_usize;
        match &self.data_type {
            DataType::Float32 => {
                if let Ok(value) = self.data_value.parse::<f32>() {
                    let mut tmp = [0_u16; 2];
                    write_be_f32_into_u16(value, &mut tmp);
                    for idx in 0..2 {
                        if let Some(register) = it.next() {
                            *register = tmp[idx];
                            registers_written += 1;
                        } else {
                            return registers_written;
                        }
                    }
                    return registers_written;
                }
            }
            DataType::Float64 => {
                if let Ok(value) = self.data_value.parse::<f64>() {
                    let mut tmp = [0_u16; 4];
                    write_be_f64_into_u16(value, &mut tmp);
                    for idx in 0..4 {
                        if let Some(register) = it.next() {
                            *register = tmp[idx];
                            registers_written += 1;
                        } else {
                            return registers_written;
                        }
                    }
                    return registers_written;
                }
            }
            _ => todo!()
        }
        return 0;
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModbusRegisterDatabase {
    db: HashMap<u16, ModbusRegisterData>,
}

impl ModbusRegisterDatabase {
    pub fn write_registers_to_be_u16(&self, register_addr: u16, registers_to_write: u16, registers: &mut Vec<u16> )
    {
        let mut register_it = registers.iter_mut();
        let mut count = registers_to_write as usize;
        let mut addr = register_addr;
        while let Some(data) = self.db.get(&addr) {
            let registers_written = data.write_to_be_u16(&mut register_it);
            println!("{}", data.data_description);
            println!("{} ===>", data.data_value);
            count -= registers_written;
            addr += registers_written as u16;
            if count <= 0 {
                break;
            }
        }
    }
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
    /// the function codes supported by the server
    pub function_codes: Vec<FunctionCode>,
    /// the register database
    pub register_data: ModbusRegisterDatabase,
}

#[derive(Debug, Deserialize)]
pub struct ModbusDeviceConfig {
    /// common configuration for all Modbus devices
    pub common: ModbusCommonConfig,
    /// configuration for Modbus client devices
    pub client: Option<ModbusClientConfig>,
    /// configuration for Modbus server devices
    pub server: Option<ModbusServerConfig>,
}

fn parse_config_str(config_str: &str) -> Result<ModbusDeviceConfig>
{
    serde_yaml::from_str(&config_str)
        .with_context(|| format!("failed to parse the config string"))
}

pub fn configure(opts: &Opts) -> Result<ModbusDeviceConfig>
{
    if let Some(config_file) = &opts.config_file {
        match fs::read_to_string(config_file) {
            Ok(config_str) => parse_config_str(&config_str),
            Err(e) => Err(e.into()),
        }
    }
    else {
        todo!()
    }
}
