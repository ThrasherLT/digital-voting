//! This is the main entry point of the client for the whole blockchain.
//! The client is implemented as a browser extension, but may be conveniently
//! build into a website for quick development and testing with hot reloading.
//! The browser extension had been set up to work with manifest version 3.

use leptos::{
    component, expect_context, mount_to_body, provide_context, view, IntoView, Show, Signal,
    SignalWith,
};

mod authentication;
mod settings;
mod state;
mod storage;
mod utils;
mod validation;
mod vote;

use state::{State, Status};

// Configuration for wasm-bindgen-test to run tests in browser.
#[cfg(test)]
mod tests {
    #![cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .init();
    console_error_panic_hook::set_once();

    mount_to_body(App);
}

#[must_use]
#[component]
pub fn App() -> impl IntoView {
    provide_context(State::new());
    let state = expect_context::<State>();

    let status = Signal::derive(move || state.get_status());

    view! {
        <Show when=move || status.with(|status| matches!(status, Status::LoggedOut)) fallback=|| ()>
            <authentication::User />
        </Show>
        <Show when=move || status.with(|status| { *status > Status::LoggedOut }) fallback=|| ()>
            <settings::SettingsPanel />
        </Show>
        <Show when=move || status.with(|status| matches!(status, Status::LoggedIn)) fallback=|| ()>
            <validation::ValidateVoter />
        </Show>
        <Show when=move || status.with(|status| matches!(status, Status::Validated)) fallback=|| ()>
            <vote::Cast />
        </Show>
    }
}
