//! The engines: status, version, capabilities, what each enables, last error, refresh
//! (FR-011 to FR-016).

use aurora_core::capabilities;
use aurora_core::domain::{EngineKind, EngineReport, EngineStatus, EnginesOverview};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::{availability_text, sentence, status_badge};
use crate::app::AppContext;

#[component]
pub fn Engines() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let refresh = move |_| {
        ctx.engines.update(|state| {
            state.refreshing = true;
            state.error = None;
        });
        spawn_local(async move {
            if let Err(error) = ctx.backend.get_value().refresh_engines().await {
                ctx.engines.update(|state| {
                    state.refreshing = false;
                    state.error = Some(error.message);
                });
            }
        });
    };

    view! {
        <div class="view-header">
            <h1>"Engines"</h1>
            <button
                class="btn primary"
                on:click=refresh
                prop:disabled=move || ctx.engines.get().refreshing
            >
                {move || if ctx.engines.get().refreshing { "Checking…" } else { "Refresh" }}
            </button>
        </div>
        <p class="muted">
            "Aurora asks each engine's binary for its version and capabilities. Set a location or a time limit in Settings."
        </p>
        <div role="status" aria-live="polite">
            {move || ctx.engines.get().error.map(|e| view! { <div class="banner error">{sentence(&e)}</div> })}
        </div>
        {move || {
            let state = ctx.engines.get();
            match state.overview {
                None => view! { <p class="muted">"Checking the engines…"</p> }.into_any(),
                Some(overview) => view! { <Overview overview=overview /> }.into_any(),
            }
        }}
    }
}

#[component]
fn Overview(overview: EnginesOverview) -> impl IntoView {
    let cards = EngineKind::ALL
        .into_iter()
        .map(|kind| {
            let report = overview.reports.iter().find(|r| r.kind == kind).cloned();
            view! { <EngineCard kind=kind report=report overview=overview.clone() /> }
        })
        .collect_view();
    let rows = overview
        .features
        .iter()
        .map(|availability| {
            let (tone, text) = availability_text(&availability.state);
            view! {
                <tr>
                    <td>{availability.feature.display_name()}</td>
                    <td>
                        <span class=format!("badge {}", tone.class())>{text}</span>
                    </td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <div class="grid">{cards}</div>
        <h2>"Features"</h2>
        <table>
            <thead>
                <tr>
                    <th scope="col">"Feature"</th>
                    <th scope="col">"State"</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>
    }
}

#[component]
fn EngineCard(
    kind: EngineKind,
    report: Option<EngineReport>,
    overview: EnginesOverview,
) -> impl IntoView {
    let Some(report) = report else {
        return view! {
            <section class="card" aria-label=kind.display_name()>
                <h2>{kind.display_name()}</h2>
                <p class="muted">"Checking…"</p>
            </section>
        }
        .into_any();
    };
    let (tone, label) = status_badge(&report.status);
    let explanation = match &report.status {
        EngineStatus::Ready => None,
        EngineStatus::InstalledWithoutContract => Some(format!(
            "{} answers --version but not `{} version --json`; update it to a version with the JSON contract.",
            kind.display_name(),
            kind.command()
        )),
        EngineStatus::Degraded { reason } => Some(sentence(reason)),
        EngineStatus::Missing => Some(format!(
            "`{}` was not found on the search path. Install the {}, or set its location in Settings.",
            kind.command(),
            kind.display_name()
        )),
    };
    let enables = capabilities::features_of(kind)
        .into_iter()
        .map(|feature| {
            let state = overview
                .features
                .iter()
                .find(|f| f.feature == feature)
                .map(|f| availability_text(&f.state));
            let (tone, text) = state.unwrap_or((super::Tone::Info, "Unknown".to_owned()));
            view! {
                <li>
                    {feature.display_name()} ": "
                    <span class=format!("badge {}", tone.class())>{text}</span>
                </li>
            }
        })
        .collect_view();
    let capabilities = report
        .capabilities
        .iter()
        .map(|c| view! { <code>{format!("{} v{}", c.name, c.schema_major)}</code> })
        .collect_view();

    view! {
        <section class="card" aria-label=kind.display_name()>
            <div class="row">
                <h2>{kind.display_name()}</h2>
                <span class=format!("badge {}", tone.class())>{label}</span>
            </div>
            <p class="muted">
                <code>{kind.command()}</code>
                {report.version.clone().map(|v| format!(" · version {v}"))}
            </p>
            {report.location.clone().map(|l| view! { <p class="path muted">{l.display().to_string()}</p> })}
            {explanation.map(|text| view! { <p>{text}</p> })}
            {(!report.capabilities.is_empty()).then(|| view! {
                <h3>"Capabilities"</h3>
                <div class="caps">{capabilities}</div>
            })}
            <h3>"Enables"</h3>
            <ul class="plain">{enables}</ul>
            {report.last_error.clone().map(|error| view! {
                <details>
                    <summary>"Last error output"</summary>
                    <pre>{error}</pre>
                </details>
            })}
            <p class="muted">{format!("Checked at {}", report.checked_at)}</p>
        </section>
    }
    .into_any()
}
