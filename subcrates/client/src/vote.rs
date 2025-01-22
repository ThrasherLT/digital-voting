use leptos::{
    component,
    prelude::{signal, ElementChild, Get, OnAttribute, Read, RwSignal, Set, Show, Signal},
    view, IntoView,
};

use crate::{
    candidate_selection,
    states::{access_tokens::AccessTokens, candidate::Candidate, user::User},
    validation, verification,
};

#[must_use]
#[component]
pub fn Vote(user: Signal<User>, blockchain: RwSignal<String>) -> impl IntoView {
    let access_tokens = RwSignal::new(
        AccessTokens::load(&user.read(), &blockchain.read()).expect("Access tokens to be loaded"),
    );
    let (candidate, set_candidate) =
        signal(Candidate::load(&user.read(), &blockchain.read()).expect("Candidate to load"));

    view! {
        <button on:click=move |_| {
            if candidate.get().is_some() {
                Candidate::delete(&user.read(), &blockchain.read());
            }
            set_candidate.set(None);
        }>"DEBUG: Reset candidate"</button>

        <button on:click=move |_| {
            blockchain.set(String::new());
        }>"Back to blockchain select"</button>

        <Show when=move || !access_tokens.read().is_complete() fallback=|| ()>
            <validation::Validation
                user=user
                blockchain=blockchain.read_only()
                access_tokens=access_tokens
            />
        </Show>
        <Show
            when=move || access_tokens.read().is_complete() && candidate.get().is_none()
            fallback=|| ()
        >
            <candidate_selection::CandidateSelection
                user=user
                blockchain=blockchain.read_only()
                access_tokens=access_tokens.read_only()
                set_candidate=set_candidate
            />
        </Show>
        <Show when=move || candidate.get().is_some() fallback=|| ()>
            <verification::Verification candidate=candidate />
        </Show>
    }
}
