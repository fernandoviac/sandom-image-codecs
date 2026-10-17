#!/bin/bash
set -euo pipefail

# Builds the JPEG decoder (zune-jpeg + the thin shim in src/lib.rs) into
# dist/jpeg.wasm. Requires: rustup with the wasm32-unknown-unknown
# target (`rustup target add wasm32-unknown-unknown`) — no emsdk, no C
# toolchain.
#
# Unlike msdfgen (whose emscripten build emits a .mjs loader beside the
# .wasm), this module imports NOTHING — no WASI, no env, no JS glue — so
# the committed artifact is a bare .wasm that a host binding (Sandom's
# src/jpeg.js) instantiates directly.

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Write straight into dist/ at the repository root, so the committed
# artifact IS the build output.
# A source checkout has no dist/: GitHub carries the source alone.
mkdir -p "$HERE/../dist"
OUT_DIR="$(cd "$HERE/../dist" && pwd)"

cd "$HERE"
cargo build --target wasm32-unknown-unknown --release

BUILT="$HERE/target/wasm32-unknown-unknown/release/jpeg_wasm.wasm"
cp "$BUILT" "$OUT_DIR/jpeg.wasm"

echo "Built $OUT_DIR/jpeg.wasm"
ls -la "$OUT_DIR/jpeg.wasm"
