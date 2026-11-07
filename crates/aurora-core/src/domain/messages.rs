//! Messages between projects (FEAT-031 to FEAT-034, ADR-0009): what Aurora shows of the
//! conversations `dcp-integrations` reads and writes, and the checks Aurora makes before
//! it asks the provider to write anything.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::ProjectId;

/// Rounds per cycle (ADR-0009).
pub const MAX_ROUNDS: u32 = 3;
/// The largest message body Aurora sends to the provider.
pub const MAX_BODY_BYTES: usize = 256 * 1024;

/// The kind of a message; kinds Aurora does not know are kept as text.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum MessageKind {
    Question,
    Proposal,
    Notice,
    Answer,
    Counter,
    Agreement,
    Impasse,
    Ack,
    Decision,
    Guidance,
    Other(String),
}

impl MessageKind {
    /// The kinds that open a conversation.
    pub const OPENING: [MessageKind; 3] = [
        MessageKind::Question,
        MessageKind::Proposal,
        MessageKind::Notice,
    ];
    /// The kinds of a reply.
    pub const REPLIES: [MessageKind; 6] = [
        MessageKind::Answer,
        MessageKind::Counter,
        MessageKind::Agreement,
        MessageKind::Impasse,
        MessageKind::Question,
        MessageKind::Ack,
    ];

    pub fn as_str(&self) -> &str {
        match self {
            MessageKind::Question => "question",
            MessageKind::Proposal => "proposal",
            MessageKind::Notice => "notice",
            MessageKind::Answer => "answer",
            MessageKind::Counter => "counter",
            MessageKind::Agreement => "agreement",
            MessageKind::Impasse => "impasse",
            MessageKind::Ack => "ack",
            MessageKind::Decision => "decision",
            MessageKind::Guidance => "guidance",
            MessageKind::Other(text) => text,
        }
    }

    /// An agreement or an impasse: a closing record for the user.
    pub fn is_closing(&self) -> bool {
        matches!(self, MessageKind::Agreement | MessageKind::Impasse)
    }

    /// Kinds that do not count in the rounds.
    pub fn is_outside_rounds(&self) -> bool {
        matches!(
            self,
            MessageKind::Ack | MessageKind::Notice | MessageKind::Decision | MessageKind::Guidance
        )
    }
}

impl From<String> for MessageKind {
    fn from(text: String) -> Self {
        match text.as_str() {
            "question" => MessageKind::Question,
            "proposal" => MessageKind::Proposal,
            "notice" => MessageKind::Notice,
            "answer" => MessageKind::Answer,
            "counter" => MessageKind::Counter,
            "agreement" => MessageKind::Agreement,
            "impasse" => MessageKind::Impasse,
            "ack" => MessageKind::Ack,
            "decision" => MessageKind::Decision,
            "guidance" => MessageKind::Guidance,
            _ => MessageKind::Other(text),
        }
    }
}

impl From<MessageKind> for String {
    fn from(kind: MessageKind) -> Self {
        kind.as_str().to_owned()
    }
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageStatus {
    Open,
    Seen,
    Done,
}

/// The header of one message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageHeader {
    pub id: String,
    pub kind: MessageKind,
    pub from: String,
    pub to: String,
    /// RFC 3339, UTC.
    pub date: String,
    pub subject: String,
    pub status: MessageStatus,
    pub reply_to: String,
    /// Why the receiver set it aside; empty when it did not.
    pub deferred: String,
    pub thread: String,
    pub cycle: u32,
    pub round: Option<u32>,
    pub needs_user: bool,
    /// Version of the tool that wrote it.
    pub tool: String,
}

/// One `##` or `###` section of a message body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub name: String,
    pub text: String,
}

impl Section {
    pub fn new(name: &str, text: &str) -> Self {
        Self {
            name: name.to_owned(),
            text: text.to_owned(),
        }
    }
}

/// A message with its sections in their order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub header: MessageHeader,
    pub sections: Vec<Section>,
    pub body: String,
}

/// One conversation as one project sees it: what it received and what it sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub thread: String,
    pub subject: String,
    pub projects: Vec<String>,
    /// The project that opened it, when known.
    pub opener: Option<String>,
    pub cycle: u32,
    /// The last round used in the current cycle.
    pub round: Option<u32>,
    /// The round the next message would be in.
    pub next_round: u32,
    pub messages: u32,
    pub last: MessageHeader,
    pub waiting_for_user: bool,
    pub user_requests: Vec<String>,
    /// Received messages of this conversation not read yet by the project.
    #[serde(default)]
    pub unread: u32,
}

/// Where a conversation stands for the user (spec 009, FR-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConversationState {
    /// A closing record waits for the user's decision.
    WaitingForYou,
    /// Rounds still running or an answer pending.
    InProgress,
    /// Decided, or only notices and acks.
    Closed,
}

impl ConversationState {
    /// The groups of the view, in their order.
    pub const ALL: [ConversationState; 3] = [
        ConversationState::WaitingForYou,
        ConversationState::InProgress,
        ConversationState::Closed,
    ];
}

impl Conversation {
    /// Waiting for the user first; then closed when the current cycle ended in a
    /// decision or an ack, holds only a notice, or is a tool 0.1 message set to done;
    /// otherwise in progress.
    pub fn state(&self) -> ConversationState {
        if self.waiting_for_user {
            return ConversationState::WaitingForYou;
        }
        let closed = match &self.last.kind {
            MessageKind::Decision => true,
            // An ack only acknowledges (a decision, a notice): the exchange ended.
            MessageKind::Ack => true,
            MessageKind::Notice => self.round.is_none(),
            MessageKind::Other(_) => self.last.status == MessageStatus::Done,
            _ => false,
        };
        if closed {
            ConversationState::Closed
        } else {
            ConversationState::InProgress
        }
    }

    /// The user should look at it: a record waits, or something arrived unread.
    pub fn needs_attention(&self) -> bool {
        self.waiting_for_user || self.unread > 0
    }

    /// The last message was sent by `me`, the conversation is not closed and
    /// nothing came back yet (a closed conversation never waits for an answer, BUG-007).
    pub fn no_answer_yet(&self, me: &str) -> bool {
        self.last.from == me
            && self.state() != ConversationState::Closed
            && !self.last.kind.is_outside_rounds()
            && !self.last.kind.is_closing()
    }

    /// The kinds a reply may have now: after the rounds of the cycle, only a closing
    /// record or an ack.
    pub fn reply_kinds(&self) -> Vec<MessageKind> {
        allowed_replies(self.next_round)
    }
}

/// The reply kinds allowed when the next message would be in round `next_round`.
pub fn allowed_replies(next_round: u32) -> Vec<MessageKind> {
    MessageKind::REPLIES
        .into_iter()
        .filter(|kind| next_round <= MAX_ROUNDS || kind.is_closing() || kind.is_outside_rounds())
        .collect()
}

/// What the user reads before deciding (ADR-0009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosingRecord {
    pub thread: String,
    pub subject: String,
    pub projects: Vec<String>,
    pub rounds: u32,
    pub cycle: u32,
    pub outcome: MessageKind,
    pub conclusion: String,
    pub justification: String,
    pub positions: String,
    pub open_points: String,
    pub decision_needed: String,
    pub options: Vec<String>,
    pub after_approval: String,
    pub user_requests: Vec<String>,
}

/// The numbered lines of a "Decision needed" section, without their numbers.
pub fn options(decision_needed: &str) -> Vec<String> {
    decision_needed
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let digits = line.find(|c: char| !c.is_ascii_digit())?;
            if digits == 0 {
                return None;
            }
            let rest = line[digits..].strip_prefix(['.', ')'])?;
            let text = rest.trim();
            (!text.is_empty()).then(|| text.to_owned())
        })
        .collect()
}

/// One section of a template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSection {
    pub name: String,
    pub hint: String,
    pub required: bool,
}

/// Whether a project can take part in messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum InboxSetup {
    Ready,
    /// No `.aurora/integrations.json`: the command that sets it up.
    NotSetUp {
        command: String,
    },
    /// The provider is missing, the project's folder is missing, or an inbox could not
    /// be read.
    Unavailable {
        reason: String,
    },
}

/// The messages of one registered project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInbox {
    pub project: ProjectId,
    pub name: String,
    /// The project's slug in messages, once set up.
    pub slug: Option<String>,
    pub counterparts: Vec<String>,
    pub setup: InboxSetup,
    /// In the order of [`order_conversations`].
    pub conversations: Vec<Conversation>,
}

impl ProjectInbox {
    /// Conversations that wait for the user or have unread received messages.
    pub fn open_count(&self) -> usize {
        self.conversations
            .iter()
            .filter(|c| c.needs_attention())
            .count()
    }

    pub fn waiting_count(&self) -> usize {
        self.conversations
            .iter()
            .filter(|c| c.waiting_for_user)
            .count()
    }
}

/// Waiting for the user first, then conversations in a cycle the user asked for, then
/// the rest; the most recent first within each group.
pub fn order_conversations(conversations: &mut [Conversation]) {
    conversations.sort_by(|a, b| {
        let group = |c: &Conversation| {
            if c.waiting_for_user {
                0
            } else if c.cycle > 1
                && !c.last.kind.is_closing()
                && c.last.kind != MessageKind::Decision
            {
                1
            } else {
                2
            }
        };
        group(a)
            .cmp(&group(b))
            .then_with(|| b.last.date.cmp(&a.last.date))
    });
}

/// A new conversation or a reply, as the user filled it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub to: String,
    pub kind: MessageKind,
    pub subject: String,
    pub sections: Vec<Section>,
}

/// The body of a message from its sections, empty ones left out.
pub fn body(sections: &[Section]) -> String {
    let mut out = String::new();
    for section in sections.iter().filter(|s| !s.text.trim().is_empty()) {
        if section.name.is_empty() {
            out.push_str(section.text.trim());
        } else {
            out.push_str(&format!("### {}\n{}", section.name, section.text.trim()));
        }
        out.push_str("\n\n");
    }
    out.trim_end().to_owned() + "\n"
}

/// What the user asks Aurora to do in a project's name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum UserAction {
    /// Approve one option of a closing record, or a choice of the user's own.
    Decide {
        thread: String,
        choice: String,
        note: Option<String>,
    },
    /// Ask for changes on a closing record: a new cycle.
    Revise {
        thread: String,
        text: String,
    },
    /// Step into any conversation: a new cycle.
    Guide {
        thread: String,
        text: String,
    },
    Send {
        draft: Draft,
    },
    #[serde(rename_all = "camelCase")]
    Reply {
        id: String,
        kind: MessageKind,
        sections: Vec<Section>,
    },
    MarkSeen {
        id: String,
    },
}

/// Why the provider could not do what Aurora asked.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MessagesError {
    #[error("dcp-integrations is not available: {hint}")]
    ProviderMissing { hint: String },
    #[error("dcp-integrations answers schema version {found}: update Aurora or the provider")]
    Unsupported { found: String },
    #[error("this project has no .aurora/integrations.json: run `{command}` in it")]
    NotSetUp {
        command: String,
        /// The project's repository URL (its `origin`), when the provider found it: the
        /// key to the slug the other projects already use for it (BUG-009).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        repository: Option<String>,
    },
    #[error("the project's folder is missing")]
    ProjectMissing,
    #[error("{message}")]
    #[serde(rename_all = "camelCase")]
    Refused {
        message: String,
        missing_sections: Vec<String>,
    },
    #[error("{message}")]
    NotFound { message: String },
    #[error("Git failed: {message}")]
    Git { message: String, stderr: String },
    #[error("dcp-integrations did not answer within {} s", *timeout_ms as f64 / 1000.0)]
    #[serde(rename_all = "camelCase")]
    Timeout { timeout_ms: u64 },
    #[error("dcp-integrations' reply is invalid: {message}")]
    InvalidReply { message: String },
}

impl MessagesError {
    fn refused(message: impl Into<String>, missing: Vec<String>) -> Self {
        MessagesError::Refused {
            message: message.into(),
            missing_sections: missing,
        }
    }
}

/// The result of polling one project's inbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResult {
    pub project: ProjectId,
    pub changed: bool,
    pub summary: String,
    /// The project has its `.aurora/integrations.json` (IMP-004).
    #[serde(default = "set_up_by_default")]
    pub set_up: bool,
}

fn set_up_by_default() -> bool {
    true
}

/// What Aurora last saw of each project's setup, so a file added by a pull or a
/// checkout (or removed) reads as news without reopening the view (IMP-004).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetupMemory {
    seen: std::collections::HashMap<ProjectId, bool>,
}

impl SetupMemory {
    /// Notes the setup the Messages view just read; unreachable projects say nothing.
    pub fn remember(&mut self, inboxes: &[ProjectInbox]) {
        for inbox in inboxes {
            match inbox.setup {
                InboxSetup::Ready => {
                    self.seen.insert(inbox.project.clone(), true);
                }
                InboxSetup::NotSetUp { .. } => {
                    self.seen.insert(inbox.project.clone(), false);
                }
                InboxSetup::Unavailable { .. } => {}
            }
        }
    }

    /// Marks as changed every result whose setup differs from the last one seen, and
    /// remembers the new ones. A project seen for the first time is not a change.
    pub fn note(&mut self, results: &mut [PollResult]) {
        for result in results {
            let before = self.seen.insert(result.project.clone(), result.set_up);
            if before.is_some_and(|b| b != result.set_up) {
                result.changed = true;
                if result.summary.is_empty() {
                    result.summary = if result.set_up {
                        format!("{} is now set up for messages.", result.project.0)
                    } else {
                        format!("{} is no longer set up for messages.", result.project.0)
                    };
                }
            }
        }
    }
}

/// The required sections left empty.
pub fn missing_sections(template: &[TemplateSection], sections: &[Section]) -> Vec<String> {
    template
        .iter()
        .filter(|t| t.required)
        .filter(|t| {
            !sections
                .iter()
                .any(|s| s.name.eq_ignore_ascii_case(&t.name) && !s.text.trim().is_empty())
        })
        .map(|t| t.name.clone())
        .collect()
}

fn check_body(sections: &[Section]) -> Result<(), MessagesError> {
    if body(sections).len() > MAX_BODY_BYTES {
        return Err(MessagesError::refused(
            format!("the message is larger than {} KiB", MAX_BODY_BYTES / 1024),
            Vec::new(),
        ));
    }
    Ok(())
}

fn check_text(text: &str, what: &str) -> Result<(), MessagesError> {
    if text.trim().is_empty() {
        return Err(MessagesError::refused(
            format!("{what} is empty"),
            Vec::new(),
        ));
    }
    if text.len() > MAX_BODY_BYTES {
        return Err(MessagesError::refused(
            format!("{what} is larger than {} KiB", MAX_BODY_BYTES / 1024),
            Vec::new(),
        ));
    }
    Ok(())
}

/// The checks before an action runs: empty text, required sections, kinds allowed at
/// this point of the conversation (`conversation`, for a reply) and size. `template` is
/// the template of the draft's or reply's kind, empty for free kinds.
pub fn check_action(
    action: &UserAction,
    template: &[TemplateSection],
    conversation: Option<&Conversation>,
) -> Result<(), MessagesError> {
    match action {
        UserAction::Decide { choice, .. } => check_text(choice, "the choice"),
        UserAction::Revise { text, .. } | UserAction::Guide { text, .. } => {
            check_text(text, "the request")
        }
        UserAction::MarkSeen { .. } => Ok(()),
        UserAction::Send { draft } => {
            if !MessageKind::OPENING.contains(&draft.kind) {
                return Err(MessagesError::refused(
                    "a new conversation starts with a question, a proposal or a notice",
                    Vec::new(),
                ));
            }
            check_text(&draft.to, "the receiver")?;
            check_text(&draft.subject, "the subject")?;
            check_sections(&draft.kind, template, &draft.sections)
        }
        UserAction::Reply { kind, sections, .. } => {
            if !MessageKind::REPLIES.contains(kind) {
                return Err(MessagesError::refused(
                    format!("'{kind}' is not a reply"),
                    Vec::new(),
                ));
            }
            if let Some(conversation) = conversation
                && !conversation.reply_kinds().contains(kind)
            {
                return Err(MessagesError::refused(
                    format!(
                        "the conversation has used its {MAX_ROUNDS} rounds in cycle {}: close it with an agreement or an impasse",
                        conversation.cycle
                    ),
                    Vec::new(),
                ));
            }
            check_sections(kind, template, sections)
        }
    }
}

fn check_sections(
    kind: &MessageKind,
    template: &[TemplateSection],
    sections: &[Section],
) -> Result<(), MessagesError> {
    let missing = missing_sections(template, sections);
    if !missing.is_empty() {
        return Err(MessagesError::refused(
            format!(
                "a '{kind}' message needs these sections, not empty: {}",
                missing.join(", ")
            ),
            missing,
        ));
    }
    if template.is_empty() && body(sections).trim().is_empty() {
        return Err(MessagesError::refused("the message is empty", Vec::new()));
    }
    check_body(sections)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(id: &str, kind: MessageKind, from: &str, date: &str) -> MessageHeader {
        MessageHeader {
            id: id.into(),
            kind,
            from: from.into(),
            to: "other".into(),
            date: date.into(),
            subject: "S".into(),
            status: MessageStatus::Open,
            reply_to: String::new(),
            deferred: String::new(),
            thread: id.into(),
            cycle: 1,
            round: Some(1),
            needs_user: false,
            tool: "1.0.0".into(),
        }
    }

    fn conversation(
        thread: &str,
        date: &str,
        kind: MessageKind,
        waiting: bool,
        cycle: u32,
    ) -> Conversation {
        Conversation {
            thread: thread.into(),
            subject: "S".into(),
            projects: vec!["a".into(), "b".into()],
            opener: Some("a".into()),
            cycle,
            round: Some(1),
            next_round: 2,
            messages: 2,
            last: header(thread, kind, "b", date),
            waiting_for_user: waiting,
            user_requests: Vec::new(),
            unread: 0,
        }
    }

    #[test]
    fn the_state_follows_the_conversation() {
        let state = |kind: MessageKind, waiting: bool, round: Option<u32>| {
            let mut c = conversation("t", "2026-11-06T10:00:00Z", kind, waiting, 1);
            c.round = round;
            c.state()
        };
        use ConversationState::*;
        assert_eq!(state(MessageKind::Agreement, true, Some(2)), WaitingForYou);
        assert_eq!(state(MessageKind::Question, false, Some(1)), InProgress);
        assert_eq!(state(MessageKind::Answer, false, Some(1)), InProgress);
        assert_eq!(state(MessageKind::Decision, false, Some(2)), Closed);
        // A revision after the decision opens cycle 2.
        assert_eq!(state(MessageKind::Guidance, false, None), InProgress);
        assert_eq!(state(MessageKind::Notice, false, None), Closed);
        assert_eq!(state(MessageKind::Ack, false, None), Closed);
        // The other side acknowledges a decision: still closed.
        assert_eq!(state(MessageKind::Ack, false, Some(2)), Closed);
        let mut old = conversation("t", "d", MessageKind::Other("message".into()), false, 1);
        assert_eq!(old.state(), InProgress);
        old.last.status = MessageStatus::Done;
        assert_eq!(old.state(), Closed);
    }

    #[test]
    fn the_open_count_adds_waiting_and_unread() {
        let mut quiet = conversation("q", "d", MessageKind::Answer, false, 1);
        let waiting = conversation("w", "d", MessageKind::Agreement, true, 1);
        let mut unread = conversation("u", "d", MessageKind::Question, false, 1);
        unread.unread = 2;
        let mut both = waiting.clone();
        both.unread = 1;
        let inbox = ProjectInbox {
            project: crate::domain::ProjectId("p".into()),
            name: "p".into(),
            slug: Some("a".into()),
            counterparts: Vec::new(),
            setup: InboxSetup::Ready,
            conversations: vec![quiet.clone(), waiting, unread, both],
        };
        assert_eq!(inbox.open_count(), 3);
        quiet.unread = 0;
        assert!(!quiet.needs_attention());
    }

    fn t(name: &str, required: bool) -> TemplateSection {
        TemplateSection {
            name: name.into(),
            hint: String::new(),
            required,
        }
    }

    #[test]
    fn kinds_round_trip_and_keep_unknown_ones() {
        for kind in MessageKind::REPLIES.iter().chain(&MessageKind::OPENING) {
            let json = serde_json::to_string(kind).unwrap();
            assert_eq!(&serde_json::from_str::<MessageKind>(&json).unwrap(), kind);
        }
        assert_eq!(
            serde_json::from_str::<MessageKind>("\"message\"").unwrap(),
            MessageKind::Other("message".into())
        );
        assert!(MessageKind::Impasse.is_closing() && MessageKind::Guidance.is_outside_rounds());
    }

    #[test]
    fn options_are_the_numbered_lines() {
        let text = "Choose one:\n1. Ship now (recommended)\n2) Wait\n  3. \nnot 4 this\n10. Later";
        assert_eq!(options(text), ["Ship now (recommended)", "Wait", "Later"]);
        assert!(options("").is_empty());
    }

    #[test]
    fn overview_order_is_waiting_then_user_cycles_then_recent() {
        let mut list = vec![
            conversation("old", "2026-11-01", MessageKind::Answer, false, 1),
            conversation("new", "2026-11-05", MessageKind::Answer, false, 1),
            conversation("cycle", "2026-11-02", MessageKind::Counter, false, 2),
            conversation("wait", "2026-10-30", MessageKind::Agreement, true, 1),
            conversation("decided", "2026-11-06", MessageKind::Decision, false, 2),
        ];
        order_conversations(&mut list);
        let order: Vec<&str> = list.iter().map(|c| c.thread.as_str()).collect();
        assert_eq!(order, ["wait", "cycle", "decided", "new", "old"]);
    }

    #[test]
    fn after_three_rounds_only_closing_or_ack() {
        assert_eq!(allowed_replies(3).len(), 6);
        assert_eq!(
            allowed_replies(4),
            [
                MessageKind::Agreement,
                MessageKind::Impasse,
                MessageKind::Ack
            ]
        );
        let mut c = conversation("x", "d", MessageKind::Counter, false, 1);
        c.next_round = 4;
        let reply = |kind| UserAction::Reply {
            id: "x".into(),
            kind,
            sections: vec![Section::new("Answer", "No.")],
        };
        let err = check_action(&reply(MessageKind::Counter), &[], Some(&c)).unwrap_err();
        assert!(err.to_string().contains("3 rounds in cycle 1"), "{err}");
        assert!(check_action(&reply(MessageKind::Ack), &[], Some(&c)).is_ok());
        assert!(c.no_answer_yet("b") && !c.no_answer_yet("a"));
    }

    #[test]
    fn the_setup_memory_reports_only_changes() {
        let result = |id: &str, set_up: bool| PollResult {
            project: ProjectId(id.into()),
            changed: false,
            summary: String::new(),
            set_up,
        };
        let inbox = |id: &str, setup: InboxSetup| ProjectInbox {
            project: ProjectId(id.into()),
            name: id.into(),
            slug: None,
            counterparts: Vec::new(),
            setup,
            conversations: Vec::new(),
        };
        let mut memory = SetupMemory::default();
        memory.remember(&[
            inbox(
                "a",
                InboxSetup::NotSetUp {
                    command: "c".into(),
                },
            ),
            inbox("b", InboxSetup::Ready),
            inbox("c", InboxSetup::Unavailable { reason: "r".into() }),
        ]);
        // a got its file, b lost it, c is seen for the first time, d did not change.
        let mut results = [result("a", true), result("b", false), result("c", false)];
        memory.note(&mut results);
        assert!(results[0].changed && results[1].changed && !results[2].changed);
        assert_eq!(results[0].summary, "a is now set up for messages.");
        assert_eq!(results[1].summary, "b is no longer set up for messages.");
        let mut again = [result("a", true), result("b", false), result("c", false)];
        memory.note(&mut again);
        assert!(again.iter().all(|r| !r.changed));
        let mut back = [result("c", true)];
        memory.note(&mut back);
        assert!(back[0].changed);
    }

    #[test]
    fn a_closed_conversation_waits_for_no_answer() {
        // BUG-007: a tool 0.1 reply of mine, marked done by the other side, ended it.
        let mut c = conversation("x", "d", MessageKind::Other("message".into()), false, 1);
        c.last.status = MessageStatus::Done;
        assert_eq!(c.state(), ConversationState::Closed);
        assert!(!c.no_answer_yet("b"));
        // Still open on the other side: it waits for an answer.
        c.last.status = MessageStatus::Seen;
        assert!(c.no_answer_yet("b"));
    }

    #[test]
    fn checks_before_running() {
        let template = [t("Ask", true), t("Context", true), t("Constraints", false)];
        let draft = |sections: Vec<Section>| UserAction::Send {
            draft: Draft {
                to: "gedo".into(),
                kind: MessageKind::Question,
                subject: "Dates".into(),
                sections,
            },
        };
        let Err(MessagesError::Refused {
            missing_sections, ..
        }) = check_action(
            &draft(vec![
                Section::new("Ask", "Why?"),
                Section::new("Context", " "),
            ]),
            &template,
            None,
        )
        else {
            panic!("refused");
        };
        assert_eq!(missing_sections, ["Context"]);
        let full = vec![
            Section::new("Ask", "Why?"),
            Section::new("context", "This."),
        ];
        assert!(check_action(&draft(full), &template, None).is_ok());
        let huge = vec![
            Section::new("Ask", &"x".repeat(MAX_BODY_BYTES)),
            Section::new("Context", "c"),
        ];
        assert!(check_action(&draft(huge), &template, None).is_err());
        let guide = UserAction::Guide {
            thread: "t".into(),
            text: "  ".into(),
        };
        assert!(check_action(&guide, &[], None).is_err());
        let free = UserAction::Reply {
            id: "x".into(),
            kind: MessageKind::Ack,
            sections: vec![Section::new("", "")],
        };
        assert!(check_action(&free, &[], None).is_err(), "an empty ack");
        let not_opening = UserAction::Send {
            draft: Draft {
                to: "gedo".into(),
                kind: MessageKind::Answer,
                subject: "x".into(),
                sections: Vec::new(),
            },
        };
        assert!(check_action(&not_opening, &[], None).is_err());
    }

    #[test]
    fn body_from_sections() {
        let sections = [
            Section::new("", "Free text."),
            Section::new("Ask", "Why?\n"),
            Section::new("Constraints", ""),
        ];
        assert_eq!(body(&sections), "Free text.\n\n### Ask\nWhy?\n");
    }

    #[test]
    fn actions_serialize_with_a_tag() {
        let action = UserAction::Revise {
            thread: "t".into(),
            text: "x".into(),
        };
        let json = serde_json::to_value(&action).unwrap();
        assert_eq!(json["action"], "revise");
        let back: UserAction = serde_json::from_value(json).unwrap();
        assert_eq!(back, action);
        let error = MessagesError::Refused {
            message: "m".into(),
            missing_sections: vec!["Ask".into()],
        };
        assert_eq!(
            serde_json::to_value(&error).unwrap()["missingSections"][0],
            "Ask"
        );
    }
}
