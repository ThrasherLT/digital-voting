use leptos::*;

fn main() {
    tracing_subscriber::fmt()
        .with_writer(
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .without_time()
        .init();
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <h1>"Hello World!"</h1> })
}
