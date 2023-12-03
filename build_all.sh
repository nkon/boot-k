#!/bin/bash

set -eux

cargo clean

cd bootloader && cargo build && cd ..
cd app-blinky && cargo build && cd ..
cd bintool && cargo build && cargo run && cd ..
cd blxlib && cargo build && cargo test && cd ..
