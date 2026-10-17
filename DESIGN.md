# Design

## Surface

Each decoder is one import-free WebAssembly module with module-global state
and this export surface:

| Export | Signature | Meaning |
| --- | --- | --- |
| `memory` | memory | Linear memory that holds the input and output buffers. |
| `alloc` | `(length: i32) -> i32` | Reserves `length` bytes for the encoded file and returns their address. Discards any earlier decode. |
| `decode` | `() -> i32` | Decodes the bytes at the `alloc` address. Returns a status code. |
| `width` | `() -> i32` | Width of the decoded image, or 0. |
| `height` | `() -> i32` | Height of the decoded image, or 0. |
| `pixels` | `() -> i32` | Address of the decoded RGBA8 pixels. |
| `release` | `() -> ()` | Frees the input and output buffers. |
| `frames` | `() -> i32` | WebP only: frame count of an animation that `decode` refused with -4, or 0. |

A host calls `alloc`, writes the encoded bytes at the returned address, calls
`decode`, reads `width`, `height` and `width * height * 4` bytes at `pixels`,
and then calls `release`. The host copies the pixels out before `release` and
before the next `alloc`, because a later allocation can grow and detach the
memory buffer.

The output is RGBA8, tightly packed, row-major, with straight alpha. The JPEG
decoder sets every alpha byte to 255. The WebP decoder keeps the source alpha
and widens an RGB source to RGBA with alpha 255.

## Status codes

| Code | JPEG | WebP |
| --- | --- | --- |
| 0 | Success | Success |
| -1 | `decode` before `alloc` | `decode` before `alloc` |
| -2 | Not a decodable JPEG: truncated, corrupt, or an unsupported variant such as 12-bit or arithmetic-coded | Not a decodable WebP: truncated, corrupt, or an unsupported variant |
| -3 | The decoder output is not `width * height * 4` bytes | The decoder output is neither `width * height * 3` nor `width * height * 4` bytes |
| -4 | — | Animated WebP; `frames` gives the frame count |

A decoder never returns the first frame of an animation as a still image.

## Runtime

The modules are single-threaded and hold one decode at a time. A panic traps
the module (`panic = "abort"`); the host reports the trap. The JPEG decoder is
a `no_std` build with `dlmalloc`. The WebP decoder is a `std` build, because
`image-webp` reads through `std::io` traits.

## Language

Repository-owned WebAssembly written from scratch is WAT across the programme.
These decoders are Rust because they are thin shims over vetted third-party
implementations, `zune-jpeg` and `image-webp`. The crate closures are pinned by
`Cargo.lock` and recorded in `docs/legal/image-decoders.md`.

## Exclusions

The package does not convert colour, read ICC profiles or Exif orientation,
sniff formats, encode, or load itself. Hosts own instantiation, the binding,
colour management and orientation.
