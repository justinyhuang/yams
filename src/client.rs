use std::fs;
use colored::*;
use tokio_modbus::prelude::*;
use tokio::time::{sleep, Duration};

use crate::{
    config::*,
    util::*,
    types::*};

pub async fn start_modbus_client(config: ModbusDeviceConfig) -> Result<(), Box<dyn std::error::Error>>
{
    print_configuration(&config);
    let client_requests = config.client.ok_or("Client config missing")?.requests;
    for request in client_requests {
        let ip_addr = request.server_address
            .ok_or("Server IP address missing in config")?;
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

        let mut ctx = tcp::connect(ip_addr).await?;
        let mut section_repeat_times = request.repeat_times.or_else(|| Some(1)).unwrap();
        #[allow(unused_parens)]
        let section_indefinite_loop = (section_repeat_times == REPEAT_TIME_INDEFINITE);
        while section_repeat_times > 0 {
            if !section_indefinite_loop {
                section_repeat_times -= 1;
            }
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
                    println!("{}", ">>>>".blue());
                    let response = match &r.function_code {
                        FunctionCode::ReadInputRegisters => {
                            vprintln(&format!("reading {} input registers starting at {}", count, start_addr),
                                     "white",
                                     config.verbose_mode);
                            ModbusRequestReturnType::ResultWithU16Vec(
                                ctx.read_input_registers(start_addr, count).await)
                        },
                        FunctionCode::ReadHoldingRegisters => {
                            vprintln(&format!("reading {} holding registers starting at {}", count, start_addr),
                                     "white",
                                     config.verbose_mode);
                            ModbusRequestReturnType::ResultWithU16Vec(
                                ctx.read_holding_registers(start_addr, count).await)
                        },
                        FunctionCode::WriteMultipleRegisters => {
                            let new_values = r.new_values.as_ref().expect("no new value for write");
                            let mut data = Vec::<u16>::new();
                            match r.data_type {
                                DataType::Float32 => {
                                    for v in new_values {
                                        if let Ok(f) = v.parse::<f32>() {
                                            data.extend(write_be_f32_into_u16(f));
                                        }
                                    }
                                },
                                _ => todo!()
                            }
                            vprintln(&format!("writing registers starting at {} with values:", start_addr),
                                     "white",
                                     config.verbose_mode);
                            vprintln(&format!("{:?}", &data),
                                     "white",
                                     config.verbose_mode);
                            ModbusRequestReturnType::ResultWithNothing(
                                ctx.write_multiple_registers(start_addr, &data).await)
                        },
                        _ => todo!()
                    };
                    println!("{}", r.description.white());
                    match response {
                        ModbusRequestReturnType::ResultWithU16Vec(Ok(response)) => {
                            match r.data_type {
                                DataType::Float32 =>
                                    println!("{}", format!("===> {:?}", write_be_u16_into_f32(response.as_slice())).white()),
                                DataType::Float64 =>
                                    println!("{}", format!("===> {:?}", write_be_u16_into_f64(&response)).white()),
                                _ => todo!()
                            }
                        },
                        ModbusRequestReturnType::ResultWithNothing(Ok(())) => {
                            println!("{}", "===> done".white());
                        }
                        ModbusRequestReturnType::ResultWithNothing(Err(e)) |
                        ModbusRequestReturnType::ResultWithU16Vec(Err(e)) => {
                            println!("{}{}", "failure".red(), format!(": {}", e).white());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
