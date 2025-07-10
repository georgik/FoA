#!/bin/bash

if [ "$2" = "esp32c6" ]; then
    TARGET="riscv32imac-unknown-none-elf"
else
    TARGET="xtensa-$2-none-elf"
fi

SSID=$3 DEFMT_LOG=$4 cargo run -r --features $2 --target $TARGET --bin $1
