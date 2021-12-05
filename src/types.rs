use clap::ArgEnum;
use serde::Deserialize;
use std::io;

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

#[derive(ArgEnum, Clone, Debug, Copy, PartialEq, Deserialize)]
pub enum DataModelType {
    DiscretesInput,
    Coils,
    DiscretesInputOrCoils,
    InputRegister,
    HoldingRegister,
    HoldingOrInputRegister,
    AllType,
}

#[derive(ArgEnum, Clone, Debug, Copy, PartialEq, Deserialize)]
pub enum DataAccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub enum ModbusRequestReturnType {
    ResultWithU16Vec(Result<Vec<u16>, io::Error>),
    ResultWithNothing(Result<(), io::Error>),
}

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

/* The tokio-modbus crate doesn't make the exception code public
 * hence the definitions below
 */
#[derive(Debug)]
pub enum ModbusExceptionCode {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
}

