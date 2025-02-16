use std::{str::FromStr, time::Duration};

use anyhow::{bail, Result};
use leptos::{
    component,
    prelude::{
        event_target_value, signal, ClassAttribute, CollectView, ElementChild, Get, OnAttribute,
        Read, ReadSignal, Set, Show, Signal, WriteSignal,
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
    candidate_id: ReadSignal<Option<CandidateId>>,
    access_tokens: ReadSignal<AccessTokens>,
    set_candidate: WriteSignal<Option<Candidate>>,
    set_error: WriteSignal<Option<String>>,
) -> Result<()> {
    let Some(candidate_id) = candidate_id.get() else {
        bail!("Candidate ID is not selected");
    };

    let vote = Vote::new(
        &Signature::load(&user.read(), &blockchain.read())?.signer,
        candidate_id,
        chrono::Utc::now(),
        access_tokens.read().prepare()?,
    )?;

    // For some reason `leptos` throws a "time not implemented on this platform" error,
    // if we read from any signal within the `spawn_local` future.
    let addr = blockchain.get().to_string();
    let user = user.read();

    spawn_local(async move {
        if let Err(e) = fetch::submit_vote(addr.clone(), Duration::from_secs(5), vote).await {
            set_error.set(Some(format!("Failed to submit vote: {e}")));
        } else {
            match Candidate::choose(candidate_id, &user, &addr) {
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
    let (selected, set_selected) = signal(None);

    let config =
        Config::load(&user.read(), &blockchain.read()).expect("Config to exist at this point");
    let candidates = config
        .election_config
        .candidates
        .into_iter()
        .map(|candidate| {
            view! {
                <label>
                    {candidate.name}
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

        <label>"Select the candidate for whom you wish to vote for:" {candidates}</label>

        <button
            disabled=move || selected.read().is_none()
            on:click=move |_| {
                if let Err(e) = send_vote(
                    user,
                    blockchain,
                    selected,
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
