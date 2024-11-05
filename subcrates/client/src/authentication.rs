use crypto::encryption::symmetric;
use leptos::{
    component, create_node_ref, create_signal, event_target_checked, view, IntoView, NodeRef,
    ReadSignal, Show, SignalGet, SignalSet, SignalWith, WriteSignal,
};

#[component]
pub fn User(
    metadata: ReadSignal<Option<symmetric::MetaData>>,
    set_encryption: WriteSignal<Option<symmetric::Encryption>>,
) -> impl IntoView {
    view! {
        <Show when=move || metadata.with(Option::is_some) fallback=|| ()>
            <Login metadata=metadata set_encryption=set_encryption />
        </Show>
        <Register set_encryption=set_encryption />
    }
}

#[component]
fn Login(
    metadata: ReadSignal<Option<symmetric::MetaData>>,
    set_encryption: WriteSignal<Option<symmetric::Encryption>>,
) -> impl IntoView {
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
        metadata.with(|metadata| {
            if let Ok(encryption) = symmetric::Encryption::load(
                username.as_bytes(),
                password.as_bytes(),
                metadata.as_ref().expect("Metadata to exist"),
            ) {
                set_encryption.set(Some(encryption));
                set_error.set(None);
            } else {
                set_error.set(Some("Username or password is is incrrect.".to_owned()));
            }
        });
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
fn Register(set_encryption: WriteSignal<Option<symmetric::Encryption>>) -> impl IntoView {
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
            set_error.set(Some("Passwords do not match.".to_owned()));
            return;
        }
        if let Ok(new_encryption) =
            symmetric::Encryption::new(username.as_bytes(), password.as_bytes())
        {
            set_error.set(None);
            set_encryption.set(Some(new_encryption));
        } else {
            set_error.set(Some("Username or password is is incrrect.".to_owned()));
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
