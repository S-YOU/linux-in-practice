#!/bin/bash

cargo build --release

TARGET_BIN="$(dirname $0)/../target/release/cache"

for i in 4 8 16 32 64 128 256 512 1024 2048 4096 8192 16384 32768 ; do printf "%6d\t" $i ; $TARGET_BIN $i ; done
