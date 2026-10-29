use aurora_core::domain::Feature;
use leptos::prelude::*;

use super::ComingLater;

#[component]
pub fn Inbox() -> impl IntoView {
    view! {
        <ComingLater
            title="Inbox"
            what="The decisions waiting for you across every project: gates to approve, release candidates to accept and blocking bugs."
            release="v0.2.0"
            feature=Feature::Inbox
        />
    }
}
