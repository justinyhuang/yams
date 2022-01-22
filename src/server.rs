use crate::{config::*, types::*, util::*};
use futures::future;
use std::sync::{Arc, Mutex};
use tokio_modbus::prelude::*;
use tokio_modbus::server::{self, Service};

struct MbServer {
    db: Arc<Mutex<ModbusDeviceConfig>>,
    counter: Arc<Mutex<u16>>,
}

impl Service for MbServer {
    type Request = Request;
    type Response = Response;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        /* since the tokio-mobus crate doesn't support server sending exception response (yet),
         * the custom response type is used as a workaround to send exception response below.
         */
        let mut db = self.db.lock().unwrap();
        let mut counter = self.counter.lock().unwrap();

        *counter += 1;
        println!(
            "{}",
            ansi_term::Colour::Blue.paint(format!(">>{:04}>>", counter))
        );
        vprintln(&format!("received request {:?}", req), db.verbose_mode);

        let mut server = db.server.take().unwrap();
        let rd_file = server
            .register_data_file
            .as_ref()
            .unwrap();
        let cd_file = server.coil_data_file.as_ref().unwrap();

        if db.external_mode {
            let r_data =
                std::fs::read_to_string(rd_file).expect("failed to read register data file");
            let c_data = std::fs::read_to_string(cd_file).expect("failed to read coil data file");
            server.register_data = serde_yaml::from_str(&r_data).unwrap();
            server.coil_data = serde_yaml::from_str(&c_data).unwrap();
        }

        let future = match req {
            Request::ReadInputRegisters(addr, cnt) => {
                match server
                    .register_data
                    .request_u16_registers(addr, cnt, FunctionCode::ReadInputRegisters)
                {
                    Ok(registers) => {
                        vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                        vprintln(
                            &format!(": input register values {:#06X?}", registers),
                            db.verbose_mode,
                        );
                        future::ready(Ok(Response::ReadInputRegisters(registers)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadInputRegisters.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                match server
                    .register_data
                    .request_u16_registers(addr, cnt, FunctionCode::ReadHoldingRegisters)
                {
                    Ok(registers) => {
                        vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                        vprintln(
                            &format!(": holding register values {:#06X?}", registers),
                            db.verbose_mode,
                        );
                        future::ready(Ok(Response::ReadHoldingRegisters(registers)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadHoldingRegisters.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::WriteMultipleRegisters(addr, values) => match server
                .register_data
                .update_u16_registers(addr, values, FunctionCode::WriteMultipleRegisters)
            {
                Ok(reg_num) => {
                    vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                    vprintln(&format!(": {} registers updated", reg_num), db.verbose_mode);
                    future::ready(Ok(Response::WriteMultipleRegisters(addr, reg_num as u16)))
                }
                Err(e) => {
                    vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                    vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                    future::ready(Ok(Response::Custom(
                        FunctionCode::WriteMultipleRegisters.get_exception_code(),
                        vec![e as u8],
                    )))
                }
            },
            Request::WriteSingleRegister(addr, value) => {
                let values = vec![value];
                match server
                    .register_data
                    .update_u16_registers(addr, values, FunctionCode::WriteSingleRegister)
                {
                    Ok(_) => {
                        vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                        vprintln(&format!("register updated"), db.verbose_mode);
                        future::ready(Ok(Response::WriteSingleRegister(addr, value)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::WriteSingleRegister.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::ReadWriteMultipleRegisters(read_addr, cnt, write_addr, values) => match server
                .register_data
                .update_u16_registers(write_addr, values, FunctionCode::ReadWriteMultipleRegisters)
            {
                Ok(_) => {
                    match server
                        .register_data
                        .request_u16_registers(
                            read_addr,
                            cnt,
                            FunctionCode::ReadWriteMultipleRegisters,
                        ) {
                        Ok(registers) => {
                            vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                            vprintln(
                                &format!(": after write, register values {:#06X?}", registers),
                                db.verbose_mode,
                            );
                            future::ready(Ok(Response::ReadWriteMultipleRegisters(registers)))
                        }
                        Err(e) => {
                            vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                            vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                            future::ready(Ok(Response::Custom(
                                FunctionCode::ReadWriteMultipleRegisters.get_exception_code(),
                                vec![e as u8],
                            )))
                        }
                    }
                }
                Err(e) => {
                    vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                    vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                    future::ready(Ok(Response::Custom(
                        FunctionCode::WriteMultipleRegisters.get_exception_code(),
                        vec![e as u8],
                    )))
                }
            },
            Request::WriteMultipleCoils(addr, values) => {
                match server.coil_data.update_coils(
                    addr,
                    values,
                    FunctionCode::WriteMultipleCoils,
                    &mut server.register_data,
                ) {
                    Ok(coil_num) => {
                        vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                        vprintln(&format!(": {} coils updated", coil_num), db.verbose_mode);
                        future::ready(Ok(Response::WriteMultipleCoils(addr, coil_num as u16)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::WriteMultipleCoils.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::ReadCoils(addr, cnt) => {
                match server.coil_data.read_coils(
                    addr,
                    cnt,
                    FunctionCode::ReadCoils,
                    &server.register_data,
                ) {
                    Ok(coils) => {
                        vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                        vprintln(&format!(": coil values {:#06X?}", coils), db.verbose_mode);
                        future::ready(Ok(Response::ReadCoils(coils)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadCoils.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::ReadDiscreteInputs(addr, cnt) => {
                match server.coil_data.read_coils(
                    addr,
                    cnt,
                    FunctionCode::ReadDiscreteInputs,
                    &server.register_data,
                ) {
                    Ok(coils) => {
                        vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                        vprintln(&format!(": coil values {:#06X?}", coils), db.verbose_mode);
                        future::ready(Ok(Response::ReadDiscreteInputs(coils)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadDiscreteInputs.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::WriteSingleCoil(addr, value) => match server.coil_data.update_coils(
                addr,
                vec![value],
                FunctionCode::WriteSingleCoil,
                &mut server.register_data,
            ) {
                Ok(_) => {
                    vprint("Ok", ansi_term::Colour::Green, db.verbose_mode);
                    vprintln(&format!(": coil is set to {}", value), db.verbose_mode);
                    future::ready(Ok(Response::WriteSingleCoil(addr, value)))
                }
                Err(e) => {
                    vprint("Err", ansi_term::Colour::Red, db.verbose_mode);
                    vprintln(&format!(": {:?} Exception", e), db.verbose_mode);
                    future::ready(Ok(Response::Custom(
                        FunctionCode::WriteSingleCoil.get_exception_code(),
                        vec![e as u8],
                    )))
                }
            },
            _ => unimplemented!(),
        };
        if db.external_mode {
            let r_str = serde_yaml::to_string(&server.register_data).unwrap();
            let c_str = serde_yaml::to_string(&server.coil_data).unwrap();
            std::fs::write(
                server
                    .register_data_file
                    .as_ref()
                    .unwrap(),
                r_str,
            )
            .unwrap();
            std::fs::write(server.coil_data_file.as_ref().unwrap(), c_str).unwrap();
        }
        db.server = Some(server);
        future
    }
}

pub async fn start_modbus_server(
    config: ModbusDeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let _enabled = ansi_term::enable_ansi_support();

    print_configuration(&config);

    if config.external_mode {
        let server = config.server.as_ref().unwrap();
        let r_str = serde_yaml::to_string(&server.register_data).unwrap();
        let c_str = serde_yaml::to_string(&server.coil_data).unwrap();
        std::fs::write(
            server
                .register_data_file
                .as_ref()
                .unwrap(),
            r_str,
        )
        .unwrap();
        std::fs::write(server.coil_data_file.as_ref().unwrap(), c_str).unwrap();
    }

    match config.common.protocol_type {
        ProtocolType::TCP => {
            let ip_addr = config
                .common
                .ip_address
                .expect("IP address missing");
            let server = server::tcp::Server::new(ip_addr);
            server
                .serve(move || {
                    Ok(MbServer {
                        db: Arc::new(Mutex::new(config.clone())),
                        counter: Arc::new(Mutex::new(0)),
                    })
                })
                .await
                .unwrap();
        }
        ProtocolType::RTU => {
            let serial = build_serial(&config).ok_or("failed in building the serial server")?;
            let server = server::rtu::Server::new(serial);
            server
                .serve_forever(move || {
                    Ok(MbServer {
                        db: Arc::new(Mutex::new(config.clone())),
                        counter: Arc::new(Mutex::new(0)),
                    })
                })
                .await;
        }
    };
    Ok(())
}
