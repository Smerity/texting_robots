#!/bin/bash

# After running `cargo install cargo-wasi`

echo Compiling to WASI...
echo ====================
cargo wasi build --release

echo
echo Native execution:
echo =================
cargo run --release --
#Elapsed time: 1.09s / 100000 = 10.86µs per parsed robots.txt
#Elapsed time: 896.38ms / 1000000 = 896.00ns per allow check

echo
echo Wasmtime execution:
echo ===================
wasmtime run target/wasm32-wasi/release/texting_robots.wasm --dir=.
#Elapsed time: 2.90s / 100000 = 29.04µs per parsed robots.txt
#Elapsed time: 2.03s / 1000000 = 2.03µs per allow check

echo
echo Wasmer \(LLVM\) execution:
echo ========================
wasmer target/wasm32-wasi/release/texting_robots.wasm --dir . --llvm
#Elapsed time: 2.32s / 100000 = 23.21µs per parsed robots.txt
#Elapsed time: 1.48s / 1000000 = 1.48µs per allow check
