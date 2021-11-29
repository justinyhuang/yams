use futures::future;

use tokio_modbus::prelude::*;
use tokio_modbus::server::{self, Service};
use crate::*;

struct MbServer {
    db: ModbusRegisterDatabase,
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
        match req {
            Request::ReadInputRegisters(addr, cnt) =>
                match self.db.request_u16_registers(addr, cnt, FunctionCode::ReadInputRegisters) {
                    Ok(registers) =>
                        future::ready(Ok(Response::ReadInputRegisters(registers))),
                    Err(e) =>
                        future::ready(Ok(Response::Custom(FunctionCode::ReadInputRegisters.get_exception_code(),
                                                          vec![e as u8]))),
            },
            Request::ReadHoldingRegisters(addr, cnt) =>
                match self.db.request_u16_registers(addr, cnt, FunctionCode::ReadHoldingRegisters) {
                    Ok(registers) =>
                        future::ready(Ok(Response::ReadHoldingRegisters(registers))),
                    Err(e) =>
                        future::ready(Ok(Response::Custom(FunctionCode::ReadHoldingRegisters.get_exception_code(),
                                                          vec![e as u8]))),
            },
            _ => unimplemented!(),
        }
    }
}

pub async fn start_modbus_server(config: ModbusDeviceConfig) -> Result<(), Box<dyn std::error::Error>>
{
    let ip_addr = config
                  .common
                  .device_ip_address
                  .ok_or("IP address doesn't exist")?;
    let register_data = config
                        .server
                        .ok_or("Server config doesn't exist")?
                        .register_data;
    let server = server::tcp::Server::new(ip_addr);
    server.serve(move || Ok(MbServer {db: register_data.clone()})).await.unwrap();
    Ok(())
}

