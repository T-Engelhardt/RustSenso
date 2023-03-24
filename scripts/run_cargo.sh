#!/usr/bin/env bash
set -e
source secret
cargo run --bin $1 -- -s $VaillantSerial --user $VaillantAppUSER --pwd $VaillantAppPWD