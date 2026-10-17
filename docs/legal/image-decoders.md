# Image decoder provenance

This package contains two bare WebAssembly image decoders, JPEG and WebP.
Package-owned source implements the ABI and the codec policy. Committed Cargo
lockfiles pin the third-party runtime source. Both artifacts have no WASI,
environment or other WebAssembly imports.

The source, lockfiles and artifacts came from Sandom's `vendor/jpeg-wasm-rs/`,
`vendor/webp-wasm-rs/`, `vendor/jpeg.wasm` and `vendor/webp.wasm` at Sandom
commit `b614a0dd`. A rebuild in this repository gives byte-identical
artifacts.

## JPEG decoder

| Field | Value |
| --- | --- |
| Package source | `jpeg-wasm-rs/`, Apache-2.0 |
| Runtime artifact | `dist/jpeg.wasm` — SHA-256 `066968c552972ee5e0c7657857aa39eb6bb1dd0a40b688eb28577c957eb34b5c`, 119,065 bytes |
| Lockfile | SHA-256 `3f281089be78fed5b3b5c2f25e7d9abca789ed8ab75c0f591ef51a2ba3804767` |
| Build script | `jpeg-wasm-rs/build.sh` — SHA-256 `406650f25b2822c5dc2d7ab6cabedab5eabaacb3d408f706b20b2a27ebf25ded` |
| Compiler / target | `rustc 1.92.0 (ded5c06cf 2025-12-08)`, `wasm32-unknown-unknown` |
| Imports | None |

The wasm32 closure is `zune-jpeg 0.5.15` → `zune-core 0.5.1`, plus
`dlmalloc 0.2.14` → `cfg-if 1.0.4`. Each is a crates.io artifact with the
exact registry checksum in `Cargo.lock`. `zune-jpeg` and `zune-core` offer
Apache-2.0, MIT or zlib terms. `dlmalloc` and `cfg-if` offer Apache-2.0 or
MIT terms. The per-crate license texts are in
[licenses/rust-crates/](licenses/rust-crates/).

## WebP decoder

| Field | Value |
| --- | --- |
| Package source | `webp-wasm-rs/`, Apache-2.0 |
| Runtime artifact | `dist/webp.wasm` — SHA-256 `47c5f79ec000e18201facac0bcf6709f60334ea89c775cac3388ca9dcb8eae4f`, 162,886 bytes |
| Lockfile | SHA-256 `06e38f22c3eda63b024c303fb3235a2def68159d87a3bc6b82c5a58039679809` |
| Build script | `webp-wasm-rs/build.sh` — SHA-256 `09cd40e4d30c9fd55b213bd8fa2e91cf7f996b54f6d10a7fded0837504ef1ce1` |
| Compiler / target | `rustc 1.92.0 (ded5c06cf 2025-12-08)`, `wasm32-unknown-unknown` |
| Imports | None |

The wasm32 closure is `image-webp 0.2.4` → `byteorder-lite 0.1.0` and
`quick-error 2.0.1`. `image-webp` and `quick-error` offer Apache-2.0 or MIT
terms. `byteorder-lite` is MIT. The lockfile keeps each locked registry
checksum, and [licenses/rust-crates/](licenses/rust-crates/) keeps each
license text.

## Changes and rebuild

The package does not modify or copy third-party Rust crates. JPEG and WebP use
thin ABI shims. Build both source trees with a Rust toolchain that has
`wasm32-unknown-unknown` installed:

```sh
rustup target add wasm32-unknown-unknown
deno task build
```

The scripts write directly to the committed `dist/*.wasm` paths. The WebP
build remaps the Cargo home and crate paths, so the artifact names no
workstation path. A changed lockfile, manifest, Rust toolchain or output digest
is a new runtime artifact: record it here before acceptance, and verify it
with `deno task test`.

## Test fixtures

The files in `test/fixtures/` are DevBlanket AB-authored test images, copied
from Sandom's `test-assets/`. They need no third-party notice.

| File | SHA-256 | Role |
| --- | --- | --- |
| `test-photo-baseline.jpg` | `546fc6a946fdc219dd0503c002a5584f2bd48a3eaef3e6c8e8e42d909b085352` | 120×180 baseline JPEG |
| `test-photo-progressive.jpg` | `d320fc49afcfe973d79e6b6218a55f3bc80f2ba5d8ba363a29296271d6db126b` | 120×180 progressive JPEG of the same source |
| `test-photo-grayscale.jpg` | `5e1eacb5d083e48d7490f3fa997d027ee0ffebd4daea078ce646a76c5353ab9d` | 120×180 grayscale JPEG of the same source |
| `test-marker-lossless.webp` | `36620f54ccdea2a124f91ee6cb5cba23ff740608bf13262c828495609ebafb76` | 8×8 opaque lossless WebP |
| `test-alpha-lossless.webp` | `80e35de0677913d81aabf4e49d8c4e70896685fb6b00724a5292e27b8535a5a8` | Lossless WebP with three alpha regions |
| `test-photo-lossy.webp` | `70cd665d54d367bda28bf75157e489cdb1a1edd015a652f4c0ab386f1c85c346` | 360×540 lossy VP8 WebP |
| `test-anim.webp` | `71840ba513f96bc7af29f0b3a9c10bcf1baa25af013811ed8d465dee58187eea` | Three-frame 8×8 animated WebP |
