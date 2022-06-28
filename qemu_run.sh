#!/bin/sh

script_dir=$(cd $(dirname $0); pwd)

OVMF_CODE=${script_dir}/ovmf/OVMF_CODE.fd
OVMF_VARS=${script_dir}/ovmf/OVMF_VARS.fd
BUILD_DIR=$(dirname $1)

qemu-system-x86_64 \
    -nodefaults \
    -machine q35,accel=kvm:tcg \
    -nographic \
    -m 128M \
    -serial stdio \
    -drive if=pflash,format=raw,file=$OVMF_CODE,readonly=on \
    -drive if=pflash,format=raw,file=$OVMF_VARS,readonly=on \
    -drive format=raw,file=fat:rw:$BUILD_DIR \

