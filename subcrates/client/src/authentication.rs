// TODO Docummentation
// TODO local storage only works on pages that have the same origin

use crate::state::State;
use leptos::{
    component, create_node_ref, create_signal, event_target_checked, expect_context, view,
    IntoView, NodeRef, Show, SignalGet, SignalSet,
};

#[component]
pub fn User() -> impl IntoView {
    view! {
        <Show when=State::can_login fallback=|| ()>
            <Login />
        </Show>
        <Register />
    }
}

#[component]
fn Login() -> impl IntoView {
    let mut state = expect_context::<State>();
    let (get_error, set_error) = create_signal(None);

    let (password_visible, set_password_visible) = create_signal(false);
    let get_password_visibility = move || {
        if password_visible.get() {
            "text"
        } else {
            "password"
        }
    };
    let username_ref: NodeRef<leptos::html::Input> = create_node_ref();
    let password_ref: NodeRef<leptos::html::Input> = create_node_ref();
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
        if let Err(e) = state.login_user(&username, &password) {
            // TODO Better error reporting:
            set_error.set(Some(format!("Error occured: {e}")));
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
        <Show when=move || get_error.get().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}

#[component]
fn Register() -> impl IntoView {
    let (get_error, set_error) = create_signal(None);
    let (password_visible, set_password_visible) = create_signal(false);
    let get_password_visibility = move || {
        if password_visible.get() {
            "text"
        } else {
            "password"
        }
    };
    let username_ref: NodeRef<leptos::html::Input> = create_node_ref();
    let password_ref: NodeRef<leptos::html::Input> = create_node_ref();
    let repeat_password_ref: NodeRef<leptos::html::Input> = create_node_ref();
    let mut state = expect_context::<State>();
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
        if let Err(e) = state.register_user(&username, &password) {
            // TODO Better error reporting:
            set_error.set(Some(format!("Error occured: {e}")));
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
        <Show when=move || get_error.get().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}
