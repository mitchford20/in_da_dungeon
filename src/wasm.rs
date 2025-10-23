//! Helpers for WebAssembly builds. By default Rust panics just call `abort` in WASM; installing a
//! panic hook pipes the message into the browser console instead, aiding debugging.

#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
