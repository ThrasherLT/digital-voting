//! File containing code which handles the user settings for the browser extension.

use crate::states::user::User;
use leptos::{
    component,
    prelude::{signal, ElementChild, Get, OnAttribute, RwSignal, Set, Show, Update},
    view, IntoView,
};

#[component]
pub fn SettingsPanel(user: RwSignal<Option<User>>) -> impl IntoView {
    let (show_settings, set_show_settings) = signal(false);

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
            <Show when=move || user.get().is_some() fallback=move || ()>
                <User user=user />
            </Show>
        </Show>
    }
}

#[component]
fn User(user: RwSignal<Option<User>>) -> impl IntoView {
    let (double_check, set_double_check) = signal(false);

    view! {
        <button on:click=move |_| {
            user.set(None);
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
                user.update(|user| {
                    let mut user = user.take();
                    user.map(|mut user| user.delete());
                });
            }>"Yes, I'm sure"</button>
            <button on:click=move |_| {
                set_double_check.set(false);
            }>"No, cancel"</button>
        </Show>
    }
}
