#!/bin/bash

set -uex

arch=${arch:-"thumbv6m-none-eabi"}
debug="release"
# debug="debug"
target_dir="../target/${arch}/${debug}"

### must build release mode.
### debug build image is toolarge to fit memory map.
cargo build --release
# cargo build

# arm-none-eabi-objcopy --only-section=".boot2" \
#     -O binary ${target_dir}/bootloader ${target_dir}/boot2.bin
cp ../../rp2040-boot2/bin/boot2_ram_memcpy.padded.bin ${target_dir}/boot2.bin
arm-none-eabi-objcopy --only-section=".vector_table" \
    --only-section=".text" --only-section=".rodata" \
    -O binary ${target_dir}/bootloader ${target_dir}/bootloader.bin

cat ${target_dir}/bootloader.bin >> ${target_dir}/boot2.bin

