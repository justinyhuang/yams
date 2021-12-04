# YAML based Configurations

## Server Configurations

```yaml
---
  common: >
  Section for common configurable items

      protocol_type: >
      Specifies the protocol type: either "TCP" or "RTU"

      device_port: >
      Specifies the (serial) port of the device used for Modbus RTU.
      This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
      Example: /dev/ttyS0

      device_ip_address: >
      Specifies the (TCP/IP) address of the device used for Modbus TCP, including the port.
      This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
      Example: 127.0.0.1:5502

      device_type: >
      Specifies the type of the simulated Modbus device: either "Server" or "Client"

      device_id: >
      Specifies the Modbus ID of the device.

      baudrate: >
      Specifies the baudrate when used for Modbus RTU.
      This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
      Example: 19200

  client: >
  Section for a Modbus Client device. Leave it as "~" for a Server.

  server: >
  Section for a Modbus Server device. Leave it as "~" for a Server.

      register_data: >
      Defines the register data maintained by the Server.

          db: { >
          "db" stands for "database"

              xxxxx: { >
              "xxxxx" should be the register address of a data item.
              Example: 40001

                 data_description: >
                 Sets a human-friendly description for this data item.
                 This description string will be printed out when the Server handles an access to this item.
                 Example: "Flowrate"

                 data_model_type: >
                 Sets the data model type of this data item.
                 Valid options are:
                     "DiscretesInput",
                     "Coils",
                     "DiscretesInputOrCoils",
                     "InputRegister",
                     "HoldingRegister",
                     "HoldingOrInputRegister",
                     "AllType",
                 where "AllType" means this data item can be accessed as a register or a coil.

                 data_access_type: >
                 Sets the access type of this data item.
                 Valid options are:
                     "ReadOnly",
                     "WriteOnly",
                     "ReadWrite",
                 This determines whether the item can be read/written by a client.
                 This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
                 When not specified the access type will be default to "ReadWrite".

                 data_type: >
                 Sets the data type of the data item.
                 Valid options are:
                     "Float32",
                     "Float64",
                     "Uint32",
                     "Uint64",
                     "Int32",
                     "Int64",

                 data_value: >
                 Sets the initial value of the data item.
                 Example: 3.141592653589793

                 },
              >
              More register data can be set in the "db" block
          }
```

## Client Configurations

```yaml
---
  common: >
  Section for common configurable items. Same as that for a Server thus ignored here.

  client: >
  Section for a Modbus Client device. Leave it as "~" for a Server.

      requests: [ >
      Defines a list of requests the Client will send to the Server(s)

          { >
          Beginning of a request block

              server_id: >
              Defines the ID of the Server this request is sent to.
              This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
              Example: 1

              server_address: >
              Defines the IP address of the Server this request is sent to.
              This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
              Example: "127.0.0.1:5502"

              repeat_times: >
              Defines how many times this request block shoud be repeated.
              To repeat indefinetely use 0xFFFF.
              This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
              When not specified the repeat_times is default to 1.
              Example: 25, or 0xFFFF

              request_files: [ >
              Specifies a list of files where the detailed requests are defined.

                  "path/to/the/request.yaml.file",
                  "more/request/yaml/file",
              ]
          }, >
          More request blocks can be defined after this
      ]
      register_data: >
      Defines register data maintained by the client (not supported at the moment)

  server: >
  Section for a Modbus Server device. Leave it as "~" for a Server.
```

YAMS supports defining a detailed request in its own request file.
The request files serve as building material for request blocks in the Client configuration file shown as above.
A request YAML file would look like:

```yaml
---
    description: >
    Sets a human-friendly description for this request.
    This description string will be printed out when a Client executes this request.

    function_code: >
    Specifies the function code to use in this request.
    Valid options are:
        "ReadCoils",
        "ReadDiscreteInputs",
        "ReadHoldingRegisters",
        "ReadInputRegisters",
        "WriteSingleCoil",
        "WriteSingleRegister",
        "ReadExceptionStatus",
        "Diagnostics",
        "GetCommeventCounter",
        "GetcommEventLog",
        "WriteMultipleCoils",
        "WriteMultipleRegisters",
    Note that at the moment not all the function codes are supported/implemented.

    access_start_address: >
    Specifies the start register address of the request.
    Example: 40001

    access_quantity: >
    Specifies the number of registers to access in the request.
    Example: 2

    new_values: >
    Specifies new values for a "Write" request.
    This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
    Example: [42.0, 99.1]

    repeat_times: >
    Specifies the number of times to repeat this single request.
    To repeat indefinetely use 0xFFFF.
    This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
    When not specified the repeat_times is default to 1.
    Example: 25, or 0xFFFF

    delay: >
    Specifies the delay time before sending this request, in 100 ms.
    This is an optional configurable item, and when not used just leave it as "~", or do not specify this item.
    When not specified the delay time would be default to 0.
    Example: 10 (to have a 1-second delay)

    data_type: >
    Specifies the type of the data for the request.
    Valid options are:
        "Float32",
        "Float64",
        "Uint32",
        "Uint64",
        "Int32",
        "Int64",
```

See the configuration file examples in `test/`
