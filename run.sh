#!/bin/bash

set -eu

script_dir=$(cd $(dirname $0); pwd)

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
cp ${script_dir}/target/${KERNEL_TARGET}/${PROFILE}/kernel ${script_dir}/disk/kernel
# run gazami
cargo run --bin gazami ${CARGO_PROFILE}
