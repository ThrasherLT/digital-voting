use leptos::{
    component,
    html::ElementChild,
    prelude::{Get, ReadSignal},
    view, IntoView,
};

use crate::states::candidate::Candidate;

#[component]
pub fn Verification(candidate: ReadSignal<Option<Candidate>>) -> impl IntoView {
    view! {
        <h4>"Election Verification"</h4>
        <p>{format!("You have successfully voted for {:?}", candidate.get())}</p>
        <p>"WIP"</p>
    }
}
