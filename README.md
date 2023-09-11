# RustSenso (WIP)

Libary + 2 binarys to read out the sensor/usage data from a Vaillant communication module sensoNET.

**!!Work in progress!!**
Project only works for a specific configuration and therefore can not be used universally.

## Building

Build both binarys `sensor` and `usage`.
```
cargo build --release
```

## Details

### sensor
Reads out sensors for hot water temperature, water pressure, heating flow temperature and outside temperature and inserts the data into a sqlite database.
```
Insert vaillant api sensor data from a facility into a sqlite database

Usage: sensor [OPTIONS] --serial <SERIAL> --user <USER> --pwd <PWD>

Options:
  -s, --serial <SERIAL>          Specify the serial of the facility
  -d, --db-file <DB_FILE>        Path of the Sqlite file. Creates a new file if not found [default: ./data.db]
      --user <USER>              User name for login
      --pwd <PWD>                Password for login
  -t, --token-file <TOKEN_FILE>  Path to token file. Creates a new file if not found [default: ./token]
  -h, --help                     Print help
```

### usage
Reads out power usage and yield for the heat pump and boiler for one day in the past and inserts the data into a sqlite database.

Additionally the coefficient of performance(COP) for the day is calculated.
```
Insert vaillant api usage data from a facility into a sqlite database. Prints to stdout if no db_file is set

Usage: usage [OPTIONS] --serial <SERIAL> --user <USER> --pwd <PWD>

Options:
  -s, --serial <SERIAL>          Specify the serial of the facility
  -d, --db-file <DB_FILE>        Path of the Sqlite file. Creates a new file if not found
      --user <USER>              User name for login
      --pwd <PWD>                Password for login
  -t, --token-file <TOKEN_FILE>  Path to token file. Creates a new file if not found [default: ./token]
      --delta <DELTA>            how many days back from today in UTC. 1 => yesterday [default: 1]
  -h, --help                     Print help
  -V, --version                  Print version
```

## Test
To run all test run one of the following commands:
```
cargo test --all -- --nocapture
cargo t
```
For test coverage run:
```
cargo +stable install cargo-llvm-cov --locked
cargo lcov # for lcov.info
cargo covhtml # for html
```
## Credit

Login flow and URLS based on [pymultiMATIC](https://github.com/thomasgermain/pymultiMATIC).

Cargo subcommand for code coverage [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov).

## Disclaimer

I am not affiliated, associated, authorized, endorsed by, or in any way officially connected with Vaillant GmbH, or any of its subsidiaries or its affiliates.