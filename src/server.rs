use crate::{config::*, data::*, types::*, util::*};
use futures::future;
use std::sync::{Arc, Mutex};
use tokio_modbus::prelude::*;
use tokio_modbus::server::{self, Service};

struct MbServer {
    rdb: Arc<Mutex<ModbusRegisterDatabase>>,
    cdb: Arc<Mutex<ModbusCoilDatabase>>,
    verbose_mode: bool,
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
        let mut rdb = self.rdb.lock().unwrap();
        let mut cdb = self.cdb.lock().unwrap();
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        println!(
            "{}",
            ansi_term::Colour::Blue.paint(format!(">>{:04}>>", counter))
        );
        vprintln(&format!("received request {:?}", req), self.verbose_mode);
        match req {
            Request::ReadInputRegisters(addr, cnt) => {
                match (*rdb).request_u16_registers(addr, cnt, FunctionCode::ReadInputRegisters) {
                    Ok(registers) => {
                        vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                        vprintln(
                            &format!(": input register values {:#06X?}", registers),
                            self.verbose_mode,
                        );
                        future::ready(Ok(Response::ReadInputRegisters(registers)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadInputRegisters.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                match (*rdb).request_u16_registers(addr, cnt, FunctionCode::ReadHoldingRegisters) {
                    Ok(registers) => {
                        vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                        vprintln(
                            &format!(": holding register values {:#06X?}", registers),
                            self.verbose_mode,
                        );
                        future::ready(Ok(Response::ReadHoldingRegisters(registers)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadHoldingRegisters.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::WriteMultipleRegisters(addr, values) => match (*rdb).update_u16_registers(
                addr,
                values,
                FunctionCode::WriteMultipleRegisters,
            ) {
                Ok(reg_num) => {
                    vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                    vprintln(
                        &format!(": {} registers updated", reg_num),
                        self.verbose_mode,
                    );
                    future::ready(Ok(Response::WriteMultipleRegisters(addr, reg_num as u16)))
                }
                Err(e) => {
                    vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                    vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                    future::ready(Ok(Response::Custom(
                        FunctionCode::WriteMultipleRegisters.get_exception_code(),
                        vec![e as u8],
                    )))
                }
            },
            Request::WriteSingleRegister(addr, value) => {
                let values = vec![value];
                match (*rdb).update_u16_registers(
                    addr,
                    values,
                    FunctionCode::WriteSingleRegister,
                    ) {
                    Ok(_) => {
                        vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                        vprintln(
                            &format!("register updated"),
                            self.verbose_mode,
                            );
                        future::ready(Ok(Response::WriteSingleRegister(addr, value)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(
                                    FunctionCode::WriteSingleRegister.get_exception_code(),
                                    vec![e as u8],
                                    )))
                    }
                }
            },
            Request::ReadWriteMultipleRegisters(read_addr, cnt, write_addr, values) => {
                match (*rdb).update_u16_registers(
                    write_addr,
                    values,
                    FunctionCode::ReadWriteMultipleRegisters,
                ) {
                    Ok(_) => {
                        match (*rdb).request_u16_registers(read_addr, cnt, FunctionCode::ReadWriteMultipleRegisters) {
                            Ok(registers) => {
                                vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                                vprintln(
                                    &format!(": after write, register values {:#06X?}", registers),
                                    self.verbose_mode,
                                );
                                future::ready(Ok(Response::ReadWriteMultipleRegisters(registers)))
                            }
                            Err(e) => {
                                vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                                vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                                future::ready(Ok(Response::Custom(
                                            FunctionCode::ReadWriteMultipleRegisters.get_exception_code(),
                                            vec![e as u8],
                                )))
                            }
                        }
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(
                                    FunctionCode::WriteMultipleRegisters.get_exception_code(),
                                    vec![e as u8],
                        )))
                    }
                }
            },
            Request::WriteMultipleCoils(addr, values) => {
                match (*cdb).update_coils(addr, values, FunctionCode::WriteMultipleCoils, &mut rdb)
                {
                    Ok(coil_num) => {
                        vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                        vprintln(&format!(": {} coils updated", coil_num), self.verbose_mode);
                        future::ready(Ok(Response::WriteMultipleCoils(addr, coil_num as u16)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::WriteMultipleCoils.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::ReadCoils(addr, cnt) => {
                match (*cdb).read_coils(addr, cnt, FunctionCode::ReadCoils, &rdb) {
                    Ok(coils) => {
                        vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                        vprintln(&format!(": coil values {:#06X?}", coils), self.verbose_mode);
                        future::ready(Ok(Response::ReadCoils(coils)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadCoils.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::ReadDiscreteInputs(addr, cnt) => {
                match (*cdb).read_coils(addr, cnt, FunctionCode::ReadDiscreteInputs, &rdb) {
                    Ok(coils) => {
                        vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                        vprintln(&format!(": coil values {:#06X?}", coils), self.verbose_mode);
                        future::ready(Ok(Response::ReadDiscreteInputs(coils)))
                    }
                    Err(e) => {
                        vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(
                            FunctionCode::ReadDiscreteInputs.get_exception_code(),
                            vec![e as u8],
                        )))
                    }
                }
            }
            Request::WriteSingleCoil(addr, value) => match (*cdb).update_coils(
                addr,
                vec![value],
                FunctionCode::WriteSingleCoil,
                &mut rdb,
            ) {
                Ok(_) => {
                    vprint("Ok", ansi_term::Colour::Green, self.verbose_mode);
                    vprintln(&format!(": coil is set to {}", value), self.verbose_mode);
                    future::ready(Ok(Response::WriteSingleCoil(addr, value)))
                }
                Err(e) => {
                    vprint("Err", ansi_term::Colour::Red, self.verbose_mode);
                    vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                    future::ready(Ok(Response::Custom(
                        FunctionCode::WriteSingleCoil.get_exception_code(),
                        vec![e as u8],
                    )))
                }
            },
            _ => unimplemented!(),
        }
    }
}

pub async fn start_modbus_server(
    mut config: ModbusDeviceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    let _enabled = ansi_term::enable_ansi_support();

    print_configuration(&config);
    let (register_data, coil_data) = config
        .server
        .take()
        .expect("Server config missing")
        .get_db();
    match config.common.protocol_type {
        ProtocolType::TCP => {
            let ip_addr = config.common.ip_address.take().expect("IP address missing");
            let server = server::tcp::Server::new(ip_addr);
            server
                .serve(move || {
                    Ok(MbServer {
                        rdb: Arc::new(Mutex::new(register_data.clone())),
                        cdb: Arc::new(Mutex::new(coil_data.clone())),
                        verbose_mode: config.verbose_mode,
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
                        rdb: Arc::new(Mutex::new(register_data.clone())),
                        cdb: Arc::new(Mutex::new(coil_data.clone())),
                        verbose_mode: config.verbose_mode,
                        counter: Arc::new(Mutex::new(0)),
                    })
                })
                .await;
        }
    };
    Ok(())
}
