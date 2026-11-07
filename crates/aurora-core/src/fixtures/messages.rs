//! An in-memory `MessagesSource` with sample conversations, for tests and the interface
//! preview: `aurora-dev-suite`, `any-context` and `ai-code-agent-skills` (as
//! `dev-cycle-protocol`) are set up, `gedo` is not. The registries also give `gedo` its
//! repository URL, and a folder `levix-gedo` is a clone of it under another name.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use super::{REPOS, lock};
use crate::domain::{
    ClosingRecord, Conversation, EngineReport, EngineSettings, MAX_ROUNDS, Message, MessageHeader,
    MessageKind, MessageStatus, MessagesError, Section, TemplateSection, UserAction, message_body,
    record_options,
};
use crate::ports::{InboxRead, ListedProject, MessagesSource};

const PROJECTS: [(&str, &str); 4] = [
    ("aurora-dev-suite", "aurora-dev-suite"),
    ("any-context", "any-context"),
    ("ai-code-agent-skills", "dev-cycle-protocol"),
    ("levix-platform", "levix-platform"),
];

/// The repository URL of the project in `folder`, as the registries list it.
fn listed_url(folder: &str) -> String {
    format!("https://github.com/Levix-Digital/{folder}.git")
}

/// The `origin` of a folder not set up, spelled unlike the registries on purpose.
fn origin_of(folder: &str) -> String {
    match folder {
        "levix-gedo" => "git@github.com:Levix-Digital/gedo".to_owned(),
        _ => format!("https://github.com/Levix-Digital/{folder}/"),
    }
}

#[derive(Debug, Default)]
struct State {
    messages: Vec<Message>,
    clock: u32,
    polled: BTreeSet<String>,
    calls: usize,
}

/// The sample conversations, changed by the actions.
#[derive(Debug, Default)]
pub struct FixtureMessages {
    state: Mutex<State>,
}

fn slug_of(root: &Path) -> Result<&'static str, MessagesError> {
    let name = root
        .strip_prefix(REPOS)
        .ok()
        .map(|r| r.to_string_lossy().into_owned())
        .unwrap_or_default();
    PROJECTS
        .iter()
        .find(|(folder, _)| *folder == name)
        .map(|(_, slug)| *slug)
        .ok_or_else(|| MessagesError::NotSetUp {
            command: crate::usecases::messages::SETUP_COMMAND.to_owned(),
            repository: Some(origin_of(&name)),
        })
}

fn date(n: u32) -> String {
    format!(
        "2026-11-0{}T{:02}:{:02}:{:02}.000Z",
        1 + (n / 600).min(8),
        9 + (n / 60) % 10,
        n % 60,
        (n * 7) % 60
    )
}

fn sections(text: &[(&str, &str)]) -> Vec<Section> {
    text.iter().map(|(n, t)| Section::new(n, t)).collect()
}

#[allow(clippy::too_many_arguments)]
fn message(
    n: u32,
    thread: &str,
    kind: MessageKind,
    from: &str,
    to: &str,
    subject: &str,
    round: Option<u32>,
    cycle: u32,
    body: &[(&str, &str)],
) -> Message {
    let slug: String = subject
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let id = format!(
        "2026110{}-{:06}-{from}-{}",
        1 + (n / 600).min(8),
        n,
        slug.trim_matches('-')
    );
    let sections = sections(body);
    Message {
        header: MessageHeader {
            thread: if thread.is_empty() {
                id.clone()
            } else {
                thread.to_owned()
            },
            id,
            needs_user: kind.is_closing(),
            kind,
            from: from.into(),
            to: to.into(),
            date: date(n),
            subject: subject.into(),
            status: MessageStatus::Open,
            reply_to: String::new(),
            deferred: String::new(),
            cycle,
            round,
            tool: "0.3.0".into(),
        },
        body: message_body(&sections),
        sections,
        text: String::new(),
        notes: Vec::new(),
    }
}

const ADS: &str = "aurora-dev-suite";
const DCP: &str = "dev-cycle-protocol";

fn sample_messages() -> Vec<Message> {
    let mut out = Vec::new();
    let q = message(
        1,
        "",
        MessageKind::Question,
        ADS,
        DCP,
        "Shared attribution settings file",
        Some(1),
        1,
        &[
            (
                "Ask",
                "Can the shared settings file that turns attribution off be committed?",
            ),
            (
                "Context",
                "Every project keeps the rule that no repository content mentions the tools that wrote it.",
            ),
            (
                "Closes when",
                "Both sides record the same sentence in their instructions.",
            ),
        ],
    );
    let t1 = q.header.id.clone();
    out.push(q);
    out.push(message(
        2,
        &t1,
        MessageKind::Answer,
        DCP,
        ADS,
        "Re: Shared attribution settings file",
        Some(1),
        1,
        &[
            ("Answer", "Agree, with one change."),
            (
                "Per point",
                "1. Commit only the shared file, never personal settings.",
            ),
            ("Next step", "Aurora answers the change."),
        ],
    ));
    out.push(message(
        3,
        &t1,
        MessageKind::Answer,
        ADS,
        DCP,
        "Re: Shared attribution settings file",
        Some(2),
        1,
        &[
            ("Answer", "Agree."),
            (
                "Next step",
                "The Dev Cycle Protocol closes the conversation.",
            ),
        ],
    ));
    out.push(message(4, &t1, MessageKind::Agreement, DCP, ADS, "Re: Shared attribution settings file", Some(2), 1, &[
        ("Answer", "Agree."),
        ("Conclusion", "The shared settings file is committed; personal settings never are."),
        ("Justification", "It keeps the rule in one place for every contributor."),
        ("Decision needed", "1. Adopt the new sentence in every project (recommended)\n2. Keep the current sentence"),
        ("After approval", "Each project updates its instructions and records it in its backlog."),
    ]));
    let p = message(
        20,
        "",
        MessageKind::Proposal,
        DCP,
        ADS,
        "Hook timeouts",
        Some(1),
        1,
        &[
            ("Ask", "Agree on a time limit per hook point."),
            ("Context", "DCP FEAT-018, draft contract."),
            (
                "Proposal",
                "A `timeouts` object in `describe`, up to 300 seconds.",
            ),
            ("Closes when", "Both sides accept the limits."),
        ],
    );
    let t2 = p.header.id.clone();
    out.push(p);
    out.push(message(
        21,
        &t2,
        MessageKind::Counter,
        ADS,
        DCP,
        "Re: Hook timeouts",
        Some(1),
        1,
        &[
            ("Answer", "Counter."),
            ("Per point", "1. 120 seconds for after-merge is enough."),
            ("Next step", "The Dev Cycle Protocol answers."),
        ],
    ));
    out.push(message(
        22,
        &t2,
        MessageKind::Answer,
        DCP,
        ADS,
        "Re: Hook timeouts",
        Some(2),
        1,
        &[
            (
                "Answer",
                "Partly: keep 300 as the cap, 120 as the after-merge default.",
            ),
            ("Next step", "Aurora answers or closes."),
        ],
    ));
    out.push(message(
        23,
        &t2,
        MessageKind::Agreement,
        ADS,
        DCP,
        "Re: Hook timeouts",
        Some(2),
        1,
        &[
            ("Answer", "Agree."),
            (
                "Conclusion",
                "A 300-second cap, 120 seconds by default after a merge.",
            ),
            (
                "Justification",
                "Long checks still finish; a hung hook never blocks for long.",
            ),
            (
                "Decision needed",
                "1. Accept the limits (recommended)\n2. Keep one limit of 30 seconds",
            ),
            (
                "After approval",
                "The Dev Cycle Protocol updates the draft contract.",
            ),
        ],
    ));
    out.push(message(
        40,
        "",
        MessageKind::Question,
        ADS,
        "any-context",
        "Knowledge search contract",
        Some(1),
        1,
        &[
            ("Ask", "Which JSON schema will `actx ask --json` answer?"),
            ("Context", "Aurora's knowledge view, v0.5.0."),
            ("Closes when", "A draft schema is published."),
        ],
    ));
    // A conversation that ended in a decision.
    const LP: &str = "levix-platform";
    let first = message(
        50,
        "",
        MessageKind::Question,
        ADS,
        LP,
        "Shared logging crate",
        Some(1),
        1,
        &[
            (
                "Ask",
                "Can levix-platform host the logging setup both apps use?",
            ),
            ("Context", "Aurora and Gedo configure tracing the same way."),
            ("Closes when", "A crate name and an owner are agreed."),
        ],
    );
    let t3 = first.header.thread.clone();
    out.push(first);
    out.push(message(
        51,
        &t3,
        MessageKind::Agreement,
        LP,
        ADS,
        "Re: Shared logging crate",
        Some(1),
        1,
        &[
            ("Conclusion", "levix-platform hosts `levix-logging`."),
            ("Justification", "Both apps already share the same setup."),
            (
                "Decision needed",
                "1. Create the crate (recommended)\n2. Keep it in each app",
            ),
            ("After approval", "levix-platform opens the crate."),
        ],
    ));
    out.push(message(
        52,
        &t3,
        MessageKind::Decision,
        ADS,
        LP,
        "Re: Shared logging crate",
        None,
        1,
        &[("Decision", "Create the crate")],
    ));
    // A notice, with nothing to answer.
    out.push(message(
        60,
        "",
        MessageKind::Notice,
        DCP,
        ADS,
        "dcp v2.11.0 renames the binary",
        None,
        1,
        &[("Notice", "The engine binary is `dcp` from v2.11.0.")],
    ));
    // Everything was read except the last message of each conversation; closed ones
    // were handled.
    let mut last_of: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, m) in out.iter().enumerate() {
        last_of.insert(m.header.thread.clone(), i);
    }
    for (i, m) in out.iter_mut().enumerate() {
        let closed = m.header.thread == t3 || m.header.kind == MessageKind::Notice;
        if closed {
            m.header.status = MessageStatus::Done;
        } else if last_of.get(&m.header.thread) != Some(&i) {
            m.header.status = MessageStatus::Seen;
        }
    }
    out
}

fn rounds_used(messages: &[&Message]) -> (u32, u32, Option<u32>) {
    let cut = messages
        .iter()
        .rposition(|m| m.header.kind == MessageKind::Guidance);
    let cycle = 1 + messages
        .iter()
        .filter(|m| m.header.kind == MessageKind::Guidance)
        .count() as u32;
    let current = &messages[cut.map_or(0, |i| i + 1)..];
    let counted = current
        .iter()
        .filter(|m| !m.header.kind.is_outside_rounds())
        .count() as u32;
    let round = current.iter().filter_map(|m| m.header.round).max();
    (cycle, counted / 2 + 1, round)
}

fn template_of(kind: &MessageKind) -> Vec<TemplateSection> {
    let t = |name: &str, hint: &str, required: bool| TemplateSection {
        name: name.into(),
        hint: hint.into(),
        required,
    };
    let ask = t(
        "Ask",
        "One or two sentences: what the receiver should do or answer.",
        true,
    );
    let context = t("Context", "Only what the receiver needs.", true);
    let constraints = t("Constraints", "Limits the answer must respect.", false);
    let closes = t(
        "Closes when",
        "The answer that ends the conversation.",
        true,
    );
    let answer = t(
        "Answer",
        "Agree | counter | question | impasse, in one line.",
        true,
    );
    let next = |required| {
        t(
            "Next step",
            "Who does what, and the record each project keeps.",
            required,
        )
    };
    let open = t("Open points", "What is still undecided.", false);
    match kind {
        MessageKind::Question => vec![ask, context, constraints, closes],
        MessageKind::Proposal => vec![
            ask,
            context,
            t("Proposal", "What you propose, point by point.", true),
            constraints,
            closes,
        ],
        MessageKind::Answer => vec![
            answer,
            t("Per point", "One line per point.", false),
            open,
            next(true),
        ],
        MessageKind::Counter => vec![
            answer,
            t("Per point", "One line per point.", true),
            open,
            next(true),
        ],
        MessageKind::Agreement | MessageKind::Impasse => {
            let mut v = vec![
                answer,
                t("Conclusion", "What was concluded.", true),
                t("Justification", "Why.", true),
            ];
            if *kind == MessageKind::Impasse {
                v.push(t("Positions", "Each side's position.", true));
            }
            v.push(t("Decision needed", "Numbered options for the user.", true));
            v.push(t("After approval", "What each project does then.", true));
            v
        }
        _ => Vec::new(),
    }
}

impl FixtureMessages {
    pub fn sample() -> Self {
        Self {
            state: Mutex::new(State {
                messages: sample_messages(),
                clock: 100,
                ..State::default()
            }),
        }
    }

    /// How many actions reached it.
    pub fn calls(&self) -> usize {
        lock(&self.state).calls
    }

    fn conversations(state: &State, me: &str) -> Vec<Conversation> {
        let mine: Vec<&Message> = state
            .messages
            .iter()
            .filter(|m| m.header.from == me || m.header.to == me)
            .collect();
        let mut threads: Vec<&str> = Vec::new();
        for m in &mine {
            if !threads.contains(&m.header.thread.as_str()) {
                threads.push(&m.header.thread);
            }
        }
        threads
            .into_iter()
            .map(|t| {
                let list: Vec<&Message> = mine
                    .iter()
                    .copied()
                    .filter(|m| m.header.thread == t)
                    .collect();
                let first = list[0];
                let last = list[list.len() - 1];
                let (cycle, next_round, round) = rounds_used(&list);
                let opener = PROJECTS
                    .iter()
                    .map(|(_, s)| *s)
                    .find(|s| t.contains(&format!("-{s}-")));
                let waiting = last.header.kind.is_closing()
                    && last.header.status != MessageStatus::Done
                    && last.header.to == me
                    && opener == Some(me);
                let mut projects: Vec<String> = list
                    .iter()
                    .flat_map(|m| [m.header.from.clone(), m.header.to.clone()])
                    .collect();
                projects.sort();
                projects.dedup();
                Conversation {
                    thread: t.to_owned(),
                    subject: first.header.subject.clone(),
                    projects,
                    opener: opener.map(str::to_owned),
                    cycle,
                    round,
                    next_round,
                    messages: list.len() as u32,
                    last: last.header.clone(),
                    waiting_for_user: waiting,
                    user_requests: list
                        .iter()
                        .filter(|m| m.header.kind == MessageKind::Guidance)
                        .filter_map(|m| m.sections.first().map(|s| s.text.clone()))
                        .collect(),
                    unread: list
                        .iter()
                        .filter(|m| m.header.to == me && m.header.status == MessageStatus::Open)
                        .count() as u32,
                    items: Vec::new(),
                    record: None,
                }
            })
            .collect()
    }

    fn push(state: &mut State, mut m: Message) -> MessageHeader {
        state.clock += 1;
        m.header.date = date(state.clock);
        m.header.id = format!("{}-{}", m.header.id, state.clock);
        if m.header.thread.is_empty() {
            m.header.thread = m.header.id.clone();
        }
        state.polled.clear();
        let header = m.header.clone();
        state.messages.push(m);
        header
    }

    fn close_records(state: &mut State, me: &str, thread: &str) {
        for m in state.messages.iter_mut().filter(|m| {
            m.header.thread == thread && m.header.to == me && m.header.kind.is_closing()
        }) {
            m.header.status = MessageStatus::Done;
        }
    }

    fn step_in(
        state: &mut State,
        me: &str,
        thread: &str,
        kind: MessageKind,
        sections: &[(&str, &str)],
    ) -> Vec<MessageHeader> {
        let list: Vec<&Message> = state
            .messages
            .iter()
            .filter(|m| m.header.thread == thread)
            .collect();
        let (cycle, _, _) = rounds_used(&list);
        let subject = list
            .first()
            .map(|m| m.header.subject.clone())
            .unwrap_or_default();
        let mut sides: Vec<String> = list
            .iter()
            .flat_map(|m| [m.header.from.clone(), m.header.to.clone()])
            .filter(|p| p != me)
            .collect();
        sides.sort();
        sides.dedup();
        let cycle = if kind == MessageKind::Guidance {
            cycle + 1
        } else {
            cycle
        };
        let prefix = if kind == MessageKind::Guidance {
            "User request"
        } else {
            "Decision"
        };
        let out = sides
            .iter()
            .map(|side| {
                let m = message(
                    0,
                    thread,
                    kind.clone(),
                    me,
                    side,
                    &format!("{prefix}: {subject}"),
                    None,
                    cycle,
                    sections,
                );
                Self::push(state, m)
            })
            .collect();
        Self::close_records(state, me, thread);
        out
    }
}

impl MessagesSource for FixtureMessages {
    fn inbox(
        &self,
        _engine: &EngineReport,
        _settings: &EngineSettings,
        root: &Path,
    ) -> Result<InboxRead, MessagesError> {
        let me = slug_of(root)?;
        let state = lock(&self.state);
        Ok(InboxRead {
            slug: me.to_owned(),
            counterparts: PROJECTS
                .iter()
                .map(|(_, s)| (*s).to_owned())
                .filter(|s| s != me)
                .collect(),
            listed: PROJECTS
                .iter()
                .chain([&("gedo", "gedo")])
                .map(|(folder, slug)| ListedProject {
                    slug: (*slug).to_owned(),
                    url: listed_url(folder),
                })
                .collect(),
            conversations: Self::conversations(&state, me),
        })
    }

    fn thread(
        &self,
        _engine: &EngineReport,
        _settings: &EngineSettings,
        root: &Path,
        thread: &str,
    ) -> Result<Vec<Message>, MessagesError> {
        let me = slug_of(root)?;
        let state = lock(&self.state);
        let list: Vec<Message> = state
            .messages
            .iter()
            .filter(|m| m.header.thread == thread && (m.header.from == me || m.header.to == me))
            .cloned()
            .collect();
        if list.is_empty() {
            return Err(MessagesError::NotFound {
                message: format!("thread {thread} is not known to {me}"),
            });
        }
        Ok(list)
    }

    fn record(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        thread: &str,
    ) -> Result<ClosingRecord, MessagesError> {
        let list = self.thread(engine, settings, root, thread)?;
        let refs: Vec<&Message> = list.iter().collect();
        let close = list
            .iter()
            .rev()
            .find(|m| m.header.kind.is_closing())
            .ok_or_else(|| MessagesError::NotFound {
                message: format!("thread {thread} has no agreement or impasse yet"),
            })?;
        let get = |name: &str| {
            close
                .sections
                .iter()
                .find(|s| s.name == name)
                .map(|s| s.text.clone())
                .unwrap_or_default()
        };
        let (cycle, _, round) = rounds_used(&refs);
        let mut projects: Vec<String> = list
            .iter()
            .flat_map(|m| [m.header.from.clone(), m.header.to.clone()])
            .collect();
        projects.sort();
        projects.dedup();
        let decision_needed = get("Decision needed");
        Ok(ClosingRecord {
            thread: thread.to_owned(),
            subject: list[0].header.subject.clone(),
            projects,
            rounds: round.unwrap_or(1).min(MAX_ROUNDS),
            cycle,
            outcome: close.header.kind.clone(),
            conclusion: get("Conclusion"),
            justification: get("Justification"),
            positions: get("Positions"),
            open_points: get("Open points"),
            options: record_options(&decision_needed),
            recommended: None,
            decision_needed,
            after_approval: get("After approval"),
            user_requests: list
                .iter()
                .filter(|m| m.header.kind == MessageKind::Guidance)
                .filter_map(|m| m.sections.first().map(|s| s.text.clone()))
                .collect(),
        })
    }

    fn template(
        &self,
        _engine: &EngineReport,
        _settings: &EngineSettings,
        kind: &MessageKind,
    ) -> Result<Vec<TemplateSection>, MessagesError> {
        Ok(template_of(kind))
    }

    fn act(
        &self,
        _engine: &EngineReport,
        _settings: &EngineSettings,
        root: &Path,
        action: &UserAction,
    ) -> Result<Vec<MessageHeader>, MessagesError> {
        let me = slug_of(root)?;
        let mut state = lock(&self.state);
        state.calls += 1;
        let state = &mut *state;
        match action {
            UserAction::Decide { thread, choice, .. } => Ok(Self::step_in(
                state,
                me,
                thread,
                MessageKind::Decision,
                &[("Decision", choice)],
            )),
            UserAction::Revise { thread, text } | UserAction::Guide { thread, text } => {
                Ok(Self::step_in(
                    state,
                    me,
                    thread,
                    MessageKind::Guidance,
                    &[("User request", text)],
                ))
            }
            UserAction::Send { draft } => {
                let mut m = message(
                    0,
                    "",
                    draft.kind.clone(),
                    me,
                    &draft.to,
                    &draft.subject,
                    Some(1),
                    1,
                    &[],
                );
                m.header.thread = String::new();
                m.body = message_body(&draft.sections);
                m.sections = draft.sections.clone();
                Ok(vec![Self::push(state, m)])
            }
            UserAction::Reply { id, kind, sections } => {
                let original = state
                    .messages
                    .iter()
                    .find(|m| &m.header.id == id && m.header.to == me)
                    .cloned()
                    .ok_or_else(|| MessagesError::NotFound {
                        message: format!("message {id} is not in the inbox of {me}"),
                    })?;
                let list: Vec<&Message> = state
                    .messages
                    .iter()
                    .filter(|m| m.header.thread == original.header.thread)
                    .collect();
                let (cycle, next_round, _) = rounds_used(&list);
                if next_round > MAX_ROUNDS && !kind.is_closing() && !kind.is_outside_rounds() {
                    return Err(MessagesError::Refused {
                        message: format!("the conversation has used its {MAX_ROUNDS} rounds"),
                        missing_sections: Vec::new(),
                    });
                }
                let subject = format!("Re: {}", original.header.subject.trim_start_matches("Re: "));
                let round = (!kind.is_outside_rounds()).then_some(next_round.min(MAX_ROUNDS));
                let mut m = message(
                    0,
                    &original.header.thread,
                    kind.clone(),
                    me,
                    &original.header.from,
                    &subject,
                    round,
                    cycle,
                    &[],
                );
                m.header.reply_to = id.clone();
                m.body = message_body(sections);
                m.sections = sections.clone();
                Ok(vec![Self::push(state, m)])
            }
            UserAction::MarkSeen { id } => {
                let m = state
                    .messages
                    .iter_mut()
                    .find(|m| &m.header.id == id && m.header.to == me)
                    .ok_or_else(|| MessagesError::NotFound {
                        message: format!("message {id} is not in the inbox of {me}"),
                    })?;
                m.header.status = MessageStatus::Seen;
                Ok(vec![m.header.clone()])
            }
        }
    }

    fn poll(
        &self,
        _engine: &EngineReport,
        _settings: &EngineSettings,
        root: &Path,
    ) -> Result<Option<String>, MessagesError> {
        let me = slug_of(root)?;
        let mut state = lock(&self.state);
        if !state.polled.insert(me.to_owned()) {
            return Ok(None);
        }
        let open = state
            .messages
            .iter()
            .filter(|m| m.header.to == me && m.header.status == MessageStatus::Open)
            .count();
        Ok((open > 0).then(|| format!("Inbox ({me}): {open} new message(s).")))
    }
}
