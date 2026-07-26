#!/usr/bin/env bash
# Builds CounterLogic.xcframework: the Rust counter logic compiled for
# iOS devices and simulators, plus UniFFI-generated Swift bindings.
#
# Outputs (both get added to the Xcode app target):
#   CounterLogic.xcframework      — the compiled Rust library
#   generated/counter_logic.swift — the Swift bindings that call into it
set -euo pipefail
cd "$(dirname "$0")"

DEVICE_TARGET=aarch64-apple-ios
SIM_TARGET=aarch64-apple-ios-sim   # Apple Silicon simulator

rustup target add "$DEVICE_TARGET" "$SIM_TARGET"

echo "==> Building Rust library for iOS targets"
cargo build --release --target "$DEVICE_TARGET"
cargo build --release --target "$SIM_TARGET"

echo "==> Generating UniFFI Swift bindings"
cargo build --release   # host dylib for library-mode bindgen
cargo run --release --bin uniffi-bindgen -- generate \
    --library target/release/libcounter_logic.dylib \
    --language swift \
    --out-dir generated

# xcodebuild expects a headers directory with a module.modulemap.
rm -rf headers CounterLogic.xcframework
mkdir -p headers
cp generated/counter_logicFFI.h headers/
cp generated/counter_logicFFI.modulemap headers/module.modulemap

echo "==> Creating CounterLogic.xcframework"
xcodebuild -create-xcframework \
    -library "target/$DEVICE_TARGET/release/libcounter_logic.a" -headers headers \
    -library "target/$SIM_TARGET/release/libcounter_logic.a" -headers headers \
    -output CounterLogic.xcframework

rm -rf headers
echo "==> Done: CounterLogic.xcframework + generated/counter_logic.swift"
