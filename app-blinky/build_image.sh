#!/bin/bash

set -uex

arch=${arch:-"thumbv6m-none-eabi"}
debug=${debug:-"debug"}

if [[ "X$debug" == "Xrelease" ]]; then
  debug_option="--release"
fi

debug_option=${debug_option:-""}

cargo build ${debug_option}

arm-none-eabi-objcopy -O binary ../target/${arch}/${debug}/app-blinky ../target/${arch}/${debug}/app-blinky.bin
cd ../bintool && \
  cargo run bintool -c all -i ../target/${arch}/${debug}/app-blinky.bin -o ../target/${arch}/${debug}/app-blinky.base && \
  cargo run bintool -c info -i ../target/${arch}/${debug}/app-blinky.base
