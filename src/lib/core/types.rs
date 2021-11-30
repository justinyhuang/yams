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

