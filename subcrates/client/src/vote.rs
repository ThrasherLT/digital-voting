//! This file contains the logic for casting an actual vote.

use leptos::{
    component, create_node_ref, create_signal, event_target_value, expect_context, html, view,
    IntoView, NodeRef, Show, SignalGet, SignalSet,
};

use crate::state::State;

#[component]
pub fn Cast() -> impl IntoView {
    let mut state = expect_context::<State>();
    let (get_error, set_error) = create_signal(None);
    // Temporarily holding the selected candidate in this signal before commiting it to the state.
    let (candidate, set_candidate) = create_signal(Option::<String>::None);

    let blockchain_addr_ref: NodeRef<html::Input> = create_node_ref();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // stop the page from reloading:
        ev.prevent_default();
        let blockchain_addr = blockchain_addr_ref
            .get()
            .expect("Blockchain URL input should be mounted")
            .value();

        if blockchain_addr.is_empty() {
            set_error.set(Some("Blockchain URL cannot be empty".to_owned()));
            return;
        }
        let selected_candidate = candidate.get();
        match selected_candidate {
            Some(selected_candidate) => {
                if let Err(e) = state.vote(&selected_candidate, &blockchain_addr) {
                    set_error.set(Some(format!("Failed to vote: {e}")));
                }
            }
            None => set_error.set(Some("Candidate not selected".to_owned())),
        }
    };

    view! {
        <label>
            <form on:submit=on_submit>
                <label>
                    "Enter the URL of the blockchain:"
                    <input
                        type="text"
                        node_ref=blockchain_addr_ref
                        name="blockchain_url"
                        placeholder="Paste the URL of the blockchain here"
                    />
                </label>
                <label>
                    "Select the ID of the candidate for whom you wish to vote for"
                    <select
                        on:change=move |ev| {
                            set_candidate.set(Some(event_target_value(&ev)));
                        }
                        prop:value=move || candidate.get()
                    >
                        <Config />
                    </select> <button type="submit">Vote</button>
                </label>
            </form>
        </label>
        <Show when=move || get_error.get().is_some() fallback=|| ()>
            <p class="error">{get_error.get().expect("Error to be some")}</p>
        </Show>
    }
}

#[component]
fn Config() -> impl IntoView {
    // TODO For now candidate config is hardcoded:
    view! {
        <option value="0">"First Candidate"</option>
        <option value="1">"Second Candidate"</option>
        <option value="2">"Third Candidate"</option>
    }
}
