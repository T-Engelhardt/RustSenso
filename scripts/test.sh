#!/bin/sh
export RUST_LOG='senso=debug'
cargo test --all --features local_url -- --nocapture