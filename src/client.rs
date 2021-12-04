use std::fs;
use tokio_modbus::prelude::*;
use tokio::time::{sleep, Duration};
use tokio_serial::SerialStream;

use crate::{
    config::*,
    util::*,
    types::*};

pub async fn start_modbus_client(config: ModbusDeviceConfig) -> Result<(), Box<dyn std::error::Error>>
{
    let _enabled = ansi_term::enable_ansi_support();
    print_configuration(&config);
    let client_requests = config.client.ok_or("Client config missing")?.requests;
    let mut counter: u16 = 0;
    for request in client_requests {
        let server_id = request.server_id.ok_or("server id missing")?;
        let server = Slave(server_id);
        let mut ctx = match config.common.protocol_type {
            ProtocolType::TCP => {
        let ip_addr = request.server_address
            .ok_or("Server IP address missing in config")?;
                tcp::connect_slave(ip_addr, server).await?
            },
            ProtocolType::RTU => {
                let device = config.common.device_port.as_ref()
                    .ok_or("client port missing")?;
                let baudrate = config.common.baudrate
                    .ok_or("baudrate missing")?;
                let builder = tokio_serial::new(device, baudrate);
                let port = SerialStream::open(&builder).unwrap();
                rtu::connect_slave(port, server).await?
            },
        };
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
                    counter += 1;
                    println!("{}", ansi_term::Colour::Blue.paint(format!(">>{:04}>>", counter)));
                    let response = match &r.function_code {
                        FunctionCode::ReadInputRegisters => {
                            vprintln(&format!("reading {} input registers starting at {}", count, start_addr),
                                     config.verbose_mode);
                            ModbusRequestReturnType::ResultWithU16Vec(
                                ctx.read_input_registers(start_addr, count).await)
                        },
                        FunctionCode::ReadHoldingRegisters => {
                            vprintln(&format!("reading {} holding registers starting at {}", count, start_addr),
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
                                     config.verbose_mode);
                            vprintln(&format!("{:?}", &data),
                                     config.verbose_mode);
                            ModbusRequestReturnType::ResultWithNothing(
                                ctx.write_multiple_registers(start_addr, &data).await)
                        },
                        _ => todo!()
                    };
                    println!("{}", r.description);
                    match response {
                        ModbusRequestReturnType::ResultWithU16Vec(Ok(response)) => {
                            match r.data_type {
                                DataType::Float32 =>
                                    println!("===> {:?}", write_be_u16_into_f32(response.as_slice())),
                                DataType::Float64 =>
                                    println!("===> {:?}", write_be_u16_into_f64(&response)),
                                _ => todo!()
                            }
                        },
                        ModbusRequestReturnType::ResultWithNothing(Ok(())) => {
                            println!("===> done");
                        }
                        ModbusRequestReturnType::ResultWithNothing(Err(e)) |
                        ModbusRequestReturnType::ResultWithU16Vec(Err(e)) => {
                            vprint("failure ", ansi_term::Colour::Red, true);
                            println!("{}", e);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
