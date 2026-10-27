//! The registered projects: add, rename, remove, locate (FR-005 to FR-010).

use std::path::PathBuf;

use aurora_core::domain::{AddOutcome, Feature, Presence, Project, ProjectList, RegistryNotice};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::{FeatureState, Message, StatusLine, Tone};
use crate::app::AppContext;
use crate::backend::Reply;

#[component]
pub fn Portfolio() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let list = RwSignal::new(None::<ProjectList>);
    let message = RwSignal::new(None::<(Tone, String)>);
    let path = RwSignal::new(String::new());

    let reload = move || {
        spawn_local(async move {
            match ctx.backend.get_value().list_projects().await {
                Ok(projects) => list.set(Some(projects)),
                Err(error) => message.set(Some((Tone::Error, error.message))),
            }
        });
    };
    reload();
    let reload = Callback::new(move |()| reload());

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
            <button class="btn primary" on:click=pick>"Add a project…"</button>
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
                    <For
                        each=move || list.get().map(|l| l.projects).unwrap_or_default()
                        key=|p: &Project| format!("{}|{}|{}|{:?}", p.id, p.name, p.root.display(), p.presence)
                        children=move |project: Project| {
                            view! { <ProjectRow project=project reload=reload message=message /> }
                        }
                    />
                }
                    .into_any()
            }
        }}
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
fn ProjectRow(project: Project, reload: Callback<()>, message: RwSignal<Message>) -> impl IntoView {
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
            {move || match mode.get() {
                RowMode::Idle => {
                    let start_rename = start_rename.clone();
                    let start_locate = start_locate.clone();
                    view! {
                        <div class="actions">
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
            <p class="muted">"Cycle state: coming in v0.2.0 with the Dev Cycle Protocol."</p>
        </div>
    }
}
