//! One project's development cycle: every section of its cycle state, the gates and the
//! next step, with the Portfolio's phase and waiting rules (spec 004, FEAT-015).

use aurora_core::domain::{
    CycleState, CycleSummary, Feature, Gate, ItemRef, Presence, ProjectList,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::portfolio::{phase_tone, release_text, waiting_text};
use super::{CycleFailure, FeatureState, StatusLine, Tone};
use crate::app::AppContext;
use crate::backend::CommandError;

/// What the view knows of the chosen project's cycle state.
#[derive(Debug, Clone, PartialEq)]
enum Detail {
    Reading,
    Read {
        state: Box<CycleState>,
        refreshing: bool,
    },
    Failed {
        error: CommandError,
        refreshing: bool,
    },
}

impl Detail {
    /// The state while a new read runs: the last answer stays, marked as refreshing.
    fn refreshing(self) -> Self {
        match self {
            Detail::Read { state, .. } => Detail::Read {
                state,
                refreshing: true,
            },
            Detail::Failed { error, .. } => Detail::Failed {
                error,
                refreshing: true,
            },
            Detail::Reading => Detail::Reading,
        }
    }
}

#[component]
pub fn ProjectView() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let list = RwSignal::new(None::<ProjectList>);
    let message = RwSignal::new(None::<(Tone, String)>);
    let detail = RwSignal::new(Detail::Reading);
    // The number of the latest read; an answer to an earlier one is dropped (research R4).
    let latest = StoredValue::new(0_u64);

    let read = move |id: String, refresh: bool| {
        let token = latest.get_value() + 1;
        latest.set_value(token);
        detail.update(|detail| {
            *detail = if refresh {
                std::mem::replace(detail, Detail::Reading).refreshing()
            } else {
                Detail::Reading
            };
        });
        spawn_local(async move {
            let answer = ctx.backend.get_value().project_cycle_state(id).await;
            if latest.get_value() != token {
                return;
            }
            detail.set(match answer {
                Ok(state) => Detail::Read {
                    state: Box::new(state),
                    refreshing: false,
                },
                Err(error) => Detail::Failed {
                    error,
                    refreshing: false,
                },
            });
        });
    };

    // The registered projects; the choice falls back to the first one when it is gone.
    spawn_local(async move {
        match ctx.backend.get_value().list_projects().await {
            Ok(projects) => {
                let chosen = ctx.project.get_untracked();
                let known = chosen
                    .as_ref()
                    .is_some_and(|id| projects.projects.iter().any(|p| &p.id.0 == id));
                if !known {
                    ctx.project
                        .set(projects.projects.first().map(|p| p.id.0.clone()));
                }
                list.set(Some(projects));
            }
            Err(error) => message.set(Some((Tone::Error, error.message))),
        }
    });
    Effect::new(move |_| {
        if let Some(id) = ctx.project.get()
            && list.with(Option::is_some)
        {
            read(id, false);
        }
    });

    let refresh = move |_| {
        if let Some(id) = ctx.project.get_untracked() {
            read(id, true);
        }
    };
    let chosen = move || {
        let id = ctx.project.get()?;
        list.with(|list| {
            list.as_ref()?
                .projects
                .iter()
                .find(|p| p.id.0 == id)
                .cloned()
        })
    };

    view! {
        <div class="view-header">
            <h1>{move || chosen().map_or_else(|| "Project".to_owned(), |p| p.name.to_string())}</h1>
            <div class="actions">
                {move || list.get().filter(|l| !l.projects.is_empty()).map(|l| view! {
                    <label>
                        "Project "
                        <select on:change=move |ev| ctx.project.set(Some(event_target_value(&ev)))>
                            {l.projects.into_iter().map(|p| {
                                let selected = ctx.project.get_untracked().as_deref() == Some(p.id.0.as_str());
                                view! { <option value=p.id.0.clone() selected=selected>{p.name.to_string()}</option> }
                            }).collect_view()}
                        </select>
                    </label>
                    <button class="btn" on:click=refresh>"Refresh"</button>
                })}
            </div>
        </div>
        <FeatureState feature=Feature::ProjectView />
        <StatusLine message=message />
        {move || match list.get() {
            None => view! { <p class="muted">"Loading the projects…"</p> }.into_any(),
            Some(l) if l.projects.is_empty() => view! {
                <div class="card empty">
                    <h2>"No projects yet"</h2>
                    <p>"Add a Git repository in the Portfolio to see its cycle here."</p>
                </div>
            }
            .into_any(),
            Some(_) => view! {
                {move || chosen().map(|p| view! {
                    <div class="path muted">{p.root.display().to_string()}</div>
                    {(p.presence == Presence::Missing).then(|| view! {
                        <span class="badge warn">"Missing: folder moved or deleted"</span>
                    })}
                })}
                {move || match detail.get() {
                    Detail::Reading => view! { <p class="muted">"Reading the cycle state…"</p> }.into_any(),
                    Detail::Read { state, refreshing } => view! {
                        {refreshing.then(|| view! { <p class="muted">"Refreshing…"</p> })}
                        <Sections state=*state />
                    }
                    .into_any(),
                    Detail::Failed { error, refreshing } => view! {
                        <div class="card cycle">
                            <CycleFailure error=error />
                            {refreshing.then(|| view! { <span class="muted">"Refreshing…"</span> })}
                        </div>
                    }
                    .into_any(),
                }}
            }
            .into_any(),
        }}
    }
}

/// An item with its priority or severity, for example "BUG-012 (high) Merge falls back".
fn item_text(item: &ItemRef) -> String {
    match &item.rank {
        Some(rank) => format!("{} ({rank}) {}", item.id, item.title),
        None => format!("{} {}", item.id, item.title),
    }
}

/// A card with a heading, or one line when there is nothing to show (spec 004 US1 8).
#[component]
fn Section(title: &'static str, empty: Option<&'static str>, children: Children) -> impl IntoView {
    view! {
        <section class="card section">
            <h2>{title}</h2>
            {match empty {
                Some(text) => view! { <p class="muted">{text}</p> }.into_any(),
                None => children().into_any(),
            }}
        </section>
    }
}

/// Every section of one cycle state (spec 004 FR-004).
#[component]
fn Sections(state: CycleState) -> impl IntoView {
    let summary = CycleSummary::of(&state);
    let gates = Gate::all(&state);
    let options: Vec<_> = state
        .next
        .ordered()
        .into_iter()
        .map(|(option, recommended)| (option.clone(), recommended))
        .collect();
    let release = state.release.clone();
    let open_items = release
        .as_ref()
        .map(|r| r.open_items.clone())
        .unwrap_or_default();

    view! {
        <section class="card section">
            <h2>"Overview"</h2>
            <div class="cycle">
                <span class=format!("badge {}", phase_tone(summary.phase).class())>{summary.phase.label()}</span>
                <span class="release">{release_text(&summary)}</span>
                <strong class="waiting">{waiting_text(&summary)}</strong>
            </div>
            <p>{state.summary.clone()}</p>
            <p class="muted">{format!("Read at {}", state.read_at)}</p>
            {(!state.findings.is_empty()).then(|| view! {
                <p>"Part of the project could not be read:"</p>
                <ul>{state.findings.iter().map(|f| view! { <li>{f.clone()}</li> }).collect_view()}</ul>
            })}
        </section>
        <Section title="Gates" empty=gates.is_empty().then_some("No gate waits for you")>
            <ul class="items">
                {gates.iter().map(|gate| view! {
                    <li><span class="badge warn">{gate.kind.label()}</span>" "{gate.detail.clone()}</li>
                }).collect_view()}
            </ul>
        </Section>
        <Section title="Next step" empty=options.is_empty().then_some("No step recommended")>
            <ol class="items">
                {options.iter().map(|(option, recommended)| view! {
                    <li>
                        {recommended.then(|| view! { <span class="badge ok">"Recommended"</span>" " })}
                        <strong>{option.title.clone()}</strong>
                        {option.reason.clone().map(|reason| view! { <span class="muted">{format!(": {reason}")}</span> })}
                    </li>
                }).collect_view()}
            </ol>
        </Section>
        <Section title="What waits for you" empty=summary.waiting.is_empty().then_some("Nothing waits for you")>
            <ul class="items">
                {summary.waiting.iter().map(|item| view! { <li>{item.title.clone()}</li> }).collect_view()}
            </ul>
        </Section>
        <Section title="Plans and milestones" empty=state.plans.is_empty().then_some("No plans in progress")>
            <ul class="items">
                {state.plans.iter().map(|plan| view! {
                    <li>
                        <strong>{plan.title.clone()}</strong>" "
                        <span class="muted">{format!("({})", plan.status)}</span>
                        <div class="progress-line">
                            <progress max="1" value=plan.fraction() aria-label=format!("{} progress", plan.title)></progress>
                            <span>{plan.progress_label()}</span>
                        </div>
                    </li>
                }).collect_view()}
            </ul>
        </Section>
        <Section title="Roadmap" empty=release.is_none().then_some("No roadmap")>
            {release.as_ref().map(|r| view! { <p><strong>{format!("{} {}", r.version, r.name)}</strong></p> })}
            {if open_items.is_empty() {
                view! { <p class="muted">"No open items in this release"</p> }.into_any()
            } else {
                view! {
                    <ul class="items">
                        {open_items.iter().map(|item| view! { <li>{item_text(item)}</li> }).collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </Section>
        <Section title="Release candidate" empty=state.release_candidate.is_none().then_some("No release candidate")>
            {state.release_candidate.clone().map(|rc| view! {
                <p>
                    <strong>{rc.run.clone()}</strong>
                    {format!(" of {}: {} not run, {} failed", rc.version, rc.not_run, rc.failed)}
                    " "
                    {if rc.released {
                        view! { <span class="badge ok">"Released"</span> }.into_any()
                    } else {
                        view! { <span class="badge warn">"Not released"</span> }.into_any()
                    }}
                </p>
            })}
        </Section>
        <Section title="Pull requests" empty=state.pull_requests.is_empty().then_some("No open pull requests")>
            <ul class="items">
                {state.pull_requests.iter().map(|pr| view! {
                    <li>
                        <strong>{format!("#{} {}", pr.number, pr.title)}</strong>
                        {pr.needs_action.then(|| view! { " "<span class="badge warn">"Needs your action"</span> })}
                        <div class="muted">
                            {pr.summary.clone()}
                            {(pr.threads_for_user > 0).then(|| match pr.threads_for_user {
                                1 => ". 1 review thread for you".to_owned(),
                                n => format!(". {n} review threads for you"),
                            })}
                        </div>
                    </li>
                }).collect_view()}
            </ul>
        </Section>
        <Section title="Blocking bugs" empty=state.blocking_bugs.is_empty().then_some("No blocking bugs")>
            <ul class="items">
                {state.blocking_bugs.iter().map(|bug| view! { <li>{item_text(bug)}</li> }).collect_view()}
            </ul>
        </Section>
        <Section title="Untriaged bugs" empty=state.untriaged_bugs.is_empty().then_some("No untriaged bugs")>
            <ul class="items">
                {state.untriaged_bugs.iter().map(|bug| view! {
                    <li>{item_text(&bug.item)}<span class="muted">{format!(": {}", bug.reason)}</span></li>
                }).collect_view()}
            </ul>
        </Section>
    }
}
