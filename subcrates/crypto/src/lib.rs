pub mod commitment;
pub mod encryption;
pub mod hash_storage;
pub mod merkle;
pub mod signature;
pub(crate) mod utils;

// Configuration for wasm-bindgen-test to run tests in browser.
#[cfg(test)]
mod tests {
    #![cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
}
