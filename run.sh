#!/usr/bin/env bash
set -euo pipefail

SDK="/var/folders/62/_whbk8x14hz73d0whtv2mdbc0000gn/T/opencode/macmon-sdk/MacOSX26.1.sdk"

export PATH="/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/lld/bin:$PATH"
export SDKROOT="$SDK"
export RUSTFLAGS="-C linker=/opt/homebrew/opt/llvm/bin/clang -C link-arg=-fuse-ld=lld -C link-arg=--sysroot=$SDK -C link-arg=-Wno-missing-sysroot"

exec cargo "${@:-run}"
