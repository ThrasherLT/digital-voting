//! Crate which describes the protocol and fundamental operation of the blockchain.

pub mod config;
pub mod timestamp;
pub mod vote;

// Configuration for wasm-bindgen-test to run tests in browser.
#[cfg(test)]
mod tests {
    #![cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
}
