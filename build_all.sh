#!/bin/bash

set -eux

cargo clean

cd bootloader && ./build_image.sh && cargo clippy && cd ..
cd app-blinky && cargo build && cargo clippy && ./build_image.sh && cd ..
cd bintool && cargo build && cargo clippy && cargo run && cd ..
cd blxlib && cargo build && cargo clippy && cargo test && cd ..
