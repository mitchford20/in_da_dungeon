#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
