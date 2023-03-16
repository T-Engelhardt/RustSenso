#!/bin/sh
# https://www.reddit.com/r/rust/comments/r60fzb/m1_users_how_are_you_cross_compiling/
# https://github.com/messense/homebrew-macos-cross-toolchains
export CC_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-gcc
export CXX_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-g++
export AR_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc
cargo build -r --target x86_64-unknown-linux-gnu