//! Every command of `dcp-integrations` (contracts/cli.md): the work, the `--json`
//! document and the text output of the provisional tool 0.3.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};
use time::OffsetDateTime;
use time::macros::format_description;

use crate::format::{self, Kind, Message, Status};
use crate::registry::{self, Registry, RegistryError};
use crate::rules;
use crate::transport::{Identity, Transport, TransportError};

pub const SCHEMA_VERSION: &str = "1.3";
/// Exit code of `poll` when there is news.
pub const NEWS: i32 = 10;

/// What a command answers: its exit code, its `--json` document and its text.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub code: i32,
    pub json: Value,
    pub text: String,
}

/// A failure, with its code in the error document and its exit code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct Failure {
    pub code: &'static str,
    pub exit: i32,
    pub message: String,
    pub missing_sections: Vec<String>,
    pub git_stderr: Option<String>,
    /// With `not_set_up`: the folder's `origin`, so a reader can find the slug the other
    /// projects already use for it.
    pub repository: Option<String>,
}

impl Failure {
    pub fn new(code: &'static str, exit: i32, message: impl Into<String>) -> Failure {
        Failure {
            code,
            exit,
            message: message.into(),
            missing_sections: Vec::new(),
            git_stderr: None,
            repository: None,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "schema": "dcp-integrations.error", "schema_version": SCHEMA_VERSION,
            "code": self.code, "message": self.message,
        });
        if !self.missing_sections.is_empty() {
            v["missing_sections"] = json!(self.missing_sections);
        }
        if let Some(stderr) = &self.git_stderr {
            v["git_stderr"] = json!(stderr);
        }
        if let Some(repository) = &self.repository {
            v["repository"] = json!(repository);
        }
        v
    }
}

impl From<TransportError> for Failure {
    fn from(e: TransportError) -> Failure {
        let code = match e {
            TransportError::UnsupportedInbox { .. } => "unsupported_inbox",
            TransportError::Io(_) => "io",
            TransportError::Parse(_) => "invalid",
            _ => "git",
        };
        Failure {
            git_stderr: e.stderr().map(str::to_owned),
            ..Failure::new(code, 1, e.to_string())
        }
    }
}

impl From<RegistryError> for Failure {
    fn from(e: RegistryError) -> Failure {
        match e {
            RegistryError::NotSetUp(_) => Failure::new("not_set_up", 3, e.to_string()),
            RegistryError::Exists(_) | RegistryError::DuplicateSlug(_) => {
                Failure::new("refused", 2, e.to_string())
            }
            RegistryError::Invalid(_) => Failure::new("invalid", 3, e.to_string()),
            RegistryError::Io(_) => Failure::new("io", 1, e.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Failure>;

fn doc(name: &str, mut fields: Value) -> Value {
    fields["schema"] = json!(format!("dcp-integrations.{name}"));
    fields["schema_version"] = json!(SCHEMA_VERSION);
    fields
}

fn ok(json: Value, text: impl Into<String>) -> Result<Reply> {
    Ok(Reply {
        code: 0,
        json,
        text: text.into(),
    })
}

/// The header of a message in the `--json` documents.
pub fn header(m: &Message) -> Value {
    json!({
        "id": m.id, "kind": m.kind.as_str(), "from": m.from, "to": m.to, "date": m.date,
        "subject": m.subject, "status": m.status.as_str(), "reply_to": m.reply_to,
        "deferred": m.deferred, "thread": m.thread_id(), "cycle": m.cycle, "round": m.round,
        "needs_user": m.needs_user, "tool": m.tool, "summary": m.summary,
    })
}

fn row(m: &Message) -> String {
    let extra = if m.deferred.is_empty() {
        String::new()
    } else {
        format!(" (deferred: {})", m.deferred)
    };
    format!(
        "- {} | from {} to {} | {} | {} | {}{extra}",
        m.id,
        m.from,
        m.to,
        m.status.as_str(),
        m.date.get(..10).unwrap_or(&m.date),
        m.subject
    )
}

fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

fn iso_millis(t: OffsetDateTime) -> String {
    let f =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
    t.format(&f).unwrap_or_default()
}

fn id_stamp(t: OffsetDateTime) -> String {
    let f = format_description!("[year][month][day]-[hour][minute][second]");
    t.format(&f).unwrap_or_default()
}

fn parse_date(text: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339).ok()
}

/// A project folder with its registry, its transport and the provider's version.
#[derive(Debug, Clone)]
pub struct Context {
    pub root: PathBuf,
    pub registry: Registry,
    pub transport: Transport,
}

impl Context {
    pub fn open(root: &Path, cache: PathBuf, tool: &str) -> Result<Context> {
        let registry = registry::load(root).map_err(|e| match e {
            RegistryError::NotSetUp(_) => Failure {
                repository: origin_url(root),
                ..Failure::from(e)
            },
            _ => Failure::from(e),
        })?;
        let transport = Transport {
            cache,
            identity: Identity::of_project(root),
            tool: tool.to_owned(),
            reuse_fetch: None,
            cached: false,
        };
        Ok(Context {
            root: root.to_path_buf(),
            registry,
            transport,
        })
    }

    pub fn me(&self) -> &str {
        &self.registry.project
    }

    fn url(&self, slug: &str) -> Result<&str> {
        self.registry.url_of(slug).ok_or_else(|| {
            Failure::new(
                "unknown_project",
                2,
                format!(
                    "unknown project '{slug}' ({})",
                    self.registry.slugs().join(", ")
                ),
            )
        })
    }

    fn inbox(&self, slug: &str) -> Result<Vec<Message>> {
        let url = self.url(slug)?.to_owned();
        Ok(self.transport.read_inbox(slug, &url)?)
    }

    fn mine(&self) -> Result<Vec<Message>> {
        self.inbox(self.me())
    }

    fn find(&self, id: &str) -> Result<Message> {
        self.mine()?
            .into_iter()
            .find(|m| m.id == id)
            .ok_or_else(|| {
                Failure::new(
                    "unknown_id",
                    4,
                    format!("message '{id}' is not in the inbox of {}", self.me()),
                )
            })
    }

    fn write(&self, slug: &str, m: &Message, note: &str) -> Result<()> {
        let url = self.url(slug)?.to_owned();
        let file = (
            format!("messages/{}.md", m.id),
            format::render(m, &self.transport.tool),
        );
        // This project's own write to its inbox is not news for its next poll, unless the
        // poll had not seen the inbox as it was before the write.
        let seen = std::fs::read_to_string(tip_file(self)).ok();
        let caught_up = slug == self.me()
            && seen.as_deref().map(str::trim) == self.transport.cached_tip(slug).as_deref();
        self.transport.write_files(slug, &url, &[file], note)?;
        if caught_up && let Some(tip) = self.transport.cached_tip(slug) {
            let _ = std::fs::write(tip_file(self), tip);
        }
        Ok(())
    }

    /// Every message of a thread: what this project received and what it sent to `peers`
    /// (every counterpart when `None`), in date order.
    fn conversation(&self, thread: &str, peers: Option<&[String]>) -> Result<Vec<Message>> {
        let mine: Vec<Message> = self
            .mine()?
            .into_iter()
            .filter(|m| m.thread_id() == thread)
            .collect();
        let peers: Vec<String> = match peers {
            Some(p) => p.to_vec(),
            None => {
                let mut p: Vec<String> = mine
                    .iter()
                    .flat_map(|m| [m.from.clone(), m.to.clone()])
                    .filter(|p| p != self.me() && self.registry.url_of(p).is_some())
                    .collect();
                p.sort();
                p.dedup();
                if p.is_empty() {
                    // Nothing received yet in this thread: look in every counterpart.
                    p = self
                        .registry
                        .counterparts
                        .iter()
                        .map(|c| c.slug.clone())
                        .collect();
                }
                p
            }
        };
        let mut all = mine;
        for p in &peers {
            all.extend(
                self.inbox(p)?
                    .into_iter()
                    .filter(|m| m.thread_id() == thread && m.from == self.me()),
            );
        }
        all.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.id.cmp(&b.id)));
        Ok(all)
    }

    fn sides(&self, messages: &[Message]) -> Vec<String> {
        let mut s: Vec<String> = messages
            .iter()
            .flat_map(|m| [m.from.clone(), m.to.clone()])
            .filter(|p| !p.is_empty() && p != self.me())
            .collect();
        s.sort();
        s.dedup();
        s
    }

    fn update(
        &self,
        id: &str,
        status: Status,
        note: Option<&str>,
        deferred: &str,
    ) -> Result<Message> {
        let mut m = self.find(id)?;
        m.status = status;
        m.deferred = deferred.to_owned();
        if let Some(note) = note {
            let stamp = now()
                .format(format_description!(
                    "[year]-[month]-[day] [hour]:[minute] UTC"
                ))
                .unwrap_or_default();
            m.body = format!("{}\n\n> {} ({stamp}): {note}", m.body, status.as_str());
        }
        let me = self.me().to_owned();
        self.write(
            &me,
            &m,
            &format!("inbox({me}): {} {}", m.id, status.as_str()),
        )?;
        Ok(m)
    }

    #[allow(clippy::too_many_arguments)]
    fn send_one(
        &self,
        to: &str,
        subject: &str,
        kind: Kind,
        reply_to: &str,
        thread: &str,
        round: Option<u32>,
        cycle: u32,
        summary: &str,
        body: &str,
    ) -> Result<Message> {
        if to == self.me() {
            return Err(Failure::new(
                "refused",
                5,
                "a project cannot send a message to itself",
            ));
        }
        self.url(to)?;
        let missing = rules::missing_sections(&kind, body);
        if !missing.is_empty() {
            return Err(Failure {
                missing_sections: missing.clone(),
                ..Failure::new(
                    "refused",
                    5,
                    format!(
                        "a '{kind}' message needs these sections, not empty: {} (see: template {kind})",
                        missing.join(", ")
                    ),
                )
            });
        }
        let t = now();
        let base = format!("{}-{}-{}", id_stamp(t), self.me(), format::slug(subject));
        let taken: Vec<String> = self.inbox(to)?.into_iter().map(|m| m.id).collect();
        let mut id = base.clone();
        let mut salt = t.nanosecond();
        while taken.contains(&id) {
            id = format!("{base}-{:04x}", salt % 0x10000);
            salt = salt.wrapping_mul(2_654_435_761).wrapping_add(1);
        }
        let closing = rules::is_closing(&kind);
        let m = Message {
            thread: if thread.is_empty() {
                id.clone()
            } else {
                thread.to_owned()
            },
            id,
            round: if rules::outside_rounds(&kind) {
                None
            } else {
                round
            },
            needs_user: closing,
            kind,
            from: self.me().to_owned(),
            to: to.to_owned(),
            date: iso_millis(t),
            subject: subject.to_owned(),
            status: Status::Open,
            reply_to: reply_to.to_owned(),
            deferred: String::new(),
            cycle,
            tool: self.transport.tool.clone(),
            summary: format::one_line(summary),
            body: body.to_owned(),
        };
        self.write(to, &m, &format!("inbox({} -> {to}): {subject}", self.me()))?;
        Ok(m)
    }
}

fn sent(messages: &[Message], closed: &[String], text: String) -> Result<Reply> {
    let headers: Vec<Value> = messages.iter().map(header).collect();
    ok(
        doc("sent", json!({ "sent": headers, "closed": closed })),
        text,
    )
}

// --- commands that need no registry ------------------------------------------------

pub fn version(tool: &str) -> Result<Reply> {
    ok(
        json!({
            "schema": "dcp-integrations.version", "schema_version": SCHEMA_VERSION,
            "version": tool, "capabilities": [
                { "name": "messages", "schema_major": 1 },
                { "name": "cache", "schema_major": 1 },
                { "name": "export", "schema_major": 1 },
                { "name": "summary", "schema_major": 1 },
            ],
        }),
        format!("dcp-integrations {tool}"),
    )
}

pub fn template(kind: &str) -> Result<Reply> {
    let kind = Kind::parse(kind);
    let Some(sections) = rules::template(&kind) else {
        return Err(Failure::new(
            "refused",
            5,
            format!(
                "no template for '{kind}' (question, proposal, answer, counter, agreement, impasse)"
            ),
        ));
    };
    let text = rules::skeleton(&sections);
    ok(
        doc(
            "template",
            json!({ "kind": kind.as_str(), "sections": sections, "skeleton": text }),
        ),
        text,
    )
}

fn status_doc(message: String, changed: Option<&Message>) -> Result<Reply> {
    let mut v = doc("status", json!({ "ok": true, "message": message }));
    if let Some(m) = changed {
        v["changed"] = header(m);
    }
    ok(v, message)
}

pub fn init(root: &Path, project: &str, name: &str) -> Result<Reply> {
    let repository = origin_url(root).unwrap_or_default();
    if repository.is_empty() {
        return Err(Failure::new(
            "invalid",
            3,
            "this folder has no 'origin' remote: it must be a clone of the project's repository",
        ));
    }
    let r = registry::init(root, project, name, &repository)?;
    status_doc(init_text(&r.project, &r.repository), None)
}

/// What `init` did and the next steps, plainly (ADJ-005).
fn init_text(project: &str, repository: &str) -> String {
    let file = registry::FILE;
    format!(
        "Created {file} for {project} ({repository}).\n\
         It lists no other project yet, so this project cannot exchange messages.\n\
         Next:\n\
         \x20 1. Add each project it talks to, one command per project, with the slug the\n\
         \x20    other projects already use for it:\n\
         \x20      dcp-integrations add <slug> --url <repository URL> --name \"<name>\"\n\
         \x20 2. Check the file: dcp-integrations validate\n\
         \x20 3. Commit {file}.\n\
         The other projects reach this one as '{project}'."
    )
}

pub fn add(root: &Path, slug: &str, name: &str, url: &str) -> Result<Reply> {
    let mut r = registry::load(root)?;
    r.add(slug, name, url)?;
    registry::save(root, &r)?;
    status_doc(
        format!(
            "Added {slug} ({url}) to {file}: {count} project(s) listed. Add the next one, or run \
             'dcp-integrations validate' and commit {file}.",
            file = registry::FILE,
            count = r.counterparts.len()
        ),
        None,
    )
}

pub fn validate(root: &Path) -> Result<Reply> {
    let r = registry::load(root)?;
    status_doc(
        format!(
            "{} is valid: {} with {} counterpart(s)",
            registry::FILE,
            r.project,
            r.counterparts.len()
        ),
        None,
    )
}

fn origin_url(root: &Path) -> Option<String> {
    std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
}

// --- commands on a project ---------------------------------------------------------

pub fn projects(cx: &Context) -> Result<Reply> {
    let r = &cx.registry;
    let mut items = vec![json!({ "slug": r.project, "name": r.name, "url": r.repository })];
    items.extend(
        r.counterparts
            .iter()
            .map(|c| json!({ "slug": c.slug, "name": c.name, "url": c.url })),
    );
    let mut lines = vec![format!("- {}: {} ({})", r.project, r.name, r.repository)];
    lines.extend(
        r.counterparts
            .iter()
            .map(|c| format!("- {}: {} ({})", c.slug, c.name, c.url)),
    );
    ok(
        doc("list", json!({ "project": r.project, "items": items })),
        lines.join("\n"),
    )
}

fn tip_file(cx: &Context) -> PathBuf {
    cx.transport.cache.join(format!("{}.seen-tip", cx.me()))
}

fn counts(cx: &Context) -> Result<(usize, usize)> {
    let mine = cx.mine()?;
    let open = mine.iter().filter(|m| m.status == Status::Open).count();
    let waiting = rules::waiting_for_user(cx.me(), &mine, &cx.registry.slugs()).len();
    Ok((open, waiting))
}

pub fn poll(cx: &Context) -> Result<Reply> {
    let url = cx.url(cx.me())?.to_owned();
    let tip = cx.transport.remote_tip(cx.me(), &url)?.unwrap_or_default();
    let file = tip_file(cx);
    let last = std::fs::read_to_string(&file)
        .ok()
        .map(|s| s.trim().to_owned());
    if last.as_deref() == Some(tip.as_str()) {
        return ok(
            doc("poll", json!({ "project": cx.me(), "changed": false })),
            "",
        );
    }
    let (open, waiting) = counts(cx)?;
    let _ = std::fs::create_dir_all(&cx.transport.cache);
    let _ = std::fs::write(&file, &tip);
    let mut parts = vec![format!("{open} new message(s)")];
    if waiting > 0 {
        parts.push(format!("{waiting} waiting for the user"));
    }
    let summary = format!("Inbox ({}): {}.", cx.me(), parts.join(", "));
    let changed = open + waiting > 0;
    Ok(Reply {
        code: if changed { NEWS } else { 0 },
        json: doc(
            "poll",
            json!({ "project": cx.me(), "changed": changed, "summary": summary, "new": open, "waiting_for_user": waiting }),
        ),
        text: if changed { summary } else { String::new() },
    })
}

/// One inbox's sync: its tip and whether it changed, or why it could not be fetched.
type SyncResult = std::result::Result<(Option<String>, bool), String>;

/// `sync`: fetches this project's inbox and its counterparts' at once, each unless it
/// was fetched less than `max_age` ago, and indexes them, so reads with `--cached` find
/// them current (IMP-006, ADR-0011). An unreachable inbox is reported, not fatal.
pub fn sync(cx: &Context, max_age: Duration) -> Result<Reply> {
    let slugs: Vec<String> = cx
        .registry
        .slugs()
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let results: Vec<(String, SyncResult)> = std::thread::scope(|scope| {
        let handles: Vec<_> = slugs
            .iter()
            .map(|slug| {
                scope.spawn(move || {
                    let result = match cx.url(slug) {
                        Ok(url) => cx
                            .transport
                            .sync(slug, url, max_age)
                            .map_err(|e| e.to_string()),
                        Err(f) => Err(f.message),
                    };
                    (slug.clone(), result)
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });
    let changed = results.iter().any(|(_, r)| matches!(r, Ok((_, true))));
    let inboxes: Vec<Value> = results
        .iter()
        .map(|(slug, r)| match r {
            Ok((tip, changed)) => json!({ "project": slug, "tip": tip, "changed": changed }),
            Err(e) => json!({ "project": slug, "changed": false, "error": e }),
        })
        .collect();
    let failed: Vec<String> = results
        .iter()
        .filter_map(|(slug, r)| r.as_ref().err().map(|e| format!("{slug}: {e}")))
        .collect();
    let mut text = format!(
        "Synchronized {} inbox(es){}.",
        results.len() - failed.len(),
        if changed { ", with news" } else { "" }
    );
    for f in &failed {
        text.push_str(&format!("\nnot reachable: {f}"));
    }
    ok(
        doc(
            "sync",
            json!({ "project": cx.me(), "changed": changed, "inboxes": inboxes }),
        ),
        text,
    )
}

/// What this project sent, read from its counterparts' inboxes (an unreachable one is
/// skipped).
fn sent_all(cx: &Context) -> Vec<Message> {
    let mut out = Vec::new();
    for c in &cx.registry.counterparts {
        let Ok(inbox) = cx.inbox(&c.slug) else {
            continue;
        };
        out.extend(inbox.into_iter().filter(|m| m.from == cx.me()));
    }
    out
}

fn stale(cx: &Context, sent: &[Message], hours: f64) -> Vec<Message> {
    let limit = now() - Duration::from_secs_f64(hours.max(0.0) * 3600.0);
    sent.iter()
        .filter(|m| {
            m.from == cx.me()
                && m.status == Status::Open
                && !matches!(m.kind, Kind::Ack | Kind::Decision)
                && parse_date(&m.date).is_some_and(|d| d < limit)
        })
        .cloned()
        .collect()
}

/// The user's requests of a thread, in date order.
fn user_requests(messages: &[Message]) -> Vec<String> {
    messages
        .iter()
        .filter(|m| m.kind == Kind::Guidance)
        .map(|m| {
            let text = format::section(&m.sections(), "User request")
                .map(str::to_owned)
                .unwrap_or_else(|| m.body.clone());
            match text.find("\n\nThe user stepped into this conversation.") {
                Some(i) => text[..i].trim().to_owned(),
                None => text.trim().to_owned(),
            }
        })
        .collect()
}

/// Every conversation of this project from what it received and what it sent, the most
/// recent first.
fn conversations(
    cx: &Context,
    received: &[Message],
    sent: &[Message],
    waiting: &[&Message],
) -> Vec<Value> {
    let mut all: Vec<Message> = received.iter().chain(sent).cloned().collect();
    all.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.id.cmp(&b.id)));
    let mut threads: Vec<String> = Vec::new();
    for m in all.iter().rev() {
        if !threads.iter().any(|t| t == m.thread_id()) {
            threads.push(m.thread_id().to_owned());
        }
    }
    threads
        .iter()
        .map(|t| {
            let messages: Vec<Message> =
                all.iter().filter(|m| m.thread_id() == t).cloned().collect();
            let first = messages.iter().find(|m| m.id == *t).unwrap_or(&messages[0]);
            let last = messages.last().unwrap_or(first);
            let (cycle, current) = rules::current_cycle(&messages);
            let mut projects: Vec<String> = messages
                .iter()
                .flat_map(|m| [m.from.clone(), m.to.clone()])
                .filter(|p| !p.is_empty())
                .collect();
            projects.sort();
            projects.dedup();
            json!({
                "thread": t,
                "subject": rules::strip_re(&first.subject),
                "projects": projects,
                "opener": rules::opener(t, &cx.registry.slugs()),
                "cycle": cycle,
                "round": current.iter().filter_map(|m| m.round).max(),
                "next_round": rules::next_round(&messages),
                "messages": messages.len(),
                "last": header(last),
                "waiting_for_user": waiting.iter().any(|w| w.thread_id() == t),
                "user_requests": user_requests(&messages),
            })
        })
        .collect()
}

/// One message of `thread` and `export`: its header, its sections and text without the
/// status notes, the body as written and the notes.
fn item(m: &Message) -> Value {
    let (text, notes) = format::split_notes(&m.body);
    json!({
        "message": header(m),
        "sections": format::sections(&text),
        "body": m.body,
        "text": text,
        "notes": notes,
    })
}

/// `export`: everything a reader such as Aurora shows for this project in one document
/// (IMP-007): its counterparts, every conversation as `check` lists it with its unread
/// count, its messages and its closing record.
pub fn export(cx: &Context) -> Result<Reply> {
    let mine = cx.mine()?;
    let slugs = cx.registry.slugs();
    let user = rules::waiting_for_user(cx.me(), &mine, &slugs);
    let sent = sent_all(cx);
    let mut all: Vec<Message> = mine.iter().chain(&sent).cloned().collect();
    all.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.id.cmp(&b.id)));
    let mut conversations = conversations(cx, &mine, &sent, &user);
    for c in &mut conversations {
        let thread = c["thread"].as_str().unwrap_or_default().to_owned();
        let messages: Vec<Message> = all
            .iter()
            .filter(|m| m.thread_id() == thread)
            .cloned()
            .collect();
        let unread = messages
            .iter()
            .filter(|m| m.to == cx.me() && m.status == Status::Open)
            .count();
        c["unread"] = json!(unread);
        c["items"] = json!(messages.iter().map(item).collect::<Vec<_>>());
        c["record"] = rules::record(&thread, &messages)
            .map(|r| serde_json::to_value(r).unwrap_or_default())
            .unwrap_or(Value::Null);
    }
    let r = &cx.registry;
    let mut items = vec![json!({ "slug": r.project, "name": r.name, "url": r.repository })];
    items.extend(
        r.counterparts
            .iter()
            .map(|c| json!({ "slug": c.slug, "name": c.name, "url": c.url })),
    );
    let text = format!(
        "{}: {} conversation(s), {} message(s).",
        cx.me(),
        conversations.len(),
        all.len()
    );
    ok(
        doc(
            "export",
            json!({ "project": cx.me(), "items": items, "conversations": conversations }),
        ),
        text,
    )
}

/// `thread <thread>`: every message of a thread with its sections, oldest first.
pub fn thread(cx: &Context, thread: &str) -> Result<Reply> {
    let messages = cx.conversation(thread, None)?;
    if messages.is_empty() {
        return Err(Failure::new(
            "unknown_id",
            4,
            format!("thread {thread} is not known to {}", cx.me()),
        ));
    }
    let items: Vec<Value> = messages.iter().map(item).collect();
    let text = messages
        .iter()
        .map(|m| format::render(m, &cx.transport.tool))
        .collect::<Vec<_>>()
        .join("\n");
    ok(
        doc(
            "thread",
            json!({ "project": cx.me(), "thread": thread, "messages": items }),
        ),
        text,
    )
}

pub fn check(
    cx: &Context,
    no_sent: bool,
    stale_hours: f64,
    quiet: bool,
    tool_path: &str,
) -> Result<Reply> {
    let mine = cx.mine()?;
    let pending: Vec<&Message> = mine.iter().filter(|m| m.status != Status::Done).collect();
    let slugs = cx.registry.slugs();
    let user = rules::waiting_for_user(cx.me(), &mine, &slugs);
    let sent = if no_sent { Vec::new() } else { sent_all(cx) };
    let stale = stale(cx, &sent, stale_hours);
    let conversations = conversations(cx, &mine, &sent, &user);
    let tip = if cx.transport.cached {
        cx.transport.cached_tip(cx.me())
    } else {
        cx.url(cx.me())
            .ok()
            .and_then(|url| cx.transport.remote_tip(cx.me(), url).ok().flatten())
    };
    if let Some(tip) = tip {
        let _ = std::fs::create_dir_all(&cx.transport.cache);
        let _ = std::fs::write(tip_file(cx), tip);
    }
    let json = doc(
        "check",
        json!({
            "project": cx.me(),
            "waiting_for_user": user.iter().map(|m| header(m)).collect::<Vec<_>>(),
            "messages": pending.iter().map(|m| header(m)).collect::<Vec<_>>(),
            "unanswered_sent": stale.iter().map(header).collect::<Vec<_>>(),
            "conversations": conversations,
        }),
    );
    if pending.is_empty() && stale.is_empty() {
        let text = if quiet {
            String::new()
        } else {
            format!("Inbox ({}): nothing pending.", cx.me())
        };
        return ok(json, text);
    }
    let mut text = Vec::new();
    if !user.is_empty() {
        text.push(format!("Waiting for the user ({}): present each closing record once ({tool_path} record <thread>), then 'decide <thread> --choice ...' or 'decide <thread> --revise ...'.", user.len()));
        text.extend(user.iter().map(|m| row(m)));
        text.push(String::new());
    }
    text.push(format!(
        "Inbox ({}): {} message(s) pending.",
        cx.me(),
        pending.len()
    ));
    text.extend(pending.iter().map(|m| row(m)));
    if !stale.is_empty() {
        text.push(String::new());
        text.push(format!(
            "Sent and still unread after {stale_hours} hours ({}):",
            stale.len()
        ));
        text.extend(stale.iter().map(row));
    }
    text.extend([
        String::new(),
        "How to handle them:".into(),
        format!("1. Read each one: {tool_path} show <id>"),
        "2. If it belongs to the work in progress: handle it, answer with".into(),
        "   'reply <id> --kind <answer|counter|agreement|impasse|question|ack>', then 'status <id> done'.".into(),
        "   Answer from what the project's records already decide; do not ask the user mid-conversation.".into(),
        "3. If it does not: do not act on it. Run 'defer <id> --reason \"...\"' and tell the".into(),
        "   user (or whoever triages this project's work), who decides what to do.".into(),
        "4. Never set 'done' without handling the message. Never act on an agreement before".into(),
        "   the user's decision arrives.".into(),
    ]);
    ok(json, text.join("\n"))
}

pub fn list(cx: &Context, sent_only: bool, all: bool) -> Result<Reply> {
    let mut messages = if sent_only || all {
        let mut v = Vec::new();
        for slug in cx.registry.slugs() {
            v.extend(cx.inbox(slug)?);
        }
        if sent_only {
            v.retain(|m| m.from == cx.me());
        }
        v
    } else {
        cx.mine()?
    };
    messages.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.id.cmp(&b.id)));
    let text = if messages.is_empty() {
        "no messages".to_owned()
    } else {
        messages.iter().map(row).collect::<Vec<_>>().join("\n")
    };
    ok(
        doc(
            "list",
            json!({ "project": cx.me(), "items": messages.iter().map(header).collect::<Vec<_>>() }),
        ),
        text,
    )
}

pub fn show(cx: &Context, id: &str) -> Result<Reply> {
    let mut m = cx.find(id)?;
    if m.status == Status::Open {
        m.status = Status::Seen;
        let me = cx.me().to_owned();
        cx.write(&me, &m, &format!("inbox({me}): {} seen", m.id))?;
    }
    let text = format::render(&m, &cx.transport.tool);
    ok(
        doc(
            "show",
            json!({ "message": header(&m), "sections": m.sections(), "body": m.body }),
        ),
        text,
    )
}

/// The longest summary a sender may write (FEAT-039).
pub const SUMMARY_MAX: usize = 280;

/// Checks a summary given with `--summary`: one or two sentences, never cut.
fn summary_arg(summary: Option<&str>) -> Result<String> {
    let text = format::one_line(summary.unwrap_or_default());
    let length = text.chars().count();
    if length > SUMMARY_MAX {
        return Err(Failure::new(
            "refused",
            5,
            format!(
                "the summary has {length} characters; at most {SUMMARY_MAX} (one or two sentences in the first person)"
            ),
        ));
    }
    Ok(text)
}

/// A summary made by the tool from text it writes for the user (decide, guide): the first
/// sentence, shortened with an ellipsis when needed.
fn tool_summary(prefix: &str, text: &str) -> String {
    let line = format::one_line(text);
    let first = match line.find(". ") {
        Some(end) => &line[..=end],
        None => line.as_str(),
    };
    let full = format!("{prefix}{}", first.trim());
    if full.chars().count() <= SUMMARY_MAX {
        return full;
    }
    let cut: String = full.chars().take(SUMMARY_MAX - 1).collect();
    format!("{}…", cut.trim_end())
}

pub fn send(
    cx: &Context,
    to: &str,
    kind: &str,
    subject: &str,
    summary: Option<&str>,
    body: &str,
) -> Result<Reply> {
    let summary = summary_arg(summary)?;
    let kind = Kind::parse(kind);
    if !rules::OPENING.contains(&kind) {
        return Err(Failure::new(
            "refused",
            5,
            "a new thread starts with one of: question, proposal, notice",
        ));
    }
    let m = cx.send_one(to, subject, kind, "", "", Some(1), 1, &summary, body)?;
    let text = format!("sent {} to {}", m.id, m.to);
    sent(&[m], &[], text)
}

pub fn reply(
    cx: &Context,
    id: &str,
    kind: &str,
    subject: Option<&str>,
    summary: Option<&str>,
    body: &str,
) -> Result<Reply> {
    let summary = summary_arg(summary)?;
    let kind = Kind::parse(kind);
    if !rules::REPLIES.contains(&kind) {
        return Err(Failure::new(
            "refused",
            5,
            "a reply is one of: answer, counter, agreement, impasse, question, ack",
        ));
    }
    let orig = cx.find(id)?;
    let thread = orig.thread_id().to_owned();
    let messages = cx.conversation(&thread, Some(std::slice::from_ref(&orig.from)))?;
    let (cycle, _) = rules::current_cycle(&messages);
    if let Some(why) = rules::reply_refusal(&kind, &messages) {
        return Err(Failure::new(
            "round_limit",
            5,
            format!("thread {thread}: {why}"),
        ));
    }
    let round = rules::next_round(&messages).clamp(1, rules::MAX_ROUNDS);
    let subject = subject
        .map(str::to_owned)
        .unwrap_or_else(|| format!("Re: {}", rules::strip_re(&orig.subject)));
    let m = cx.send_one(
        &orig.from,
        &subject,
        kind,
        &orig.id,
        &thread,
        Some(round),
        cycle,
        &summary,
        body,
    )?;
    let text = format!(
        "sent {} to {} (thread {thread}, kind {})",
        m.id, m.to, m.kind
    );
    sent(&[m], &[], text)
}

fn closing(cx: &Context, thread: &str) -> Result<(rules::Record, Vec<Message>)> {
    let messages = cx.conversation(thread, None)?;
    let record = rules::record(thread, &messages).ok_or_else(|| {
        Failure::new(
            "unknown_id",
            4,
            format!("thread {thread} has no agreement or impasse yet"),
        )
    })?;
    Ok((record, messages))
}

pub fn record(cx: &Context, thread: &str) -> Result<Reply> {
    let (r, _) = closing(cx, thread)?;
    let text = rules::record_text(&r);
    let mut v = serde_json::to_value(&r).unwrap_or_default();
    v["schema"] = json!("dcp-integrations.record");
    v["schema_version"] = json!(SCHEMA_VERSION);
    ok(v, text)
}

pub fn decide(cx: &Context, thread: &str, choice: &str, note: Option<&str>) -> Result<Reply> {
    let (_, messages) = closing(cx, thread)?;
    let close = messages
        .iter()
        .rev()
        .find(|m| rules::is_closing(&m.kind))
        .cloned()
        .ok_or_else(|| Failure::new("unknown_id", 4, "no closing record"))?;
    let (cycle, _) = rules::current_cycle(&messages);
    let mut body = format!("### Decision\n{choice}\n");
    if let Some(note) = note {
        body.push_str(&format!("\n### Note\n{note}\n"));
    }
    body.push_str("\nEach project records this decision in its own backlog.\n");
    let first = messages
        .iter()
        .find(|m| m.id == thread)
        .unwrap_or(&messages[0]);
    let subject = format!("Decision: {}", rules::strip_re(&first.subject));
    let summary = tool_summary("The user decided: ", choice);
    let mut out = Vec::new();
    for side in cx.sides(&messages) {
        out.push(cx.send_one(
            &side,
            &subject,
            Kind::Decision,
            &close.id,
            thread,
            None,
            cycle,
            &summary,
            &body,
        )?);
    }
    let mut closed = Vec::new();
    if close.to == cx.me() && close.status != Status::Done {
        cx.update(
            &close.id,
            Status::Done,
            Some(&format!("user decided: {choice}")),
            "",
        )?;
        closed.push(close.id.clone());
    }
    let text = out
        .iter()
        .map(|m| format!("sent {} to {}", m.id, m.to))
        .collect::<Vec<_>>()
        .join("\n");
    sent(&out, &closed, text)
}

pub fn guide(cx: &Context, thread: &str, text: &str) -> Result<Reply> {
    if text.trim().is_empty() {
        return Err(Failure::new("refused", 5, "the user's request is empty"));
    }
    let messages = cx.conversation(thread, None)?;
    let Some(last) = messages.last() else {
        return Err(Failure::new(
            "unknown_id",
            4,
            format!("thread {thread} is not known to {}", cx.me()),
        ));
    };
    let cycle = rules::current_cycle(&messages).0 + 1;
    let first = messages
        .iter()
        .find(|m| m.id == thread)
        .unwrap_or(&messages[0]);
    let subject = format!("User request: {}", rules::strip_re(&first.subject));
    let summary = tool_summary("The user asks: ", text);
    let body = format!(
        "### User request\n{}\n\n### Cycle\nThe user stepped into this conversation. Cycle {cycle} starts: up to {} new rounds that take this request into account, closed by a new agreement or impasse for the user.\n",
        text.trim(),
        rules::MAX_ROUNDS
    );
    let mut out = Vec::new();
    for side in cx.sides(&messages) {
        out.push(cx.send_one(
            &side,
            &subject,
            Kind::Guidance,
            &last.id,
            thread,
            None,
            cycle,
            &summary,
            &body,
        )?);
    }
    let mut closed = Vec::new();
    for m in messages
        .iter()
        .filter(|m| m.to == cx.me() && rules::is_closing(&m.kind) && m.status != Status::Done)
    {
        cx.update(
            &m.id,
            Status::Done,
            Some(&format!("user asked for changes: {}", text.trim())),
            "",
        )?;
        closed.push(m.id.clone());
    }
    let text = out
        .iter()
        .map(|m| format!("sent {} to {} (cycle {})", m.id, m.to, m.cycle))
        .collect::<Vec<_>>()
        .join("\n");
    sent(&out, &closed, text)
}

pub fn status(cx: &Context, id: &str, status: &str, note: Option<&str>) -> Result<Reply> {
    let status = Status::parse(status).ok_or_else(|| {
        Failure::new(
            "invalid",
            2,
            format!("unknown status '{status}' (open, seen, done)"),
        )
    })?;
    let m = cx.update(id, status, note, "")?;
    status_doc(format!("{}: {}", m.id, m.status.as_str()), Some(&m))
}

pub fn defer(cx: &Context, id: &str, reason: &str) -> Result<Reply> {
    let m = cx.update(
        id,
        Status::Seen,
        Some(&format!("deferred: {reason}")),
        reason,
    )?;
    status_doc(format!("{}: deferred ({reason})", m.id), Some(&m))
}

/// One pass of `watch`: polls each target and runs `exec` in its folder on news.
pub fn watch_once(
    targets: &[(String, PathBuf)],
    cache: &Path,
    tool: &str,
    exec: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    for (slug, folder) in targets {
        let reply = Context::open(folder, cache.to_path_buf(), tool).and_then(|cx| {
            if cx.me() != slug {
                return Err(Failure::new(
                    "invalid",
                    3,
                    format!(
                        "{} is {} in its {}, not {slug}",
                        folder.display(),
                        cx.me(),
                        registry::FILE
                    ),
                ));
            }
            poll(&cx)
        });
        match reply {
            Ok(r) if r.code == NEWS => {
                lines.push(r.text.clone());
                let mut cmd = if cfg!(windows) {
                    let mut c = std::process::Command::new("cmd");
                    c.args(["/C", exec]);
                    c
                } else {
                    let mut c = std::process::Command::new("sh");
                    c.args(["-c", exec]);
                    c
                };
                let _ = cmd
                    .current_dir(folder)
                    .env("AURORA_PROJECT", slug)
                    .env("AURORA_INBOX_SUMMARY", &r.text)
                    .status();
            }
            Ok(_) => {}
            Err(e) => lines.push(format!("dcp-integrations: {e}")),
        }
    }
    lines
}
