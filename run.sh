#!/bin/bash

set -eu

readonly SCRIPT_DIR=$(cd $(dirname $0); pwd)
cd $SCRIPT_DIR

readonly KERNEL_TARGET=x86_64-unknown-none
readonly PROFILE=${1:-debug}

if [ $PROFILE == "debug" ]
then
    readonly CARGO_PROFILE=""
else
    readonly CARGO_PROFILE="--release"
fi

# kernel build
cargo build ${CARGO_PROFILE} --bin kernel --target ${KERNEL_TARGET}
# copy binary into disk
cp target/${KERNEL_TARGET}/${PROFILE}/kernel disk/kernel
# run gazami
cargo run --bin gazami ${CARGO_PROFILE}

# # === Use C++ binary for debug ===
# # kernel build
# clang++ -O2 -Wall -g --target=x86_64-elf -ffreestanding -mno-red-zone \
# -fno-exceptions -fno-rtti -std=c++17 -c src/bin/kernel.cpp
# ld.lld --entry KernelMain -z norelro --image-base 0x100000 --static -o \
# kernel.elf kernel.o
# # copy binary into disk
# mv kernel.elf disk/kernel
# # run gazami
# cargo run --bin gazami ${CARGO_PROFILE}
