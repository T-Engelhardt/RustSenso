#!/bin/sh
export RUST_LOG='senso=debug'
cargo test --all -- --nocapture