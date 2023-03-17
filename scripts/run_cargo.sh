#!/usr/bin/env bash
set -e
source secret
cargo run --bin $1 -- -s 21223900202609620938071939N6 --user $VaillantAppUSER --pwd $VaillantAppPWD