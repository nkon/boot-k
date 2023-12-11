#!/bin/bash

set -eux

cargo clean

cd bootloader && cargo build && cargo clippy && cd ..
cd app-blinky && cargo build && cargo clippy && cd ..
cd bintool && cargo build && cargo clippy && cargo run && cd ..
cd blxlib && cargo build && cargo clippy && cargo test && cd ..
