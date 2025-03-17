#!/usr/bin/env bash
loongarch64-unknown-linux-gnu-gdb \
    -ex 'file target/loongarch64-unknown-none/release/os' \
    -ex 'target remote localhost:1234'