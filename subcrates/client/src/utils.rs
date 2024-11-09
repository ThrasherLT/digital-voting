use leptos::{component, view, IntoView, Show, Signal, SignalGet, SignalWith};
use leptos_use::{use_clipboard, UseClipboardReturn};

#[allow(non_snake_case)]
#[component]
pub fn Copyable(value: Signal<Option<String>>) -> impl IntoView {
    let UseClipboardReturn {
        is_supported,
        copied,
        copy,
        ..
    } = use_clipboard();
    view! {
        <Show when=move || value.with(Option::is_some) fallback=move || ()>
            <input type="text" value=move || value.get() readonly />
        </Show>
        <Show when=move || is_supported.get() && value.with(Option::is_some) fallback=|| ()>
            <button on:click={
                let copy = copy.clone();
                move |_| copy(&value.get().expect("Copyable value to be some after Show"))
            }>
                <Show when=move || copied.get() fallback=move || "Copy">
                    "Copied!"
                </Show>
            </button>
        </Show>
    }
}

// TODO leptos_use doesn't currently really support creating a paste button, but it shouldn't be too
// complicated to either create a PR for it or create it here locally.
