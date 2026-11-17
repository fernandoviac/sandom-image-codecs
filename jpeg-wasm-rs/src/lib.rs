//! JPEG -> RGBA8 for sandom, as a freestanding wasm module.
//!
//! The whole surface is five exports and one piece of module-global state,
//! deliberately: this is called from JS (src/jpeg.js) exactly once per
//! distinct image src, so there is no concurrency to design for and no
//! handle table worth its own bug surface.
//!
//! Call order is fixed and enforced by src/jpeg.js, which is the only
//! caller: `alloc` -> write bytes -> `decode` -> read `width`/`height`/
//! `pixels` -> `release`.
//!
//! Output is RGBA8, tightly packed, row-major, alpha always 255 - the same
//! shape src/images.js's decodePNG returns, so both feed the image pipeline
//! without a per-format branch downstream.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use zune_jpeg::zune_core::bytestream::ZCursor;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

// panic = "abort" in the release profile, so a panic traps the module
// rather than unwinding into JS. src/jpeg.js reports the trap with the
// source path; nothing here tries to recover or to return a partial image.
#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

/// The input buffer JS writes encoded bytes into, and the decoded RGBA8
/// output. Both are held across calls because `decode` returns only a
/// status code - the caller reads geometry and pixels through the
/// accessors below, then calls `release`.
struct State {
    input: Vec<u8>,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

static mut STATE: Option<State> = None;

fn state() -> Option<&'static mut State> {
    // Single-threaded by construction: wasm32-unknown-unknown without
    // threads, driven by one JS caller.
    unsafe { (*core::ptr::addr_of_mut!(STATE)).as_mut() }
}

/// Reserves `len` bytes for the encoded JPEG and returns a pointer into
/// linear memory for JS to write them at. Discards any previous decode.
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut input = Vec::new();
    input.resize(len, 0);
    let ptr = input.as_mut_ptr();
    unsafe {
        STATE = Some(State { input, pixels: Vec::new(), width: 0, height: 0 });
    }
    ptr
}

/// Decodes the bytes previously written into `alloc`'s buffer.
/// Returns 0 on success, -1 if `alloc` was never called, -2 if the bytes
/// are not a decodable JPEG, and -3 if the decoded size does not match the
/// RGBA8 output contract.
#[no_mangle]
pub extern "C" fn decode() -> i32 {
    let Some(st) = state() else { return -1 };

    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    // ZCursor, not a bare slice: zune-core only implements its reader trait
    // for std's BufRead+Seek, which this no_std build does not have.
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(st.input.as_slice()), options);
    let Ok(pixels) = decoder.decode() else { return -2 };
    let Some((w, h)) = decoder.dimensions() else { return -2 };

    // Guarded rather than assumed: the RGBA request above is an option, and
    // a decoder that honoured it partially (grayscale passthrough, say)
    // would otherwise hand JS a short buffer that reads as a torn image
    // instead of a failure.
    if pixels.len() != w.saturating_mul(h).saturating_mul(4) {
        return -3;
    }

    st.pixels = pixels;
    st.width = w as u32;
    st.height = h as u32;
    // The encoded input is dead the moment the pixels exist, and a photo's
    // worth of it is not worth holding until `release`.
    st.input = Vec::new();
    0
}

#[no_mangle]
pub extern "C" fn width() -> u32 {
    state().map_or(0, |s| s.width)
}

#[no_mangle]
pub extern "C" fn height() -> u32 {
    state().map_or(0, |s| s.height)
}

#[no_mangle]
pub extern "C" fn pixels() -> *const u8 {
    state().map_or(core::ptr::null(), |s| s.pixels.as_ptr())
}

/// The text of a nonzero `decode` status, UTF-8 at `status_message` with
/// `status_message_length` bytes; "unknown decoder status" for another
/// value.
fn message(status: i32) -> &'static str {
    match status {
        -1 => "internal error: decode() called before alloc()",
        -2 => "not a decodable JPEG (truncated, corrupt, or an unsupported variant such as 12-bit or arithmetic-coded)",
        -3 => "internal error: decoder returned a buffer that is not width*height*4 RGBA bytes",
        _ => "unknown decoder status",
    }
}

#[no_mangle]
pub extern "C" fn status_message(status: i32) -> *const u8 {
    message(status).as_ptr()
}

#[no_mangle]
pub extern "C" fn status_message_length(status: i32) -> usize {
    message(status).len()
}

/// Frees both buffers. JS calls this after copying the pixels out, so a
/// decoded photo does not sit in linear memory until the next decode
/// happens to overwrite it.
#[no_mangle]
pub extern "C" fn release() {
    unsafe {
        STATE = None;
    }
}
