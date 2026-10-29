use aurora_core::domain::Feature;
use leptos::prelude::*;

use super::ComingLater;

#[component]
pub fn ProjectView() -> impl IntoView {
    view! {
        <ComingLater
            title="Project"
            what="One project's development cycle: its phase, plan, milestones, gates and release candidates, read from the Dev Cycle Protocol."
            release="v0.2.0"
            feature=Feature::ProjectView
        />
    }
}
