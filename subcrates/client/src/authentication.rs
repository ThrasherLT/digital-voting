//! File for the code required to authenticate the user.

// TODO Docummentation
// TODO local storage only works on pages that have the same origin

use crate::states::user::User;
use leptos::{
    component,
    prelude::{
        event_target_checked, signal, ClassAttribute, ElementChild, Get, NodeRef, NodeRefAttribute,
        OnAttribute, Read, RwSignal, Set, Show,
    },
    view, IntoView,
};

#[component]
pub fn Authentication(user_state: RwSignal<Option<User>>) -> impl IntoView {
    view! {
        <Login user_state=user_state />
        <Register user_state=user_state />
    }
}

#[component]
fn Login(user_state: RwSignal<Option<User>>) -> impl IntoView {
    let (get_error, set_error) = signal(None);

    let (password_visible, set_password_visible) = signal(false);
    let get_password_visibility = move || {
        if *password_visible.read() {
            "text"
        } else {
            "password"
        }
    };
    let username_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let password_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let username = username_ref
            .get()
            .expect("Username should be mounted")
            .value();
        let password = password_ref
            .get()
            .expect("Password should be mounted")
            .value();
        match User::login(username, &password) {
            Ok(user) => user_state.set(Some(user)),
            Err(e) => set_error.set(Some(format!("Error occured: {e}"))),
        }
    };

    view! {
        <h3>Login</h3>
        <form on:submit=on_submit>
            <label>
                "Username:"
                <input
                    type="text"
                    name="username"
                    node_ref=username_ref
                    placeholder="Enter your username"
                />
            </label>
            <label>
                "Password:"
                <input
                    type=get_password_visibility
                    name="password"
                    node_ref=password_ref
                    placeholder="Enter your password"
                />
            </label>
            <label>
                <input
                    type="checkbox"
                    on:change=move |e| set_password_visible.set(event_target_checked(&e))
                />
                "Show Password"
            </label>
            <button type="submit">Login</button>
        </form>
        <Show when=move || get_error.read().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}

#[component]
fn Register(user_state: RwSignal<Option<User>>) -> impl IntoView {
    let (get_error, set_error) = signal(None);
    let (password_visible, set_password_visible) = signal(false);
    let get_password_visibility = move || {
        if *password_visible.read() {
            "text"
        } else {
            "password"
        }
    };
    let username_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let password_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let repeat_password_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let username = username_ref
            .get()
            .expect("Username should be mounted")
            .value();
        let password = password_ref
            .get()
            .expect("Password should be mounted")
            .value();
        let repeat_password = repeat_password_ref
            .get()
            .expect("Repeat password should be mounted")
            .value();
        if password != repeat_password {
            set_error.set(Some("Passwords do not match".to_owned()));
            return;
        }
        if password.is_empty() || repeat_password.is_empty() || username.is_empty() {
            set_error.set(Some("Username and password cannot be empty".to_owned()));
            return;
        }
        match User::register(username, &password) {
            Ok(user) => user_state.set(Some(user)),
            Err(e) => set_error.set(Some(format!("Error occured: {e}"))),
        }
    };

    view! {
        <form on:submit=on_submit>
            <h3>Register</h3>
            <label>
                "Username:"
                <input
                    type="text"
                    name="username"
                    node_ref=username_ref
                    placeholder="Enter your username"
                />
            </label>
            <label>
                "Password:"
                <input
                    type=get_password_visibility
                    name="password"
                    node_ref=password_ref
                    placeholder="Enter your password"
                />
                <input
                    type=get_password_visibility
                    name="repeat_password"
                    node_ref=repeat_password_ref
                    placeholder="Repeat your password"
                />
            </label>
            <label>
                <input
                    type="checkbox"
                    on:change=move |e| set_password_visible.set(event_target_checked(&e))
                />
                "Show Password"
            </label>
            <button type="submit">Register</button>
        </form>
        <Show when=move || get_error.read().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}
