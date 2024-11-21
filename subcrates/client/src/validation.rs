//! This file contains the logic for validating the voter's right to vote.

use std::str::FromStr;

use crypto::signature::blind_sign;
use leptos::{
    component, create_node_ref, create_signal, expect_context, html, view, IntoView, NodeRef, Show,
    Signal, SignalGet, SignalSet, SignalWith,
};

use crate::state::State;
use crate::utils;

#[must_use]
#[component]
pub fn ValidateVoter() -> impl IntoView {
    let state = expect_context::<State>();

    view! {
        <AuthKeyInput />
        <BlindedPkDisplay />
        <Show when=move || state.get_blinded_pk().with(Option::is_some) fallback=|| ()>
            <UnblindAccessToken />
        </Show>
    }
}

#[must_use]
#[component]
pub fn AuthKeyInput() -> impl IntoView {
    let mut state = expect_context::<State>();
    let (get_error, set_error) = create_signal(None);

    let authority_pk_ref: NodeRef<html::Input> = create_node_ref();
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // stop the page from reloading:
        ev.prevent_default();
        let authority_pk = authority_pk_ref
            .get()
            .expect("Authority public key input should be mounted")
            .value();

        match blind_sign::PublicKey::from_str(&authority_pk) {
            Ok(authority_pk) => {
                if let Err(e) = state.blind(authority_pk) {
                    // TODO better error reporting:
                    set_error.set(Some(format!("Error had occured: {e}")));
                }
            }
            Err(e) => {
                set_error.set(Some(format!("Invalid election authority public key: {e}")));
            }
        }
    };

    view! {
        <form on:submit=on_submit>
            <h3>"Blinded Public Key Setup"</h3>
            <label>
                "Enter the public key of the election authority:"
                <input
                    type="text"
                    node_ref=authority_pk_ref
                    name="authority_pk"
                    placeholder="Paste the public key of the election authority here"
                /> <button type="submit">Blind</button>
            </label>
        </form>
        <Show when=move || get_error.get().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}

#[must_use]
#[component]
pub fn BlindedPkDisplay() -> impl IntoView {
    let state = expect_context::<State>();
    // TODO Check if nothing is overwriting this value, saw some flicker on the screen.
    let display_value =
        Signal::derive(move || state.get_blinded_pk().get().map(|val| val.to_string()));

    view! { <utils::Copyable value=display_value /> }
}

#[must_use]
#[component]
pub fn UnblindAccessToken() -> impl IntoView {
    let mut state = expect_context::<State>();
    let (get_error, set_error) = create_signal(None);

    let input_ref: NodeRef<html::Input> = create_node_ref();
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // stop the page from reloading!
        ev.prevent_default();

        // here, we'll extract the value from the input
        let blind_signature = input_ref.get().expect("Input to be mounted").value();

        match blind_sign::BlindSignature::from_str(&blind_signature) {
            Ok(blind_signature) => {
                if let Err(e) = state.unblind(blind_signature) {
                    // TODO better error reporting:
                    set_error.set(Some(format!("Error had occured: {e}")));
                }
            }
            Err(e) => {
                set_error.set(Some(format!("Invalid blinded authority signature: {e}")));
            }
        }
    };

    view! {
        <form on:submit=on_submit>
            <label>
                "Enter the blind signature of the election authority:"
                <input type="text" node_ref=input_ref /> <input type="submit" value="Unblind" />
            </label>
        </form>
        <Show when=move || get_error.get().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}
