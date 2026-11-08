//! `MessagesSource` over `dcp-integrations --json` (spec 008, ADR-0010). Aurora runs no
//! Git: the provider reads and writes the inbox branches. Message bodies go to the
//! provider in a temporary file that is removed afterwards, and are never logged.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use aurora_core::domain::{
    ClosingRecord, Conversation, EngineReport, EngineSettings, Message, MessageHeader, MessageKind,
    MessageStatus, MessagesError, Section, StatusNote, TemplateSection, UserAction, message_body,
};
use aurora_core::ports::{InboxRead, ListedProject, MessagesSource, export_by_parts};
use aurora_core::usecases::messages::SETUP_COMMAND;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::dcp::elapsed_ms;
use crate::runner::{self, CommandSpec, RunError};

pub const SCHEMA_MAJOR: u32 = 1;
/// The exit code of `poll` when there is news.
pub const NEWS: i32 = 10;
/// Writes push to other repositories: they get at least this long.
pub const MIN_WRITE_TIMEOUT_MS: u64 = 30_000;

/// Runs `dcp-integrations` at the location of its latest report.
#[derive(Debug, Default, Clone)]
pub struct ProviderMessagesSource;

#[derive(Deserialize)]
struct RawHeader {
    id: String,
    kind: String,
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    reply_to: Option<String>,
    #[serde(default)]
    deferred: Option<String>,
    #[serde(default)]
    thread: String,
    #[serde(default)]
    cycle: Option<u32>,
    #[serde(default)]
    round: Option<u32>,
    #[serde(default)]
    needs_user: bool,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    summary: String,
}

impl From<RawHeader> for MessageHeader {
    fn from(raw: RawHeader) -> Self {
        MessageHeader {
            thread: if raw.thread.is_empty() {
                raw.id.clone()
            } else {
                raw.thread
            },
            id: raw.id,
            kind: MessageKind::from(raw.kind),
            from: raw.from,
            to: raw.to,
            date: raw.date,
            subject: raw.subject,
            status: match raw.status.as_str() {
                "done" => MessageStatus::Done,
                "seen" => MessageStatus::Seen,
                _ => MessageStatus::Open,
            },
            reply_to: raw.reply_to.unwrap_or_default(),
            deferred: raw.deferred.unwrap_or_default(),
            cycle: raw.cycle.unwrap_or(1).max(1),
            round: raw.round,
            needs_user: raw.needs_user,
            tool: raw.tool.unwrap_or_default(),
            summary: raw.summary,
        }
    }
}

#[derive(Deserialize)]
struct RawConversation {
    thread: String,
    subject: String,
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    opener: Option<String>,
    cycle: u32,
    #[serde(default)]
    round: Option<u32>,
    next_round: u32,
    #[serde(default)]
    messages: u32,
    last: RawHeader,
    waiting_for_user: bool,
    #[serde(default)]
    user_requests: Vec<String>,
}

#[derive(Deserialize)]
struct RawCheck {
    #[serde(default)]
    conversations: Vec<RawConversation>,
    /// The received messages not done yet.
    #[serde(default)]
    messages: Vec<RawPending>,
}

#[derive(Deserialize)]
struct RawPending {
    #[serde(default)]
    thread: String,
    #[serde(default)]
    status: String,
}

/// Received messages still open (not read), per thread.
fn unread_by_thread(pending: &[RawPending]) -> std::collections::HashMap<&str, u32> {
    let mut out = std::collections::HashMap::new();
    for m in pending.iter().filter(|m| m.status == "open") {
        *out.entry(m.thread.as_str()).or_insert(0) += 1;
    }
    out
}

#[derive(Deserialize)]
struct RawItem {
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Deserialize)]
struct RawList {
    project: String,
    items: Vec<RawItem>,
}

#[derive(Deserialize)]
struct RawSection {
    name: String,
    text: String,
}

#[derive(Deserialize)]
struct RawNote {
    status: String,
    date: String,
    #[serde(default)]
    note: String,
}

#[derive(Deserialize)]
struct RawMessage {
    message: RawHeader,
    sections: Vec<RawSection>,
    body: String,
    /// The body without the status notes (schema 1.2).
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    notes: Vec<RawNote>,
}

impl From<RawMessage> for Message {
    fn from(m: RawMessage) -> Self {
        let mut message = Message {
            header: m.message.into(),
            sections: m
                .sections
                .into_iter()
                .map(|s| Section {
                    name: s.name,
                    text: s.text,
                })
                .collect(),
            body: m.body,
            text: m.text.unwrap_or_default(),
            notes: m
                .notes
                .into_iter()
                .map(|n| StatusNote {
                    status: n.status,
                    date: n.date,
                    note: n.note,
                })
                .collect(),
        };
        // A provider before 1.2 leaves the notes in the body.
        message.split_notes();
        message
    }
}

#[derive(Deserialize)]
struct RawThread {
    messages: Vec<RawMessage>,
}

#[derive(Deserialize)]
struct RawRecord {
    thread: String,
    subject: String,
    projects: Vec<String>,
    rounds: u32,
    cycle: u32,
    outcome: String,
    conclusion: String,
    justification: String,
    #[serde(default)]
    positions: String,
    #[serde(default)]
    open_points: String,
    decision_needed: String,
    #[serde(default)]
    options: Vec<String>,
    #[serde(default)]
    recommended: Option<usize>,
    after_approval: String,
    #[serde(default)]
    user_requests: Vec<String>,
}

impl From<RawRecord> for ClosingRecord {
    fn from(r: RawRecord) -> Self {
        ClosingRecord {
            thread: r.thread,
            subject: r.subject,
            projects: r.projects,
            rounds: r.rounds,
            cycle: r.cycle,
            outcome: MessageKind::from(r.outcome),
            conclusion: r.conclusion,
            justification: r.justification,
            positions: r.positions,
            open_points: r.open_points,
            decision_needed: r.decision_needed,
            options: r.options,
            recommended: r.recommended,
            after_approval: r.after_approval,
            user_requests: r.user_requests,
        }
    }
}

#[derive(Deserialize)]
struct RawTemplate {
    sections: Vec<TemplateSection>,
}

#[derive(Deserialize)]
struct RawSent {
    sent: Vec<RawHeader>,
}

#[derive(Deserialize)]
struct RawStatus {
    #[serde(default)]
    changed: Option<RawHeader>,
}

#[derive(Deserialize)]
struct RawPoll {
    changed: bool,
    #[serde(default)]
    summary: String,
}

#[derive(Deserialize)]
struct RawError {
    code: String,
    message: String,
    #[serde(default)]
    missing_sections: Vec<String>,
    #[serde(default)]
    git_stderr: Option<String>,
    #[serde(default)]
    repository: Option<String>,
}

fn invalid(message: impl Into<String>) -> MessagesError {
    MessagesError::InvalidReply {
        message: message.into(),
    }
}

/// The provider's error document as a `MessagesError`.
fn from_error(raw: RawError) -> MessagesError {
    match raw.code.as_str() {
        "not_set_up" => MessagesError::NotSetUp {
            command: SETUP_COMMAND.to_owned(),
            repository: raw.repository.filter(|r| !r.is_empty()),
        },
        "refused" | "round_limit" | "invalid" => MessagesError::Refused {
            message: raw.message,
            missing_sections: raw.missing_sections,
        },
        "unknown_id" | "unknown_project" => MessagesError::NotFound {
            message: raw.message,
        },
        "unsupported_inbox" => MessagesError::Unsupported { found: raw.message },
        _ => MessagesError::Git {
            message: raw.message,
            stderr: raw.git_stderr.unwrap_or_default(),
        },
    }
}

/// The reply's document once its envelope is `dcp-integrations.<name>` at MAJOR 1; the
/// error document as its error.
pub fn parse<T: DeserializeOwned>(bytes: &[u8], name: &str) -> Result<T, MessagesError> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| invalid("the reply is not JSON"))?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let major = version
        .split('.')
        .next()
        .and_then(|m| m.parse::<u32>().ok());
    if schema.is_empty() || major.is_none() {
        return Err(invalid("the reply is not a versioned JSON envelope"));
    }
    if major != Some(SCHEMA_MAJOR) {
        return Err(MessagesError::Unsupported {
            found: version.to_owned(),
        });
    }
    if schema == "dcp-integrations.error" {
        let raw: RawError = serde_json::from_value(value)
            .map_err(|e| invalid(format!("the error document is invalid: {e}")))?;
        return Err(from_error(raw));
    }
    let expected = format!("dcp-integrations.{name}");
    if schema != expected {
        return Err(invalid(format!(
            "the reply has schema `{schema}`, expected `{expected}`"
        )));
    }
    serde_json::from_value(value).map_err(|e| invalid(format!("`{expected}`: {e}")))
}

/// A temporary file for a message body, removed when dropped.
#[derive(Debug)]
struct BodyFile(PathBuf);

impl BodyFile {
    fn new(body: &str) -> Result<BodyFile, MessagesError> {
        let path = std::env::temp_dir().join(format!("aurora-message-{}.md", uuid::Uuid::new_v4()));
        std::fs::write(&path, body)
            .map_err(|e| invalid(format!("cannot write the message to a temporary file: {e}")))?;
        Ok(BodyFile(path))
    }

    fn arg(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
}

impl Drop for BodyFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The arguments of the provider for `action`, after `--json --dir <root>`; the body file
/// when the action has one.
pub fn act_args(
    action: &UserAction,
) -> Result<(Vec<String>, Option<BodyFileHandle>), MessagesError> {
    let s = |v: &[&str]| v.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>();
    Ok(match action {
        UserAction::Decide {
            thread,
            choice,
            note,
        } => {
            let mut args = s(&["decide", thread, "--choice", choice]);
            if let Some(note) = note.as_deref().filter(|n| !n.trim().is_empty()) {
                args.extend(s(&["--note", note]));
            }
            (args, None)
        }
        UserAction::Revise { thread, text } => (s(&["decide", thread, "--revise", text]), None),
        UserAction::Guide { thread, text } => (s(&["guide", thread, "--text", text]), None),
        UserAction::MarkSeen { id } => (s(&["status", id, "seen"]), None),
        UserAction::Send { draft } => {
            let file = BodyFile::new(&message_body(&draft.sections))?;
            let mut args = s(&[
                "send",
                "--to",
                &draft.to,
                "--kind",
                draft.kind.as_str(),
                "--subject",
                &draft.subject,
                "--body-file",
                &file.arg(),
            ]);
            with_summary(&mut args, &draft.summary);
            (args, Some(BodyFileHandle(file)))
        }
        UserAction::Reply {
            id,
            kind,
            sections,
            summary,
        } => {
            let file = BodyFile::new(&message_body(sections))?;
            let mut args = s(&[
                "reply",
                id,
                "--kind",
                kind.as_str(),
                "--body-file",
                &file.arg(),
            ]);
            with_summary(&mut args, summary);
            (args, Some(BodyFileHandle(file)))
        }
    })
}

/// Adds `--summary` when the user wrote one (FEAT-039); a provider before 1.3 never gets it
/// from an empty summary.
fn with_summary(args: &mut Vec<String>, summary: &str) {
    let summary = summary.split_whitespace().collect::<Vec<_>>().join(" ");
    if !summary.is_empty() {
        args.push("--summary".to_owned());
        args.push(summary);
    }
}

/// Keeps the body file of an action until the provider has run.
#[derive(Debug)]
pub struct BodyFileHandle(BodyFile);

impl BodyFileHandle {
    pub fn path(&self) -> &Path {
        &self.0.0
    }
}

impl ProviderMessagesSource {
    fn run(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: Option<&Path>,
        args: Vec<String>,
        timeout_ms: u64,
        what: &str,
    ) -> Result<(Vec<u8>, i32), MessagesError> {
        let Some(program) = &engine.location else {
            return Err(MessagesError::ProviderMissing {
                hint: "dcp-integrations was not found".to_owned(),
            });
        };
        let mut all = vec!["--json".to_owned()];
        if let Some(root) = root {
            all.extend(["--dir".to_owned(), root.to_string_lossy().into_owned()]);
        }
        all.extend(args);
        let mut spec = CommandSpec::new(
            program,
            &[],
            Duration::from_millis(timeout_ms.max(settings.timeout_ms)),
        );
        spec.args = all;
        spec.working_dir = root.map(Path::to_path_buf);
        let started = Instant::now();
        let result = runner::run(&spec);
        let elapsed_ms = elapsed_ms(started);
        let root_text = root.map(|r| r.display().to_string()).unwrap_or_default();
        match result {
            Ok(output) => {
                let code = output.exit_code.unwrap_or(-1);
                tracing::info!(command = what, root = %root_text, elapsed_ms, code, "dcp-integrations");
                if output.stdout.is_empty() {
                    return Err(MessagesError::Git {
                        message: format!("dcp-integrations {what} failed with exit code {code}"),
                        stderr: output.stderr,
                    });
                }
                Ok((output.stdout, code))
            }
            Err(error) => {
                tracing::warn!(command = what, root = %root_text, elapsed_ms, %error, "dcp-integrations failed");
                Err(match error {
                    RunError::TimedOut { timeout, .. } => MessagesError::Timeout {
                        timeout_ms: u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX),
                    },
                    RunError::TooLarge { .. } => invalid("the reply is too large"),
                    other => MessagesError::ProviderMissing {
                        hint: other.to_string(),
                    },
                })
            }
        }
    }

    fn read<T: DeserializeOwned>(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: Option<&Path>,
        args: &[&str],
        name: &str,
    ) -> Result<T, MessagesError> {
        let mut args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
        // A provider with local copies answers from them; `sync` updates them (ADR-0011).
        if root.is_some() && caches(engine) {
            args.push("--cached".to_owned());
        }
        let (stdout, _) = self.run(engine, settings, root, args, 0, name)?;
        parse(&stdout, name)
    }
}

/// The provider keeps local copies of the inboxes (`cache` capability, ADR-0011).
pub fn caches(engine: &EngineReport) -> bool {
    engine.has_capability(CACHE_CAPABILITY, SCHEMA_MAJOR)
}

/// The capability of a provider that answers reads from its local copies.
pub const CACHE_CAPABILITY: &str = "cache";

#[derive(Deserialize)]
struct RawSync {
    #[serde(default)]
    changed: bool,
}

fn conversation(raw: RawConversation) -> Conversation {
    Conversation {
        thread: raw.thread,
        subject: raw.subject,
        projects: raw.projects,
        opener: raw.opener,
        cycle: raw.cycle.max(1),
        round: raw.round,
        next_round: raw.next_round.max(1),
        messages: raw.messages,
        last: raw.last.into(),
        waiting_for_user: raw.waiting_for_user,
        user_requests: raw.user_requests,
        unread: 0,
        items: Vec::new(),
        record: None,
    }
}

/// The capability of a provider with `export` (IMP-007).
pub const EXPORT_CAPABILITY: &str = "export";

#[derive(Deserialize)]
struct RawExportConversation {
    #[serde(flatten)]
    conversation: RawConversation,
    #[serde(default)]
    unread: u32,
    #[serde(default)]
    items: Vec<RawMessage>,
    #[serde(default)]
    record: Option<RawRecord>,
}

#[derive(Deserialize)]
struct RawExport {
    project: String,
    #[serde(default)]
    items: Vec<RawItem>,
    #[serde(default)]
    conversations: Vec<RawExportConversation>,
}

/// The registry's projects and the conversations as one reading.
fn inbox_read(list: RawList, conversations: Vec<Conversation>) -> InboxRead {
    InboxRead {
        listed: list
            .items
            .iter()
            .filter_map(|i| match (&i.slug, &i.url) {
                (Some(slug), Some(url)) => Some(ListedProject {
                    slug: slug.clone(),
                    url: url.clone(),
                }),
                _ => None,
            })
            .collect(),
        counterparts: list
            .items
            .into_iter()
            .filter_map(|i| i.slug)
            .filter(|s| *s != list.project)
            .collect(),
        slug: list.project,
        conversations,
    }
}

impl MessagesSource for ProviderMessagesSource {
    fn inbox(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<InboxRead, MessagesError> {
        let list: RawList = self.read(engine, settings, Some(root), &["projects"], "list")?;
        let check: RawCheck = self.read(engine, settings, Some(root), &["check"], "check")?;
        let unread = unread_by_thread(&check.messages);
        let counts: Vec<u32> = check
            .conversations
            .iter()
            .map(|c| unread.get(c.thread.as_str()).copied().unwrap_or(0))
            .collect();
        let conversations = check
            .conversations
            .into_iter()
            .zip(counts)
            .map(|(raw, unread)| Conversation {
                unread,
                ..conversation(raw)
            })
            .collect();
        Ok(inbox_read(list, conversations))
    }

    fn export(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<InboxRead, MessagesError> {
        if !engine.has_capability(EXPORT_CAPABILITY, SCHEMA_MAJOR) {
            return export_by_parts(self, engine, settings, root);
        }
        let raw: RawExport = self.read(engine, settings, Some(root), &["export"], "export")?;
        let list = RawList {
            project: raw.project,
            items: raw.items,
        };
        let conversations = raw
            .conversations
            .into_iter()
            .map(|c| Conversation {
                unread: c.unread,
                items: c.items.into_iter().map(Message::from).collect(),
                record: c.record.map(ClosingRecord::from),
                ..conversation(c.conversation)
            })
            .collect();
        Ok(inbox_read(list, conversations))
    }

    fn thread(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        thread: &str,
    ) -> Result<Vec<Message>, MessagesError> {
        let raw: RawThread =
            self.read(engine, settings, Some(root), &["thread", thread], "thread")?;
        Ok(raw.messages.into_iter().map(Message::from).collect())
    }

    fn record(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        thread: &str,
    ) -> Result<ClosingRecord, MessagesError> {
        let r: RawRecord =
            self.read(engine, settings, Some(root), &["record", thread], "record")?;
        Ok(r.into())
    }

    fn template(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        kind: &MessageKind,
    ) -> Result<Vec<TemplateSection>, MessagesError> {
        let raw: RawTemplate = self.read(
            engine,
            settings,
            None,
            &["template", kind.as_str()],
            "template",
        )?;
        Ok(raw.sections)
    }

    fn act(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        action: &UserAction,
    ) -> Result<Vec<MessageHeader>, MessagesError> {
        let (args, body) = act_args(action)?;
        let what = args.first().cloned().unwrap_or_default();
        let (stdout, _) = self.run(
            engine,
            settings,
            Some(root),
            args,
            MIN_WRITE_TIMEOUT_MS,
            &what,
        )?;
        drop(body);
        if matches!(action, UserAction::MarkSeen { .. }) {
            let status: RawStatus = parse(&stdout, "status")?;
            return Ok(status.changed.into_iter().map(Into::into).collect());
        }
        let sent: RawSent = parse(&stdout, "sent")?;
        Ok(sent.sent.into_iter().map(Into::into).collect())
    }

    fn poll(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<Option<String>, MessagesError> {
        let (stdout, code) = self.run(
            engine,
            settings,
            Some(root),
            vec!["poll".to_owned()],
            0,
            "poll",
        )?;
        let poll: RawPoll = parse(&stdout, "poll")?;
        Ok((poll.changed || code == NEWS).then_some(poll.summary))
    }

    fn sync(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<bool, MessagesError> {
        if !caches(engine) {
            return Ok(false);
        }
        let (stdout, _) = self.run(
            engine,
            settings,
            Some(root),
            vec!["sync".to_owned()],
            0,
            "sync",
        )?;
        let sync: RawSync = parse(&stdout, "sync")?;
        Ok(sync.changed)
    }
}

#[cfg(test)]
mod unread_tests {
    use super::*;

    #[test]
    fn only_open_messages_count_as_unread() {
        let pending = |thread: &str, status: &str| RawPending {
            thread: thread.into(),
            status: status.into(),
        };
        let list = [
            pending("a", "open"),
            pending("a", "open"),
            pending("a", "seen"),
            pending("b", "seen"),
        ];
        let unread = unread_by_thread(&list);
        assert_eq!(unread.get("a"), Some(&2));
        assert_eq!(unread.get("b"), None);
    }
}

#[cfg(test)]
mod summary_tests {
    use super::*;
    use aurora_core::domain::{Draft, Section};

    #[test]
    fn a_summary_is_passed_only_when_written() {
        let draft = |summary: &str| UserAction::Send {
            draft: Draft {
                to: "beta".into(),
                kind: MessageKind::Question,
                subject: "Format".into(),
                summary: summary.into(),
                sections: vec![Section::new("Ask", "Adopt 1.0?")],
            },
        };
        let (args, _file) = act_args(&draft("  ")).expect("args");
        assert!(!args.iter().any(|a| a == "--summary"));
        let (args, _file) = act_args(&draft("I need\nyour format.")).expect("args");
        let at = args.iter().position(|a| a == "--summary").expect("summary");
        assert_eq!(args[at + 1], "I need your format.");
        let reply = UserAction::Reply {
            id: "m1".into(),
            kind: MessageKind::Answer,
            sections: vec![Section::new("Answer", "Yes.")],
            summary: "Yes, from v1.1.".into(),
        };
        let (args, _file) = act_args(&reply).expect("args");
        assert!(
            args.windows(2)
                .any(|w| w == ["--summary", "Yes, from v1.1."])
        );
    }
}
