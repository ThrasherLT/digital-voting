use std::time::Duration;

use anyhow::{anyhow, Result};
use crypto::signature::blind_sign;
use futures::future::{select, Either};
use leptos::{
    component,
    html::ElementChild,
    mount::mount_to_body,
    prelude::{
        signal, ClassAttribute, Get, NodeRef, NodeRefAttribute, OnAttribute, Read, ReadSignal, Set,
        Show, With,
    },
    task::spawn_local,
    view, IntoView,
};
use leptos_use::{use_clipboard, UseClipboardReturn};
use reqwasm::http::Response;

#[component]
fn App() -> impl IntoView {
    view! {
        <h1>"Welcome to the mock election authority!"</h1>
        <p>"This is used only for testing and demonstration purposes."</p>
        <BlindSigning />
    }
}

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(App);
}

#[component]
fn BlindSigning() -> impl IntoView {
    let (get_error, set_error) = signal(Option::<String>::None);
    let binded_user_pk_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let (blind_signature, set_blind_signature) = signal(Option::<blind_sign::BlindSignature>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let blinded_user_pk = binded_user_pk_ref
            .get()
            .expect("Blinded user public key should be mounted")
            .value();

        spawn_local(async move {
            let current_url = web_sys::window()
                .and_then(|win| win.location().host().ok())
                .unwrap_or_else(|| "Unknown".to_string());

            match post(
                format!("{{\"blinded_pkey\":\"{}\"}}", blinded_user_pk),
                &format!("http://{}/authenticate", current_url),
                Duration::from_secs(5),
            )
            .await
            {
                Ok(response) => {
                    if response.status() != 200 {
                        set_error.set(Some(format!(
                            "Failed to reach server: {}",
                            response.status()
                        )));
                    }
                    match response.json().await {
                        Ok(blind_signature) => {
                            set_blind_signature.set(Some(blind_signature));
                        }
                        Err(e) => set_error.set(Some(format!("Failed to parse response: {e}"))),
                    }
                }
                Err(e) => set_error.set(Some(format!("Request failed: {e}"))),
            }
        })
    };

    view! {
        <form on:submit=on_submit>
            <label>
                "Blinded User Public Key:"
                <input
                    type="text"
                    name="blinded_pk"
                    node_ref=binded_user_pk_ref
                    placeholder="Enter blinded user public key"
                />
            </label>
            <button type="submit">"Sign"</button>
        </form>
        <Copyable value=blind_signature />

        <Show when=move || get_error.read().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}

async fn post(payload: String, addr: &str, timeout: Duration) -> Result<Response> {
    let fetch_future = std::pin::pin!(async {
        match reqwasm::http::Request::post(&addr)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(_) => Err(anyhow!("Request failed".to_string())),
        }
    });

    let timeout_future = gloo_timers::future::TimeoutFuture::new(timeout.as_millis().try_into()?);

    match select(fetch_future, timeout_future).await {
        Either::Left((result, _)) => result,
        Either::Right(_) => Err(anyhow!("Request timed out".to_string())),
    }
}

#[allow(non_snake_case)]
#[component]
pub fn Copyable(value: ReadSignal<Option<blind_sign::BlindSignature>>) -> impl IntoView {
    let UseClipboardReturn {
        is_supported,
        copied,
        copy,
        ..
    } = use_clipboard();
    view! {
        <Show when=move || value.with(Option::is_some) fallback=move || ()>
            <input
                type="text"
                value=move || value.get().expect("Value to be some at this point").to_string()
                readonly
            />
        </Show>
        <Show when=move || is_supported.get() && value.with(Option::is_some) fallback=|| ()>
            <button on:click={
                let copy = copy.clone();
                move |_| copy(
                    &value.get().expect("Copyable value to be some after Show").to_string(),
                )
            }>
                <Show when=move || copied.get() fallback=move || "Copy">
                    "Copied!"
                </Show>
            </button>
        </Show>
    }
}
