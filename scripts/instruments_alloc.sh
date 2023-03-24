#!/usr/bin/env bash
set -e
source secret
cargo instruments -t Allocations --bin $1 -- -s $VaillantSerial --user $VaillantAppUSER --pwd $VaillantAppPWD