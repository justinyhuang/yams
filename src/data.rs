use crate::{types::*, util::*};
use anyhow;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Write as FmtWrite};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn is_function_code_supported(&self, function_code: FunctionCode) -> bool {
        let access_type = self
            .data_access_type
            .or_else(|| Some(DataAccessType::ReadWrite))
            .unwrap();
        match (access_type, self.data_model_type) {
            (DataAccessType::ReadOnly, DataModelType::DiscreteInputs) => {
                if function_code == FunctionCode::ReadDiscreteInputs {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::Coils) => {
                if function_code == FunctionCode::ReadCoils {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::DiscreteInputsOrCoils) => {
                if function_code == FunctionCode::ReadCoils
                    || function_code == FunctionCode::ReadDiscreteInputs
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::InputRegister) => {
                if function_code == FunctionCode::ReadInputRegisters {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::HoldingRegister) => {
                if function_code == FunctionCode::ReadHoldingRegisters {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::HoldingOrInputRegister) => {
                if function_code == FunctionCode::ReadHoldingRegisters
                    || function_code == FunctionCode::ReadInputRegisters
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::AllType) => {
                if function_code == FunctionCode::ReadHoldingRegisters
                    || function_code == FunctionCode::ReadInputRegisters
                    || function_code == FunctionCode::ReadDiscreteInputs
                    || function_code == FunctionCode::ReadCoils
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::WriteOnly, DataModelType::DiscreteInputs)
            | (DataAccessType::WriteOnly, DataModelType::Coils)
            | (DataAccessType::WriteOnly, DataModelType::DiscreteInputsOrCoils) => {
                if function_code == FunctionCode::WriteMultipleCoils
                    || function_code == FunctionCode::WriteSingleCoil
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::WriteOnly, DataModelType::InputRegister)
            | (DataAccessType::WriteOnly, DataModelType::HoldingRegister)
            | (DataAccessType::WriteOnly, DataModelType::HoldingOrInputRegister) => {
                if function_code == FunctionCode::WriteMultipleRegisters
                    || function_code == FunctionCode::WriteSingleRegister
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::WriteOnly, DataModelType::AllType) => {
                if function_code == FunctionCode::WriteMultipleRegisters
                    || function_code == FunctionCode::WriteSingleRegister
                    || function_code == FunctionCode::WriteMultipleCoils
                    || function_code == FunctionCode::WriteSingleCoil
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::InputRegister) => {
                if function_code == FunctionCode::WriteMultipleRegisters
                    || function_code == FunctionCode::WriteSingleRegister
                    || function_code == FunctionCode::ReadInputRegisters
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::HoldingRegister) => {
                if function_code == FunctionCode::WriteMultipleRegisters
                    || function_code == FunctionCode::WriteSingleRegister
                    || function_code == FunctionCode::ReadHoldingRegisters
                    || function_code == FunctionCode::ReadWriteMultipleRegisters
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::HoldingOrInputRegister) => {
                if function_code == FunctionCode::WriteMultipleRegisters
                    || function_code == FunctionCode::WriteSingleRegister
                    || function_code == FunctionCode::ReadInputRegisters
                    || function_code == FunctionCode::ReadHoldingRegisters
                    || function_code == FunctionCode::ReadWriteMultipleRegisters
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::AllType) => true,
            _ => false,
        }
    }

    pub fn write_into_u16(&self, registers: &mut Vec<u16>, e: EndiannessType) -> usize {
        match &self.data_type {
            DataType::Float32 => {
                if let Ok(value) = self.data_value.parse::<f32>() {
                    let tmp = write_f32_into_u16(value, e);
                    registers.extend(tmp);
                    return 2;
                }
            }
            DataType::Float64 => {
                if let Ok(value) = self.data_value.parse::<f64>() {
                    let tmp = write_f64_into_u16(value, e);
                    registers.extend(tmp);
                    return 4;
                }
            }
            DataType::Uint32 => {
                if let Ok(value) = parse_int::parse::<u32>(&self.data_value) {
                    let tmp = vec![(value & 0xFFFF) as u16, (value >> 16) as u16];
                    registers.extend(tmp);
                    return 2;
                }
            }
            DataType::Uint16 => {
                if let Ok(value) = parse_int::parse::<u16>(&self.data_value) {
                    registers.push(value);
                    return 1;
                }
            }
            _ => todo!(),
        }
        return 0;
    }

    pub fn read_from_u16(
        &mut self,
        it: &mut std::iter::Peekable<std::slice::Iter<u16>>,
        e: EndiannessType
    ) -> usize {
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
                self.data_value = write_u16_into_f32(&tmp, e).to_string();
                return 2;
            }
            DataType::Float64 => {
                let mut tmp = [0_u16; 4];
                for idx in 0..4 {
                    if let Some(data) = it.next() {
                        tmp[idx] = *data;
                    } else {
                        return 0;
                    }
                }
                self.data_value = write_u16_into_f64(&tmp, e).to_string();
                return 4;
            }
            DataType::Uint32 => {
                let mut tmp = [0_u16; 2];
                for idx in 0..2 {
                    if let Some(data) = it.next() {
                        tmp[idx] = *data;
                    } else {
                        return 0;
                    }
                }
                self.data_value = (tmp[0] as u32 | ((tmp[1] as u32) << 16)).to_string();
                return 2;
            }
            DataType::Uint16 => {
                if let Some(data) = it.next() {
                    self.data_value = data.to_string();
                    return 1;
                } else {
                    return 0;
                }
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusRegisterDatabase {
    db: HashMap<u16, ModbusRegisterData>,
}

impl ModbusRegisterDatabase {
    pub fn update_u16_registers(
        &mut self,
        register_addr: u16,
        values: Vec<u16>,
        function_code: FunctionCode,
        e: EndiannessType,
    ) -> anyhow::Result<usize, ModbusExceptionCode> {
        let mut value_it = values.iter().peekable();
        let mut total_updated = 0_usize;
        let mut addr = register_addr;
        while let Some(data) = self.db.get_mut(&addr) {
            if data.is_function_code_supported(function_code) {
                let registers_updated = data.read_from_u16(&mut value_it, e);
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

    pub fn request_u16_registers(
        &self,
        register_addr: u16,
        registers_to_write: u16,
        function_code: FunctionCode,
        e: EndiannessType,
    ) -> anyhow::Result<Vec<u16>, ModbusExceptionCode> {
        let mut registers = Vec::<u16>::new();
        let mut count = registers_to_write as usize;
        let mut addr = register_addr;
        let mut printout = String::new();
        while let Some(data) = self.db.get(&addr) {
            if data.is_function_code_supported(function_code) {
                let registers_written = data.write_into_u16(&mut registers, e);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependentCoil {
    value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterBitCoil {
    register: u16,
    bit: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModbusCoilDataValueType {
    Independent(IndependentCoil),
    RegisterBit(RegisterBitCoil),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusCoilData {
    /// the data description
    pub data_description: String,
    /// data model type
    pub data_model_type: DataModelType,
    /// data access type
    pub data_access_type: Option<DataAccessType>,
    /// (boolean) data value, or map to a bit of a registers
    pub data_value: ModbusCoilDataValueType,
    /// external program to run after a register is updated
    pub external_program: Option<String>,
}

impl ModbusCoilData {
    pub fn is_function_code_supported(&self, function_code: FunctionCode) -> bool {
        let access_type = self
            .data_access_type
            .or_else(|| Some(DataAccessType::ReadWrite))
            .unwrap();
        match (access_type, self.data_model_type) {
            (DataAccessType::ReadOnly, DataModelType::DiscreteInputs) => {
                if function_code == FunctionCode::ReadDiscreteInputs {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::Coils) => {
                if function_code == FunctionCode::ReadCoils {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::DiscreteInputsOrCoils) => {
                if function_code == FunctionCode::ReadCoils
                    || function_code == FunctionCode::ReadDiscreteInputs
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadOnly, DataModelType::AllType) => {
                if function_code == FunctionCode::ReadDiscreteInputs
                    || function_code == FunctionCode::ReadCoils
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::WriteOnly, DataModelType::DiscreteInputs)
            | (DataAccessType::WriteOnly, DataModelType::Coils)
            | (DataAccessType::WriteOnly, DataModelType::DiscreteInputsOrCoils) => {
                if function_code == FunctionCode::WriteMultipleCoils
                    || function_code == FunctionCode::WriteSingleCoil
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::WriteOnly, DataModelType::AllType) => {
                if function_code == FunctionCode::WriteMultipleCoils
                    || function_code == FunctionCode::WriteSingleCoil
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::Coils) => {
                if function_code == FunctionCode::WriteMultipleCoils
                    || function_code == FunctionCode::WriteSingleCoil
                    || function_code == FunctionCode::ReadCoils
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::DiscreteInputs) => {
                if function_code == FunctionCode::WriteMultipleCoils
                    || function_code == FunctionCode::WriteSingleCoil
                    || function_code == FunctionCode::ReadDiscreteInputs
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::DiscreteInputsOrCoils) => {
                if function_code == FunctionCode::WriteMultipleCoils
                    || function_code == FunctionCode::WriteSingleCoil
                    || function_code == FunctionCode::ReadCoils
                    || function_code == FunctionCode::ReadDiscreteInputs
                {
                    true
                } else {
                    false
                }
            }
            (DataAccessType::ReadWrite, DataModelType::AllType) => true,
            _ => false,
        }
    }

    pub fn update(&mut self, value: bool, rdb: &mut ModbusRegisterDatabase, e: EndiannessType) {
        let d = &mut self.data_value;
        match d {
            ModbusCoilDataValueType::Independent(_) => {
                *d = ModbusCoilDataValueType::Independent(IndependentCoil { value })
            }
            ModbusCoilDataValueType::RegisterBit(c) => {
                let register = rdb
                    .db
                    .get_mut(&c.register)
                    .expect(&format!("missing register @ {}", c.register));
                let mut current_values = Vec::<u16>::new();
                let _ = register.write_into_u16(&mut current_values, e);
                let register_idx = (c.bit / 16) as usize;
                let bit_idx = (c.bit % 16) as usize;
                if value == true {
                    current_values[register_idx] = current_values[register_idx] | (1 << bit_idx);
                } else {
                    current_values[register_idx] = current_values[register_idx] & (!(1 << bit_idx));
                }
                register.read_from_u16(&mut current_values.iter().peekable(), e);
            }
        }
    }

    pub fn read(&self, rdb: &ModbusRegisterDatabase, e: EndiannessType) -> bool {
        let d = &self.data_value;
        match d {
            ModbusCoilDataValueType::Independent(IndependentCoil { value }) => *value,
            ModbusCoilDataValueType::RegisterBit(c) => {
                let register = rdb
                    .db
                    .get(&c.register)
                    .expect(&format!("missing register @ {}", c.register));
                let mut current_values = Vec::<u16>::new();
                let _ = register.write_into_u16(&mut current_values, e);
                let register_idx = (c.bit / 16) as usize;
                let bit_idx = (c.bit % 16) as usize;
                current_values[register_idx] & (1 << bit_idx) != 0
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusCoilDatabase {
    db: HashMap<u16, ModbusCoilData>,
}

impl ModbusCoilDatabase {
    pub fn update_coils(
        &mut self,
        coil_addr: u16,
        values: Vec<bool>,
        function_code: FunctionCode,
        rdb: &mut ModbusRegisterDatabase,
        e: EndiannessType,
    ) -> anyhow::Result<usize, ModbusExceptionCode> {
        let mut value_it = values.iter().peekable();
        let mut total_updated = 0_usize;
        let mut addr = coil_addr;
        while let Some(data) = self.db.get_mut(&addr) {
            if data.is_function_code_supported(function_code) {
                if let Some(new_data) = value_it.next() {
                    data.update(*new_data, rdb, e);
                    total_updated += 1;
                } else {
                    return Err(ModbusExceptionCode::IllegalDataValue);
                }
                if value_it.peek().is_none() {
                    return Ok(total_updated);
                }
            } else {
                return Err(ModbusExceptionCode::IllegalFunction);
            }
            addr += 1;
        }
        return Err(ModbusExceptionCode::IllegalDataAddress);
    }

    pub fn read_coils(
        &self,
        coil_addr: u16,
        count: u16,
        function_code: FunctionCode,
        rdb: &ModbusRegisterDatabase,
        e: EndiannessType,
    ) -> anyhow::Result<Vec<bool>, ModbusExceptionCode> {
        let mut coils = Vec::<bool>::new();
        let mut count = count as usize;
        let mut addr = coil_addr;
        //let mut printout = String::new();
        while let Some(data) = self.db.get(&addr) {
            if data.is_function_code_supported(function_code) {
                coils.push(data.read(rdb, e));
                count -= 1;
                if count == 0 {
                    return Ok(coils);
                }
            } else {
                return Err(ModbusExceptionCode::IllegalFunction);
            }
            addr += 1;
        }
        return Err(ModbusExceptionCode::IllegalDataAddress);
    }
}
