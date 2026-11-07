//! Messages between projects (spec 008, spec 010): a mailbox of conversations in two
//! tabs, Active and Closed, each row colored by its status; a conversation opens right
//! below its row like an e-mail thread, with a decision card when a closing record waits
//! for the user. Everything comes from the messages worker's snapshot (IMP-007), so the
//! view never waits for the provider to read.

use std::collections::HashSet;

use aurora_core::domain::{
    ClosingRecord, ConversationState, ConversationView, Draft, FOLDED_SECTIONS, Feature,
    InboxSetup, LEAD_SECTIONS, Message, MessageKind, MessageStatus, MessagesError,
    MessagesSnapshot, NEXT_STEP, ProjectInbox, Section, TemplateSection, UserAction,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::{FeatureState, StatusLine, Tone, sentence};
use crate::app::AppContext;
use crate::backend::{Backend, CommandError};

/// The two tabs of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Active,
    Closed,
}

impl Tab {
    pub fn keeps(self, state: ConversationState) -> bool {
        match self {
            Tab::Active => state.is_active(),
            Tab::Closed => !state.is_active(),
        }
    }
}

/// The status filter: every status, or one.
pub fn status_keeps(filter: Option<ConversationState>, state: ConversationState) -> bool {
    filter.is_none_or(|f| f == state)
}

/// `2026-11-06T23:48:00Z` as `2026-11-06 23:48`.
fn short_date(date: &str) -> String {
    date.get(..16).unwrap_or(date).replace('T', " ")
}

/// A refusal or failure, with the sections that are missing.
fn failure_text(error: &CommandError) -> (String, Vec<String>) {
    match error.messages.as_deref() {
        Some(MessagesError::Refused {
            message,
            missing_sections,
        }) => (sentence(message), missing_sections.clone()),
        _ => (sentence(&error.message), Vec::new()),
    }
}

#[component]
fn Failure(error: RwSignal<Option<CommandError>>) -> impl IntoView {
    move || {
        error.get().map(|error| {
            let (text, missing) = failure_text(&error);
            let stderr = match error.messages.as_deref() {
                Some(MessagesError::Git { stderr, .. }) if !stderr.is_empty() => {
                    Some(stderr.clone())
                }
                _ => None,
            };
            view! {
                <div class="banner error" role="alert">
                    <p>{text}</p>
                    {(!missing.is_empty()).then(|| view! {
                        <p>"Fill in: " {missing.join(", ")}</p>
                    })}
                    {stderr.map(|stderr| view! {
                        <details><summary>"Git output"</summary><pre>{stderr}</pre></details>
                    })}
                </div>
            }
        })
    }
}

/// The view's shared state: the snapshot, and what the user did that the next snapshot
/// has not caught up with yet (IMP-007).
#[derive(Clone, Copy)]
struct Mailbox {
    snapshot: RwSignal<Option<MessagesSnapshot>>,
    /// Conversations the user opened: read at once, written behind.
    read: RwSignal<HashSet<String>>,
    /// Conversations whose decision was sent: no longer waiting.
    decided: RwSignal<HashSet<String>>,
    status: RwSignal<Option<(Tone, String)>>,
    /// Kept here because `reload` also runs from timers, outside any reactive owner.
    backend: StoredValue<Backend>,
}

impl Mailbox {
    /// Every conversation once, with what the user just did applied.
    fn views(&self) -> Vec<ConversationView> {
        let (read, decided) = (self.read.get(), self.decided.get());
        self.snapshot
            .get()
            .map(|s| s.views())
            .unwrap_or_default()
            .into_iter()
            .map(|mut v| {
                if read.contains(&v.conversation.thread) {
                    v.conversation.unread = 0;
                }
                if decided.contains(&v.conversation.thread) {
                    v.conversation.waiting_for_user = false;
                }
                v
            })
            .collect()
    }

    /// Asks for the snapshot again (after a write, or "Check now").
    fn reload(self) {
        let backend = self.backend.get_value();
        spawn_local(async move {
            if let Ok(snapshot) = backend.messages_snapshot().await {
                self.snapshot.set(Some(snapshot));
            }
        });
    }

    fn done(self, text: String) {
        self.status.set(Some((Tone::Ok, text)));
        self.reload();
    }
}

#[component]
pub fn MessagesView() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let mailbox = Mailbox {
        snapshot: ctx.messages,
        read: RwSignal::new(HashSet::new()),
        decided: RwSignal::new(HashSet::new()),
        status: RwSignal::new(None),
        backend: ctx.backend,
    };
    if ctx.messages.get_untracked().is_none() {
        mailbox.reload();
    }
    let tab = RwSignal::new(Tab::Active);
    let project = RwSignal::new(String::new());
    let filter = RwSignal::new(None::<ConversationState>);
    let open = RwSignal::new(None::<String>);
    let composing = RwSignal::new(false);
    let checking = RwSignal::new(false);

    let inboxes = Signal::derive(move || ctx.messages.get().map(|s| s.inboxes).unwrap_or_default());
    let views = Memo::new(move |_| mailbox.views());
    // The conversations of the project chosen, before the tab and the status.
    let of_project = Memo::new(move |_| {
        let wanted = project.get();
        let ids: Vec<String> = if wanted.is_empty() {
            Vec::new()
        } else {
            vec![wanted]
        };
        views
            .get()
            .into_iter()
            .filter(|v| {
                ids.is_empty()
                    || ids.contains(&v.project.0)
                    || inboxes.with(|list| {
                        list.iter()
                            .filter(|i| ids.contains(&i.project.0))
                            .filter_map(|i| i.slug.as_ref())
                            .any(|slug| v.conversation.projects.contains(slug))
                    })
            })
            .collect::<Vec<_>>()
    });
    let rows = Memo::new(move |_| {
        let (t, f) = (tab.get(), filter.get());
        of_project
            .get()
            .into_iter()
            .filter(|v| t.keeps(v.state()) && status_keeps(f, v.state()))
            .map(|v| v.conversation.thread)
            .collect::<Vec<_>>()
    });
    let count = move |t: Tab| {
        of_project
            .get()
            .iter()
            .filter(|v| t.keeps(v.state()))
            .count()
    };

    let check_now = move |_| {
        checking.set(true);
        spawn_local(async move {
            match ctx.backend.get_value().messages_refresh().await {
                Ok(results) => {
                    let news: Vec<String> = results
                        .into_iter()
                        .filter(|r| r.changed)
                        .map(|r| r.summary)
                        .collect();
                    mailbox.status.set(Some(if news.is_empty() {
                        (Tone::Info, "Nothing new.".to_owned())
                    } else {
                        (Tone::Ok, news.join(" "))
                    }));
                }
                Err(error) => mailbox.status.set(Some((Tone::Error, error.message))),
            }
            checking.set(false);
            mailbox.reload();
        });
    };

    view! {
        <section aria-labelledby="messages-title">
            <div class="view-header">
                <h1 id="messages-title">"Messages"</h1>
                <div class="actions">
                    <button class="btn" disabled=move || checking.get() on:click=check_now>
                        {move || if checking.get() { "Checking…" } else { "Check now" }}
                    </button>
                    <button class="btn primary" on:click=move |_| composing.update(|c| *c = !*c)>
                        "New conversation"
                    </button>
                </div>
            </div>
            <FeatureState feature=Feature::Messages />
            <StatusLine message=mailbox.status />
            {move || composing.get().then(|| view! {
                <NewConversation inboxes=inboxes on_done=Callback::new(move |text| {
                    composing.set(false);
                    mailbox.done(text);
                }) />
            })}
            {move || match ctx.messages.get() {
                None => view! { <p class="muted">"Reading the inboxes…"</p> }.into_any(),
                Some(snapshot) => view! {
                    <Setup inboxes=snapshot.inboxes.clone() />
                    <Stale snapshot=snapshot />
                }.into_any(),
            }}
            <div class="tabs" role="tablist" aria-label="Conversations">
                {[(Tab::Active, "Active"), (Tab::Closed, "Closed")].into_iter().map(|(t, label)| view! {
                    <button
                        class="tab"
                        role="tab"
                        aria-selected=move || (tab.get() == t).to_string()
                        on:click=move |_| {
                            tab.set(t);
                            filter.set(None);
                        }
                    >
                        {label} " " <span class="tab-count">{move || count(t)}</span>
                    </button>
                }).collect_view()}
            </div>
            <Filters inboxes=inboxes project=project filter=filter tab=tab />
            {move || rows.with(Vec::is_empty).then(|| view! {
                <p class="muted empty">"No conversation to show."</p>
            })}
            <ul class="conversations" role="list">
                <For
                    each=move || rows.get()
                    key=|thread| thread.clone()
                    children=move |thread| view! {
                        <ConversationRow thread=thread views=views open=open mailbox=mailbox />
                    }
                />
            </ul>
        </section>
    }
}

/// The projects that cannot take part yet, and why.
#[component]
fn Setup(inboxes: Vec<ProjectInbox>) -> impl IntoView {
    let lines: Vec<(String, InboxSetup)> = inboxes
        .into_iter()
        .filter(|i| i.setup != InboxSetup::Ready)
        .map(|i| (i.name, i.setup))
        .collect();
    (!lines.is_empty()).then(|| view! {
        <details class="setup-notes">
            <summary>{format!("{} project(s) not taking part", lines.len())}</summary>
            <ul class="items">
                {lines.into_iter().map(|(name, setup)| match setup {
                    InboxSetup::NotSetUp { command } => view! {
                        <li><strong>{name}</strong>" — not set up on the branch checked out: run " <code>{command}</code>", then commit .aurora/integrations.json."</li>
                    }.into_any(),
                    InboxSetup::Unavailable { reason } => view! {
                        <li><strong>{name}</strong>" — not reachable: " {sentence(&reason)}</li>
                    }.into_any(),
                    InboxSetup::Ready => ().into_any(),
                }).collect_view()}
            </ul>
        </details>
    })
}

/// Projects the last reading could not reach: their conversations are the last ones read.
#[component]
fn Stale(snapshot: MessagesSnapshot) -> impl IntoView {
    let names: Vec<String> = snapshot
        .stale
        .iter()
        .map(|s| {
            let name = snapshot
                .inboxes
                .iter()
                .find(|i| i.project == s.project)
                .map_or_else(|| s.project.0.clone(), |i| i.name.clone());
            format!("{name} (since {})", short_date(&s.since))
        })
        .collect();
    (!names.is_empty()).then(|| view! {
        <p class="muted stale" title="Aurora shows what it read last and keeps trying">
            {format!("Not reachable now, showing the last messages read: {}.", names.join(", "))}
        </p>
    })
}

#[component]
fn Filters(
    inboxes: Signal<Vec<ProjectInbox>>,
    project: RwSignal<String>,
    filter: RwSignal<Option<ConversationState>>,
    tab: RwSignal<Tab>,
) -> impl IntoView {
    let ready = move || {
        inboxes
            .get()
            .into_iter()
            .filter(|i| i.setup == InboxSetup::Ready)
            .map(|i| (i.project.0, i.name))
            .collect::<Vec<_>>()
    };
    let states = move || {
        ConversationState::ALL
            .into_iter()
            .filter(|s| tab.get().keeps(*s))
            .collect::<Vec<_>>()
    };
    view! {
        <div class="settings-toolbar mail-filters" role="group" aria-label="Filters">
            <label for="messages-project">"Project"</label>
            <select id="messages-project" on:change=move |ev| project.set(event_target_value(&ev))>
                <option value="" selected=move || project.get().is_empty()>"Every project"</option>
                {move || ready().into_iter().map(|(id, name)| {
                    let selected = id.clone();
                    view! { <option value=id selected=move || project.get() == selected>{name}</option> }
                }).collect_view()}
            </select>
            {move || (tab.get() == Tab::Active).then(|| view! {
                <label for="messages-status">"Status"</label>
                <select id="messages-status" on:change=move |ev| {
                    let key = event_target_value(&ev);
                    filter.set(ConversationState::ALL.into_iter().find(|s| s.key() == key));
                }>
                    <option value="" selected=move || filter.get().is_none()>"Every status"</option>
                    {states().into_iter().map(|s| view! {
                        <option value=s.key() selected=move || filter.get() == Some(s)>{s.label()}</option>
                    }).collect_view()}
                </select>
            })}
            <ul class="legend" aria-label="Status colors">
                {move || states().into_iter().map(|s| view! {
                    <li class=format!("status-{}", s.key())>{s.label()}</li>
                }).collect_view()}
            </ul>
        </div>
    }
}

/// `opener → counterpart`.
fn route_text((from, to): (String, String)) -> String {
    format!("{from} → {to}")
}

/// One conversation: its row, and the conversation right below it when open. It follows
/// the snapshot by its thread, so a new message updates it without closing it.
#[component]
fn ConversationRow(
    thread: String,
    views: Memo<Vec<ConversationView>>,
    open: RwSignal<Option<String>>,
    mailbox: Mailbox,
) -> impl IntoView {
    let view_of = {
        let thread = thread.clone();
        Memo::new(move |_| {
            views
                .get()
                .into_iter()
                .find(|v| v.conversation.thread == thread)
        })
    };
    let is_open = {
        let thread = thread.clone();
        Memo::new(move |_| open.get().as_deref() == Some(thread.as_str()))
    };
    let row = NodeRef::<leptos::html::Button>::new();
    let toggle = {
        let thread = thread.clone();
        move |_| {
            if is_open.get_untracked() {
                open.set(None);
            } else {
                open.set(Some(thread.clone()));
            }
        }
    };
    let close = Callback::new(move |()| {
        open.set(None);
        if let Some(button) = row.get_untracked() {
            let _ = button.focus();
        }
    });
    move || {
        let Some(v) = view_of.get() else {
            return ().into_any();
        };
        let state = v.state();
        let c = v.conversation.clone();
        let last = format!(
            "Last: {} · {} · {}",
            c.last.kind,
            short_date(&c.last.date),
            c.last.from
        );
        view! {
            <li class=format!("conversation-row status-{}", state.key())>
                <button
                    class="row-button"
                    node_ref=row
                    aria-expanded=move || if is_open.get() { "true" } else { "false" }
                    on:click=toggle.clone()
                >
                    <span class="route">{route_text(c.route())}</span>
                    <span class="subject">{c.subject.clone()}</span>
                    <span class="meta">{last}</span>
                </button>
                <span class="row-side">
                    {c.waiting_for_user.then(|| view! {
                        <span class="pill status-waiting">"Waiting for you"</span>
                    })}
                    {(c.unread > 0).then(|| view! {
                        <span class="unread" title=format!("{} unread", c.unread) aria-label=format!("{} unread", c.unread)></span>
                    })}
                </span>
                {move || is_open.get().then(|| view! {
                    <ConversationPanel view=view_of on_close=close mailbox=mailbox />
                })}
            </li>
        }
        .into_any()
    }
}

/// The messages of a conversation received by `me` and not read yet.
fn unread_ids(messages: &[Message], me: &str) -> Vec<String> {
    messages
        .iter()
        .filter(|m| m.header.to == me && m.header.status == MessageStatus::Open)
        .map(|m| m.header.id.clone())
        .collect()
}

/// What a conversation panel shows open below its header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Form {
    None,
    Reply,
    StepIn,
}

#[component]
fn ConversationPanel(
    view: Memo<Option<ConversationView>>,
    on_close: Callback<()>,
    mailbox: Mailbox,
) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let form = RwSignal::new(Form::None);
    let Some(first) = view.get_untracked() else {
        return ().into_any();
    };
    let thread = first.conversation.thread.clone();
    // Read now: the view says so at once, the worker writes it behind (IMP-005, IMP-007).
    let unread = unread_ids(&first.conversation.items, &first.me);
    if !unread.is_empty() || first.conversation.unread > 0 {
        mailbox.read.update(|r| {
            r.insert(thread.clone());
        });
        let project = first.project.0.clone();
        spawn_local(async move {
            for id in unread {
                let action = UserAction::MarkSeen { id };
                if let Err(e) = ctx
                    .backend
                    .get_value()
                    .messages_act(project.clone(), action)
                    .await
                {
                    leptos::logging::warn!("mark as read: {}", e.message);
                }
            }
        });
    }
    // The focus moves into the conversation when it opens (spec 009, US3).
    let panel = NodeRef::<leptos::html::Section>::new();
    Effect::new(move |_| {
        if let Some(section) = panel.get() {
            let _ = section.focus();
        }
    });
    let header = move || {
        view.get().map(|v| {
            let c = &v.conversation;
            let state = v.state();
            let round = c
                .round
                .map(|r| format!(" · round {r} of {}", aurora_core::domain::MAX_ROUNDS))
                .unwrap_or_default();
            let next = c.next_step().map(str::to_owned);
            view! {
                <div class="panel-head">
                    <div>
                        <h2 id="conversation-title" class="route big">{route_text(c.route())}</h2>
                        <p class="panel-subject">{c.subject.clone()}</p>
                        <p class="panel-meta">
                            <span class=format!("pill status-{}", state.key())>{state.label()}</span>
                            {format!(" {} message(s){round}", c.messages)}
                        </p>
                        {next.map(|n| view! { <p class="next-line"><strong>"Next: "</strong>{n}</p> })}
                    </div>
                    <button class="btn" on:click=move |_| on_close.run(())>"Close ✕"</button>
                </div>
            }
        })
    };
    let decision = move || {
        view.get().and_then(|v| {
            (v.conversation.waiting_for_user)
                .then(|| v.conversation.record.clone().map(|r| (v.project.0.clone(), r)))
                .flatten()
                .map(|(project, record)| view! { <DecisionCard project=project record=record mailbox=mailbox /> })
        })
    };
    let items = Memo::new(move |_| view.get().map(|v| v.conversation.items).unwrap_or_default());
    let newest = Memo::new(move |_| items.with(|list| list.last().map(|m| m.header.id.clone())));
    view! {
        <section
            class=move || format!("card conversation status-{}", view.get().map(|v| v.state().key()).unwrap_or("awaiting"))
            aria-labelledby="conversation-title"
            tabindex="-1"
            node_ref=panel
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    ev.stop_propagation();
                    on_close.run(());
                }
            }
        >
            {header}
            {decision}
            {move || items.with(Vec::is_empty).then(|| view! {
                <p class="muted">"The messages of this conversation were not read yet."</p>
            })}
            <ol class="messages">
                <For
                    each=move || items.get()
                    key=|m| m.header.id.clone()
                    children=move |m| {
                        let expanded = RwSignal::new(newest.get_untracked().as_deref() == Some(m.header.id.as_str()));
                        // A message that arrives while the conversation is open opens too.
                        let id = m.header.id.clone();
                        Effect::new(move |_| {
                            if newest.get().as_deref() == Some(id.as_str()) {
                                expanded.set(true);
                            }
                        });
                        view! { <MessageItem message=m expanded=expanded /> }
                    }
                />
            </ol>
            {move || view.get().map(|v| view! { <Forms view=v form=form mailbox=mailbox /> })}
        </section>
    }
    .into_any()
}

/// One message: one line when collapsed, the whole message when open (spec 010, US2).
#[component]
fn MessageItem(message: Message, expanded: RwSignal<bool>) -> impl IntoView {
    let h = message.header.clone();
    let route = format!("{} → {}", h.from, h.to);
    let handled = message.handled().cloned();
    let side = match &handled {
        Some(n) => format!("✓ Handled · {}", short_date(&n.date)),
        None => short_date(&h.date),
    };
    let preview = message.preview();
    let m = message.clone();
    view! {
        <li class=format!("message kind-{}", h.kind)>
            <button
                class="message-head"
                aria-expanded=move || if expanded.get() { "true" } else { "false" }
                on:click=move |_| expanded.update(|e| *e = !*e)
            >
                <span class="chevron" aria-hidden="true">{move || if expanded.get() { "▾" } else { "▸" }}</span>
                <strong class="message-route">{route}</strong>
                <span class="kind">{h.kind.to_string()}</span>
                <span class="side">{move || if expanded.get() { short_date(&h.date) } else { side.clone() }}</span>
            </button>
            {move || (!expanded.get()).then(|| view! { <p class="preview">{preview.clone()}</p> })}
            {move || expanded.get().then(|| view! { <MessageBody message=m.clone() /> })}
        </li>
    }
}

/// Whether a section is shown under its name in the body (not the lead, not folded,
/// not the Next step).
fn plain_section(s: &Section) -> bool {
    let is = |names: &[&str]| names.iter().any(|n| s.name.eq_ignore_ascii_case(n));
    !is(&LEAD_SECTIONS) && !is(&FOLDED_SECTIONS) && !s.name.eq_ignore_ascii_case(NEXT_STEP)
}

#[component]
fn MessageBody(message: Message) -> impl IntoView {
    let lead = message.lead().map(|s| s.text.clone());
    let plain: Vec<Section> = message
        .sections
        .iter()
        .filter(|s| plain_section(s) && !s.text.trim().is_empty())
        .cloned()
        .collect();
    let folded: Vec<Section> = message
        .sections
        .iter()
        .filter(|s| {
            FOLDED_SECTIONS
                .iter()
                .any(|n| s.name.eq_ignore_ascii_case(n))
                && !s.text.trim().is_empty()
                && !s.text.trim().eq_ignore_ascii_case("none")
        })
        .cloned()
        .collect();
    let next = message.next_step().map(str::to_owned);
    let free = message
        .sections
        .is_empty()
        .then(|| message.text_or_body().trim().to_owned());
    let handled = message.handled().cloned();
    let receiver = message.header.to.clone();
    view! {
        <div class="message-body">
            {lead.map(|l| view! { <p class="lead">{l}</p> })}
            {free.map(|t| view! { <p class="body">{t}</p> })}
            {plain.into_iter().map(|s| view! {
                <h4>{s.name}</h4>
                <p class="body">{s.text}</p>
            }).collect_view()}
            {next.map(|n| view! {
                <div class="next-step">
                    <span class="label">"➜ Next step"</span>
                    <p>{n}</p>
                </div>
            })}
            {(!folded.is_empty()).then(|| {
                let title = folded.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(" and ");
                view! {
                    <details class="folded">
                        <summary>{title}</summary>
                        {folded.into_iter().map(|s| view! {
                            <h4>{s.name}</h4>
                            <p class="body">{s.text}</p>
                        }).collect_view()}
                    </details>
                }
            })}
            {handled.map(|n| view! {
                <p class="handled" title=n.note.clone()>
                    <strong>{format!("✓ Handled by {receiver}")}</strong>
                    " " {short_date(&n.date)}
                    {(!n.note.is_empty()).then(|| view! { <span class="note">{format!(" · {}", n.note)}</span> })}
                </p>
            })}
        </div>
    }
}

/// How long a decision can be undone before it is sent.
const UNDO_MS: u64 = 5_000;

/// The decision card of a closing record (spec 010, US3): a click sends an option after
/// a short Undo window; Something else sends the user's own words.
#[component]
fn DecisionCard(project: String, record: ClosingRecord, mailbox: Mailbox) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let pending = RwSignal::new(None::<(u32, String)>);
    let token = StoredValue::new(0_u32);
    let other_open = RwSignal::new(false);
    let other = RwSignal::new(String::new());
    let error = RwSignal::new(None::<CommandError>);
    let busy = RwSignal::new(false);
    let thread = record.thread.clone();
    let recommended = record.recommended_option();
    let question = {
        let lines: Vec<&str> = record
            .decision_needed
            .lines()
            .map(str::trim)
            .filter(|l| {
                !l.is_empty()
                    && !l.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && !l.to_lowercase().starts_with("recommended:")
            })
            .collect();
        if lines.is_empty() {
            "What do you decide?".to_owned()
        } else {
            lines.join(" ")
        }
    };
    let act = {
        let (project, thread) = (project.clone(), thread.clone());
        move |action: UserAction, done: String| {
            let (project, thread) = (project.clone(), thread.clone());
            busy.set(true);
            error.set(None);
            spawn_local(async move {
                match ctx.backend.get_value().messages_act(project, action).await {
                    Ok(_) => {
                        mailbox.decided.update(|d| {
                            d.insert(thread);
                        });
                        mailbox.done(done);
                    }
                    Err(e) => {
                        pending.set(None);
                        error.set(Some(e));
                    }
                }
                busy.set(false);
            });
        }
    };
    let choose = {
        let (act, thread) = (act.clone(), thread.clone());
        move |n: usize, option: String| {
            let mine = token.get_value() + 1;
            token.set_value(mine);
            pending.set(Some((mine, ClosingRecord::option_label(&option))));
            let (act, thread) = (act.clone(), thread.clone());
            set_timeout(
                move || {
                    if pending.get_untracked().is_some_and(|(t, _)| t == mine) {
                        act(
                            UserAction::Decide {
                                thread,
                                choice: option.clone(),
                                note: None,
                            },
                            format!(
                                "Decision sent: option {n}, {}.",
                                ClosingRecord::option_label(&option)
                            ),
                        );
                    }
                },
                std::time::Duration::from_millis(UNDO_MS),
            );
        }
    };
    let send_other = {
        let (act, thread) = (act.clone(), thread.clone());
        move |_| {
            let text = other.get_untracked().trim().to_owned();
            if text.is_empty() {
                return;
            }
            act(
                UserAction::Decide {
                    thread: thread.clone(),
                    choice: text.clone(),
                    note: None,
                },
                format!("Decision sent: {text}"),
            );
        }
    };
    let rework = {
        let (act, thread) = (act.clone(), thread.clone());
        move |_| {
            let text = other.get_untracked().trim().to_owned();
            if text.is_empty() {
                error.set(Some(CommandError::new(
                    "messages",
                    "write what the projects should change first",
                )));
                return;
            }
            act(
                UserAction::Revise {
                    thread: thread.clone(),
                    text,
                },
                "Your request was sent: a new cycle of up to three rounds starts.".to_owned(),
            );
        }
    };
    let details: Vec<(&str, String)> = [
        ("Conclusion", record.conclusion.clone()),
        ("Justification", record.justification.clone()),
        ("Positions", record.positions.clone()),
        ("Open points", record.open_points.clone()),
        ("After approval", record.after_approval.clone()),
        ("Your requests", record.user_requests.join(" / ")),
    ]
    .into_iter()
    .filter(|(_, t)| !t.trim().is_empty() && !t.trim().eq_ignore_ascii_case("none"))
    .collect();
    let options = record.options.clone();
    let others = record.projects.join(" and ");
    view! {
        <article class="decision-card" aria-label="Your decision">
            <p class="label">"⚑ Your decision"</p>
            <p class="question">{question}</p>
            {(!record.conclusion.trim().is_empty()).then(|| view! { <p class="context">{record.conclusion.clone()}</p> })}
            {move || match pending.get() {
                Some((_, label)) => view! {
                    <div class="undo" role="status">
                        <strong>{format!("✓ {label}")}</strong>
                        <span>{format!("sending to {others} in 5 s")}</span>
                        <span class="countdown" aria-hidden="true"><i></i></span>
                        <button class="btn" disabled=move || busy.get() on:click=move |_| pending.set(None)>"Undo"</button>
                    </div>
                }.into_any(),
                None => view! {
                    <div class="options">
                        {options.iter().enumerate().map(|(i, option)| {
                            let n = i + 1;
                            let (choose, option) = (choose.clone(), option.clone());
                            let label = ClosingRecord::option_label(&option);
                            let is_recommended = recommended == Some(n);
                            view! {
                                <button
                                    class=if is_recommended { "option recommended" } else { "option" }
                                    disabled=move || busy.get()
                                    on:click=move |_| choose(n, option.clone())
                                >
                                    <span class="n">{n}</span>
                                    <span class="text">{label}</span>
                                    {is_recommended.then(|| view! { <span class="tag">"Recommended"</span> })}
                                </button>
                            }
                        }).collect_view()}
                        <button
                            class="option other"
                            aria-expanded=move || other_open.get().to_string()
                            on:click=move |_| other_open.update(|o| *o = !*o)
                        >
                            <span class="n">"✎"</span>
                            <span class="text">"Something else…"</span>
                        </button>
                    </div>
                }.into_any(),
            }}
            {move || other_open.get().then(|| view! {
                <div class="field">
                    <label for=format!("other-{}", thread) class="visually-hidden">"Your decision"</label>
                    <textarea
                        id=format!("other-{}", thread)
                        rows="3"
                        placeholder="Write exactly what you decided."
                        prop:value=move || other.get()
                        on:input=move |ev| other.set(event_target_value(&ev))
                    ></textarea>
                    <div class="actions">
                        <button class="btn primary" disabled=move || busy.get() || other.get().trim().is_empty() on:click=send_other.clone()>"Send decision"</button>
                        <button class="btn" on:click=move |_| other_open.set(false)>"Cancel"</button>
                        <span class="muted rework">
                            "or "
                            <button class="link" disabled=move || busy.get() on:click=rework.clone()>"ask the projects to rework it"</button>
                            " (new cycle)"
                        </span>
                    </div>
                </div>
            })}
            <Failure error=error />
            {(!details.is_empty()).then(|| view! {
                <details class="folded">
                    <summary>"Conclusion, positions and what follows"</summary>
                    <dl class="record-fields">
                        {details.into_iter().map(|(name, text)| view! { <dt>{name}</dt><dd>{text}</dd> }).collect_view()}
                    </dl>
                </details>
            })}
        </article>
    }
}

/// Reply and Step in, closed until their button is pressed (spec 010, US2).
#[component]
fn Forms(view: ConversationView, form: RwSignal<Form>, mailbox: Mailbox) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let c = view.conversation.clone();
    let me = view.me.clone();
    let project = view.project.0.clone();
    let thread = c.thread.clone();
    // The message this project would answer: the last one it received.
    let answering = c
        .items
        .iter()
        .rev()
        .find(|m| m.header.to == me)
        .map(|m| m.header.id.clone());
    let kinds = c.reply_kinds();
    let reply_kind = RwSignal::new(kinds.first().cloned().unwrap_or(MessageKind::Answer));
    let fields: Fields = RwSignal::new(Vec::new());
    let guide = RwSignal::new(String::new());
    let error = RwSignal::new(None::<CommandError>);
    let busy = RwSignal::new(false);
    Effect::new(move |_| {
        if form.get() == Form::Reply {
            load_template(ctx, reply_kind.get(), fields, error);
        }
    });
    let run = {
        let project = project.clone();
        move |action: UserAction, done: String| {
            let project = project.clone();
            busy.set(true);
            error.set(None);
            spawn_local(async move {
                match ctx.backend.get_value().messages_act(project, action).await {
                    Ok(_) => {
                        form.set(Form::None);
                        guide.set(String::new());
                        fields.set(Vec::new());
                        mailbox.done(done);
                    }
                    Err(e) => error.set(Some(e)),
                }
                busy.set(false);
            });
        }
    };
    let send_reply = {
        let (run, answering) = (run.clone(), answering.clone());
        move |_| {
            let Some(id) = answering.clone() else {
                return;
            };
            run(
                UserAction::Reply {
                    id,
                    kind: reply_kind.get_untracked(),
                    sections: sections_of(&fields.get_untracked()),
                },
                "Reply sent.".to_owned(),
            );
        }
    };
    let step_in = {
        let (run, thread) = (run.clone(), thread.clone());
        move |_| {
            run(
                UserAction::Guide {
                    thread: thread.clone(),
                    text: guide.get_untracked(),
                },
                "Your request was sent: a new cycle of up to three rounds starts.".to_owned(),
            );
        }
    };
    let limited = c.next_round > aurora_core::domain::MAX_ROUNDS;
    let can_reply = answering.is_some();
    view! {
        <div class="conversation-actions">
            {move || (form.get() == Form::None).then(|| view! {
                <div class="actions">
                    {can_reply.then(|| view! {
                        <button class="btn primary" on:click=move |_| form.set(Form::Reply)>
                            {format!("Reply as {me}")}
                        </button>
                    })}
                    <button class="btn" on:click=move |_| form.set(Form::StepIn)>"Step in"</button>
                </div>
            })}
            {move || (form.get() == Form::Reply).then(|| {
                let kinds = kinds.clone();
                let send_reply = send_reply.clone();
                view! {
                    <fieldset class="reply-form">
                        <legend>"Reply"</legend>
                        {limited.then(|| view! {
                            <p class="muted">"Three rounds were used in this cycle: close it with an agreement or an impasse."</p>
                        })}
                        <div class="field">
                            <label for="reply-kind">"Kind"</label>
                            <select id="reply-kind" on:change=move |ev| reply_kind.set(MessageKind::from(event_target_value(&ev)))>
                                {kinds.into_iter().map(|k| {
                                    let selected = k.clone();
                                    view! { <option value=k.to_string() selected=move || reply_kind.get() == selected>{k.to_string()}</option> }
                                }).collect_view()}
                            </select>
                        </div>
                        <SectionFields fields=fields prefix="reply".to_owned() missing=missing_of(error) />
                        <div class="actions">
                            <button class="btn primary" disabled=move || busy.get() on:click=send_reply>
                                {move || if busy.get() { "Sending…" } else { "Send reply" }}
                            </button>
                            <button class="btn" on:click=move |_| form.set(Form::None)>"Cancel"</button>
                        </div>
                    </fieldset>
                }
            })}
            {move || (form.get() == Form::StepIn).then(|| {
                let step_in = step_in.clone();
                view! {
                    <div class="field">
                        <label for="step-in">"Step in"</label>
                        <textarea
                            id="step-in"
                            rows="3"
                            placeholder="Tell the projects what you want: a new cycle of up to three rounds follows."
                            prop:value=move || guide.get()
                            on:input=move |ev| guide.set(event_target_value(&ev))
                        ></textarea>
                        <div class="actions">
                            <button class="btn primary" disabled=move || busy.get() || guide.get().trim().is_empty() on:click=step_in>
                                "Send your request"
                            </button>
                            <button class="btn" on:click=move |_| form.set(Form::None)>"Cancel"</button>
                        </div>
                    </div>
                }
            })}
            <Failure error=error />
        </div>
    }
}

/// The editable sections of a template, with the text the user typed.
type Fields = RwSignal<Vec<(TemplateSection, String)>>;

fn free_section() -> TemplateSection {
    TemplateSection {
        name: String::new(),
        hint: "The message.".to_owned(),
        required: true,
    }
}

/// Loads the template of `kind` into `fields`, keeping what was typed under the same name.
fn load_template(
    ctx: AppContext,
    kind: MessageKind,
    fields: Fields,
    error: RwSignal<Option<CommandError>>,
) {
    spawn_local(async move {
        match ctx.backend.get_value().messages_template(kind).await {
            Ok(template) => {
                let template = if template.is_empty() {
                    vec![free_section()]
                } else {
                    template
                };
                fields.update(|current| {
                    let typed: Vec<(String, String)> = current
                        .iter()
                        .map(|(t, v)| (t.name.clone(), v.clone()))
                        .collect();
                    *current = template
                        .into_iter()
                        .map(|t| {
                            let value = typed
                                .iter()
                                .find(|(n, _)| *n == t.name)
                                .map(|(_, v)| v.clone())
                                .unwrap_or_default();
                            (t, value)
                        })
                        .collect();
                });
            }
            Err(e) => error.set(Some(e)),
        }
    });
}

fn sections_of(fields: &[(TemplateSection, String)]) -> Vec<Section> {
    fields
        .iter()
        .map(|(t, v)| Section::new(&t.name, v))
        .collect()
}

#[component]
fn SectionFields(fields: Fields, prefix: String, missing: Signal<Vec<String>>) -> impl IntoView {
    move || {
        fields
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, (t, value))| {
                let id = format!("{prefix}-{i}");
                let label = if t.name.is_empty() { "Message".to_owned() } else { t.name.clone() };
                let is_missing = missing.get().iter().any(|m| m.eq_ignore_ascii_case(&t.name));
                view! {
                    <div class=if is_missing { "field missing" } else { "field" }>
                        <label for=id.clone()>
                            {label}
                            {t.required.then(|| view! { <span class="required" aria-label="required">" *"</span> })}
                        </label>
                        <textarea
                            id=id
                            rows="3"
                            placeholder=t.hint.clone()
                            aria-required=t.required.to_string()
                            aria-invalid=is_missing.to_string()
                            prop:value=value
                            on:input=move |ev| {
                                let text = event_target_value(&ev);
                                fields.update(|f| {
                                    if let Some(slot) = f.get_mut(i) {
                                        slot.1 = text;
                                    }
                                });
                            }
                        ></textarea>
                    </div>
                }
            })
            .collect_view()
    }
}

fn missing_of(error: RwSignal<Option<CommandError>>) -> Signal<Vec<String>> {
    Signal::derive(move || error.get().map(|e| failure_text(&e).1).unwrap_or_default())
}

#[component]
fn NewConversation(inboxes: Signal<Vec<ProjectInbox>>, on_done: Callback<String>) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let from = RwSignal::new(String::new());
    let to = RwSignal::new(String::new());
    let subject = RwSignal::new(String::new());
    let kind = RwSignal::new(MessageKind::Question);
    let fields: Fields = RwSignal::new(Vec::new());
    let error = RwSignal::new(None::<CommandError>);
    let busy = RwSignal::new(false);
    let ready = move || {
        inboxes
            .get()
            .into_iter()
            .filter(|i| i.setup == InboxSetup::Ready)
            .collect::<Vec<_>>()
    };
    Effect::new(move |_| {
        if from.get_untracked().is_empty()
            && let Some(first) = ready().first()
        {
            from.set(first.project.0.clone());
        }
    });
    let counterparts = move || {
        let f = from.get();
        ready()
            .into_iter()
            .find(|i| i.project.0 == f)
            .map(|i| i.counterparts)
            .unwrap_or_default()
    };
    Effect::new(move |_| {
        let list = counterparts();
        if !list.contains(&to.get_untracked()) {
            to.set(list.first().cloned().unwrap_or_default());
        }
    });
    Effect::new(move |_| load_template(ctx, kind.get(), fields, error));
    let send = move |_| {
        let draft = Draft {
            to: to.get_untracked(),
            kind: kind.get_untracked(),
            subject: subject.get_untracked(),
            sections: sections_of(&fields.get_untracked()),
        };
        let project = from.get_untracked();
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            match ctx
                .backend
                .get_value()
                .messages_act(project, UserAction::Send { draft })
                .await
            {
                Ok(sent) => on_done.run(format!(
                    "Sent to {}.",
                    sent.iter()
                        .map(|h| h.to.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    view! {
        <section class="card compose" aria-labelledby="compose-title">
            <h2 id="compose-title">"New conversation"</h2>
            {move || ready().is_empty().then(|| view! {
                <p class="muted">"No project is set up for messages yet."</p>
            })}
            <div class="settings-toolbar">
                <label for="compose-from">"From"</label>
                <select id="compose-from" on:change=move |ev| from.set(event_target_value(&ev))>
                    {move || ready().into_iter().map(|i| {
                        let id = i.project.0.clone();
                        view! { <option value=i.project.0.clone() selected=move || from.get() == id>{i.name.clone()}</option> }
                    }).collect_view()}
                </select>
                <label for="compose-to">"To"</label>
                <select id="compose-to" on:change=move |ev| to.set(event_target_value(&ev))>
                    {move || counterparts().into_iter().map(|slug| {
                        let s = slug.clone();
                        view! { <option value=slug.clone() selected=move || to.get() == s>{slug.clone()}</option> }
                    }).collect_view()}
                </select>
                <label for="compose-kind">"Kind"</label>
                <select id="compose-kind" on:change=move |ev| kind.set(MessageKind::from(event_target_value(&ev)))>
                    {MessageKind::OPENING.into_iter().map(|k| {
                        let selected = k.clone();
                        view! { <option value=k.to_string() selected=move || kind.get() == selected>{k.to_string()}</option> }
                    }).collect_view()}
                </select>
            </div>
            <div class="field">
                <label for="compose-subject">"Subject" <span class="required" aria-label="required">" *"</span></label>
                <input id="compose-subject" type="text" prop:value=move || subject.get() on:input=move |ev| subject.set(event_target_value(&ev)) />
            </div>
            <SectionFields fields=fields prefix="compose".to_owned() missing=missing_of(error) />
            <Failure error=error />
            <div class="actions">
                <button class="btn primary" disabled=move || busy.get() || ready().is_empty() on:click=send>"Send"</button>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_and_the_status_filter_keep_what_they_say() {
        assert!(Tab::Active.keeps(ConversationState::Answered));
        assert!(!Tab::Active.keeps(ConversationState::Closed));
        assert!(Tab::Closed.keeps(ConversationState::Closed));
        assert!(status_keeps(None, ConversationState::Deferred));
        assert!(status_keeps(
            Some(ConversationState::Deferred),
            ConversationState::Deferred
        ));
        assert!(!status_keeps(
            Some(ConversationState::Answered),
            ConversationState::Deferred
        ));
        assert_eq!(short_date("2026-11-06T10:00:00.000Z"), "2026-11-06 10:00");
        assert_eq!(route_text(("a".into(), "b".into())), "a → b");
    }

    #[test]
    fn only_plain_sections_show_under_their_name() {
        assert!(!plain_section(&Section::new("Ask", "x")));
        assert!(!plain_section(&Section::new("authority", "x")));
        assert!(!plain_section(&Section::new("Next step", "x")));
        assert!(plain_section(&Section::new("Context", "x")));
        let refused = CommandError::from(MessagesError::Refused {
            message: "a 'question' message needs these sections".into(),
            missing_sections: vec!["Ask".into()],
        });
        let (text, missing) = failure_text(&refused);
        assert!(text.starts_with('A'));
        assert_eq!(missing, ["Ask"]);
    }
}
