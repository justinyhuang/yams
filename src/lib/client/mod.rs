use std::fs;

use tokio_modbus::prelude::*;
use tokio::time::{sleep, Duration};

use crate::{ModbusDeviceConfig,
            ModbusRequest};
use crate::lib::core::util::*;
use crate::lib::core::config::*;

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

        let mut section_repeat_times = request.repeat_times.or_else(|| Some(1)).unwrap();
        #[allow(unused_parens)]
        let section_indefinite_loop = (section_repeat_times == REPEAT_TIME_INDEFINITE);
        while section_repeat_times > 0 {
            if !section_indefinite_loop {
                section_repeat_times -= 1;
            }
            let mut ctx = tcp::connect(ip_addr).await.unwrap();
            for r in &rlist {
                let start_addr = r.access_start_address;
                let count = r.access_quantity;
                let mut file_repeat_times = r.repeat_times.or_else(|| Some(1)).unwrap();
                #[allow(unused_parens)]
                let file_indefinite_loop = (file_repeat_times == REPEAT_TIME_INDEFINITE);
                let delay_in_100ms = r.delay.or_else(|| Some(0)).unwrap();
                while file_repeat_times > 0 {
                    if !file_indefinite_loop {
                        file_repeat_times -= 1;
                    }
                    sleep(Duration::from_millis(100 * delay_in_100ms)).await;
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
        }
    }
    Ok(())
}
