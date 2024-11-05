use leptos::{component, create_signal, mount_to_body, view, IntoView};

mod authentication;

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
    let (_encryption, set_encryption) = create_signal(None);
    let (metadata, _set_metadata) = create_signal(None);

    view! { <authentication::User metadata=metadata set_encryption=set_encryption /> }
}
