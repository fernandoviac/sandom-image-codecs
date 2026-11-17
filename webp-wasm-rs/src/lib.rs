//! WebP -> RGBA8 for sandom, as a freestanding wasm module.
//!
//! Deliberately the same five-export surface as jpeg-wasm-rs, for
//! the same reason: this is called from JS (src/webp.js) exactly once per
//! distinct image src, so there is no concurrency to design for and no
//! handle table worth its own bug surface.
//!
//! Call order is fixed and enforced by src/webp.js, which is the only
//! caller: `alloc` -> write bytes -> `decode` -> read `width`/`height`/
//! `pixels` -> `release`.
//!
//! Output is RGBA8, tightly packed, row-major - the same shape decodePNG
//! and decodeJPEG return, so all three feed the image pipeline without a
//! per-format branch downstream. Unlike JPEG, alpha here is REAL: a WebP
//! can carry it, so the shim normalizes RGB -> RGBA rather than forcing
//! opaque, and `output_buffer_size` is what distinguishes the two.
//!
//! This is a `std` build, unlike the JPEG shim next door. See Cargo.toml.

use std::io::Cursor;

use image_webp::WebPDecoder;

/// The input buffer JS writes encoded bytes into, and the decoded RGBA8
/// output. Both are held across calls because `decode` returns only a
/// status code - the caller reads geometry and pixels through the
/// accessors below, then calls `release`.
struct State {
    input: Vec<u8>,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    /// Frame count of an animation this decoder refused, so the JS error
    /// can name it. Zero for a still image.
    frames: u32,
}

static mut STATE: Option<State> = None;

fn state() -> Option<&'static mut State> {
    // Single-threaded by construction: wasm32-unknown-unknown without
    // threads, driven by one JS caller.
    unsafe { (*core::ptr::addr_of_mut!(STATE)).as_mut() }
}

/// Reserves `len` bytes for the encoded WebP and returns a pointer into
/// linear memory for JS to write them at. Discards any previous decode.
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut input = Vec::new();
    input.resize(len, 0);
    let ptr = input.as_mut_ptr();
    unsafe {
        STATE = Some(State {
            input,
            pixels: Vec::new(),
            width: 0,
            height: 0,
            frames: 0,
        });
    }
    ptr
}

/// Decodes the bytes previously written into `alloc`'s buffer.
///
/// Returns 0 on success, -1 if `alloc` was never called, -2 if the bytes
/// are not a decodable WebP, -3 if the decoded size does not match the
/// RGB/RGBA output contract, and -4 for an ANIMATED WebP - which is not a
/// corrupt file but a format this slice deliberately does not handle, so
/// it gets its own code and `frames()` carries the count for the message.
#[no_mangle]
pub extern "C" fn decode() -> i32 {
    let Some(st) = state() else { return -1 };

    let Ok(mut decoder) = WebPDecoder::new(Cursor::new(st.input.as_slice())) else {
        return -2;
    };

    // Refused BEFORE reading pixels, not after: read_image on an animated
    // file yields frame 1, and silently rendering the first frame of an
    // animation is exactly the half-support this project's image ground
    // rules forbid. N4 owns animation; until then this is a loud no.
    if decoder.is_animated() {
        st.frames = decoder.num_frames();
        return -4;
    }

    let (w, h) = decoder.dimensions();
    let Some(size) = decoder.output_buffer_size() else {
        return -2;
    };

    let mut buf = Vec::new();
    buf.resize(size, 0);
    if decoder.read_image(&mut buf).is_err() {
        return -2;
    }

    // has_alpha() decides the source layout; the output contract is always
    // RGBA8. Both arms are checked against the geometry rather than
    // trusted, so a decoder that disagreed with its own buffer-size
    // report would surface as a status code instead of a torn image.
    let expected_rgba = (w as usize).saturating_mul(h as usize).saturating_mul(4);
    let expected_rgb = (w as usize).saturating_mul(h as usize).saturating_mul(3);

    if size == expected_rgba {
        st.pixels = buf;
    } else if size == expected_rgb {
        let mut rgba = Vec::new();
        rgba.resize(expected_rgba, 0xFF);
        for i in 0..(w as usize).saturating_mul(h as usize) {
            rgba[i * 4] = buf[i * 3];
            rgba[i * 4 + 1] = buf[i * 3 + 1];
            rgba[i * 4 + 2] = buf[i * 3 + 2];
            // alpha stays 0xFF from the resize above
        }
        st.pixels = rgba;
    } else {
        return -3;
    }

    st.width = w;
    st.height = h;
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

/// Frame count of the animation `decode` refused with -4. Zero otherwise.
#[no_mangle]
pub extern "C" fn frames() -> u32 {
    state().map_or(0, |s| s.frames)
}

/// The text of a nonzero `decode` status other than -4 (whose message
/// names the frame count), UTF-8 at `status_message` with
/// `status_message_length` bytes; "unknown decoder status" for another
/// value.
fn message(status: i32) -> &'static str {
    match status {
        -1 => "internal error: decode() called before alloc()",
        -2 => "not a decodable WebP (truncated, corrupt, or an unsupported variant)",
        -3 => "internal error: decoder returned a buffer that is neither width*height*3 nor *4",
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
