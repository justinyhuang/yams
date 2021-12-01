use futures::future;
use tokio_modbus::prelude::*;
use tokio_modbus::server::{self, Service};
use colored::*;
use std::sync::{Arc, Mutex};
use crate::{
    config::*,
    data::*,
    types::*,
    util::*};

struct MbServer {
    db: Arc<Mutex<ModbusRegisterDatabase>>,
    verbose_mode: bool,
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
        println!("{}", ">>>>".blue());
        vprintln(&format!("received request {:?}", req), self.verbose_mode);
        match req {
            Request::ReadInputRegisters(addr, cnt) =>
                match (*db).request_u16_registers(addr, cnt, FunctionCode::ReadInputRegisters) {
                    Ok(registers) => {
                        vprint("Ok", "green", self.verbose_mode);
                        vprintln(&format!(": input register values [{:#06X?}]", registers), self.verbose_mode);
                        future::ready(Ok(Response::ReadInputRegisters(registers)))
                    },
                    Err(e) => {
                        vprint("Err", "red", self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(FunctionCode::ReadInputRegisters.get_exception_code(),
                                                          vec![e as u8])))
                    },
            },
            Request::ReadHoldingRegisters(addr, cnt) =>
                match (*db).request_u16_registers(addr, cnt, FunctionCode::ReadHoldingRegisters) {
                    Ok(registers) => {
                        vprint("Ok", "green", self.verbose_mode);
                        vprintln(&format!(": holding register values [{:#06X?}]", registers), self.verbose_mode);
                        future::ready(Ok(Response::ReadHoldingRegisters(registers)))
                    },
                    Err(e) => {
                        vprint("Err", "red", self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(FunctionCode::ReadHoldingRegisters.get_exception_code(),
                                                          vec![e as u8])))
                    },
            },
            Request::WriteMultipleRegisters(addr, values) =>
                match (*db).update_u16_registers(addr, values, FunctionCode::WriteMultipleRegisters) {
                    Ok(reg_num) => {
                        vprint("Ok", "green", self.verbose_mode);
                        vprintln(&format!(": {} registers updated", reg_num), self.verbose_mode);
                        future::ready(Ok(Response::WriteMultipleRegisters(addr, reg_num as u16)))
                    },
                    Err(e) => {
                        vprint("Err", "red", self.verbose_mode);
                        vprintln(&format!(": {:?} Exception", e), self.verbose_mode);
                        future::ready(Ok(Response::Custom(FunctionCode::WriteMultipleRegisters.get_exception_code(),
                                                          vec![e as u8])))
                    },
                }
            _ => unimplemented!(),
        }
    }
}

pub async fn start_modbus_server(config: ModbusDeviceConfig) -> Result<(), Box<dyn std::error::Error>>
{
    print_configuration(&config);
    let ip_addr = config
                  .common
                  .device_ip_address
                  .ok_or("IP address doesn't exist")?;
    let register_data = config
                        .server
                        .ok_or("Server config doesn't exist")?
                        .register_data;
    let server = server::tcp::Server::new(ip_addr);
    server.serve(move || Ok(MbServer{
                                     db: Arc::new(Mutex::new(register_data.clone())),
                                     verbose_mode: config.verbose_mode,
                                    })).await.unwrap();
    Ok(())
}

