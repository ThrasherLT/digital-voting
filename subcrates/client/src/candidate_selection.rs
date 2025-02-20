use std::{str::FromStr, time::Duration};

use anyhow::{bail, Result};
use leptos::{
    component,
    prelude::{
        event_target_value, signal, ClassAttribute, CollectView, ElementChild, Get,
        GlobalAttributes, OnAttribute, Read, ReadSignal, Set, Show, Signal, WriteSignal,
    },
    task::spawn_local,
    view, IntoView,
};
use protocol::{config::CandidateId, vote::Vote};

use crate::{
    fetch,
    states::{
        access_tokens::AccessTokens, candidate::Candidate, config::Config, signature::Signature,
        user::User,
    },
};

fn send_vote(
    user: Signal<User>,
    blockchain: ReadSignal<String>,
    node: ReadSignal<Option<String>>,
    candidate_id: ReadSignal<Option<CandidateId>>,
    access_tokens: ReadSignal<AccessTokens>,
    set_candidate: WriteSignal<Option<Candidate>>,
    set_error: WriteSignal<Option<String>>,
) -> Result<()> {
    let Some(candidate_id) = candidate_id.get() else {
        bail!("Candidate ID is not selected");
    };
    let Some(node) = node.get() else {
        bail!("Blockchain node not selected");
    };

    let vote = Vote::new(
        &Signature::load(&user.read(), &blockchain.read())?.signer,
        candidate_id,
        chrono::Utc::now(),
        access_tokens.read().prepare()?,
    )?;

    spawn_local(async move {
        if let Err(e) = fetch::submit_vote(node.clone(), Duration::from_secs(5), vote).await {
            set_error.set(Some(format!("Failed to submit vote: {e}")));
        } else {
            match Candidate::choose(candidate_id, &user.read(), &node) {
                Ok(candidate) => set_candidate.set(Some(candidate)),
                Err(e) => set_error.set(Some(format!("Failed to save candidate: {e}"))),
            }
        }
    });

    Ok(())
}

#[component]
pub fn CandidateSelection(
    user: Signal<User>,
    blockchain: ReadSignal<String>,
    set_candidate: WriteSignal<Option<Candidate>>,
    access_tokens: ReadSignal<AccessTokens>,
) -> impl IntoView {
    let (get_error, set_error) = signal(None);
    let (selected_candidate, set_selected) = signal(None);
    let (selected_node, set_selected_node) = signal(None);

    let (config, _) = signal(
        Config::load(&user.read(), &blockchain.read()).expect("Config to exist at this point"),
    );
    // TODO Extra clone is required here to appease the borrow checker, but maybe it's possible to avoid it?
    let node_options = config.read().get_nodes().clone();
    let node_options = node_options
        .iter()
        .map(|node| {
            view! { <option value=node.clone()>{node.clone()}</option> }
        })
        .collect_view();

    let candidates = config
        .read()
        .get_candidates()
        .into_iter()
        .map(|candidate| {
            view! {
                <label>
                    {candidate.name.clone()}
                    <input
                        type="radio"
                        name="single-select"
                        value=candidate.id
                        on:change=move |ev| {
                            match CandidateId::from_str(&event_target_value(&ev)) {
                                Ok(candidate_id) => set_selected.set(Some(candidate_id)),
                                Err(e) => {
                                    set_error.set(Some(format!("Candidate selection failed: {e}")))
                                }
                            }
                        }
                    />
                </label>
            }
        })
        .collect_view();

    view! {
        <h4>"Election Candidate Selection"</h4>

        <label for="node_select">"Select blockchain node: "</label>
        <select
            id="node_select"
            on:change=move |ev| set_selected_node.set(Some(event_target_value(&ev)))
        >
            <option value="" selected=selected_node.read().is_none() disabled>
                "Please select a node"
            </option>
            {node_options}
        </select>
        <p>"Selected node: " {selected_node}</p>

        <label>"Select the candidate for whom you wish to vote for:" {candidates}</label>
        <button
            disabled=move || selected_candidate.read().is_none() && selected_node.read().is_none()
            on:click=move |_| {
                if let Err(e) = send_vote(
                    user,
                    blockchain,
                    selected_node,
                    selected_candidate,
                    access_tokens,
                    set_candidate,
                    set_error,
                ) {
                    set_error.set(Some(format!("Failed to submit vote: {e}")))
                }
            }
        >
            "Confirm Candidate"
        </button>
        <Show when=move || get_error.get().is_some() fallback=|| ()>
            <p class="error">{move || get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}
