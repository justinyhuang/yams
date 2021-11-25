use futures::future;

use tokio_modbus::prelude::*;
use tokio_modbus::server::{self, Service};
use crate::{ModbusDeviceConfig, ModbusRegisterDatabase};

struct MbServer {
    db: ModbusRegisterDatabase,
}

impl Service for MbServer {
    type Request = Request;
    type Response = Response;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadInputRegisters(addr, cnt) => {
                let mut registers = vec![0_u16; cnt as usize];
                self.db
                    .write_registers_to_be_u16(addr, cnt, &mut registers);
                future::ready(Ok(Response::ReadInputRegisters(registers)))
            }
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

