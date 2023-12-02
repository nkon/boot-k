#!/bin/bash

set -uex

arch=${arch:-"thumbv6m-none-eabi"}
debug=${debug:-"debug"}

probe-rs download --chip RP2040 --protocol swd ../target/${arch}/${debug}/app-blinky
probe-rs reset --chip RP2040 --protocol swd
