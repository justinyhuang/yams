use clap::{Parser, ArgEnum};
use serde::Deserialize;
use anyhow::{self, Context};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    fmt::Write as FmtWrite,
    fs};
use crate::lib::core::util::*;
use crate::lib::core::types::*;

#[derive(ArgEnum, Clone, Copy, PartialEq, Debug, Deserialize)]
pub enum FunctionCode {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    ReadExceptionStatus = 0x07,
    Diagnostics = 0x08,
    GetCommeventCounter = 0x0B,
    GetcommEventLog = 0x0C,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10
}

impl FunctionCode {
    pub fn get_exception_code(&self) -> u8
    {
        *self as u8 + 0x80
    }
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
    pub new_values: Option<Vec<String>>,
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

/* The tokio-modbus crate doesn't make the exception code public
 * hence the definitions below
 */
#[derive(Debug)]
pub enum ModbusExceptionCode {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModbusRegisterData {
    /// the data description
    pub data_description: String,
    /// data model type
    pub data_model_type: DataModelType,
    /// data access type
    pub data_access_type: Option<DataAccessType>,
    /// data type
    pub data_type: DataType,
    /// data value in a string
    pub data_value: String,
}

impl ModbusRegisterData {
    pub fn is_function_code_supported(&self, function_code: FunctionCode) -> bool
    {
        let access_type = self.data_access_type.or_else(|| Some(DataAccessType::ReadWrite)).unwrap();
        match (self.data_model_type, access_type) {
            // DiscretesInput =>
            // Coils =>
            // DiscretesInputOrCoils =>
            (DataModelType::InputRegister, DataAccessType::ReadOnly) =>
                if function_code == FunctionCode::ReadInputRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::InputRegister, DataAccessType::WriteOnly) =>
                if function_code == FunctionCode::WriteMultipleRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::InputRegister, DataAccessType::ReadWrite) =>
                if function_code == FunctionCode::WriteMultipleRegisters ||
                   function_code == FunctionCode::ReadInputRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::HoldingRegister, DataAccessType::ReadOnly) =>
                if function_code == FunctionCode::ReadHoldingRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::HoldingRegister, DataAccessType::WriteOnly) =>
                if function_code == FunctionCode::WriteMultipleRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::HoldingRegister, DataAccessType::ReadWrite) =>
                if function_code == FunctionCode::WriteMultipleRegisters ||
                   function_code == FunctionCode::ReadHoldingRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::HoldingOrInputRegister, DataAccessType::ReadOnly) =>
                if function_code == FunctionCode::ReadHoldingRegisters ||
                   function_code == FunctionCode::ReadInputRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::HoldingOrInputRegister, DataAccessType::WriteOnly) =>
                if function_code == FunctionCode::WriteMultipleRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::HoldingOrInputRegister, DataAccessType::ReadWrite) =>
                if function_code == FunctionCode::WriteMultipleRegisters ||
                   function_code == FunctionCode::ReadInputRegisters   ||
                   function_code == FunctionCode::ReadHoldingRegisters {
                    true
                } else {
                    false
                },
            (DataModelType::AllType, DataAccessType::ReadWrite) =>
                true,
            _ => false,
        }
    }

    pub fn write_to_be_u16(&self, registers: &mut Vec<u16>) -> usize
    {
        match &self.data_type {
            DataType::Float32 => {
                if let Ok(value) = self.data_value.parse::<f32>() {
                    let tmp = write_be_f32_into_u16(value);
                    registers.extend(tmp);
                    return 2;
                }
            }
            DataType::Float64 => {
                if let Ok(value) = self.data_value.parse::<f64>() {
                    let tmp = write_be_f64_into_u16(value);
                    registers.extend(tmp);
                    return 4;
                }
            }
            _ => todo!()
        }
        return 0;
    }

    pub fn read_from_be_u16(&mut self, it: &mut std::iter::Peekable<std::slice::Iter<u16>>) -> usize
    {
        match &self.data_type {
            DataType::Float32 => {
                let mut tmp = [0_u16; 2];
                for idx in 0..2 {
                    if let Some(data) = it.next() {
                        tmp[idx] = *data;
                    } else {
                        return 0;
                    }
                }
                self.data_value = write_be_u16_into_f32(&tmp).to_string();
                return 2;
            }
            DataType::Float64 => {
                    return 4;
            }
            _ => todo!()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModbusRegisterDatabase {
    db: HashMap<u16, ModbusRegisterData>,
}

impl ModbusRegisterDatabase {
    pub fn update_u16_registers(&mut self, register_addr: u16, values: Vec<u16>, function_code: FunctionCode) -> anyhow::Result<usize, ModbusExceptionCode>
    {
        let mut value_it = values.iter().peekable();
        let mut total_updated = 0_usize;
        let mut addr = register_addr;
        while let Some(data) = self.db.get_mut(&addr) {
            if data.is_function_code_supported(function_code) {
                let registers_updated = data.read_from_be_u16(&mut value_it);
                if registers_updated != 0 {
                    addr += registers_updated as u16;
                    total_updated += registers_updated;
                } else {
                    return Err(ModbusExceptionCode::IllegalDataValue);
                }
                if value_it.peek().is_none() {
                    return Ok(total_updated);
                }
            } else {
                return Err(ModbusExceptionCode::IllegalFunction);
            }
        }
        return Err(ModbusExceptionCode::IllegalDataAddress);
    }

    pub fn request_u16_registers(&self, register_addr: u16, registers_to_write: u16, function_code: FunctionCode) -> anyhow::Result<Vec<u16>, ModbusExceptionCode>
    {
        let mut registers = Vec::<u16>::new();
        let mut count = registers_to_write as usize;
        let mut addr = register_addr;
        let mut printout = String::new();
        while let Some(data) = self.db.get(&addr) {
            if data.is_function_code_supported(function_code) {
                let registers_written = data.write_to_be_u16(&mut registers);
                writeln!(&mut printout, "{}", data.data_description).unwrap();
                writeln!(&mut printout, "{} ===>", data.data_value).unwrap();
                if count >= registers_written {
                    count -= registers_written;
                    addr += registers_written as u16;
                    if count == 0 {
                        println!("{}", printout);
                        return Ok(registers);
                    }
                } else {
                    return Err(ModbusExceptionCode::IllegalDataValue);
                }
            } else {
                return Err(ModbusExceptionCode::IllegalFunction);
            }
        }
        return Err(ModbusExceptionCode::IllegalDataAddress);
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

fn parse_config_str(config_str: &str) -> anyhow::Result<ModbusDeviceConfig>
{
    serde_yaml::from_str(&config_str)
        .with_context(|| format!("failed to parse the config string"))
}

pub fn configure(opts: &Opts) -> anyhow::Result<ModbusDeviceConfig>
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
