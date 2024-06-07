#!/bin/bash

# Build the project for the wasm32-unknown-unknown target
cargo build --target wasm32-unknown-unknown --release

# Use wasm-bindgen to generate the JavaScript bindings
wasm-bindgen target/wasm32-unknown-unknown/release/lunar_lander.wasm --out-dir ./out --target web

# Copy the HTML template to the output directory
# cp index.html out/
cp bob.html out/index.html
cp mq_js_bundle.js out/
