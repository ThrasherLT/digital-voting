pub mod commitment;
pub mod set_membership_zkp;
pub mod signature;
mod utils;

// Configuration for wasm-bindgen-test to run tests in browser.
#[cfg(test)]
mod tests {
    #![cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
}
