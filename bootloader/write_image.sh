#!/bin/bash

set -uex

arch=${arch:-"thumbv6m-none-eabi"}
debug="release"
# debug="debug"
target_dir="../target/${arch}/${debug}"

./build_image.sh

probe-rs download --chip RP2040 --protocol swd --format bin --base-address 0x10000000 --skip 0 ${target_dir}/boot2.bin
probe-rs reset --chip RP2040 --protocol swd
