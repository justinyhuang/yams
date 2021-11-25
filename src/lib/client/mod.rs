use std::fs;

use tokio_modbus::prelude::*;

use crate::{ModbusDeviceConfig,
            ModbusRequest};
use crate::lib::core::util::*;
use crate::lib::core::config::DataType;

pub async fn start_modbus_client(config: ModbusDeviceConfig) -> Result<(), Box<dyn std::error::Error>>
{
    let client_requests = config.client.ok_or("Client config doesn't exist")?.requests;
    for request in client_requests {
        let ip_addr = request.server_address
            .ok_or("Server IP address doesn't exist")?;
        let mut rlist = Vec::<ModbusRequest>::new();
        for request_file in request.request_files {
            if let Ok(request_str) = fs::read_to_string(&request_file) {
                if let Ok(r) = serde_yaml::from_str(&request_str) {
                    rlist.push(r);
                } else {
                    println!("failed in parsing request file {}", request_file.display());
                }
            } else {
                println!("failed in reading request file {}", request_file.display());
            }
        }

        let mut ctx = tcp::connect(ip_addr).await.unwrap();
        for r in rlist {
            let start_addr = r.access_start_address;
            let count = r.access_quantity;
            let response = ctx
                .read_input_registers(start_addr, count)
                .await.unwrap();
            println!("{}", r.description);
            match r.data_type {
                DataType::Float32 => {
                    let mut float32s = vec![0.0_f32; (count / 2) as usize];
                    write_be_u16_into_f32(&response, &mut float32s);
                    println!("===> {:?}", float32s);
                }
                DataType::Float64 => {
                    let mut float64s = vec![0.0_f64; (count / 4) as usize];
                    write_be_u16_into_f64(&response, &mut float64s);
                    println!("===> {:?}", float64s);
                }
                _ => todo!()
            }
        }
    }
    Ok(())
}
