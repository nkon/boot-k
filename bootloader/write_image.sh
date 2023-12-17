#!/bin/bash

set -uex

arch=${arch:-"thumbv6m-none-eabi"}
debug="release"
target_dir="../target/${arch}/${debug}"

cargo build --release

arm-none-eabi-objcopy --only-section=".boot2" \
    -O binary ${target_dir}/bootloader ${target_dir}/boot2.bin
arm-none-eabi-objcopy --only-section=".vector_table" \
    --only-section=".text" --only-section=".rodata" --only-section=".data" \
    -O binary ${target_dir}/bootloader ${target_dir}/bootloader.bin

cat ${target_dir}/bootloader.bin >> ${target_dir}/boot2.bin

probe-rs download --chip RP2040 --protocol swd --format bin --base-address 0x10000000 --skip 0 ${target_dir}/boot2.bin
probe-rs reset --chip RP2040 --protocol swd
