#!/bin/bash

# After running `cargo install cargo-wasi`

echo Compiling to WASI...
echo ====================
cargo wasi build --release

echo
echo Native execution:
echo =================
cargo run --release --
#Elapsed time: 308.53ms / 1000 = 308.53µs per loop

echo
echo Wasmtime execution:
echo ===================
wasmtime run target/wasm32-wasi/release/texting_robots.wasm --dir=.
#Elapsed time: 562.58ms / 1000 = 562.63µs per loop

echo
echo Wasmer \(LLVM\) execution:
echo ========================
wasmer target/wasm32-wasi/release/texting_robots.wasm --dir . --llvm
#Elapsed time: 450.81ms / 1000 = 450.81µs per loop
