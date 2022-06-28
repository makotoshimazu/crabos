#!/bin/sh

script_dir=$(cd $(dirname $0); pwd)
built_binary_path="$1"

DISK_DIR=${script_dir}/disk
OVMF_CODE=${script_dir}/ovmf/OVMF_CODE.fd
OVMF_VARS=${script_dir}/ovmf/OVMF_VARS.fd

# Make sure the disk dir exists
mkdir -p $DISK_DIR

# Copy the binary to the disk dir
cp $built_binary_path $DISK_DIR

# Create startup.nsh for EDK2 Shell to read.
echo "fs0:$(basename $built_binary_path)" > $DISK_DIR/startup.nsh

qemu-system-x86_64 \
    -nodefaults \
    -machine q35,accel=kvm:tcg \
    -nographic \
    -m 128M \
    -serial stdio \
    -drive if=pflash,format=raw,file=$OVMF_CODE,readonly=on \
    -drive if=pflash,format=raw,file=$OVMF_VARS,readonly=on \
    -drive format=raw,file=fat:rw:$DISK_DIR \
