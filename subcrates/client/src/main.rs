use leptos::{
    component,
    mount::mount_to_body,
    prelude::{ElementChild, Read, RwSignal, Show, Signal},
    view, IntoView,
};
use states::user::User;

mod authentication;
mod blockchain_selection;
mod candidate_selection;
mod fetch;
mod settings;
mod states;
mod storage;
mod utils;
mod validation;
mod verification;
mod vote;

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
    let blockchain = RwSignal::new(String::new());
    let user_state = RwSignal::new(Option::<User>::None);
    let get_user = Signal::derive(move || {
        user_state
            .read_only()
            .read()
            .as_ref()
            .expect("User to have existed by now")
            .to_owned()
    });

    view! {
        <h3>"Untitled Voting System"</h3>
        <Show when=move || user_state.read().is_none() fallback=|| ()>
            <authentication::Authentication user_state=user_state />
        </Show>
        <Show when=move || user_state.read().is_some() fallback=|| ()>
            <settings::SettingsPanel user=user_state />
        </Show>
        <Show
            when=move || user_state.read().is_some() && blockchain.read().is_empty()
            fallback=|| ()
        >
            <blockchain_selection::SelectBlockchain
                user=user_state
                set_blockchain=blockchain.write_only()
            />
        </Show>
        <Show when=move || !blockchain.read().is_empty() fallback=|| ()>
            <vote::Vote user=get_user blockchain=blockchain />
        </Show>
    }
}
