use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use leptos::{
    component,
    prelude::{
        signal, ClassAttribute, ElementChild, For, Get, NodeRef, NodeRefAttribute, OnAttribute,
        Read, RwSignal, Set, Show, Signal, Update, WriteSignal,
    },
    task::spawn_local,
    view, IntoView,
};
use protocol::config::ElectionConfig;

use crate::{fetch, states::user::User};

fn new_blockchain(
    new_blockchain_addr: String,
    set_user: RwSignal<Option<User>>,
    blockchain_config: ElectionConfig,
) -> Result<()> {
    let mut res = Ok(());

    set_user.update(|user| {
        if let Some(user) = user {
            if let Err(e) = user.add_blockchain(new_blockchain_addr.clone(), blockchain_config) {
                res = Err(anyhow!(format!("Error fetching blockchain configs: {e}")));
            }
        } else {
            res = Err(anyhow!("Internal user error"));
        }
    });

    res
}

fn delete_blockchain(blockchain_addr: &str, set_user: &mut Option<User>) -> Result<()> {
    let Some(user) = set_user else {
        bail!("Internal user error");
    };
    user.remove_blockchain(blockchain_addr)
}

#[component]
pub fn SelectBlockchain(
    user: RwSignal<Option<User>>,
    set_blockchain: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <h4>"Blockchain Node Selection"</h4>
        <BlockchainList user=user set_blockchain=set_blockchain />
        <NewBlockchain user=user />
    }
}

#[component]
fn NewBlockchain(user: RwSignal<Option<User>>) -> impl IntoView {
    let (get_error, set_error) = signal(Option::<String>::None);
    let new_blockchain_addr_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let new_blockchain_addr = new_blockchain_addr_ref
            .get()
            .expect("New blockchain input should be mounted")
            .value();

        spawn_local(async move {
            match fetch::blockchain_config(new_blockchain_addr.clone(), Duration::from_secs(5))
                .await
            {
                Ok(blockchain_config) => {
                    if let Err(e) = new_blockchain(new_blockchain_addr, user, blockchain_config) {
                        set_error.set(Some(format!("Error setting up new blockchain: {e}")))
                    } else {
                        set_error.set(None);
                    }
                }
                Err(e) => set_error.set(Some(format!("Error fetching blockchain configs: {e}"))),
            };
        })
    };

    view! {
        <form on:submit=on_submit>
            <label>
                "New Blockchain URL:"
                <input
                    type="text"
                    name="new_blockchain"
                    node_ref=new_blockchain_addr_ref
                    placeholder="Enter new blockchain URL"
                />
            </label>
            <button type="submit">"Add new"</button>
        </form>

        <Show when=move || get_error.read().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}

#[component]
fn BlockchainList(
    user: RwSignal<Option<User>>,
    set_blockchain: WriteSignal<String>,
) -> impl IntoView {
    let (get_error, set_error) = signal(None);

    let blockchain_list = Signal::derive(move || {
        let blockchain_list: Vec<RwSignal<String>> = user
            .read()
            .as_ref()
            .map(|user| user.blockchains.clone())
            .unwrap_or_default()
            .iter()
            .map(|blockchain| RwSignal::new(blockchain.clone()))
            .collect();
        blockchain_list
    });

    view! {
        <ul>
            <For each=move || blockchain_list.get() key=|blockchain| blockchain.clone() let:child>
                <li>
                    // TODO create separate component for buttons with confirmation prompt.
                    <button on:click=move |_| {
                        set_blockchain.set(child.get());
                    }>{child}</button>
                    <button on:click=move |_| {
                        user.update(|mut user| {
                            if let Err(e) = delete_blockchain(&child.read(), &mut user) {
                                set_error.set(Some(format!("Failed to remove blockchain: {e}")))
                            } else {
                                set_error.set(None);
                            }
                        });
                    }>"Delete"</button>
                </li>
            </For>
        </ul>

        <Show when=move || get_error.read().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}
