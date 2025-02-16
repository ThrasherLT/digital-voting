use std::str::FromStr;

use crypto::signature::blind_sign;
use leptos::{
    component,
    html::{self, ElementChild},
    prelude::{
        signal, ClassAttribute, CollectView, Get, NodeRef, NodeRefAttribute, OnAttribute, Read,
        ReadSignal, RwSignal, Set, Show, Signal, Update,
    },
    view, IntoView,
};

use crate::{
    states::{
        access_tokens::AccessTokens, config::Config, signature::Signature, user::User,
        validators::Validators,
    },
    utils,
};

// TODO Figure out a clean way to handle errors and excepts.
// TODO Blockchain validation must apply to all nodes of the same blockchain.

#[component]
pub fn Validation(
    user: Signal<User>,
    blockchain: ReadSignal<String>,
    access_tokens: RwSignal<AccessTokens>,
) -> impl IntoView {
    let config = Config::load(&user.read(), &blockchain.read()).expect("Config to be loaded");
    let (signature, _) =
        signal(Signature::load(&user.read(), &blockchain.read()).expect("Signature to be loaded"));

    let (validators, _) = signal(
        Validators::load(&config.election_config, &user.read(), &blockchain.read())
            .expect("Validators to be loaded"),
    );

    let entries = config
        .get_authorities()
        .into_iter()
        .zip(validators.read().get_blinded_pks())
        .enumerate()
        .map(|(i, (authority, blinded_pk))| {
            let (blinded_pk, _) = signal(Some(format!("{}", blinded_pk)));
            let (get_error, set_error) = signal(None);
            let blind_signature_ref: NodeRef<html::Input> = NodeRef::new();

            let on_submit = move |ev: leptos::ev::SubmitEvent| {
                // Stop the page from reloading:
                ev.prevent_default();
                let blind_signature = blind_signature_ref
                    .get()
                    .expect("Input to be mounted")
                    .value();

                match blind_sign::BlindSignature::from_str(&blind_signature) {
                    Ok(blind_signature) => {
                        match validators
                            .read()
                            .validate(&signature.read(), i, blind_signature)
                        {
                            Ok(access_token) => access_tokens.update(|access_tokens| {
                                if let Err(e) = access_tokens.set(
                                    &user.read(),
                                    &blockchain.read(),
                                    i,
                                    Some(access_token),
                                ) {
                                    set_error.set(Some(format!("Failed to save access token: {e}")))
                                }
                            }),
                            Err(e) => set_error.set(Some(format!(
                                "Invalid valid blind_signature in position {i}: {e}"
                            ))),
                        }
                    }
                    Err(e) => set_error.set(Some(format!(
                        "Not a valid blind_signature in position {i}: {e}"
                    ))),
                }
            };
            let link = authority.clone();
            view! {
                <a href=link target="_blank">
                    {authority}
                </a>
                <Show
                    when=move || access_tokens.read().get(i).map(|ac| ac.is_none()).unwrap_or(false)
                    fallback=|| view! { <p>"Access token acquired!"</p> }
                >
                    <utils::Copyable value=blinded_pk />
                    <form on:submit=on_submit>
                        <label>
                            "Enter the blind signature from the election authority:"
                            <input type="text" node_ref=blind_signature_ref />
                            <input type="submit" value="Validate" />
                        </label>
                    </form>
                    <Show when=move || get_error.get().is_some() fallback=|| ()>
                        <p class="error">{move || get_error.get().expect("Error to be some")}</p>
                    </Show>
                </Show>
            }
        })
        .collect_view();

    view! { <ul>{entries}</ul> }
}
