//! Messages between projects (spec 008): the closing records waiting for the user first,
//! with decide and "Ask for changes", then every conversation with its cycles and rounds,
//! a field to step into any of them, replies and new conversations from the templates.

use aurora_core::domain::{
    ClosingRecord, Conversation, Draft, Feature, InboxSetup, Message, MessageKind, MessagesError,
    ProjectInbox, Section, TemplateSection, UserAction,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::{FeatureState, StatusLine, Tone, sentence};
use crate::app::AppContext;
use crate::backend::CommandError;

/// Which conversations the list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Waiting,
    NoAnswer,
    UserCycle,
}

impl Filter {
    pub const ALL: [Filter; 4] = [
        Filter::All,
        Filter::Waiting,
        Filter::NoAnswer,
        Filter::UserCycle,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Waiting => "Waiting for you",
            Filter::NoAnswer => "No answer yet",
            Filter::UserCycle => "After your requests",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Filter::All => "all",
            Filter::Waiting => "waiting",
            Filter::NoAnswer => "no-answer",
            Filter::UserCycle => "user-cycle",
        }
    }

    pub fn keeps(self, conversation: &Conversation, me: &str) -> bool {
        match self {
            Filter::All => true,
            Filter::Waiting => conversation.waiting_for_user,
            Filter::NoAnswer => conversation.no_answer_yet(me),
            Filter::UserCycle => conversation.cycle > 1,
        }
    }
}

/// The total of closing records waiting for the user, for the navigation.
pub fn waiting_total(inboxes: &[ProjectInbox]) -> usize {
    inboxes.iter().map(ProjectInbox::waiting_count).sum()
}

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

#[component]
pub fn MessagesView() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let inboxes = RwSignal::new(None::<Result<Vec<ProjectInbox>, CommandError>>);
    let open = RwSignal::new(None::<(String, String)>);
    let composing = RwSignal::new(false);
    let project_filter = RwSignal::new(String::new());
    let filter = RwSignal::new(Filter::All);
    let status = RwSignal::new(None::<(Tone, String)>);
    let loading = RwSignal::new(false);

    let load = move || {
        loading.set(true);
        spawn_local(async move {
            let answer = ctx.backend.get_value().messages_overview().await;
            if let Ok(list) = &answer {
                ctx.messages_waiting.set(waiting_total(list));
            }
            inboxes.set(Some(answer));
            loading.set(false);
        });
    };
    load();
    // A background poll found news: offer to read again, so nothing being typed is lost.
    let has_news = RwSignal::new(false);
    Effect::new(move |previous: Option<u32>| {
        let news = ctx.messages_news.get();
        if previous.is_some_and(|p| p != news) {
            has_news.set(true);
        }
        news
    });
    // "Check now": the poll's summaries, then the inboxes again.
    let check_now = move |_| {
        spawn_local(async move {
            match ctx.backend.get_value().messages_poll().await {
                Ok(results) => {
                    let news: Vec<String> = results
                        .into_iter()
                        .filter(|r| r.changed)
                        .map(|r| r.summary)
                        .collect();
                    status.set(Some(if news.is_empty() {
                        (Tone::Info, "Nothing new.".to_owned())
                    } else {
                        (Tone::Ok, news.join(" "))
                    }));
                }
                Err(error) => status.set(Some((Tone::Error, error.message))),
            }
            load();
        });
    };
    let done = Callback::new(move |text: String| {
        status.set(Some((Tone::Ok, text)));
        load();
    });

    let list = move || inboxes.get().and_then(Result::ok).unwrap_or_default();

    view! {
        <section aria-labelledby="messages-title">
            <div class="view-header">
                <h1 id="messages-title">"Messages"</h1>
                <div class="actions">
                    <button class="btn" disabled=move || loading.get() on:click=check_now>
                        {move || if loading.get() { "Reading…" } else { "Check now" }}
                    </button>
                    <button class="btn primary" on:click=move |_| composing.update(|c| *c = !*c)>
                        "New conversation"
                    </button>
                </div>
            </div>
            <FeatureState feature=Feature::Messages />
            <StatusLine message=status />
            {move || has_news.get().then(|| view! {
                <div class="banner info" role="status">
                    "New messages arrived. "
                    <button class="btn" on:click=move |_| {
                        has_news.set(false);
                        load();
                    }>"Show them"</button>
                </div>
            })}
            {move || composing.get().then(|| view! {
                <NewConversation inboxes=Signal::derive(list) on_done=Callback::new(move |text| {
                    composing.set(false);
                    done.run(text);
                }) />
            })}
            {move || match inboxes.get() {
                None => view! { <p class="muted">"Reading the inboxes…"</p> }.into_any(),
                Some(Err(error)) => view! {
                    <div class="banner error" role="alert">{sentence(&error.message)}</div>
                }.into_any(),
                Some(Ok(all)) => view! {
                    <Waiting inboxes=all.clone() on_done=done />
                    <Setup inboxes=all.clone() />
                    <section class="section" aria-labelledby="conversations-title">
                        <h2 id="conversations-title">"Conversations"</h2>
                        <Filters inboxes=all.clone() project=project_filter filter=filter />
                        <ConversationList
                            inboxes=all
                            project=project_filter
                            filter=filter
                            open=open
                        />
                    </section>
                }.into_any(),
            }}
            {move || open.get().and_then(|(project, thread)| {
                let all = list();
                let inbox = all.into_iter().find(|i| i.project.0 == project)?;
                let conversation = inbox.conversations.iter().find(|c| c.thread == thread)?.clone();
                Some(view! {
                    <ConversationPanel
                        inbox=inbox.clone()
                        conversation=conversation
                        on_close=Callback::new(move |()| open.set(None))
                        on_done=done
                    />
                })
            })}
        </section>
    }
}

/// The closing records waiting for the user, first and in full.
#[component]
fn Waiting(inboxes: Vec<ProjectInbox>, on_done: Callback<String>) -> impl IntoView {
    let waiting: Vec<(ProjectInbox, Conversation)> = inboxes
        .iter()
        .flat_map(|inbox| {
            inbox
                .conversations
                .iter()
                .filter(|c| c.waiting_for_user)
                .map(move |c| (inbox.clone(), c.clone()))
        })
        .collect();
    let count = waiting.len();
    view! {
        <section class="section" aria-labelledby="waiting-title">
            <h2 id="waiting-title">{format!("Waiting for you ({count})")}</h2>
            {(count == 0).then(|| view! { <p class="muted">"No closing record waits for your decision."</p> })}
            {waiting
                .into_iter()
                .map(|(inbox, conversation)| view! {
                    <RecordCard inbox=inbox conversation=conversation on_done=on_done />
                })
                .collect_view()}
        </section>
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Pending {
    None,
    Decide(String),
}

#[component]
fn RecordCard(
    inbox: ProjectInbox,
    conversation: Conversation,
    on_done: Callback<String>,
) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let record = RwSignal::new(None::<Result<ClosingRecord, CommandError>>);
    let choice = RwSignal::new(String::new());
    let other = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let request = RwSignal::new(String::new());
    let pending = RwSignal::new(Pending::None);
    let error = RwSignal::new(None::<CommandError>);
    let busy = RwSignal::new(false);
    let project = inbox.project.0.clone();
    let thread = conversation.thread.clone();
    {
        let (project, thread) = (project.clone(), thread.clone());
        spawn_local(async move {
            let answer = ctx
                .backend
                .get_value()
                .messages_record(project, thread)
                .await;
            if let Ok(r) = &answer
                && let Some(first) = r.options.first()
            {
                choice.set(first.clone());
            }
            record.set(Some(answer));
        });
    }
    let act = {
        let (project, thread) = (project.clone(), thread.clone());
        move |action: UserAction, done: String| {
            let project = project.clone();
            busy.set(true);
            error.set(None);
            spawn_local(async move {
                match ctx.backend.get_value().messages_act(project, action).await {
                    Ok(_) => on_done.run(done),
                    Err(e) => error.set(Some(e)),
                }
                busy.set(false);
            });
            let _ = &thread;
        }
    };
    let chosen = move || {
        let c = choice.get();
        if c.is_empty() {
            other.get().trim().to_owned()
        } else {
            c
        }
    };
    let decide = {
        let (act, thread) = (act.clone(), thread.clone());
        move |_| {
            let Pending::Decide(text) = pending.get_untracked() else {
                return;
            };
            let n = note.get_untracked();
            act(
                UserAction::Decide {
                    thread: thread.clone(),
                    choice: text.clone(),
                    note: (!n.trim().is_empty()).then_some(n),
                },
                format!("Decision sent: {text}"),
            );
            pending.set(Pending::None);
        }
    };
    let revise = {
        let (act, thread) = (act.clone(), thread.clone());
        move |_| {
            act(
                UserAction::Revise {
                    thread: thread.clone(),
                    text: request.get_untracked(),
                },
                "Your request was sent: a new cycle of up to three rounds starts.".to_owned(),
            );
        }
    };
    let name_id = format!("choice-{}", conversation.thread);
    view! {
        <article class="card record" aria-label=format!("Closing record: {}", conversation.subject)>
            <div class="row">
                <h3>{conversation.subject.clone()}</h3>
                <span class="badge warn">{format!("{} · {}", inbox.name, conversation.last.kind)}</span>
            </div>
            {move || match record.get() {
                None => view! { <p class="muted">"Reading the record…"</p> }.into_any(),
                Some(Err(e)) => view! { <p class="banner error">{sentence(&e.message)}</p> }.into_any(),
                Some(Ok(r)) => {
                    let name_id = name_id.clone();
                    view! {
                        <p class="muted">
                            {format!(
                                "Discussed with {} ({} round(s), cycle {})",
                                r.projects.join(", "),
                                r.rounds,
                                r.cycle
                            )}
                        </p>
                        <dl class="record-fields">
                            <dt>"Conclusion"</dt><dd>{r.conclusion.clone()}</dd>
                            <dt>"Justification"</dt><dd>{r.justification.clone()}</dd>
                            {(!r.positions.is_empty()).then(|| view! { <dt>"Positions"</dt><dd>{r.positions.clone()}</dd> })}
                            {(!r.open_points.is_empty()).then(|| view! { <dt>"Open points"</dt><dd>{r.open_points.clone()}</dd> })}
                            {(!r.user_requests.is_empty()).then(|| view! {
                                <dt>"Your requests"</dt><dd>{r.user_requests.join(" / ")}</dd>
                            })}
                            <dt>"After approval"</dt><dd>{r.after_approval.clone()}</dd>
                        </dl>
                        <fieldset>
                            <legend>"Decision needed"</legend>
                            {r.options.iter().map(|option| {
                                let value = option.clone();
                                let checked = option.clone();
                                view! {
                                    <label class="choice">
                                        <input
                                            type="radio"
                                            name=name_id.clone()
                                            prop:checked=move || choice.get() == checked
                                            on:change=move |_| choice.set(value.clone())
                                        />
                                        " " {option.clone()}
                                    </label>
                                }
                            }).collect_view()}
                            <label class="choice">
                                <input
                                    type="radio"
                                    name=name_id.clone()
                                    prop:checked=move || choice.get().is_empty()
                                    on:change=move |_| choice.set(String::new())
                                />
                                " Another choice: "
                                <input
                                    type="text"
                                    aria-label="Another choice"
                                    prop:value=move || other.get()
                                    on:input=move |ev| {
                                        other.set(event_target_value(&ev));
                                        choice.set(String::new());
                                    }
                                />
                            </label>
                            <div class="field">
                                <label>"Note (optional)"</label>
                                <input type="text" prop:value=move || note.get() on:input=move |ev| note.set(event_target_value(&ev)) />
                            </div>
                        </fieldset>
                    }.into_any()
                }
            }}
            <Failure error=error />
            {move || match pending.get() {
                Pending::Decide(text) => view! {
                    <div class="banner warn" role="alert">
                        <p>{format!("Send the decision \"{text}\" to every project of this conversation?")}</p>
                        <div class="actions">
                            <button class="btn primary" disabled=move || busy.get() on:click=decide.clone()>"Confirm"</button>
                            <button class="btn" on:click=move |_| pending.set(Pending::None)>"Cancel"</button>
                        </div>
                    </div>
                }.into_any(),
                Pending::None => view! {
                    <div class="actions">
                        <button
                            class="btn primary"
                            disabled=move || busy.get() || chosen().is_empty()
                            on:click=move |_| pending.set(Pending::Decide(chosen()))
                        >"Decide"</button>
                    </div>
                }.into_any(),
            }}
            <div class="field ask-changes">
                <label for=format!("ask-{}", thread)>"Ask for changes"</label>
                <textarea
                    id=format!("ask-{}", thread)
                    rows="2"
                    placeholder="What the projects should change: a new cycle of up to three rounds follows."
                    prop:value=move || request.get()
                    on:input=move |ev| request.set(event_target_value(&ev))
                ></textarea>
                <div class="actions">
                    <button
                        class="btn"
                        disabled=move || busy.get() || request.get().trim().is_empty()
                        on:click=revise
                    >"Ask for changes"</button>
                </div>
            </div>
        </article>
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
        <section class="section" aria-labelledby="setup-title">
            <h2 id="setup-title">"Not taking part"</h2>
            <ul class="items">
                {lines.into_iter().map(|(name, setup)| match setup {
                    InboxSetup::NotSetUp { command } => view! {
                        <li><strong>{name}</strong>" — not set up: run " <code>{command}</code>" in its folder, then commit .aurora/integrations.json."</li>
                    }.into_any(),
                    InboxSetup::Unavailable { reason } => view! {
                        <li><strong>{name}</strong>" — not reachable: " {sentence(&reason)}</li>
                    }.into_any(),
                    InboxSetup::Ready => ().into_any(),
                }).collect_view()}
            </ul>
        </section>
    })
}

#[component]
fn Filters(
    inboxes: Vec<ProjectInbox>,
    project: RwSignal<String>,
    filter: RwSignal<Filter>,
) -> impl IntoView {
    let ready: Vec<(String, String)> = inboxes
        .iter()
        .filter(|i| i.setup == InboxSetup::Ready)
        .map(|i| (i.project.0.clone(), i.name.clone()))
        .collect();
    view! {
        <div class="settings-toolbar" role="group" aria-label="Filters">
            <label for="messages-project">"Project"</label>
            <select id="messages-project" on:change=move |ev| project.set(event_target_value(&ev))>
                <option value="" selected=move || project.get().is_empty()>"Every project"</option>
                {ready.into_iter().map(|(id, name)| {
                    let selected = id.clone();
                    view! { <option value=id selected=move || project.get() == selected>{name}</option> }
                }).collect_view()}
            </select>
            <label for="messages-filter">"Show"</label>
            <select id="messages-filter" on:change=move |ev| {
                let key = event_target_value(&ev);
                if let Some(f) = Filter::ALL.into_iter().find(|f| f.key() == key) {
                    filter.set(f);
                }
            }>
                {Filter::ALL.into_iter().map(|f| view! {
                    <option value=f.key() selected=move || filter.get() == f>{f.label()}</option>
                }).collect_view()}
            </select>
        </div>
    }
}

#[component]
fn ConversationList(
    inboxes: Vec<ProjectInbox>,
    project: RwSignal<String>,
    filter: RwSignal<Filter>,
    open: RwSignal<Option<(String, String)>>,
) -> impl IntoView {
    move || {
        let wanted = project.get();
        let f = filter.get();
        let rows: Vec<(ProjectInbox, Conversation)> = inboxes
            .iter()
            .filter(|i| {
                i.setup == InboxSetup::Ready && (wanted.is_empty() || i.project.0 == wanted)
            })
            .flat_map(|i| {
                let me = i.slug.clone().unwrap_or_default();
                i.conversations
                    .iter()
                    .filter(move |c| f.keeps(c, &me))
                    .map(move |c| (i.clone(), c.clone()))
            })
            .collect();
        if rows.is_empty() {
            return view! { <p class="muted empty">"No conversation to show."</p> }.into_any();
        }
        view! {
            <ul class="items conversations">
                {rows.into_iter().map(|(inbox, c)| {
                    let me = inbox.slug.clone().unwrap_or_default();
                    let target = (inbox.project.0.clone(), c.thread.clone());
                    let others: Vec<String> = c.projects.iter().filter(|p| **p != me).cloned().collect();
                    let round = c.round.map(|r| format!(", round {r}")).unwrap_or_default();
                    view! {
                        <li class="conversation-row">
                            <button class="link" on:click=move |_| open.set(Some(target.clone()))>
                                {c.subject.clone()}
                            </button>
                            <span class="muted">
                                {format!(
                                    " {} with {} · cycle {}{round} · last: {} from {} on {}",
                                    inbox.name,
                                    others.join(", "),
                                    c.cycle,
                                    c.last.kind,
                                    c.last.from,
                                    short_date(&c.last.date)
                                )}
                            </span>
                            {c.waiting_for_user.then(|| view! { " " <span class="badge warn">"Waiting for you"</span> })}
                            {c.no_answer_yet(&me).then(|| view! { " " <span class="badge muted">"No answer yet"</span> })}
                            {(c.cycle > 1).then(|| view! { " " <span class="badge info">{format!("{} request(s) of yours", c.user_requests.len())}</span> })}
                        </li>
                    }
                }).collect_view()}
            </ul>
        }
        .into_any()
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
fn ConversationPanel(
    inbox: ProjectInbox,
    conversation: Conversation,
    on_close: Callback<()>,
    on_done: Callback<String>,
) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let messages = RwSignal::new(None::<Result<Vec<Message>, CommandError>>);
    let guide = RwSignal::new(String::new());
    let error = RwSignal::new(None::<CommandError>);
    let busy = RwSignal::new(false);
    let me = inbox.slug.clone().unwrap_or_default();
    let project = inbox.project.0.clone();
    let thread = conversation.thread.clone();
    {
        let (project, thread) = (project.clone(), thread.clone());
        spawn_local(async move {
            messages.set(Some(
                ctx.backend
                    .get_value()
                    .messages_conversation(project, thread)
                    .await,
            ));
        });
    }
    let kinds = conversation.reply_kinds();
    let reply_kind = RwSignal::new(kinds.first().cloned().unwrap_or(MessageKind::Answer));
    let fields: Fields = RwSignal::new(Vec::new());
    Effect::new(move |_| load_template(ctx, reply_kind.get(), fields, error));

    let run = {
        let project = project.clone();
        move |action: UserAction, done: String, clear: Option<RwSignal<String>>| {
            let project = project.clone();
            busy.set(true);
            error.set(None);
            spawn_local(async move {
                match ctx.backend.get_value().messages_act(project, action).await {
                    Ok(_) => {
                        if let Some(c) = clear {
                            c.set(String::new());
                        }
                        on_done.run(done);
                    }
                    Err(e) => error.set(Some(e)),
                }
                busy.set(false);
            });
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
                Some(guide),
            )
        }
    };
    let me_for_reply = me.clone();
    let send_reply = move |_| {
        let Some(Ok(list)) = messages.get_untracked() else {
            return;
        };
        let Some(last) = list.iter().rev().find(|m| m.header.to == me_for_reply) else {
            error.set(Some(CommandError::new(
                "messages",
                "nothing was received in this conversation yet",
            )));
            return;
        };
        run(
            UserAction::Reply {
                id: last.header.id.clone(),
                kind: reply_kind.get_untracked(),
                sections: sections_of(&fields.get_untracked()),
            },
            "Reply sent.".to_owned(),
            None,
        );
    };
    let limited = conversation.next_round > aurora_core::domain::MAX_ROUNDS;
    view! {
        <section class="card conversation" aria-labelledby="conversation-title">
            <div class="row">
                <h2 id="conversation-title">{conversation.subject.clone()}</h2>
                <button class="btn" on:click=move |_| on_close.run(())>"Close"</button>
            </div>
            <p class="muted">
                {format!(
                    "{} · with {} · cycle {}{}",
                    inbox.name,
                    conversation.projects.iter().filter(|p| **p != me).cloned().collect::<Vec<_>>().join(", "),
                    conversation.cycle,
                    conversation.round.map(|r| format!(", round {r} of 3")).unwrap_or_default()
                )}
            </p>
            {(!conversation.user_requests.is_empty()).then(|| view! {
                <p>"Your requests: " {conversation.user_requests.join(" / ")}</p>
            })}
            {move || match messages.get() {
                None => view! { <p class="muted">"Reading the conversation…"</p> }.into_any(),
                Some(Err(e)) => view! { <p class="banner error">{sentence(&e.message)}</p> }.into_any(),
                Some(Ok(list)) => view! {
                    <ol class="messages">
                        {list.into_iter().map(|m| view! {
                            <li class=format!("message kind-{}", m.header.kind)>
                                <p class="muted">
                                    <strong>{m.header.from.clone()}</strong>
                                    {format!(
                                        " to {} · {} · {}{}",
                                        m.header.to,
                                        m.header.kind,
                                        short_date(&m.header.date),
                                        m.header.round.map(|r| format!(" · round {r}")).unwrap_or_default()
                                    )}
                                </p>
                                {if m.sections.is_empty() {
                                    view! { <pre class="body">{m.body.clone()}</pre> }.into_any()
                                } else {
                                    m.sections.into_iter().map(|s| view! {
                                        <h4>{s.name}</h4>
                                        <pre class="body">{s.text}</pre>
                                    }).collect_view().into_any()
                                }}
                            </li>
                        }).collect_view()}
                    </ol>
                }.into_any(),
            }}
            <Failure error=error />
            <div class="field">
                <label for="step-in">"Step in"</label>
                <textarea
                    id="step-in"
                    rows="2"
                    placeholder="Tell the projects what you want: a new cycle of up to three rounds follows."
                    prop:value=move || guide.get()
                    on:input=move |ev| guide.set(event_target_value(&ev))
                ></textarea>
                <div class="actions">
                    <button class="btn" disabled=move || busy.get() || guide.get().trim().is_empty() on:click=step_in>
                        "Send your request"
                    </button>
                </div>
            </div>
            <fieldset>
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
                    <button class="btn primary" disabled=move || busy.get() on:click=send_reply>"Send reply"</button>
                </div>
            </fieldset>
        </section>
    }
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
    use aurora_core::domain::{MessageHeader, MessageStatus, ProjectId};

    fn conversation(waiting: bool, cycle: u32, last_from: &str) -> Conversation {
        Conversation {
            thread: "t".into(),
            subject: "S".into(),
            projects: vec!["a".into(), "b".into()],
            opener: Some("a".into()),
            cycle,
            round: Some(1),
            next_round: 2,
            messages: 1,
            last: MessageHeader {
                id: "m".into(),
                kind: MessageKind::Question,
                from: last_from.into(),
                to: "b".into(),
                date: "2026-11-06T10:00:00.000Z".into(),
                subject: "S".into(),
                status: MessageStatus::Open,
                reply_to: String::new(),
                deferred: String::new(),
                thread: "t".into(),
                cycle,
                round: Some(1),
                needs_user: false,
                tool: String::new(),
            },
            waiting_for_user: waiting,
            user_requests: Vec::new(),
            unread: 0,
        }
    }

    #[test]
    fn filters_keep_what_they_say() {
        let c = conversation(true, 2, "a");
        assert!(Filter::ALL.iter().all(|f| f.keeps(&c, "a")));
        let quiet = conversation(false, 1, "b");
        assert!(!Filter::Waiting.keeps(&quiet, "a"));
        assert!(!Filter::NoAnswer.keeps(&quiet, "a"));
        assert!(!Filter::UserCycle.keeps(&quiet, "a"));
        assert_eq!(short_date("2026-11-06T10:00:00.000Z"), "2026-11-06 10:00");
    }

    #[test]
    fn the_navigation_count_adds_every_waiting_record() {
        let inbox = |waiting: bool| ProjectInbox {
            project: ProjectId("p".into()),
            name: "p".into(),
            slug: Some("a".into()),
            counterparts: Vec::new(),
            setup: InboxSetup::Ready,
            conversations: vec![conversation(waiting, 1, "b"), conversation(true, 1, "b")],
        };
        assert_eq!(waiting_total(&[inbox(true), inbox(false)]), 3);
        let refused = CommandError::from(MessagesError::Refused {
            message: "a 'question' message needs these sections".into(),
            missing_sections: vec!["Ask".into()],
        });
        let (text, missing) = failure_text(&refused);
        assert!(text.starts_with('A'));
        assert_eq!(missing, ["Ask"]);
    }
}
