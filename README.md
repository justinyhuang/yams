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
    - overlapping of coil and register is supported
    - support user defined server behavior triggered by register/coil writes:
      With this feature enabled: YAMS will share the register/coil data with a user program via data (yaml) files.
      Each time a register/coil is updated, the user specified program will be invoked to update the register/coil data
      based on user-defined logic, as long as the external program writes the new data back to the files, YAMS will
      pick up the new data and continue running.
- Modbus client behaviors:
    - requests to send to server(s)
    - support repeated request (single/multi request repeat)
    - support predefined delay before a request
- Human friendly UI:
    - flexible YAML based configuration/request organization
    - supports verbose mode showing more details across the wire
    - supports print-out of request/response indexes
    - set and show measurements in its own type: 32-bit float for example

To support quick testing, Oneshot mode is provided so that one can run YAMS with only arguments from command line,
   though it is recommended to run with YAML configuration files to make the most out of YAMS.

## Install

- Make sure Cargo is installed. See the [install page](rust-lang.org/tools/install) for details.
- Install YAMS by `cargo install yams`.

## Quick Demo:

[DEMO](https://asciinema.org/a/464218)

## Configurable Items
See [YAML based Configurations](yaml.based.configurations.md)

## Todo:

- [x] implement Modbus TCP support
- [x] implement YAML configuration/request support
- [x] implement Modbus RTU support
- [x] implement support for repeated request(s)
- [x] implement support for delay before request(s)
- [x] implement error handling
- [x] implement one-shot mode without config files
- [x] implement verbose mode to print out more detail/data
- [x] implement request/response counts printout
- [x] implement support for server behavior triggered by register/coil writes, defined by user
- [ ] implement all data support of all current functions, with tests
- implement function code support below:
  - [x] Read Coils
  - [x] Read Discrete Inputs
  - [x] Read Holding Registers
  - [x] Read Input Registers
  - [x] Write Single Coil
  - [x] Write Single Register
  - [ ] Read Exception Status
  - [ ] Diagnostics
  - [ ] Get Comm Event Counter
  - [ ] Get Comm Event Log
  - [x] Write Multiple Coils
  - [x] Write Multiple Registers
  - [ ] Report Server ID
  - [ ] Read File Record
  - [ ] Write File Record
  - [ ] Mask Write Register
  - [x] Read/Write Multiple registers
  - [ ] Read FIFO Queue
  - [ ] Encaptulated Interface Transport
  - [ ] CANopen General Reference Request and Response PDU
  - [ ] Read Device Identification
