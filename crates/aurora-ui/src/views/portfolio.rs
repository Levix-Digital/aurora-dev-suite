//! The registered projects: add, rename, remove, locate (spec 001 FR-005 to FR-010), and
//! each project's cycle summary read in parallel (spec 003, FEAT-014).

use std::collections::HashMap;
use std::path::PathBuf;

use aurora_core::domain::{
    AddOutcome, CyclePhase, CycleSummary, Feature, Presence, Project, ProjectList, RegistryNotice,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::loader::{Loader, MAX_PARALLEL_READS};
use super::reads::{RowState, Rows, read};
use super::{CycleFailure, FeatureState, Message, StatusLine, Tone};
use crate::app::AppContext;
use crate::backend::Reply;

#[component]
pub fn Portfolio() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let list = RwSignal::new(None::<ProjectList>);
    let message = RwSignal::new(None::<(Tone, String)>);
    let path = RwSignal::new(String::new());
    let rows: Rows = RwSignal::new(HashMap::new());
    let loader = StoredValue::new(Loader::new(MAX_PARALLEL_READS));

    // Lists the projects again and reads the ones without a state yet.
    let reload = move || {
        spawn_local(async move {
            let backend = ctx.backend.get_value();
            match backend.list_projects().await {
                Ok(projects) => {
                    let ids: Vec<String> =
                        projects.projects.iter().map(|p| p.id.0.clone()).collect();
                    loader.update_value(|l| l.set_listed(ids.clone()));
                    rows.update(|rows| rows.retain(|id, _| ids.contains(id)));
                    let unread = rows.with_untracked(|rows| {
                        ids.into_iter()
                            .filter(|id| !rows.contains_key(id))
                            .collect()
                    });
                    list.set(Some(projects));
                    read(backend, loader, rows, unread);
                }
                Err(error) => message.set(Some((Tone::Error, error.message))),
            }
        });
    };
    reload();
    let reload = Callback::new(move |()| reload());
    let refresh = move |_| {
        let ids = list
            .get_untracked()
            .map(|l| l.projects.into_iter().map(|p| p.id.0).collect())
            .unwrap_or_default();
        read(ctx.backend.get_value(), loader, rows, ids);
    };

    let report_added = move |result: Reply<Option<AddOutcome>>| match result {
        Ok(Some(outcome)) => {
            let mut text = format!("Added {}.", outcome.project.name);
            if outcome.used_repository_root {
                text.push_str(&format!(
                    " The folder is inside a repository, so its root {} was registered.",
                    outcome.project.root.display()
                ));
            }
            message.set(Some((Tone::Ok, text)));
            reload.run(());
        }
        Ok(None) => {}
        Err(error) => message.set(Some((Tone::Error, error.message))),
    };

    let pick = move |_| {
        spawn_local(async move {
            report_added(ctx.backend.get_value().pick_and_add_project().await);
        });
    };
    let add_typed = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let typed = path.get_untracked().trim().to_owned();
        if typed.is_empty() {
            return;
        }
        spawn_local(async move {
            let result = ctx
                .backend
                .get_value()
                .add_project(PathBuf::from(typed), None)
                .await
                .map(Some);
            if result.is_ok() {
                path.set(String::new());
            }
            report_added(result);
        });
    };

    view! {
        <div class="view-header">
            <h1>"Portfolio"</h1>
            <div class="actions">
                <button class="btn" on:click=refresh>"Refresh"</button>
                <button class="btn primary" on:click=pick>"Add a project…"</button>
            </div>
        </div>
        <FeatureState feature=Feature::Portfolio />
        <form class="row card" on:submit=add_typed>
            <label for="add-path">"Or type the path of a repository"</label>
            <input
                id="add-path"
                type="text"
                placeholder="/home/you/src/project"
                prop:value=move || path.get()
                on:input=move |ev| path.set(event_target_value(&ev))
            />
            <button class="btn" type="submit">"Add"</button>
        </form>
        <StatusLine message=message />
        {move || list.get().and_then(|l| l.notice).map(|notice| view! { <Notice notice=notice /> })}
        {move || match list.get() {
            None => view! { <p class="muted">"Loading the projects…"</p> }.into_any(),
            Some(l) if l.projects.is_empty() => {
                view! {
                    <div class="card empty">
                        <h2>"No projects yet"</h2>
                        <p>"Add the Git repositories you work on to follow them here."</p>
                    </div>
                }
                    .into_any()
            }
            Some(l) => {
                view! {
                    <h2>{match l.projects.len() { 1 => "1 project".to_owned(), n => format!("{n} projects") }}</h2>
                    <Totals rows=rows />
                    <For
                        each=move || list.get().map(|l| l.projects).unwrap_or_default()
                        key=|p: &Project| format!("{}|{}|{}|{:?}", p.id, p.name, p.root.display(), p.presence)
                        children=move |project: Project| {
                            let id = project.id.0.clone();
                            let cycle = Signal::derive(move || rows.with(|rows| rows.get(&id).cloned()));
                            view! { <ProjectRow project=project cycle=cycle reload=reload message=message /> }
                        }
                    />
                }
                    .into_any()
            }
        }}
    }
}

/// What waits across the projects read so far, and how many are still being read.
#[component]
fn Totals(rows: Rows) -> impl IntoView {
    move || {
        rows.with(|rows| {
            let waiting: usize = rows
                .values()
                .filter_map(|row| match row {
                    RowState::Read { summary, .. } => Some(summary.waiting.len()),
                    _ => None,
                })
                .sum();
            let reading = rows.values().filter(|row| row.is_reading()).count();
            let mut text = match waiting {
                0 => "Nothing waits for you".to_owned(),
                1 => "1 item waits for you".to_owned(),
                n => format!("{n} items wait for you"),
            };
            if reading > 0 {
                text.push_str(&format!(" (reading {reading}…)"));
            }
            view! { <p class="totals" aria-live="polite">{text}</p> }
        })
    }
}

#[component]
fn Notice(notice: RegistryNotice) -> impl IntoView {
    match notice {
        RegistryNotice::Recovered { moved_to } => view! {
            <div class="banner warn" role="alert">
                "The saved project list could not be read, so it starts empty. The old file was kept at "
                <span class="path">{moved_to.display().to_string()}</span>
                "."
            </div>
        }
        .into_any(),
        RegistryNotice::ReadOnly { reason } => view! {
            <div class="banner warn" role="alert">
                "Changes to the project list will not be kept after Aurora closes: "
                {reason}
            </div>
        }
        .into_any(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowMode {
    Idle,
    Renaming,
    Locating,
    ConfirmRemove,
}

#[component]
fn ProjectRow(
    project: Project,
    cycle: Signal<Option<RowState>>,
    reload: Callback<()>,
    message: RwSignal<Message>,
) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let mode = RwSignal::new(RowMode::Idle);
    let input = RwSignal::new(String::new());
    let id = project.id.0.clone();
    let name = project.name.to_string();
    let missing = project.presence == Presence::Missing;

    let finish = move |result: Reply<String>| match result {
        Ok(text) => {
            mode.set(RowMode::Idle);
            message.set(Some((Tone::Ok, text)));
            reload.run(());
        }
        Err(error) => message.set(Some((Tone::Error, error.message))),
    };

    let submit = {
        let (id, name) = (id.clone(), name.clone());
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let (id, name, value) = (id.clone(), name.clone(), input.get_untracked());
            let backend = ctx.backend.get_value();
            match mode.get_untracked() {
                RowMode::Renaming => spawn_local(async move {
                    let result = backend.rename_project(id, value).await;
                    finish(result.map(|p| format!("{name} is now {}.", p.name)));
                }),
                RowMode::Locating => spawn_local(async move {
                    let result = backend
                        .relocate_project(id, PathBuf::from(value.trim()))
                        .await;
                    finish(result.map(|p| format!("{} found at {}.", p.name, p.root.display())));
                }),
                _ => {}
            }
        }
    };
    let remove = {
        let (id, name) = (id.clone(), name.clone());
        move |_| {
            let (id, name) = (id.clone(), name.clone());
            spawn_local(async move {
                let result = ctx.backend.get_value().remove_project(id).await;
                finish(result.map(|()| {
                    format!("{name} is no longer followed. Nothing was changed on disk.")
                }));
            });
        }
    };
    let start = {
        let (name, root) = (name.clone(), project.root.display().to_string());
        move |next: RowMode| {
            input.set(match next {
                RowMode::Renaming => name.clone(),
                RowMode::Locating => root.clone(),
                _ => String::new(),
            });
            mode.set(next);
        }
    };
    let (start_rename, start_locate) = (start.clone(), start);

    view! {
        <div class="card">
            <div class="row">
                <div>
                    <h3>{name.clone()}</h3>
                    <div class="path muted">{project.root.display().to_string()}</div>
                </div>
                {if missing {
                    view! { <span class="badge warn">"Missing: folder moved or deleted"</span> }.into_any()
                } else {
                    view! { <span class="badge ok">"Present"</span> }.into_any()
                }}
            </div>
            {move || view! { <CycleLine state=cycle.get() /> }}
            {move || match mode.get() {
                RowMode::Idle => {
                    let start_rename = start_rename.clone();
                    let start_locate = start_locate.clone();
                    let id = id.clone();
                    view! {
                        <div class="actions">
                            <button class="btn primary" on:click=move |_| ctx.open_project(id.clone())>"Open"</button>
                            <button class="btn" on:click=move |_| start_rename(RowMode::Renaming)>"Rename"</button>
                            {missing.then(|| view! {
                                <button class="btn" on:click=move |_| start_locate(RowMode::Locating)>"Locate…"</button>
                            })}
                            <button class="btn danger" on:click=move |_| mode.set(RowMode::ConfirmRemove)>"Remove"</button>
                        </div>
                    }
                        .into_any()
                }
                RowMode::ConfirmRemove => {
                    let remove = remove.clone();
                    view! {
                        <div class="actions" role="group" aria-label="Confirm removal">
                            <span>"Stop following this project? The repository is not touched."</span>
                            <button class="btn danger" on:click=remove>"Remove"</button>
                            <button class="btn" on:click=move |_| mode.set(RowMode::Idle)>"Cancel"</button>
                        </div>
                    }
                        .into_any()
                }
                editing => {
                    let label = if editing == RowMode::Renaming { "New name" } else { "Path of the repository" };
                    let submit = submit.clone();
                    view! {
                        <form class="actions" on:submit=submit>
                            <label>
                                {label} " "
                                <input
                                    type="text"
                                    prop:value=move || input.get()
                                    on:input=move |ev| input.set(event_target_value(&ev))
                                />
                            </label>
                            <button class="btn primary" type="submit">"Save"</button>
                            <button class="btn" type="button" on:click=move |_| mode.set(RowMode::Idle)>"Cancel"</button>
                        </form>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// The current release, as the Portfolio row and the Project view show it.
pub fn release_text(summary: &CycleSummary) -> String {
    summary.release.as_ref().map_or_else(
        || "No roadmap".to_owned(),
        |release| format!("{} {}", release.version, release.name),
    )
}

/// How many things wait for the user, and of which kinds.
pub fn waiting_text(summary: &CycleSummary) -> String {
    if summary.waiting.is_empty() {
        return "Nothing waits for you".to_owned();
    }
    let kinds: Vec<String> = summary
        .waiting_by_kind()
        .into_iter()
        .map(|(kind, count)| kind.label(count))
        .collect();
    let count = summary.waiting.len();
    let verb = if count == 1 { "waits" } else { "wait" };
    format!("{count} {verb} for you: {}", kinds.join(", "))
}

pub const fn phase_tone(phase: CyclePhase) -> Tone {
    match phase {
        CyclePhase::ReleaseCandidate => Tone::Warn,
        CyclePhase::Idle => Tone::Ok,
        _ => Tone::Info,
    }
}

/// One project's cycle summary, or why it has none (spec 003 FR-007).
#[component]
fn CycleLine(state: Option<RowState>) -> impl IntoView {
    let refreshing_note = |refreshing: bool| refreshing.then_some(" Refreshing…");
    match state.unwrap_or(RowState::Reading) {
        RowState::Reading => {
            view! { <p class="cycle muted">"Reading the cycle state…"</p> }.into_any()
        }
        RowState::Read {
            summary,
            refreshing,
        } => {
            let tone = phase_tone(summary.phase);
            let release = release_text(&summary);
            let waiting = waiting_text(&summary);
            view! {
                <div class="cycle">
                    <span class=format!("badge {}", tone.class())>{summary.phase.label()}</span>
                    <span class="release">{release}</span>
                    <strong class="waiting">{waiting}</strong>
                    {summary.has_findings.then(|| view! {
                        <span class="muted">"Part of the project could not be read."</span>
                    })}
                    <span class="muted">{refreshing_note(refreshing)}</span>
                </div>
            }
            .into_any()
        }
        RowState::Failed { error, refreshing } => view! {
            <div class="cycle">
                <CycleFailure error=error />
                <span class="muted">{refreshing_note(refreshing)}</span>
            </div>
        }
        .into_any(),
    }
}
