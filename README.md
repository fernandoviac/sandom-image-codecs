# @sandom/image-codecs

Status: extracted from Sandom (`vendor/jpeg-wasm-rs/`, `vendor/webp-wasm-rs/`,
`vendor/jpeg.wasm`, `vendor/webp.wasm`) with byte-identical output. Not yet
published.

This project owns two import-free WebAssembly decoders:

- `dist/jpeg.wasm` decodes baseline, progressive and grayscale JPEG to RGBA8.
  It is a thin Rust shim over `zune-jpeg`.
- `dist/webp.wasm` decodes still lossy (VP8) and lossless (VP8L) WebP, with or
  without alpha, to RGBA8, and refuses an animated WebP. It is a thin Rust shim
  over `image-webp`.

It does not own colour management, ICC profiles, Exif orientation, format
sniffing, encoding, animation or the host binding. The package contains no
JavaScript, TypeScript or `.d.ts` files: a host writes its own binding against
the export surface in [DESIGN.md](DESIGN.md). Tests use JavaScript.

## Packages

- npm: `@sandom/image-codecs`
- library set: `https://cdn.urania-libs.com`, under
  `sandom-image-codecs/dist/jpeg.wasm` and `sandom-image-codecs/dist/webp.wasm`

The package ships `dist/jpeg.wasm`, `dist/webp.wasm`, `docs/`, and the license
files.

## Consumers

Sandom's handwritten bindings (`src/jpeg.js`, `src/webp.js`) load the modules
from the library set, or from this npm package in an npm install.

## Commands

```sh
deno task build   # rebuild dist/jpeg.wasm and dist/webp.wasm with cargo
deno task test    # raw-ABI tests against dist/ (needs --allow-read only)
```

The build needs a Rust toolchain with the `wasm32-unknown-unknown` target.
[docs/legal/image-decoders.md](docs/legal/image-decoders.md) records the
compiler, the artifact digests and the rebuild procedure.

## Documents

- [DESIGN.md](DESIGN.md): export surface, status codes and exclusions.
- [docs/legal/image-decoders.md](docs/legal/image-decoders.md): third-party
  closure, digests, rebuild and test fixtures.
