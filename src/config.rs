use crate::{data::*, types::*};
use anyhow::{self, Context};
use clap::Parser;
use serde::{Serialize, Deserialize};
use std::{fs, net::SocketAddr, path::PathBuf};
use tokio_serial::{SerialPort, SerialStream};

#[derive(Parser, Debug)]
#[clap(version = "0.8", author = "Justin Huang <justin.y.huang@live.com>")]
pub struct Opts {
    /// Sets the configuration file to load
    #[clap(short, long, required_unless_present_all(&["device-type"]))]
    config_file: Option<String>,

    /// Verbose mode
    #[clap(short, long)]
    pub verbose_mode: bool,
    /// enable external mode
    #[clap(short('x'), long, requires("config-file"))]
    pub external_mode: bool,

    /// the modbus protocol type
    #[clap(arg_enum, short('p'), long, required_unless_present("config-file"))]
    pub protocol_type: Option<ProtocolType>,
    /// the device type (only 'client' is supported in one-shot mode)
    #[clap(arg_enum, short('t'), long, required_unless_present("config-file"))]
    pub device_type: Option<DeviceType>,
    /// the id of the client/server
    #[clap(short('i'), long, required_unless_present("config-file"))]
    pub device_id: Option<u8>,
    /// the socket address when using Modbus TCP
    #[clap(short('a'), long, required_if_eq("protocol-type", "tcp"))]
    pub ip_address: Option<SocketAddr>,
    /// the serial port when using Modbus RTU
    #[clap(short('s'), long, required_if_eq("protocol-type", "rtu"))]
    pub serial_port: Option<String>,
    /// the baudrate when using Modbus RTU
    #[clap(short('b'), long, required_if_eq("protocol-type", "rtu"))]
    pub serial_baudrate: Option<u32>,
    /// the parity of the serial port
    #[clap(arg_enum, short('r'), long, required_if_eq("protocol-type", "rtu"))]
    pub serial_parity: Option<ParityType>,
    /// the stop bits of the serial port
    #[clap(arg_enum, short('o'), long, required_if_eq("protocol-type", "rtu"))]
    pub serial_stop_bits: Option<StopBitsType>,
    /// the data bits of the serial port
    #[clap(arg_enum, short('d'), long, required_if_eq("protocol-type", "rtu"))]
    pub serial_data_bits: Option<DataBitsType>,
    /// the function code to use in one-shot mode
    #[clap(arg_enum, short('f'), long, required_unless_present("config-file"))]
    pub function_code: Option<FunctionCode>,
    /// the start register address to use in one-shot mode
    #[clap(short('e'), long, required_if_eq_any(&[("function-code", "write-single-register"),
                                                  ("function-code", "write-multiple-registers"),
                                                  ("function-code", "write-single-coil"),
                                                  ("function-code", "write-multiple-coils"),
                                                  ("function-code", "read-coils"),
                                                  ("function-code", "read-discrete-inputs"),
                                                  ("function-code", "read-holding-registers"),
                                                  ("function-code", "read-input-registers"),
                                                  ("function-code", "write-multiple-coils")]))]
    pub start_address: Option<u16>,
    /// the quantity-of-registers-to-access to use in one-shot mode
    #[clap(short('q'), long, required_if_eq_any(&[("function-code", "write-single-register"),
                                                  ("function-code", "write-multiple-registers"),
                                                  ("function-code", "write-single-coil"),
                                                  ("function-code", "write-multiple-coils"),
                                                  ("function-code", "read-coils"),
                                                  ("function-code", "read-discrete-inputs"),
                                                  ("function-code", "read-holding-registers"),
                                                  ("function-code", "read-input-registers"),
                                                  ("function-code", "write-multiple-coils")]))]
    pub quantity: Option<u16>,
    /// the new values to set in one-shot mode
    #[clap(short('n'), long, required_if_eq_any(&[("function-code", "write-single-register"),
                                                  ("function-code", "write-multiple-registers"),
                                                  ("function-code", "write-single-coil"),
                                                  ("function-code", "write-multiple-coils")]))]
    pub new_values: Option<Vec<String>>,
    /// the times to repeat the request in one-shot mode
    #[clap(short('g'), long, required_unless_present("config-file"))]
    pub repeat_times: Option<u16>,
    /// the time (number of 100ms) to delay the request in one-shot mode
    #[clap(short('y'), long, required_unless_present("config-file"))]
    pub delay: Option<u64>,
    /// the data type used in one-shot mode
    #[clap(arg_enum, short('j'), long, required_if_eq_any(&[("function-code", "write-single-register"),
                                                            ("function-code", "write-multiple-registers")]))]
    pub data_type: Option<DataType>,
    /// the server id used in one-shot mode
    #[clap(short('k'), long, required_if_eq("device-type", "client"))]
    pub server_id: Option<u8>,
    /// the server address used in one-shot mode
    #[clap(short('l'), long, required_if_eq_any(&[("device-type", "client"),
                                                  ("protocol-type", "tcp")]))]
    pub server_address: Option<SocketAddr>,
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
    /// the id of the client/server
    pub device_id: u8,
    /// the socket address when using Modbus TCP
    pub ip_address: Option<SocketAddr>,
    /// the serial port when using Modbus RTU
    pub serial_port: Option<String>,
    /// the baudrate when using Modbus RTU
    pub serial_baudrate: Option<u32>,
    /// the parity of the serial port
    pub serial_parity: Option<ParityType>,
    /// the stop bits of the serial port
    pub serial_stop_bits: Option<StopBitsType>,
    /// the data bits of the serial port
    pub serial_data_bits: Option<DataBitsType>,
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
    /// a single request
    pub request: Option<ModbusRequest>,
}

#[derive(Debug, Deserialize)]
pub struct ModbusClientConfig {
    /// requests send by the client
    pub requests: Vec<ModbusClientRequest>,
    /// the register database
    pub register_data: Option<ModbusRegisterDatabase>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    #[serde(default)]
    pub external_mode: bool,
}

fn parse_config_str(config_str: &str) -> anyhow::Result<ModbusDeviceConfig> {
    serde_yaml::from_str(&config_str).with_context(|| format!("failed to parse the config string"))
}

pub fn configure(opts: &mut Opts) -> anyhow::Result<ModbusDeviceConfig> {
    if let Some(config_file) = &opts.config_file {
        match fs::read_to_string(config_file) {
            Ok(config_str) => parse_config_str(&config_str),
            Err(e) => Err(e.into()),
        }
    } else {
        if opts.device_type == Some(DeviceType::Client) {
            Ok(ModbusDeviceConfig {
                common: ModbusCommonConfig {
                    protocol_type: opts.protocol_type.unwrap(),
                    device_type: DeviceType::Client,
                    device_id: opts.device_id.unwrap(),
                    ip_address: opts.ip_address,
                    serial_baudrate: opts.serial_baudrate,
                    serial_data_bits: opts.serial_data_bits,
                    serial_stop_bits: opts.serial_stop_bits,
                    serial_parity: opts.serial_parity,
                    serial_port: opts.serial_port.take(),
                },
                server: None,
                client: Some(ModbusClientConfig {
                    requests: vec![ ModbusClientRequest {
                        server_id: opts.server_id,
                        server_address: opts.server_address,
                        repeat_times: None,
                        request_files: vec![],
                        request: Some(ModbusRequest {
                            description: "".to_string(),
                            function_code: opts.function_code.unwrap(),
                            access_start_address: opts.start_address.unwrap(),
                            access_quantity: opts.quantity.unwrap(),
                            new_values: opts.new_values.take(),
                            repeat_times: opts.repeat_times,
                            delay: opts.delay,
                            data_type: opts.data_type,
                        }),
                    }],
                register_data: None,
                }),
                verbose_mode: opts.verbose_mode,
                external_mode: opts.external_mode,
        })
        } else {
            println!("server is not supported in one-shot mode");
            std::process::exit(1);
        }

    }
}

pub fn build_serial(config: &ModbusDeviceConfig) -> Option<SerialStream> {
    let device = config.common.serial_port.as_ref()?;
    let baudrate = config.common.serial_baudrate?;
    let builder = tokio_serial::new(device, baudrate);
    let mut port = SerialStream::open(&builder).unwrap();

    port.set_parity(config.common.serial_parity?.into()).ok();
    port.set_stop_bits(config.common.serial_stop_bits?.into())
        .ok();
    port.set_data_bits(config.common.serial_data_bits?.into())
        .ok();
    Some(port)
}
