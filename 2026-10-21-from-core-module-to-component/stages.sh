#!/bin/sh
# Build the service in two separate stages instead of one linker command:
# stage 1 stops after wasm-ld and leaves a core module, and stage 2 turns
# that core module into a component with wasm-tools.
set -e
cargo build --target-dir target/stage1 --config 'target.wasm32-wasip2.rustflags=["-C", "link-arg=--skip-wit-component"]'
cp target/stage1/wasm32-wasip2/debug/from_core_module_to_component.wasm target/core.wasm
wasm-tools component new target/core.wasm -o target/component.wasm
echo "stage 1: target/core.wasm"
echo "stage 2: target/component.wasm"
