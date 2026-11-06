//! The message file: a front matter header and a Markdown body in sections (ADR-0009).
//!
//! The format is the one the provisional tool wrote from 0.1 to 0.3, so every message
//! already on an `aurora/inbox` branch is read unchanged.

use std::fmt;

use serde::Serialize;

/// Header fields in the order they are written.
pub const FIELDS: [&str; 13] = [
    "id",
    "kind",
    "from",
    "to",
    "date",
    "subject",
    "status",
    "reply_to",
    "deferred",
    "thread",
    "cycle",
    "round",
    "needs_user",
];

/// The kinds of message. Unknown kinds are kept as text, so a newer writer never breaks
/// an older reader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
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
    /// Free message of the provisional tool 0.1.
    Message,
    Other(String),
}

impl Kind {
    pub fn parse(text: &str) -> Kind {
        match text.trim() {
            "question" => Kind::Question,
            "proposal" => Kind::Proposal,
            "notice" => Kind::Notice,
            "answer" => Kind::Answer,
            "counter" => Kind::Counter,
            "agreement" => Kind::Agreement,
            "impasse" => Kind::Impasse,
            "ack" => Kind::Ack,
            "decision" => Kind::Decision,
            "guidance" => Kind::Guidance,
            "message" | "" => Kind::Message,
            other => Kind::Other(other.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Kind::Question => "question",
            Kind::Proposal => "proposal",
            Kind::Notice => "notice",
            Kind::Answer => "answer",
            Kind::Counter => "counter",
            Kind::Agreement => "agreement",
            Kind::Impasse => "impasse",
            Kind::Ack => "ack",
            Kind::Decision => "decision",
            Kind::Guidance => "guidance",
            Kind::Message => "message",
            Kind::Other(text) => text,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Kind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Seen,
    Done,
}

impl Status {
    pub fn parse(text: &str) -> Option<Status> {
        match text.trim() {
            "open" => Some(Status::Open),
            "seen" => Some(Status::Seen),
            "done" => Some(Status::Done),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Seen => "seen",
            Status::Done => "done",
        }
    }
}

/// A message: its header and its body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Message {
    pub id: String,
    pub kind: Kind,
    pub from: String,
    pub to: String,
    pub date: String,
    pub subject: String,
    pub status: Status,
    pub reply_to: String,
    pub deferred: String,
    /// The id of the first message; the message's own id when empty (tool 0.1).
    pub thread: String,
    /// 1 + the user's requests before it; 1 when the header has none.
    pub cycle: u32,
    /// 1 to 3 within the cycle; `None` outside rounds.
    pub round: Option<u32>,
    pub needs_user: bool,
    /// The writer's version, empty for tool 0.1.
    pub tool: String,
    #[serde(skip)]
    pub body: String,
}

impl Message {
    /// The thread the message belongs to.
    pub fn thread_id(&self) -> &str {
        if self.thread.is_empty() {
            &self.id
        } else {
            &self.thread
        }
    }

    /// The body's sections in order.
    pub fn sections(&self) -> Vec<Section> {
        sections(&self.body)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Section {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{path}: {message}")]
pub struct ParseError {
    pub path: String,
    pub message: String,
}

/// Parses a message file. `path` names it in errors and gives the id when the header has
/// none.
pub fn parse(text: &str, path: &str) -> Result<Message, ParseError> {
    let text = text.replace("\r\n", "\n");
    let (header, body) = split_front_matter(&text);
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in header.lines() {
        if let Some((key, value)) = line.split_once(':') {
            fields.push((key.trim().to_owned(), unquote(value.trim())));
        }
    }
    let get = |key: &str| {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    let stem = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".md")
        .to_owned();
    let id = match get("id") {
        id if id.is_empty() => stem,
        id => id,
    };
    let status = Status::parse(&get("status")).ok_or_else(|| ParseError {
        path: path.to_owned(),
        message: format!("unknown status '{}'", get("status")),
    })?;
    let number = |key: &str| -> Result<Option<u32>, ParseError> {
        let value = get(key);
        if value.is_empty() {
            return Ok(None);
        }
        value.parse().map(Some).map_err(|_| ParseError {
            path: path.to_owned(),
            message: format!("'{key}' is not a number: '{value}'"),
        })
    };
    Ok(Message {
        id,
        kind: Kind::parse(&get("kind")),
        from: get("from"),
        to: get("to"),
        date: get("date"),
        subject: get("subject"),
        status,
        reply_to: get("reply_to"),
        deferred: get("deferred"),
        thread: get("thread"),
        cycle: number("cycle")?.unwrap_or(1).max(1),
        round: number("round")?,
        needs_user: get("needs_user") == "true",
        tool: get("tool"),
        body: body.trim().to_owned(),
    })
}

/// Writes a message file in the format of the provisional tool 0.3.
pub fn render(message: &Message, tool: &str) -> String {
    let quoted = |value: &str| {
        if value.is_empty() {
            String::new()
        } else {
            format!("\"{}\"", value.replace('"', "'"))
        }
    };
    let values = [
        message.id.clone(),
        message.kind.to_string(),
        message.from.clone(),
        message.to.clone(),
        message.date.clone(),
        quoted(&message.subject),
        message.status.as_str().to_owned(),
        message.reply_to.clone(),
        quoted(&message.deferred),
        message.thread.clone(),
        message.cycle.to_string(),
        message.round.map(|r| r.to_string()).unwrap_or_default(),
        if message.needs_user {
            "true".to_owned()
        } else {
            String::new()
        },
    ];
    let mut out = String::from("---\n");
    for (key, value) in FIELDS.iter().zip(values.iter()) {
        out.push_str(&format!("{key}: {value}\n"));
    }
    let tool = if message.tool.is_empty() {
        tool
    } else {
        &message.tool
    };
    out.push_str(&format!("tool: {tool}\n---\n\n"));
    out.push_str(message.body.trim());
    out.push('\n');
    out
}

/// The `##` or `###` sections of a body, in order, with their text trimmed.
pub fn sections(body: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    for line in body.lines() {
        let heading = line
            .strip_prefix("### ")
            .or_else(|| line.strip_prefix("## "))
            .map(str::trim)
            .filter(|h| !h.is_empty() && !line.starts_with("####"));
        match heading {
            Some(name) => out.push(Section {
                name: name.to_owned(),
                text: String::new(),
            }),
            None => {
                if let Some(last) = out.last_mut() {
                    last.text.push_str(line);
                    last.text.push('\n');
                }
            }
        }
    }
    for section in &mut out {
        section.text = section.text.trim().to_owned();
    }
    out
}

/// A section's text by name, ignoring case.
pub fn section<'a>(sections: &'a [Section], name: &str) -> Option<&'a str> {
    sections
        .iter()
        .rev()
        .find(|s| s.name.eq_ignore_ascii_case(name))
        .map(|s| s.text.as_str())
}

fn split_front_matter(text: &str) -> (&str, &str) {
    if let Some(rest) = text.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---")
    {
        let after = &rest[end + 4..];
        return (&rest[..end], after.strip_prefix('\n').unwrap_or(after));
    }
    ("", text)
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
        .to_owned()
}

/// A short, file-safe slug of a subject, as the provisional tool makes it.
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in text.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    let mut out: String = out.trim_end_matches('-').chars().take(40).collect();
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "message".to_owned()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOL_01: &str = "---\nid: 20261029-213000-aurora-dev-suite-settings-view\nkind: message\nfrom: aurora-dev-suite\nto: dev-cycle-protocol\ndate: 2026-10-29T21:30:00Z\nsubject: \"Settings view\"\nstatus: done\nreply_to:\ndeferred:\n---\n\n## Context\nA.\n\n## What is asked\nB.\n";
    const TOOL_02: &str = "---\nid: 20261106-115643-aurora-dev-suite-re-proposal\nkind: answer\nfrom: aurora-dev-suite\nto: dev-cycle-protocol\ndate: 2026-11-06T11:56:43Z\nsubject: \"Re: Proposal\"\nstatus: open\nreply_to: 20261106-114054-dev-cycle-protocol-proposal\ndeferred: \nthread: 20261106-114054-dev-cycle-protocol-proposal\nround: 1\nneeds_user: \ntool: 0.2.0\n---\n\n### Answer\nAgree.\n\n### Next step\nNone.\n";
    const TOOL_03: &str = "---\nid: 20261106-123044-alpha-user-request-test\nkind: guidance\nfrom: alpha\nto: beta\ndate: 2026-11-06T12:30:44.123Z\nsubject: \"User request: Test\"\nstatus: open\nreply_to: x\ndeferred: \nthread: 20261106-123043-alpha-test\ncycle: 2\nround: \nneeds_user: \ntool: 0.3.0\n---\n\n### User request\nUse option 2.\n";

    #[test]
    fn reads_every_tool_version() {
        let m = parse(TOOL_01, "messages/x.md").unwrap();
        assert_eq!(m.kind, Kind::Message);
        assert_eq!(
            m.thread_id(),
            "20261029-213000-aurora-dev-suite-settings-view"
        );
        assert_eq!((m.cycle, m.round, m.status), (1, None, Status::Done));
        assert_eq!(m.subject, "Settings view");
        let m = parse(TOOL_02, "messages/y.md").unwrap();
        assert_eq!(
            (m.kind.clone(), m.cycle, m.round),
            (Kind::Answer, 1, Some(1))
        );
        assert_eq!(m.thread_id(), "20261106-114054-dev-cycle-protocol-proposal");
        let m = parse(TOOL_03, "messages/z.md").unwrap();
        assert_eq!(
            (m.kind.clone(), m.cycle, m.round),
            (Kind::Guidance, 2, None)
        );
        assert_eq!(
            section(&m.sections(), "user request"),
            Some("Use option 2.")
        );
    }

    #[test]
    fn render_then_parse_keeps_the_message() {
        for (text, path) in [(TOOL_01, "a.md"), (TOOL_02, "b.md"), (TOOL_03, "c.md")] {
            let m = parse(text, path).unwrap();
            let again = parse(&render(&m, "1.0.0"), path).unwrap();
            assert_eq!(
                again.tool,
                if m.tool.is_empty() {
                    "1.0.0".to_owned()
                } else {
                    m.tool.clone()
                }
            );
            assert_eq!(
                Message {
                    tool: m.tool.clone(),
                    ..again
                },
                m
            );
        }
    }

    #[test]
    fn sections_accept_two_or_three_hashes_and_ignore_deeper() {
        let s = sections("intro\n## Ask\nDo X\n#### detail\nmore\n### Closes when\n\nWhen done\n");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].text, "Do X\n#### detail\nmore");
        assert_eq!(section(&s, "closes WHEN"), Some("When done"));
    }

    #[test]
    fn crlf_and_unknown_kinds_are_read() {
        let text = TOOL_02
            .replace('\n', "\r\n")
            .replace("kind: answer", "kind: poll-result");
        let m = parse(&text, "b.md").unwrap();
        assert_eq!(m.kind, Kind::Other("poll-result".into()));
        assert_eq!(render(&m, "x").matches("kind: poll-result").count(), 1);
    }

    #[test]
    fn bad_status_or_number_is_an_error() {
        assert!(parse(&TOOL_02.replace("status: open", "status: lost"), "b.md").is_err());
        assert!(parse(&TOOL_02.replace("round: 1", "round: one"), "b.md").is_err());
    }

    #[test]
    fn slug_matches_the_provisional_tool() {
        assert_eq!(
            slug("Re: dcp v2.12.0 and DCP FEAT-018: status, timing"),
            "re-dcp-v2-12-0-and-dcp-feat-018-status-t"
        );
        assert_eq!(slug("!!!"), "message");
    }
}
