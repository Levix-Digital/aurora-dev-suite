//! What waits for the user across every project, the most urgent first (spec 005,
//! FEAT-017). The items are the Portfolio's waiting items, read the same way.

use std::collections::HashMap;

use aurora_core::domain::{CycleError, Feature, Inbox as InboxItems, InboxEntry, ProjectList};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::loader::{Loader, MAX_PARALLEL_READS};
use super::reads::{RowState, Rows, read};
use super::{CycleFailure, FeatureState, StatusLine, Tone};
use crate::app::AppContext;
use crate::backend::CommandError;

#[component]
pub fn Inbox() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let list = RwSignal::new(None::<ProjectList>);
    let message = RwSignal::new(None::<(Tone, String)>);
    let rows: Rows = RwSignal::new(HashMap::new());
    let loader = StoredValue::new(Loader::new(MAX_PARALLEL_READS));

    spawn_local(async move {
        let backend = ctx.backend.get_value();
        match backend.list_projects().await {
            Ok(projects) => {
                let ids: Vec<String> = projects.projects.iter().map(|p| p.id.0.clone()).collect();
                loader.update_value(|l| l.set_listed(ids.clone()));
                list.set(Some(projects));
                read(backend, loader, rows, ids);
            }
            Err(error) => message.set(Some((Tone::Error, error.message))),
        }
    });
    let refresh = move |_| {
        let ids = list
            .get_untracked()
            .map(|l| l.projects.into_iter().map(|p| p.id.0).collect())
            .unwrap_or_default();
        read(ctx.backend.get_value(), loader, rows, ids);
    };

    // The projects in the registry's order, with their names, for the core's order.
    let projects = move || {
        list.with(|list| {
            list.as_ref()
                .map(|l| {
                    l.projects
                        .iter()
                        .map(|p| (p.id.0.clone(), p.name.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
    };
    let inbox = Memo::new(move |_| {
        let projects = projects();
        rows.with(|rows| {
            let read: Vec<_> = projects
                .iter()
                .filter_map(|(id, name)| match rows.get(id) {
                    Some(RowState::Read { summary, .. }) => {
                        Some((id.as_str(), name.as_str(), summary))
                    }
                    _ => None,
                })
                .collect();
            InboxItems::of(read)
        })
    });
    let failed = Memo::new(move |_| {
        let projects = projects();
        rows.with(|rows| {
            projects
                .into_iter()
                .filter_map(|(id, name)| match rows.get(&id) {
                    Some(RowState::Failed { error, .. }) => Some((id, name, error.clone())),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
    });

    view! {
        <div class="view-header">
            <h1>"Inbox"</h1>
            <div class="actions">
                <button class="btn" on:click=refresh>"Refresh"</button>
            </div>
        </div>
        <FeatureState feature=Feature::Inbox />
        <StatusLine message=message />
        {move || match list.get() {
            None => view! { <p class="muted">"Loading the projects…"</p> }.into_any(),
            Some(l) if l.projects.is_empty() => view! {
                <div class="card empty">
                    <h2>"No projects yet"</h2>
                    <p>"Add a Git repository in the Portfolio to see what waits for you here."</p>
                </div>
            }
            .into_any(),
            Some(_) => view! {
                <Totals inbox=inbox rows=rows />
                {move || inbox.with(|inbox| {
                    inbox
                        .groups()
                        .into_iter()
                        .map(|(urgency, entries)| {
                            let entries: Vec<InboxEntry> = entries.into_iter().cloned().collect();
                            view! {
                                <section class="card section inbox-group">
                                    <h2>{format!("{} ({})", urgency.label(), entries.len())}</h2>
                                    <ul class="items">
                                        {entries.into_iter().map(|entry| view! { <Entry entry=entry /> }).collect_view()}
                                    </ul>
                                </section>
                            }
                        })
                        .collect_view()
                })}
                <NotRead failed=failed />
            }
            .into_any(),
        }}
    }
}

/// How many items wait across how many projects, how many projects are still being read,
/// and whether part of a project could not be read (spec 005 FR-006).
#[component]
fn Totals(inbox: Memo<InboxItems>, rows: Rows) -> impl IntoView {
    move || {
        let (items, projects) = inbox.with(|inbox| (inbox.entries.len(), inbox.project_count()));
        let (reading, with_findings) = rows.with(|rows| {
            let reading = rows.values().filter(|row| row.is_reading()).count();
            let with_findings = rows
                .values()
                .filter(|row| matches!(row, RowState::Read { summary, .. } if summary.has_findings))
                .count();
            (reading, with_findings)
        });
        let mut text = match (items, projects) {
            (0, _) => "Nothing waits for you".to_owned(),
            (1, _) => "1 item waits for you".to_owned(),
            (n, 1) => format!("{n} items wait for you in 1 project"),
            (n, p) => format!("{n} items wait for you across {p} projects"),
        };
        if reading > 0 {
            text.push_str(&format!(" (reading {reading}…)"));
        }
        view! {
            <p class="totals" aria-live="polite">{text}</p>
            {(with_findings > 0).then(|| view! {
                <p class="muted">
                    "Part of a project could not be read; open it in the Project view for the findings."
                </p>
            })}
        }
    }
}

/// One waiting item: its project, kind, title, detail and "Open".
#[component]
fn Entry(entry: InboxEntry) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let InboxEntry {
        project_id,
        project_name,
        item,
    } = entry;
    let label = format!("Open {project_name}");
    view! {
        <li class="inbox-item">
            <div>
                <span class="badge info">{project_name}</span>" "
                <span class="muted">{item.kind.name()}</span>" "
                <strong>{item.title}</strong>
                {item.detail.map(|detail| view! { <div class="muted">{detail}</div> })}
            </div>
            <button class="btn" aria-label=label on:click=move |_| ctx.open_project(project_id.clone())>
                "Open"
            </button>
        </li>
    }
}

/// The projects whose cycle state could not be read, so an empty Inbox is never mistaken
/// for nothing to do (spec 005 FR-007).
#[component]
fn NotRead(failed: Memo<Vec<(String, String, CommandError)>>) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    move || {
        let failed = failed.get();
        if failed.is_empty() {
            return None;
        }
        let unavailable = failed
            .iter()
            .all(|(_, _, error)| matches!(error.detail, Some(CycleError::Unavailable { .. })));
        Some(if unavailable {
            view! {
                <section class="card section">
                    <h2>"Not read"</h2>
                    <p class="muted">"Cycle state unavailable."</p>
                </section>
            }
            .into_any()
        } else {
            view! {
                <section class="card section inbox-group">
                    <h2>{format!("Not read ({})", failed.len())}</h2>
                    <ul class="items">
                        {failed.into_iter().map(|(id, name, error)| view! {
                            <li class="inbox-item">
                                <div>
                                    <strong>{name.clone()}</strong>" "
                                    <CycleFailure error=error />
                                </div>
                                <button
                                    class="btn"
                                    aria-label=format!("Open {name}")
                                    on:click=move |_| ctx.open_project(id.clone())
                                >
                                    "Open"
                                </button>
                            </li>
                        }).collect_view()}
                    </ul>
                </section>
            }
            .into_any()
        })
    }
}
