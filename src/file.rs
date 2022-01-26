use crate::config::*;
use std::fs;

pub fn write_data_to_files(server: &ModbusServerConfig) {
    let r_str = serde_yaml::to_string(&server.register_data).unwrap();
    let c_str = serde_yaml::to_string(&server.coil_data).unwrap();
    fs::write(
        server
            .register_data_file
            .as_ref()
            .unwrap(),
        r_str,
    )
    .unwrap();
    fs::write(server.coil_data_file.as_ref().unwrap(), c_str).unwrap();
}

pub fn read_data_from_files(server: &mut ModbusServerConfig) {
    let rd_file = server
        .register_data_file
        .as_ref()
        .unwrap();
    let cd_file = server.coil_data_file.as_ref().unwrap();
    let r_data = fs::read_to_string(rd_file).expect("failed to read register data file");
    let c_data = fs::read_to_string(cd_file).expect("failed to read coil data file");
    server.register_data = serde_yaml::from_str(&r_data).unwrap();
    server.coil_data = serde_yaml::from_str(&c_data).unwrap();
}
