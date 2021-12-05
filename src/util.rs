use std::mem::transmute;
use crate::{
    config::*,
    types::*};

pub fn print_configuration(config: &ModbusDeviceConfig)
{
    if let Some(_server) = &config.server {
        println!("Modbus Server (ID: {})", config.common.device_id);
        print!("runs {:?} ", config.common.protocol_type);
        match config.common.protocol_type {
            ProtocolType::TCP => {
                println!("@ {}", config.common.device_ip_address.unwrap());
            },
            ProtocolType::RTU => {
                println!("@ {}",
                         config.common.device_port.as_ref().unwrap());
            }
        }
    }
    if let Some(_client) = &config.client {
        println!("Modbus Client (ID: {})", config.common.device_id);
        print!("runs {:?} ", config.common.protocol_type);
    }
    println!("verbose mode: {:?}", config.verbose_mode);
}

pub fn vprint(s: &str, c: ansi_term::Color, v: bool)
{
    if v {
        print!("{}", c.paint(s));
    }
}

pub fn vprintln(s: &str, v: bool)
{
    if v {
        println!("{}", s);
    }
}

pub fn write_be_u16_into_f32(src: &[u16]) -> f32
{
    let data: u32 = src[0] as u32 | ((src[1] as u32) << 16);
    unsafe {
        transmute(data)
    }
}

pub fn write_be_u16_into_f64(src: &[u16]) -> f64
{
    let data: u64 = src[0] as u64 |
        ((src[1] as u64) << 16) |
        ((src[2] as u64) << 32) |
        ((src[3] as u64) << 48);
    unsafe {
        transmute(data)
    }
}

pub fn write_be_f32_into_u16(src: f32) -> Vec<u16>
{
    let data: u32 = unsafe {
        transmute(src)
    };
    let mut output = Vec::<u16>::new();
    output.push((data & 0xFFFF) as u16);
    output.push((data >> 16) as u16);
    output
}

pub fn write_be_f64_into_u16(src: f64) -> Vec<u16>
{
    let data: u64 = unsafe {
        transmute(src)
    };
    let mut output = Vec::<u16>::new();
    output.push((data & 0xFFFF) as u16);
    output.push(((data >> 16) & 0xFFFF) as u16);
    output.push(((data >> 32) & 0xFFFF) as u16);
    output.push(((data >> 48) & 0xFFFF) as u16);
    output
}

#[cfg(test)]
mod tests
{
    use super::*;
    use float_cmp::assert_approx_eq;

    #[test]
    fn given_2_u16_then_get_correct_f32()
    {
        let src = [0x0e56, 0x4049];
        assert_approx_eq!(f32, 3.1415_f32, write_be_u16_into_f32(&src));
    }

    #[test]
    fn given4_u16_then_get_correct_f64()
    {
        let src = [0x2D18, 0x5444, 0x21FB, 0x4009];
        assert_approx_eq!(f64, 3.141592653589793_f64, write_be_u16_into_f64(&src));
    }

    #[test]
    fn given_f32_then_get_correct_2_u16()
    {
        let src = 3.1415_f32;
        assert_eq!(vec![0x0e56, 0x4049], write_be_f32_into_u16(src));
    }

    #[test]
    fn given_f64_then_get_correct_4_u16()
    {
        let src = 3.141592653589793_f64;
        assert_eq!(vec![0x2D18, 0x5444, 0x21FB, 0x4009], write_be_f64_into_u16(src));
    }
}
