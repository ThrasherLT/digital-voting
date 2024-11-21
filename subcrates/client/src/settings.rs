//! File containing code which handles the user settings for the browser extension.

use crate::state::State;
use leptos::{
    component, create_signal, expect_context, view, IntoView, Show, SignalGet, SignalSet,
};

#[component]
pub fn SettingsPanel() -> impl IntoView {
    let (show_settings, set_show_settings) = create_signal(false);

    view! {
        <Show
            when=move || show_settings.get()
            fallback=move || {
                view! {
                    <button on:click=move |_| {
                        set_show_settings.set(true);
                    }>"Show settings"</button>
                }
            }
        >
            <button on:click=move |_| {
                set_show_settings.set(false);
            }>"Hide settings"</button>
            <User />
        </Show>
    }
}

#[component]
pub fn User() -> impl IntoView {
    let (double_check, set_double_check) = create_signal(false);

    view! {
        <button on:click=move |_| {
            let mut state = expect_context::<State>();
            state.logout();
        }>"Logout"</button>
        <Show
            when=move || double_check.get()
            fallback=move || {
                view! {
                    <button on:click=move |_| {
                        set_double_check.set(true);
                    }>"Delete User"</button>
                }
            }
        >
            <button on:click=move |_| {
                set_double_check.set(true);
            }>"Delete User"</button>
            <p>"Are you sure? This cannot be undone!"</p>
            <button on:click=move |_| {
                let mut state = expect_context::<State>();
                state.delete_user();
            }>"Yes, I'm sure"</button>
            <button on:click=move |_| {
                set_double_check.set(false);
            }>"No, cancel"</button>
        </Show>
    }
}
