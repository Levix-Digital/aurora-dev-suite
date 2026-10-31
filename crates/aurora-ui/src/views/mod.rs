//! The views of the window. Each section shows ready, degraded, unavailable or "not in
//! this release" (FR-004).

pub mod engines;
pub mod inbox;
pub mod loader;
pub mod portfolio;
pub mod project;
pub mod project_settings;
pub mod reads;
pub mod settings;

use aurora_core::domain::{AvailabilityState, CycleError, EngineStatus, Feature};
use leptos::prelude::*;

use crate::app::AppContext;
use crate::backend::CommandError;

/// How a message reads: the colour of its badge or banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Ok,
    Info,
    Warn,
    Error,
}

impl Tone {
    pub const fn class(self) -> &'static str {
        match self {
            Tone::Ok => "ok",
            Tone::Info => "info",
            Tone::Warn => "warn",
            Tone::Error => "error",
        }
    }
}

pub type Message = Option<(Tone, String)>;

/// A live region for the result of the last action.
#[component]
pub fn StatusLine(message: RwSignal<Message>) -> impl IntoView {
    view! {
        <div class="status-line" role="status" aria-live="polite">
            {move || {
                message
                    .get()
                    .map(|(tone, text)| {
                        view! {
                            <div class=format!("banner {}", tone.class())>{sentence(&text)}</div>
                        }
                    })
            }}
        </div>
    }
}

/// Error messages from the core start in lower case; the interface shows sentences.
pub fn sentence(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

pub fn status_badge(status: &EngineStatus) -> (Tone, &'static str) {
    match status {
        EngineStatus::Ready => (Tone::Ok, "Ready"),
        EngineStatus::InstalledWithoutContract => (Tone::Warn, "Installed without contract"),
        EngineStatus::Degraded { .. } => (Tone::Error, "Degraded"),
        EngineStatus::Missing => (Tone::Info, "Missing"),
    }
}

pub fn availability_text(state: &AvailabilityState) -> (Tone, String) {
    match state {
        AvailabilityState::Available => (Tone::Ok, "Available".to_owned()),
        AvailabilityState::Unavailable { missing, hint, .. } if missing.is_empty() => {
            (Tone::Warn, hint.clone())
        }
        AvailabilityState::Unavailable { missing, hint, .. } => {
            (Tone::Warn, format!("{hint} (needs {})", missing.join(", ")))
        }
        AvailabilityState::NotInThisRelease { release } => {
            (Tone::Info, format!("Coming in {release}"))
        }
    }
}

/// What a feature needs, from the last engine overview.
#[component]
pub fn FeatureState(feature: Feature) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    move || {
        let state = ctx.engines.get();
        let found = state
            .overview
            .as_ref()
            .and_then(|o| o.features.iter().find(|f| f.feature == feature).cloned());
        match found {
            Some(found) => {
                let (tone, text) = availability_text(&found.state);
                view! {
                    <p>
                        <span class=format!("badge {}", tone.class())>{feature.display_name()}</span>
                        " "
                        {text}
                    </p>
                }
                .into_any()
            }
            None => view! { <p class="muted">"Checking the engines…"</p> }.into_any(),
        }
    }
}

/// Why a project's cycle state could not be read, with the engine's error output to open;
/// the same text in the Portfolio and the Project view (spec 003 FR-007, spec 004 FR-008).
#[component]
pub fn CycleFailure(error: CommandError) -> impl IntoView {
    let text = match &error.detail {
        Some(CycleError::Unavailable { .. }) => "Cycle state unavailable.".to_owned(),
        Some(CycleError::ProjectMissing { .. }) => {
            "Locate the folder to read its cycle state.".to_owned()
        }
        _ => sentence(&error.message),
    };
    let stderr = error
        .detail
        .as_ref()
        .and_then(CycleError::stderr)
        .map(str::to_owned);
    view! {
        <span class="muted">{text}</span>
        {stderr.map(|stderr| view! {
            <details>
                <summary>"Engine output"</summary>
                <pre>{stderr}</pre>
            </details>
        })}
    }
}
