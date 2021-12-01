use serde::Deserialize;
use anyhow;
use std::{
    collections::HashMap,
    fmt::Write as FmtWrite};
use crate::{
    util::*,
    types::*};

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
                        print!("{}", printout);
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

