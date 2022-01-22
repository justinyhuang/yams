use std::fs;
use tokio::time::{sleep, Duration};
use tokio_modbus::prelude::*;

use crate::{config::*, data::*, types::*, util::*};

pub async fn start_modbus_client(
    mut config: ModbusDeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let _enabled = ansi_term::enable_ansi_support();

    print_configuration(&config);
    let client_requests = config
        .client
        .take()
        .expect("Client config missing")
        .requests;
    let mut counter: u16 = 0;
    for mut request in client_requests {
        let server_id = request
            .server_id
            .take()
            .expect("server id missing");
        let server = Slave(server_id);
        let mut ctx = match config.common.protocol_type {
            ProtocolType::TCP => {
                let ip_addr = request
                    .server_address
                    .take()
                    .expect("Server IP address missing in config");
                tcp::connect_slave(ip_addr, server).await?
            }
            ProtocolType::RTU => {
                let serial = build_serial(&config).ok_or("failed in building the serial client")?;
                rtu::connect_slave(serial, server).await?
            }
        };
        let mut rlist = Vec::<ModbusRequest>::new();
        for request_file in &request.request_files {
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
        if let Some(r) = request.request {
            rlist.push(r);
        }

        let mut section_repeat_times = request
            .repeat_times
            .or_else(|| Some(1))
            .unwrap();
        #[allow(unused_parens)]
        let section_indefinite_loop = (section_repeat_times == REPEAT_TIME_INDEFINITE);
        while section_repeat_times > 0 {
            if !section_indefinite_loop {
                section_repeat_times -= 1;
            }
            for r in &mut rlist {
                let start_addr = r.access_start_address;
                let count = r.access_quantity;
                let mut file_repeat_times = r
                    .repeat_times
                    .or_else(|| Some(1))
                    .unwrap();
                #[allow(unused_parens)]
                let file_indefinite_loop = (file_repeat_times == REPEAT_TIME_INDEFINITE);
                let delay_in_100ms = r.delay.or_else(|| Some(0)).unwrap();
                while file_repeat_times > 0 {
                    if !file_indefinite_loop {
                        file_repeat_times -= 1;
                    }
                    sleep(Duration::from_millis(100 * delay_in_100ms)).await;
                    counter += 1;
                    println!(
                        "{}",
                        ansi_term::Colour::Blue.paint(format!(">>{:04}>>", counter))
                    );
                    let response = match &r.function_code {
                        FunctionCode::ReadInputRegisters => {
                            vprintln(
                                &format!(
                                    "reading {} input registers starting at {}",
                                    count, start_addr
                                ),
                                config.verbose_mode,
                            );
                            ModbusRequestReturnType::ResultWithU16Vec(
                                ctx.read_input_registers(start_addr, count)
                                    .await,
                            )
                        }
                        FunctionCode::ReadHoldingRegisters => {
                            vprintln(
                                &format!(
                                    "reading {} holding registers starting at {}",
                                    count, start_addr
                                ),
                                config.verbose_mode,
                            );
                            ModbusRequestReturnType::ResultWithU16Vec(
                                ctx.read_holding_registers(start_addr, count)
                                    .await,
                            )
                        }
                        FunctionCode::WriteMultipleRegisters => {
                            let new_values = r
                                .new_values
                                .take()
                                .expect("missing value for write");
                            let mut data = Vec::<u16>::new();
                            for v in new_values {
                                let d = ModbusRegisterData {
                                    data_description: "".to_string(),
                                    data_model_type: DataModelType::HoldingOrInputRegister,
                                    data_access_type: None,
                                    data_type: r
                                        .data_type
                                        .expect("missing data type for write"),
                                    data_value: v,
                                };
                                d.write_into_be_u16(&mut data);
                            }
                            vprintln(
                                &format!(
                                    "writing registers starting at {} with values:",
                                    start_addr
                                ),
                                config.verbose_mode,
                            );
                            vprintln(&format!("{:?}", &data), config.verbose_mode);
                            ModbusRequestReturnType::ResultWithNothing(
                                ctx.write_multiple_registers(start_addr, &data)
                                    .await,
                            )
                        }
                        FunctionCode::WriteSingleRegister => {
                            let new_values = r
                                .new_values
                                .take()
                                .expect("missing value for write");
                            let mut data = Vec::<u16>::new();
                            for v in new_values {
                                let d = ModbusRegisterData {
                                    data_description: "".to_string(),
                                    data_model_type: DataModelType::HoldingOrInputRegister,
                                    data_access_type: None,
                                    data_type: r
                                        .data_type
                                        .expect("missing data type for write"),
                                    data_value: v,
                                };
                                d.write_into_be_u16(&mut data);
                            }
                            vprintln(
                                &format!("writing register at {} with value:", start_addr),
                                config.verbose_mode,
                            );
                            vprintln(&format!("{:?}", data), config.verbose_mode);
                            ModbusRequestReturnType::ResultWithNothing(
                                ctx.write_single_register(start_addr, data[0])
                                    .await,
                            )
                        }
                        FunctionCode::ReadWriteMultipleRegisters => {
                            let new_values = r
                                .new_values
                                .take()
                                .expect("missing value for write");
                            let mut data = Vec::<u16>::new();
                            for v in new_values {
                                let d = ModbusRegisterData {
                                    data_description: "".to_string(),
                                    data_model_type: DataModelType::HoldingOrInputRegister,
                                    data_access_type: None,
                                    data_type: r
                                        .data_type
                                        .expect("missing data type for write"),
                                    data_value: v,
                                };
                                d.write_into_be_u16(&mut data);
                            }
                            vprintln(
                                &format!(
                                    "writing and read registers starting at {} with values:",
                                    start_addr
                                ),
                                config.verbose_mode,
                            );
                            vprintln(&format!("{:?}", &data), config.verbose_mode);
                            ModbusRequestReturnType::ResultWithU16Vec(
                                ctx.read_write_multiple_registers(
                                    start_addr,
                                    data.len() as u16,
                                    start_addr,
                                    &data,
                                )
                                .await,
                            )
                        }
                        FunctionCode::WriteMultipleCoils => {
                            let new_values = r
                                .new_values
                                .as_ref()
                                .expect("missing value for write");
                            let mut data = Vec::<bool>::new();
                            for v in new_values {
                                if let Ok(f) = v.parse::<bool>() {
                                    data.push(f);
                                }
                            }
                            vprintln(
                                &format!("writing coils starting at {} with values:", start_addr),
                                config.verbose_mode,
                            );
                            vprintln(&format!("{:?}", &data), config.verbose_mode);
                            ModbusRequestReturnType::ResultWithNothing(
                                ctx.write_multiple_coils(start_addr, &data)
                                    .await,
                            )
                        }
                        FunctionCode::ReadCoils => {
                            vprintln(
                                &format!("reading {} coils starting at {}", count, start_addr),
                                config.verbose_mode,
                            );
                            ModbusRequestReturnType::ResultWithBoolVec(
                                ctx.read_coils(start_addr, count).await,
                            )
                        }
                        FunctionCode::ReadDiscreteInputs => {
                            vprintln(
                                &format!("reading {} coils starting at {}", count, start_addr),
                                config.verbose_mode,
                            );
                            ModbusRequestReturnType::ResultWithBoolVec(
                                ctx.read_discrete_inputs(start_addr, count)
                                    .await,
                            )
                        }
                        FunctionCode::WriteSingleCoil => {
                            let new_values = r
                                .new_values
                                .as_ref()
                                .expect("missing value for write");
                            let data = new_values[0]
                                .parse::<bool>()
                                .expect("incorrect value for bool");
                            vprintln(
                                &format!("writing coil at {} with value:", start_addr),
                                config.verbose_mode,
                            );
                            vprintln(&format!("{:?}", &data), config.verbose_mode);
                            ModbusRequestReturnType::ResultWithNothing(
                                ctx.write_single_coil(start_addr, data)
                                    .await,
                            )
                        }
                        _ => todo!(),
                    };
                    println!("{}", r.description);
                    match response {
                        ModbusRequestReturnType::ResultWithU16Vec(Ok(response)) => {
                            let data_type = r
                                .data_type
                                .as_ref()
                                .expect("missing data type for write");
                            match data_type {
                                DataType::Float32 => println!(
                                    "===> {:?}",
                                    write_be_u16_into_f32(response.as_slice())
                                ),
                                DataType::Float64 => {
                                    println!("===> {:?}", write_be_u16_into_f64(&response))
                                }
                                DataType::Uint32 => {
                                    let data = response[0] as u32 | (response[1] as u32) << 16;
                                    println!("===> {:?} ({:#010X})", data, data);
                                }
                                DataType::Uint16 => {
                                    let data = response[0];
                                    println!("===> {:?} ({:#06X})", data, data);
                                }
                                _ => todo!(),
                            }
                        }
                        ModbusRequestReturnType::ResultWithBoolVec(Ok(response)) => {
                            println!("{:?}", response)
                        }
                        ModbusRequestReturnType::ResultWithNothing(Ok(())) => {
                            println!("===> done");
                        }
                        ModbusRequestReturnType::ResultWithNothing(Err(e))
                        | ModbusRequestReturnType::ResultWithU16Vec(Err(e))
                        | ModbusRequestReturnType::ResultWithBoolVec(Err(e)) => {
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
