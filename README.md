# Yet Another Modbus Simulator

![YAMS](yams.png)


A simulator for Modbus client/server that supports YAML based configuration and behavior definition, with pure Rust.

YAMS supports Modbus device configuration and behavior definition via YAML files. This means one can predefine how the
simulator works before running the simulator, including:

- the device type: Server/Client
- the protocol type: Modbus TCP/RTU
- communication configurations: IP address, baudrate, etc.
- Modbus configurations: device ID etc.
- Modbus server properties:
    - supported function codes
    - supported registers/coils and their values
- Modbus client behaviors:
    - requests to send to server(s)
    - support repeated request (single/multi request repeat)
    - support predefined delay before a request
- Human friendly UI:
    - flexible request organization
    - set and show measurements in its own type: 32-bit float

## Quick Demo:

[DEMO](https://asciinema.org/a/452710)

## Configurable Items
See [YAML based Configurations](yaml.based.configurations.md)

## Todo:

- [x] implement Modbus TCP support
- [x] implement YAML configuration/request support
- [ ] implement Modbus RTU support
- [x] implement support for repeated request(s)
- [x] implement support for delay before request(s)
- [x] implement error handling
- [ ] implement one-shot mode without config files
- [ ] implement verbose mode to print out more detail/data
- implement function code support below:
  - [ ] Read Coils
  - [ ] Read Discrete Inputs
  - [x] Read Holding Registers
  - [x] Read Input Registers
  - [ ] Write Single Coil
  - [ ] Write Single Register
  - [ ] Read Exception Status
  - [ ] Diagnostics
  - [ ] Get Comm Event Counter
  - [ ] Get Comm Event Log
  - [ ] Write Multiple Coils
  - [x] Write Multiple Registers
  - [ ] Report Server ID
  - [ ] Read File Record
  - [ ] Write File Record
  - [ ] Mask Write Register
  - [ ] Read/Write Multiple registers
  - [ ] Read FIFO Queue
  - [ ] Encaptulated Interface Transport
  - [ ] CANopen General Reference Request and Response PDU
  - [ ] Read Device Identification
