#!/bin/bash

set -uex

arch=${arch:-"thumbv6m-none-eabi"}
debug=${debug:-"debug"}
update=${update:-""}

if [[ "X$debug" == "Xrelease" ]]; then
  debug_option="--release"
fi

if [[ "X$update" == "Xupdate" ]]; then
    base_address="0x10100000"
else 
    base_address="0x10020000"
fi

arch=${arch} debug=${debug} ./build_image.sh

probe-rs download --chip RP2040 --protocol swd \
  --format bin --base-address ${base_address} --skip 0 \
  ../target/${arch}/${debug}/app-blinky.base
probe-rs reset --chip RP2040 --protocol swd
