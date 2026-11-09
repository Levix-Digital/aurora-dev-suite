//! Messages between projects as a chat (spec 011, FEAT-039): the registered projects are
//! contacts on the left; the selected project's chat shows what it sent (right) and
//! received (left) as bubbles with the sender's summary, and the closing records as
//! decision cards; a click on a bubble slides the full message in on the right.
//! Everything comes from the messages worker's snapshot (IMP-007), so the view never
//! waits for the provider to read.

use std::collections::{HashMap, HashSet};

use aurora_core::domain::{
    ChatBubble, ChatContact, ChatEntry, ChatRecord, ClosingRecord, ConversationView, Draft,
    FOLDED_SECTIONS, Feature, InboxSetup, LEAD_SECTIONS, MAX_SUMMARY_CHARS, Message, MessageKind,
    MessageStatus, MessagesError, MessagesSnapshot, NEXT_STEP, ProjectInbox, RecordState, Section,
    TemplateSection, UserAction, avatar_color, chat_contacts, chat_peers, chat_stream, day_label,
    initials, local_time, row_time,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use super::{FeatureState, StatusLine, Tone, sentence};
use crate::app::AppContext;
use crate::backend::{Backend, CommandError};

/// `2026-11-06T23:48:00Z` as `2026-11-06 23:48` in the user's time zone.
fn short_date(date: &str) -> String {
    match local_time(date, offset_minutes()) {
        Some((day, time)) => format!("{day} {time}"),
        None => date.get(..16).unwrap_or(date).replace('T', " "),
    }
}

/// Minutes east of UTC of the user's time zone, as the browser knows it.
fn offset_minutes() -> i32 {
    #[allow(clippy::cast_possible_truncation)]
    let behind = js_sys::Date::new_0().get_timezone_offset() as i32;
    -behind
}

/// Today as `YYYY-MM-DD` in the user's time zone.
fn today() -> String {
    let now: String = js_sys::Date::new_0().to_iso_string().into();
    local_time(&now, offset_minutes())
        .map(|(day, _)| day)
        .unwrap_or_default()
}

/// The palette colors an avatar without a logo takes, by class.
const AVATAR_CLASSES: [&str; 6] = ["av-1", "av-2", "av-3", "av-4", "av-5", "av-6"];

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
    /// Messages the user opened: read at once, written behind.
    read: RwSignal<HashSet<String>>,
    /// Conversations whose decision was sent: no longer waiting.
    decided: RwSignal<HashSet<String>>,
    status: RwSignal<Option<(Tone, String)>>,
    /// Kept here because `reload` also runs from timers, outside any reactive owner.
    backend: StoredValue<Backend>,
}

impl Mailbox {
    /// The inboxes with what the user just did applied.
    fn inboxes(&self) -> Vec<ProjectInbox> {
        let (read, decided) = (self.read.get(), self.decided.get());
        let mut inboxes = self.snapshot.get().map(|s| s.inboxes).unwrap_or_default();
        for inbox in &mut inboxes {
            let me = inbox.slug.clone().unwrap_or_default();
            for c in &mut inbox.conversations {
                for m in &mut c.items {
                    if read.contains(&m.header.id) && m.header.status == MessageStatus::Open {
                        m.header.status = MessageStatus::Seen;
                    }
                }
                if !c.items.is_empty() {
                    let unread = c
                        .items
                        .iter()
                        .filter(|m| m.header.to == me && m.header.status == MessageStatus::Open)
                        .count();
                    c.unread = u32::try_from(unread).unwrap_or(u32::MAX);
                }
                if decided.contains(&c.thread) {
                    c.waiting_for_user = false;
                }
            }
        }
        inboxes
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

/// A project's picture: its logo, else its initials on a palette color.
#[component]
fn Avatar(name: String, logo: Option<String>, #[prop(optional)] small: bool) -> impl IntoView {
    let size = if small { " small" } else { "" };
    match logo {
        Some(src) => view! {
            <span class=format!("avatar logo{size}") aria-hidden="true"><img src=src alt="" /></span>
        }
        .into_any(),
        None => {
            let color = AVATAR_CLASSES[avatar_color(&name, AVATAR_CLASSES.len())];
            view! {
                <span class=format!("avatar {color}{size}") aria-hidden="true">{initials(&name)}</span>
            }
            .into_any()
        }
    }
}

/// Whether a contact matches the search: its name, or a summary or subject it holds.
fn contact_matches(contact: &ChatContact, inbox: Option<&ProjectInbox>, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() || contact.name.to_lowercase().contains(&q) {
        return true;
    }
    inbox.is_some_and(|i| {
        i.conversations.iter().any(|c| {
            c.subject.to_lowercase().contains(&q)
                || c.items.iter().any(|m| {
                    m.header.summary.to_lowercase().contains(&q)
                        || m.header.subject.to_lowercase().contains(&q)
                })
        })
    })
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
    // The projects' logos, from Aurora's own list (FEAT-039).
    let logos = RwSignal::new(HashMap::<String, String>::new());
    spawn_local(async move {
        if let Ok(list) = ctx.backend.get_value().list_projects().await {
            logos.set(
                list.projects
                    .into_iter()
                    .filter_map(|p| p.logo.map(|logo| (p.id.0, logo)))
                    .collect(),
            );
        }
    });
    let selected = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let with = RwSignal::new(None::<String>);
    let opened = RwSignal::new(None::<String>);
    let composing = RwSignal::new(false);
    let checking = RwSignal::new(false);

    let inboxes = Memo::new(move |_| mailbox.inboxes());
    let contacts = Memo::new(move |_| inboxes.with(|list| chat_contacts(list)));
    // The first contact is selected until the user picks one.
    Effect::new(move |_| {
        let current = selected.get();
        let list = contacts.get();
        if current
            .as_ref()
            .is_none_or(|id| !list.iter().any(|c| &c.project.0 == id))
            && let Some(first) = list.first()
        {
            selected.set(Some(first.project.0.clone()));
        }
    });
    let inbox = Memo::new(move |_| {
        let id = selected.get();
        inboxes.with(|list| {
            list.iter()
                .find(|i| Some(&i.project.0) == id.as_ref())
                .cloned()
        })
    });
    let pick = move |id: String| {
        if selected.get_untracked().as_deref() != Some(id.as_str()) {
            opened.set(None);
            with.set(None);
            composing.set(false);
            selected.set(Some(id));
        }
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

    let layout = move || {
        if opened.get().is_some() {
            "chat-view detail-open"
        } else {
            "chat-view"
        }
    };
    view! {
        <section class=layout aria-label="Messages"
            on:keydown=move |ev| {
                if ev.key() == "Escape" && opened.get_untracked().is_some() {
                    ev.stop_propagation();
                    opened.set(None);
                }
            }
        >
            <aside class="contacts" aria-label="Projects">
                <header class="contacts-head">
                    <div class="row">
                        <h1 id="messages-title">"Messages"</h1>
                        <div class="actions">
                            <button class="btn icon" title="Check now" aria-label="Check now"
                                disabled=move || checking.get() on:click=check_now>
                                {move || if checking.get() { "…" } else { "⟳" }}
                            </button>
                        </div>
                    </div>
                    <label for="contacts-search" class="visually-hidden">"Search projects or messages"</label>
                    <input id="contacts-search" class="search" type="text" placeholder="Search projects or messages"
                        prop:value=move || search.get()
                        on:input=move |ev| search.set(event_target_value(&ev)) />
                </header>
                <ul class="contact-list" role="list">
                    {move || {
                        let query = search.get();
                        let list = contacts.get();
                        let all = inboxes.get();
                        let today = today();
                        let shown: Vec<ChatContact> = list
                            .into_iter()
                            .filter(|c| contact_matches(c, all.iter().find(|i| i.project == c.project), &query))
                            .collect();
                        if shown.is_empty() {
                            return view! { <li class="muted empty">"No project matches."</li> }.into_any();
                        }
                        shown.into_iter().map(|c| {
                            let id = c.project.0.clone();
                            let is_selected = {
                                let id = id.clone();
                                Memo::new(move |_| selected.get().as_deref() == Some(id.as_str()))
                            };
                            let logo = logos.with(|l| l.get(&id).cloned());
                            let (line, time) = match (&c.setup, &c.last) {
                                (InboxSetup::Ready, Some(last)) => (
                                    if last.outgoing { format!("You: {}", last.text) } else { last.text.clone() },
                                    row_time(&last.date, &today, offset_minutes()),
                                ),
                                (InboxSetup::Ready, None) => ("No messages yet".to_owned(), String::new()),
                                (InboxSetup::NotSetUp { .. }, _) => ("Not set up for messages".to_owned(), String::new()),
                                (InboxSetup::Unavailable { .. }, _) => ("Not reachable now".to_owned(), String::new()),
                            };
                            let muted = c.last.is_none() || c.setup != InboxSetup::Ready;
                            view! {
                                <li>
                                    <button
                                        class=move || if is_selected.get() { "contact selected" } else { "contact" }
                                        aria-current=move || is_selected.get().then_some("true")
                                        on:click=move |_| pick(id.clone())
                                    >
                                        <Avatar name=c.name.clone() logo=logo />
                                        <span class="contact-text">
                                            <span class="contact-name">{c.name.clone()}</span>
                                            <span class=if muted { "contact-last muted" } else { "contact-last" }>{line}</span>
                                        </span>
                                        <span class="contact-side">
                                            <span class="contact-time">{time}</span>
                                            <span class="contact-flags">
                                                {(c.waiting > 0).then(|| view! {
                                                    <span class="dot waiting" title="A decision waits for you" aria-label="A decision waits for you"></span>
                                                })}
                                                {(c.unread > 0).then(|| view! {
                                                    <span class="unread-count" aria-label=format!("{} unread", c.unread)>{c.unread}</span>
                                                })}
                                            </span>
                                        </span>
                                    </button>
                                </li>
                            }
                        }).collect_view().into_any()
                    }}
                </ul>
            </aside>

            <section class="chat" aria-labelledby="chat-title">
                {move || match inbox.get() {
                    None => view! {
                        <div class="chat-empty">
                            <FeatureState feature=Feature::Messages />
                            <p class="muted">{move || if ctx.messages.get().is_none() { "Reading the inboxes…" } else { "No project is registered yet." }}</p>
                        </div>
                    }.into_any(),
                    Some(current) => view! {
                        <ChatColumn inbox=current logos=logos with=with opened=opened composing=composing mailbox=mailbox pick=Callback::new(pick) />
                    }.into_any(),
                }}
            </section>

            <aside class="detail" aria-label="Message">
                {move || opened.get().and_then(|id| {
                    let current = inbox.get()?;
                    let conversation = current
                        .conversations
                        .iter()
                        .find(|c| c.items.iter().any(|m| m.header.id == id))
                        .cloned()?;
                    let message = conversation.items.iter().find(|m| m.header.id == id).cloned()?;
                    let view = ConversationView {
                        project: current.project.clone(),
                        me: current.slug.clone().unwrap_or_default(),
                        conversation,
                    };
                    Some(view! { <MessageDetail message=message view=view opened=opened mailbox=mailbox /> })
                })}
            </aside>
        </section>
    }
}

/// The chat of the selected project.
#[component]
fn ChatColumn(
    inbox: ProjectInbox,
    logos: RwSignal<HashMap<String, String>>,
    with: RwSignal<Option<String>>,
    opened: RwSignal<Option<String>>,
    composing: RwSignal<bool>,
    mailbox: Mailbox,
    pick: Callback<String>,
) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let me = inbox.slug.clone().unwrap_or_default();
    let name = inbox.name.clone();
    let project = inbox.project.0.clone();
    let logo = logos.with_untracked(|l| l.get(&project).cloned());
    let active = inbox
        .conversations
        .iter()
        .filter(|c| c.state().is_active())
        .count();
    let waiting = inbox.waiting_count();
    let peers = chat_peers(&inbox);
    let entries = chat_stream(&inbox, with.get_untracked().as_deref(), offset_minutes());
    let today = today();
    let subline = match &inbox.setup {
        InboxSetup::Ready => {
            let mut text = format!(
                "{active} active conversation{}",
                if active == 1 { "" } else { "s" }
            );
            if waiting > 0 {
                text.push_str(&format!(
                    " · {waiting} wait{} for you",
                    if waiting == 1 { "s" } else { "" }
                ));
            }
            text
        }
        InboxSetup::NotSetUp { .. } => "Not set up for messages".to_owned(),
        InboxSetup::Unavailable { .. } => "Not reachable now".to_owned(),
    };
    // Projects registered in Aurora by slug: a pending record links to the one that decides.
    let registered: Vec<Registered> = mailbox
        .snapshot
        .get_untracked()
        .map(|s| {
            s.inboxes
                .iter()
                .filter_map(|i| {
                    i.slug.clone().map(|slug| Registered {
                        slug,
                        id: i.project.0.clone(),
                        name: i.name.clone(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let stale = mailbox.snapshot.get_untracked().and_then(|s| {
        s.stale
            .iter()
            .find(|st| st.project == inbox.project)
            .map(|st| short_date(&st.since))
    });

    // Scrolled to the newest when it opens; later news keep the place unless at the bottom.
    let stream = NodeRef::<leptos::html::Div>::new();
    let at_bottom = RwSignal::new(true);
    let to_bottom = move || {
        if let Some(el) = stream.get_untracked() {
            el.set_scroll_top(el.scroll_height());
        }
    };
    Effect::new(move |_| {
        if stream.get().is_some() {
            request_animation_frame(to_bottom);
        }
    });
    let on_scroll = move |_| {
        if let Some(el) = stream.get_untracked() {
            at_bottom.set(el.scroll_height() - el.scroll_top() - el.client_height() < 40);
        }
    };
    // Up and Down move between bubbles (FR-008).
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();
        if key != "ArrowUp" && key != "ArrowDown" {
            return;
        }
        let Some(el) = stream.get_untracked() else {
            return;
        };
        let Ok(list) = el.query_selector_all("button.bubble") else {
            return;
        };
        let active = document().active_element();
        let count = list.length();
        let mut at = None;
        for i in 0..count {
            if let (Some(node), Some(a)) = (list.item(i), active.as_ref())
                && node.is_same_node(Some(a.unchecked_ref()))
            {
                at = Some(i);
            }
        }
        let next = match (at, key.as_str()) {
            (Some(i), "ArrowUp") if i > 0 => Some(i - 1),
            (Some(i), "ArrowDown") if i + 1 < count => Some(i + 1),
            (None, _) if count > 0 => Some(count - 1),
            _ => None,
        };
        if let Some(node) = next.and_then(|i| list.item(i))
            && let Ok(button) = node.dyn_into::<web_sys::HtmlElement>()
        {
            ev.prevent_default();
            let _ = button.focus();
        }
    };

    let open_bubble = Callback::new(move |m: Message| {
        let id = m.header.id.clone();
        opened.set(Some(id.clone()));
        // Read now: the view says so at once, the worker writes it behind (IMP-007).
        if m.header.to == me
            && m.header.status == MessageStatus::Open
            && !mailbox.read.get_untracked().contains(&id)
        {
            mailbox.read.update(|r| {
                r.insert(id.clone());
            });
            let project = project.clone();
            spawn_local(async move {
                if let Err(e) = ctx
                    .backend
                    .get_value()
                    .messages_act(project, UserAction::MarkSeen { id })
                    .await
                {
                    leptos::logging::warn!("mark as read: {}", e.message);
                }
            });
        }
    });

    let inbox_for_compose = inbox.clone();
    let has_entries = !entries.is_empty();
    view! {
        <header class="chat-head">
            <Avatar name=name.clone() logo=logo />
            <div class="chat-title">
                <h2 id="chat-title">{name.clone()}</h2>
                <p class="muted">{subline}</p>
            </div>
            {(peers.len() > 1).then(|| {
                let peers = peers.clone();
                view! {
                    <label for="chat-with" class="visually-hidden">"Show the messages with"</label>
                    <select id="chat-with" class="chat-with" on:change=move |ev| {
                        let value = event_target_value(&ev);
                        opened.set(None);
                        with.set((!value.is_empty()).then_some(value));
                    }>
                        <option value="" selected=move || with.get().is_none()>"With: all projects"</option>
                        {peers.into_iter().map(|p| {
                            let selected = p.clone();
                            view! { <option value=p.clone() selected=move || with.get().as_deref() == Some(selected.as_str())>{format!("With: {p}")}</option> }
                        }).collect_view()}
                    </select>
                }
            })}
        </header>
        <StatusLine message=mailbox.status />
        {stale.map(|since| view! {
            <p class="muted stale" title="Aurora shows what it read last and keeps trying">
                {format!("Not reachable now, showing the last messages read (since {since}).")}
            </p>
        })}
        {match inbox.setup.clone() {
            InboxSetup::NotSetUp { command } => Some(view! {
                <div class="banner warn">"Not set up on the branch checked out: run " <code>{command}</code>", then commit .aurora/integrations.json."</div>
            }.into_any()),
            InboxSetup::Unavailable { reason } => Some(view! {
                <div class="banner warn">"Not reachable: " {sentence(&reason)}</div>
            }.into_any()),
            InboxSetup::Ready => None,
        }}
        <div class="stream" node_ref=stream on:scroll=on_scroll on:keydown=on_keydown role="log" aria-label=format!("Messages of {name}")>
            {(!has_entries).then(|| view! {
                <p class="muted empty">"No messages yet. Start a conversation below."</p>
            })}
            {entries.into_iter().map(|entry| match entry {
                ChatEntry::Day { date } => view! {
                    <div class="day" role="separator"><span>{day_label(&date, &today)}</span></div>
                }.into_any(),
                ChatEntry::Bubble(b) => view! {
                    <BubbleLine bubble=*b opened=opened on_open=open_bubble logos=logos registered=registered.clone() />
                }.into_any(),
                ChatEntry::Record(r) => view! {
                    <RecordLine record=*r project=project_id(&inbox) registered=registered.clone() mailbox=mailbox pick=pick />
                }.into_any(),
            }).collect_view()}
        </div>
        {move || (!at_bottom.get()).then(|| view! {
            <button class="btn to-bottom" on:click=move |_| to_bottom()>"Newest ↓"</button>
        })}
        <div class="compose-bar">
            {move || if composing.get() {
                let from = inbox_for_compose.clone();
                view! {
                    <NewConversation inbox=from on_done=Callback::new(move |text| {
                        composing.set(false);
                        mailbox.done(text);
                    }) on_cancel=Callback::new(move |()| composing.set(false)) />
                }.into_any()
            } else {
                view! {
                    <span class="muted hint">"Click a message to read it in full and reply."</span>
                    <button class="btn primary" disabled=inbox.setup != InboxSetup::Ready on:click=move |_| composing.set(true)>"New conversation"</button>
                }.into_any()
            }}
        </div>
    }
}

/// A project registered in Aurora, by its slug in messages.
#[derive(Debug, Clone)]
struct Registered {
    slug: String,
    id: String,
    name: String,
}

fn project_id(inbox: &ProjectInbox) -> String {
    inbox.project.0.clone()
}

/// One bubble: received on the left with the sender, sent on the right.
#[component]
fn BubbleLine(
    bubble: ChatBubble,
    opened: RwSignal<Option<String>>,
    on_open: Callback<Message>,
    logos: RwSignal<HashMap<String, String>>,
    registered: Vec<Registered>,
) -> impl IntoView {
    let h = bubble.message.header.clone();
    let id = h.id.clone();
    let is_open = {
        let id = id.clone();
        Memo::new(move |_| opened.get().as_deref() == Some(id.as_str()))
    };
    let side = if bubble.outgoing {
        "line out"
    } else {
        "line in"
    };
    let time = local_time(&h.date, offset_minutes())
        .map(|(_, t)| t)
        .unwrap_or_default();
    let round = h.round.map(|r| format!(" · round {r}")).unwrap_or_default();
    let sender = registered.iter().find(|r| r.slug == h.from);
    let sender_logo = sender.and_then(|r| logos.with_untracked(|l| l.get(&r.id).cloned()));
    // A registered sender has the same picture and color as its contact.
    let sender_name = sender.map_or_else(|| h.from.clone(), |r| r.name.clone());
    let color = AVATAR_CLASSES[avatar_color(&sender_name, AVATAR_CLASSES.len())];
    let message = bubble.message.clone();
    let quote_target = bubble.quote.as_ref().map(|q| q.id.clone());
    let mark = bubble.mark;
    view! {
        <div class=side id=format!("msg-{id}")>
            {(!bubble.outgoing).then(|| view! {
                <Avatar name=sender_name.clone() logo=sender_logo small=true />
            })}
            <button
                class=move || if is_open.get() { "bubble selected" } else { "bubble" }
                aria-pressed=move || is_open.get().to_string()
                on:click=move |_| on_open.run(message.clone())
            >
                {(!bubble.outgoing).then(|| view! { <span class=format!("who {color}")>{h.from.clone()}</span> })}
                {bubble.opens.then(|| view! {
                    <span class="subj"><b>{bubble.subject.clone()}</b>{format!(" · {}{round}", h.kind)}</span>
                })}
                {bubble.quote.clone().map(|q| {
                    let target = quote_target.clone();
                    view! {
                        <span class="quote" role="link" tabindex="-1"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                if let Some(t) = &target
                                    && let Some(el) = document().get_element_by_id(&format!("msg-{t}"))
                                {
                                    el.scroll_into_view();
                                }
                            }
                        >
                            <span class="qw">{q.from}</span>
                            <span class="qt">{q.text}</span>
                        </span>
                    }
                })}
                <span class="txt">{bubble.text.clone()}</span>
                <span class="foot">
                    {(!bubble.opens).then(|| view! { <span class="kind">{h.kind.to_string()}</span> })}
                    {bubble.outgoing.then(|| view! { <span class="to">{format!("to {}", h.to)}</span> })}
                    <span>{time}</span>
                    {bubble.outgoing.then(|| view! {
                        <span class=format!("ticks mark-{}", mark.key()) title=mark.label() aria-label=mark.label()>{mark.symbol()}</span>
                    })}
                    {bubble.unread.then(|| view! { <span class="new" aria-label="Unread">"● new"</span> })}
                </span>
            </button>
        </div>
    }
}

/// A closing record in the stream: the decision card while it waits for the user, then a
/// closed card with the choice.
#[component]
fn RecordLine(
    record: ChatRecord,
    project: String,
    registered: Vec<Registered>,
    mailbox: Mailbox,
    pick: Callback<String>,
) -> impl IntoView {
    let decided_here = mailbox
        .decided
        .with_untracked(|d| d.contains(&record.thread));
    let context = format!("{} · {}", record.subject, record.kind);
    let class = if record.kind == MessageKind::Impasse {
        "record impasse"
    } else {
        "record"
    };
    match record.state {
        RecordState::Waiting if !decided_here => view! {
            <div class=class>
                <DecisionCard project=project record=record.record mailbox=mailbox context=context />
            </div>
        }
        .into_any(),
        RecordState::Waiting => view! {
            <div class="record decided">
                <p class="label">{format!("✓ Decision sent · {context}")}</p>
            </div>
        }
        .into_any(),
        RecordState::Decided { choice, date } => view! {
            <div class="record decided">
                <p class="label">{format!("✓ Decided · {context}")}</p>
                <p class="choice">{choice}</p>
                <p class="muted">{short_date(&date)}</p>
            </div>
        }
        .into_any(),
        RecordState::Pending => {
            let opener = record.record.projects.iter().find_map(|slug| {
                registered
                    .iter()
                    .find(|r| r.slug == *slug && r.id != project)
                    .map(|r| (r.slug.clone(), r.id.clone()))
            });
            view! {
                <div class=format!("{class} pending")>
                    <p class="label">{format!("⚑ Closing record · {context}")}</p>
                    <p class="question">{record.record.conclusion.clone()}</p>
                    {opener.map(|(slug, id)| view! {
                        <p class="muted">"Decide it in the chat of " <button class="link" on:click=move |_| pick.run(id.clone())>{slug}</button> "."</p>
                    })}
                </div>
            }
            .into_any()
        }
    }
}

/// The full message in the panel on the right (US3).
#[component]
fn MessageDetail(
    message: Message,
    view: ConversationView,
    opened: RwSignal<Option<String>>,
    mailbox: Mailbox,
) -> impl IntoView {
    let h = message.header.clone();
    let round = h
        .round
        .map(|r| format!(" · round {r} of {}", aurora_core::domain::MAX_ROUNDS))
        .unwrap_or_default();
    let summary = h.summary.trim().to_owned();
    let form = RwSignal::new(Form::None);
    let answering = (h.to == view.me).then(|| h.id.clone());
    let panel = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        if let Some(el) = panel.get() {
            let _ = el.focus();
        }
    });
    view! {
        <div class="detail-inner" tabindex="-1" node_ref=panel>
            <div class="detail-head">
                <span class="badge info">{h.kind.to_string()}</span>
                <span class="muted">{format!("{}{round}", short_date(&h.date))}</span>
                <button class="btn icon close" aria-label="Close the message" title="Close (Esc)" on:click=move |_| opened.set(None)>"✕"</button>
            </div>
            <div class="detail-body">
                <h2 class="route">{format!("{} → {}", h.from, h.to)}</h2>
                <p class="panel-subject">{h.subject.clone()}</p>
                {(!summary.is_empty()).then(|| view! {
                    <div class="sumbox"><span class="k">"Summary by the sender"</span><p>{summary.clone()}</p></div>
                })}
                <MessageBody message=message.clone() />
                <Forms view=view answering=answering form=form mailbox=mailbox />
            </div>
        </div>
    }
}

/// What the detail panel shows open below the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Form {
    None,
    Reply,
    StepIn,
}

/// The summary field of Reply and New conversation (US5).
#[component]
fn SummaryField(value: RwSignal<String>, id: String) -> impl IntoView {
    let count = move || {
        value.with(|v| {
            v.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .count()
        })
    };
    view! {
        <div class=move || { if count() > MAX_SUMMARY_CHARS { "field missing" } else { "field" } }>
            <label for=id.clone()>"Summary " <span class="muted">"(what the receiver reads in the chat)"</span></label>
            <textarea id=id rows="2"
                placeholder="One or two sentences in the first person, for example: I agree, with one change to the cap."
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))></textarea>
            <span class=move || { if count() > MAX_SUMMARY_CHARS { "counter over" } else { "counter muted" } }>
                {move || format!("{} / {MAX_SUMMARY_CHARS}", count())}
            </span>
        </div>
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
fn DecisionCard(
    project: String,
    record: ClosingRecord,
    mailbox: Mailbox,
    #[prop(optional)] context: String,
) -> impl IntoView {
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
            <p class="label">{if context.is_empty() { "⚑ Your decision".to_owned() } else { format!("⚑ Your decision · {context}") }}</p>
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
fn Forms(
    view: ConversationView,
    answering: Option<String>,
    form: RwSignal<Form>,
    mailbox: Mailbox,
) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let c = view.conversation.clone();
    let me = view.me.clone();
    let project = view.project.0.clone();
    let thread = c.thread.clone();
    let summary = RwSignal::new(String::new());
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
                        summary.set(String::new());
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
                    summary: summary.get_untracked(),
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
                        <SummaryField value=summary id="reply-summary".to_owned() />
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
fn NewConversation(
    inbox: ProjectInbox,
    on_done: Callback<String>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let from = inbox.project.0.clone();
    let from_name = inbox.name.clone();
    let counterparts = inbox.counterparts.clone();
    let to = RwSignal::new(counterparts.first().cloned().unwrap_or_default());
    let summary = RwSignal::new(String::new());
    let subject = RwSignal::new(String::new());
    let kind = RwSignal::new(MessageKind::Question);
    let fields: Fields = RwSignal::new(Vec::new());
    let error = RwSignal::new(None::<CommandError>);
    let busy = RwSignal::new(false);
    Effect::new(move |_| load_template(ctx, kind.get(), fields, error));
    let send = move |_| {
        let draft = Draft {
            to: to.get_untracked(),
            kind: kind.get_untracked(),
            subject: subject.get_untracked(),
            summary: summary.get_untracked(),
            sections: sections_of(&fields.get_untracked()),
        };
        let project = from.clone();
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
    let has_peers = !counterparts.is_empty();
    view! {
        <section class="card compose" aria-labelledby="compose-title">
            <h2 id="compose-title">{format!("New conversation from {from_name}")}</h2>
            {(!has_peers).then(|| view! {
                <p class="muted">"This project has no counterpart in .aurora/integrations.json yet."</p>
            })}
            <div class="settings-toolbar">
                <label for="compose-to">"To"</label>
                <select id="compose-to" on:change=move |ev| to.set(event_target_value(&ev))>
                    {counterparts.into_iter().map(|slug| {
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
            <SummaryField value=summary id="compose-summary".to_owned() />
            <SectionFields fields=fields prefix="compose".to_owned() missing=missing_of(error) />
            <Failure error=error />
            <div class="actions">
                <button class="btn primary" disabled=move || busy.get() || !has_peers on:click=send>"Send"</button>
                <button class="btn" on:click=move |_| on_cancel.run(())>"Cancel"</button>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
