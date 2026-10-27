//! Theme and per-engine location and time limit (FR-003, FR-011).

use std::path::PathBuf;

use aurora_core::domain::{EngineKind, EngineSettings, Settings, ThemeChoice};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::{StatusLine, Tone};
use crate::app::AppContext;

fn engine_mut(settings: &mut Settings, kind: EngineKind) -> &mut EngineSettings {
    match kind {
        EngineKind::DevCycle => &mut settings.engines.dev_cycle,
        EngineKind::Gedo => &mut settings.engines.gedo,
        EngineKind::AnyContext => &mut settings.engines.any_context,
    }
}

#[component]
pub fn SettingsView() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let draft = RwSignal::new(ctx.settings.get_untracked());
    let message = RwSignal::new(None::<(Tone, String)>);

    let save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let settings = draft.get_untracked();
        spawn_local(async move {
            match ctx.backend.get_value().save_settings(settings).await {
                Ok(saved) => {
                    ctx.settings.set(saved);
                    message.set(Some((
                        Tone::Ok,
                        "Settings saved. Refresh the Engines view to use new locations.".to_owned(),
                    )));
                }
                Err(error) => message.set(Some((Tone::Error, error.message))),
            }
        });
    };

    let themes = [
        (ThemeChoice::System, "Follow the system"),
        (ThemeChoice::Light, "Light"),
        (ThemeChoice::Dark, "Dark"),
    ]
    .into_iter()
    .map(|(choice, label)| {
        view! {
            <label>
                <input
                    type="radio"
                    name="theme"
                    prop:checked=move || draft.get().theme == choice
                    on:change=move |_| draft.update(|s| s.theme = choice)
                />
                " " {label}
            </label>
            " "
        }
    })
    .collect_view();

    let engines = EngineKind::ALL
        .into_iter()
        .map(|kind| {
            let path_id = format!("path-{}", kind.command());
            let timeout_id = format!("timeout-{}", kind.command());
            view! {
                <fieldset>
                    <legend>{kind.display_name()}</legend>
                    <div class="field">
                        <label for=path_id.clone()>{format!("Location of {}", kind.command())}</label>
                        <input
                            id=path_id
                            type="text"
                            placeholder="Empty: look on the search path"
                            prop:value=move || {
                                draft.get().engines.get(kind).path.as_ref()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_default()
                            }
                            on:change=move |ev| {
                                let value = event_target_value(&ev).trim().to_owned();
                                draft.update(|s| {
                                    engine_mut(s, kind).path = (!value.is_empty()).then(|| PathBuf::from(value));
                                });
                            }
                        />
                    </div>
                    <div class="field">
                        <label for=timeout_id.clone()>"Time limit (seconds)"</label>
                        <input
                            id=timeout_id
                            type="number"
                            min="1"
                            max="120"
                            prop:value=move || (draft.get().engines.get(kind).timeout_ms / 1000).to_string()
                            on:change=move |ev| {
                                if let Ok(seconds) = event_target_value(&ev).trim().parse::<u64>() {
                                    draft.update(|s| engine_mut(s, kind).timeout_ms = seconds.clamp(1, 120) * 1000);
                                }
                            }
                        />
                    </div>
                </fieldset>
            }
        })
        .collect_view();

    view! {
        <div class="view-header">
            <h1>"Settings"</h1>
        </div>
        <form on:submit=save>
            <fieldset>
                <legend>"Theme"</legend>
                {themes}
            </fieldset>
            <h2>"Engines"</h2>
            {engines}
            <button class="btn primary" type="submit">"Save"</button>
        </form>
        <StatusLine message=message />
    }
}
