#!/bin/bash
set -euo pipefail

# Builds the WebP decoder (image-webp + the thin shim in src/lib.rs) into
# dist/webp.wasm. Requires: rustup with the wasm32-unknown-unknown
# target (`rustup target add wasm32-unknown-unknown`) — no emsdk, no C
# toolchain, same as jpeg-wasm-rs next door.
#
# Like the JPEG module, this one imports NOTHING — no WASI, no env, no JS
# glue — so the committed artifact is a bare .wasm that a host binding
# (Sandom's src/webp.js) instantiates directly. That is worth stating
# because this IS a `std` build (image-webp's decoder is generic over
# BufRead + Seek): std on wasm32-unknown-unknown pulls in no imports of its
# own (verified with WebAssembly.Module.imports() — empty), it just costs
# module size.
# Measured: 163 KB here vs jpeg.wasm's 116 KB no_std build.

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Same convention as jpeg-wasm-rs/build.sh: write straight into dist/ at the
# repository root, so the committed artifact IS the build output.
mkdir -p "$HERE/../dist"
OUT_DIR="$(cd "$HERE/../dist" && pwd)"

# --remap-path-prefix keeps this machine out of the artifact. This is a `std`
# build, so every panic site embeds its source location: without the remap the
# committed webp.wasm carries ~/.cargo/registry paths from whatever
# workstation built it — and it did, through sandom 0.2.1 and 0.3.0. The
# mapped names still identify the crate and line, they just do not name a home
# directory. jpeg-wasm-rs next door is `no_std` and embeds no locations, which
# is why only this artifact carried them.
cd "$HERE"
RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo --remap-path-prefix=$HERE=/webp-wasm-rs" \
  cargo build --target wasm32-unknown-unknown --release

BUILT="$HERE/target/wasm32-unknown-unknown/release/webp_wasm.wasm"
cp "$BUILT" "$OUT_DIR/webp.wasm"

echo "Built $OUT_DIR/webp.wasm"
ls -la "$OUT_DIR/webp.wasm"
