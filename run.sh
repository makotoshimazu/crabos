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
