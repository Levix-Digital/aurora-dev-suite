//! The Messages view as a chat between projects (spec 011, FEAT-039): each registered
//! project is a contact, and its chat is one stream of what it sent and received, as
//! bubbles with the sender's summary, with the closing records as decision cards.
//!
//! Everything here is pure: it reads the snapshot's inboxes and never calls a provider.

use serde::{Deserialize, Serialize};

use super::messages::{
    ClosingRecord, InboxSetup, Message, MessageHeader, MessageKind, MessageStatus, ProjectInbox,
};
use super::project::ProjectId;

/// The text of a bubble: the sender's summary, else the subject without its `Re:`.
pub fn bubble_text(header: &MessageHeader) -> String {
    let summary = header.summary.trim();
    if summary.is_empty() {
        strip_re(&header.subject).to_owned()
    } else {
        summary.to_owned()
    }
}

/// A subject without its leading `Re: ` (any number of them).
pub fn strip_re(subject: &str) -> &str {
    let mut s = subject.trim();
    while let Some(rest) = s
        .strip_prefix("Re: ")
        .or_else(|| s.strip_prefix("RE: "))
        .or_else(|| s.strip_prefix("re: "))
    {
        s = rest.trim_start();
    }
    s
}

/// The last message of a contact, for its row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatLast {
    pub text: String,
    /// RFC 3339, UTC.
    pub date: String,
    /// The project sent it (the row shows `You:`).
    pub outgoing: bool,
}

/// One registered project in the list of contacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatContact {
    pub project: ProjectId,
    pub name: String,
    pub slug: Option<String>,
    pub setup: InboxSetup,
    pub last: Option<ChatLast>,
    /// Received messages not read yet.
    pub unread: u32,
    /// Conversations whose closing record waits for the user.
    pub waiting: u32,
}

/// Every message the inbox knows, oldest first: the conversations' items, or their last
/// header when the reading had no items.
fn messages_of(inbox: &ProjectInbox) -> Vec<Message> {
    let mut all: Vec<Message> = Vec::new();
    for c in &inbox.conversations {
        if c.items.is_empty() {
            all.push(Message {
                header: c.last.clone(),
                sections: Vec::new(),
                body: String::new(),
                text: String::new(),
                notes: Vec::new(),
            });
        } else {
            all.extend(c.items.iter().cloned());
        }
    }
    all.sort_by(|a, b| {
        a.header
            .date
            .cmp(&b.header.date)
            .then_with(|| a.header.id.cmp(&b.header.id))
    });
    all.dedup_by(|a, b| a.header.id == b.header.id);
    all
}

/// The contacts: projects with messages first, the most recent first; then the ones
/// without messages, then the ones not taking part, each by name.
pub fn chat_contacts(inboxes: &[ProjectInbox]) -> Vec<ChatContact> {
    let mut out: Vec<ChatContact> = inboxes
        .iter()
        .map(|inbox| {
            let me = inbox.slug.clone().unwrap_or_default();
            let last = messages_of(inbox)
                .into_iter()
                .rfind(|m| m.header.kind != MessageKind::Decision || m.header.from == me)
                .map(|m| ChatLast {
                    text: bubble_text(&m.header),
                    outgoing: m.header.from == me,
                    date: m.header.date,
                });
            ChatContact {
                project: inbox.project.clone(),
                name: inbox.name.clone(),
                slug: inbox.slug.clone(),
                setup: inbox.setup.clone(),
                last,
                unread: inbox.conversations.iter().map(|c| c.unread).sum(),
                waiting: u32::try_from(inbox.waiting_count()).unwrap_or(u32::MAX),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        let group = |c: &ChatContact| match (&c.setup, &c.last) {
            (InboxSetup::Ready, Some(_)) => 0,
            (InboxSetup::Ready, None) => 1,
            _ => 2,
        };
        group(a)
            .cmp(&group(b))
            .then_with(|| {
                let date = |c: &ChatContact| c.last.as_ref().map(|l| l.date.clone());
                date(b).cmp(&date(a))
            })
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    out
}

/// What became of a message, as its sender sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeliveryMark {
    /// Written, not read yet: `✓`.
    Sent,
    /// Read by the receiver: `✓✓`.
    Seen,
    /// Handled by the receiver: `✓✓` in the accent color.
    Handled,
    /// Set aside by the receiver: a clock.
    Deferred,
}

impl DeliveryMark {
    pub fn of(header: &MessageHeader) -> Self {
        if !header.deferred.trim().is_empty() {
            return DeliveryMark::Deferred;
        }
        match header.status {
            MessageStatus::Open => DeliveryMark::Sent,
            MessageStatus::Seen => DeliveryMark::Seen,
            MessageStatus::Done => DeliveryMark::Handled,
        }
    }

    pub const fn symbol(self) -> &'static str {
        match self {
            DeliveryMark::Sent => "✓",
            DeliveryMark::Seen | DeliveryMark::Handled => "✓✓",
            DeliveryMark::Deferred => "◷",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            DeliveryMark::Sent => "Sent, not read yet",
            DeliveryMark::Seen => "Read",
            DeliveryMark::Handled => "Handled",
            DeliveryMark::Deferred => "Set aside",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            DeliveryMark::Sent => "sent",
            DeliveryMark::Seen => "seen",
            DeliveryMark::Handled => "handled",
            DeliveryMark::Deferred => "deferred",
        }
    }
}

/// The message a reply answers, quoted on top of its bubble.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatQuote {
    pub id: String,
    pub from: String,
    pub text: String,
}

/// One message in the chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatBubble {
    pub message: Message,
    /// The summary, else the subject.
    pub text: String,
    /// The conversation's subject, without `Re:`.
    pub subject: String,
    /// The project sent it: a bubble on the right.
    pub outgoing: bool,
    /// The other project of this message.
    pub peer: String,
    /// The first message of its conversation in the stream: shows subject, kind and round.
    pub opens: bool,
    /// The message it answers, when that is not the bubble right above it.
    pub quote: Option<ChatQuote>,
    pub mark: DeliveryMark,
    /// Received and not read yet.
    pub unread: bool,
}

/// How a closing record stands in the chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum RecordState {
    /// The user has to decide: the card has its options.
    Waiting,
    /// The user decided: the card is closed with the choice.
    Decided { choice: String, date: String },
    /// Neither: the decision is taken through the project that opened the conversation.
    Pending,
}

/// A closing record shown as a card in the stream, right after the message that closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRecord {
    pub thread: String,
    pub subject: String,
    /// Agreement or impasse.
    pub kind: MessageKind,
    pub record: ClosingRecord,
    pub state: RecordState,
    /// The date of the closing message.
    pub date: String,
}

/// One line of the chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "entry", rename_all = "camelCase")]
pub enum ChatEntry {
    /// A day separator: `date` is `YYYY-MM-DD` in the user's time zone.
    Day {
        date: String,
    },
    Bubble(Box<ChatBubble>),
    Record(Box<ChatRecord>),
}

impl ChatEntry {
    /// A stable key for the view's list.
    pub fn key(&self) -> String {
        match self {
            ChatEntry::Day { date } => format!("day:{date}"),
            ChatEntry::Bubble(b) => {
                format!("msg:{}:{:?}:{}", b.message.header.id, b.mark, b.unread)
            }
            ChatEntry::Record(r) => format!("rec:{}:{:?}", r.thread, r.state),
        }
    }
}

/// The text of the Decision section of a decision message.
fn decision_choice(m: &Message) -> String {
    m.sections
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case("Decision"))
        .map(|s| s.text.trim().to_owned())
        .unwrap_or_else(|| bubble_text(&m.header))
}

/// The chat of one project: every message it sent or received, oldest first, with a day
/// separator when the day changes (in the time zone `offset_minutes` east of UTC) and the
/// closing record as a card after the message that closed. `with` keeps only the
/// messages exchanged with one counterpart. Decision messages are not bubbles: they close
/// their record's card.
pub fn chat_stream(
    inbox: &ProjectInbox,
    with: Option<&str>,
    offset_minutes: i32,
) -> Vec<ChatEntry> {
    let me = inbox.slug.clone().unwrap_or_default();
    let all = messages_of(inbox);
    let keeps = |m: &Message| with.is_none_or(|peer| m.header.from == peer || m.header.to == peer);
    let mut out: Vec<ChatEntry> = Vec::new();
    let mut day = String::new();
    let mut seen_threads: Vec<String> = Vec::new();
    let mut previous: Option<String> = None;
    for m in all.iter().filter(|m| keeps(m)) {
        let h = &m.header;
        if h.kind == MessageKind::Decision {
            continue;
        }
        let local_day = local_time(&h.date, offset_minutes)
            .map(|(d, _)| d)
            .unwrap_or_else(|| h.date.get(..10).unwrap_or_default().to_owned());
        if local_day != day {
            day.clone_from(&local_day);
            out.push(ChatEntry::Day { date: local_day });
        }
        let outgoing = h.from == me;
        let conversation = inbox.conversations.iter().find(|c| c.thread == h.thread);
        let subject = conversation
            .map(|c| strip_re(&c.subject).to_owned())
            .unwrap_or_else(|| strip_re(&h.subject).to_owned());
        let opens = !seen_threads.contains(&h.thread);
        if opens {
            seen_threads.push(h.thread.clone());
        }
        let quote = (!h.reply_to.is_empty() && previous.as_deref() != Some(h.reply_to.as_str()))
            .then(|| all.iter().find(|o| o.header.id == h.reply_to))
            .flatten()
            .map(|o| ChatQuote {
                id: o.header.id.clone(),
                from: o.header.from.clone(),
                text: bubble_text(&o.header),
            });
        out.push(ChatEntry::Bubble(Box::new(ChatBubble {
            text: bubble_text(h),
            subject,
            outgoing,
            peer: if outgoing {
                h.to.clone()
            } else {
                h.from.clone()
            },
            opens,
            quote,
            mark: DeliveryMark::of(h),
            unread: !outgoing && h.status == MessageStatus::Open,
            message: m.clone(),
        })));
        previous = Some(h.id.clone());
        // The record follows the last closing message of its conversation.
        if let Some(c) = conversation
            && h.kind.is_closing()
            && let Some(record) = &c.record
            && all
                .iter()
                .rfind(|o| o.header.thread == h.thread && o.header.kind.is_closing())
                .is_some_and(|last| last.header.id == h.id)
        {
            let decision = all
                .iter()
                .filter(|o| o.header.thread == h.thread && o.header.kind == MessageKind::Decision)
                .rfind(|o| o.header.date >= h.date);
            let state = match decision {
                Some(d) => RecordState::Decided {
                    choice: decision_choice(d),
                    date: d.header.date.clone(),
                },
                None if c.waiting_for_user => RecordState::Waiting,
                None => RecordState::Pending,
            };
            out.push(ChatEntry::Record(Box::new(ChatRecord {
                thread: h.thread.clone(),
                subject: strip_re(&c.subject).to_owned(),
                kind: h.kind.clone(),
                record: record.clone(),
                state,
                date: h.date.clone(),
            })));
        }
    }
    out
}

/// The counterparts a project has exchanged messages with, by name.
pub fn chat_peers(inbox: &ProjectInbox) -> Vec<String> {
    let me = inbox.slug.clone().unwrap_or_default();
    let mut peers: Vec<String> = messages_of(inbox)
        .iter()
        .flat_map(|m| [m.header.from.clone(), m.header.to.clone()])
        .filter(|p| !p.is_empty() && *p != me)
        .collect();
    peers.sort();
    peers.dedup();
    peers
}

// --- dates ---------------------------------------------------------------------------

/// Days since 1970-01-01 of a civil date (H. Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date of a number of days since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn number(text: &str, range: std::ops::Range<usize>) -> Option<i64> {
    text.get(range)?.parse().ok()
}

/// Minutes since 1970-01-01 of an RFC 3339 date in UTC (`Z`, or no zone).
fn utc_minutes(date: &str) -> Option<i64> {
    let days = days_from_civil(
        number(date, 0..4)?,
        number(date, 5..7)?,
        number(date, 8..10)?,
    );
    let (hour, minute) = if date.len() >= 16 {
        (number(date, 11..13)?, number(date, 14..16)?)
    } else {
        (0, 0)
    };
    Some(days * 1440 + hour * 60 + minute)
}

/// `YYYY-MM-DD` and `HH:MM` of an RFC 3339 UTC date in the time zone `offset_minutes`
/// east of UTC.
pub fn local_time(date: &str, offset_minutes: i32) -> Option<(String, String)> {
    let minutes = utc_minutes(date)? + i64::from(offset_minutes);
    let (y, m, d) = civil_from_days(minutes.div_euclid(1440));
    let rest = minutes.rem_euclid(1440);
    Some((
        format!("{y:04}-{m:02}-{d:02}"),
        format!("{:02}:{:02}", rest / 60, rest % 60),
    ))
}

const WEEKDAYS: [&str; 7] = [
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn day_number(ymd: &str) -> Option<i64> {
    Some(days_from_civil(
        number(ymd, 0..4)?,
        number(ymd, 5..7)?,
        number(ymd, 8..10)?,
    ))
}

fn weekday(days: i64) -> &'static str {
    WEEKDAYS[usize::try_from(days.rem_euclid(7)).unwrap_or(0)]
}

/// The label of a day separator: `Today`, `Yesterday`, `Saturday, 1 November`, or with
/// the year when it is not `today`'s.
pub fn day_label(ymd: &str, today: &str) -> String {
    let (Some(day), Some(now)) = (day_number(ymd), day_number(today)) else {
        return ymd.to_owned();
    };
    match now - day {
        0 => "Today".to_owned(),
        1 => "Yesterday".to_owned(),
        _ => {
            let (y, m, d) = civil_from_days(day);
            let month = MONTHS[usize::try_from(m - 1).unwrap_or(0)];
            if ymd.get(..4) == today.get(..4) {
                format!("{}, {d} {month}", weekday(day))
            } else {
                format!("{}, {d} {month} {y}", weekday(day))
            }
        }
    }
}

/// The time on a contact's row: the time today, `Yesterday`, the weekday within a week,
/// else the day and short month.
pub fn row_time(date: &str, today: &str, offset_minutes: i32) -> String {
    let Some((ymd, hm)) = local_time(date, offset_minutes) else {
        return date.to_owned();
    };
    let (Some(day), Some(now)) = (day_number(&ymd), day_number(today)) else {
        return ymd;
    };
    match now - day {
        0 => hm,
        1 => "Yesterday".to_owned(),
        2..=6 => weekday(day).to_owned(),
        _ => {
            let (y, m, d) = civil_from_days(day);
            let month = &MONTHS[usize::try_from(m - 1).unwrap_or(0)][..3];
            if ymd.get(..4) == today.get(..4) {
                format!("{d} {month}")
            } else {
                format!("{d} {month} {y}")
            }
        }
    }
}

/// Two letters for a project without a logo: the first letters of its first two words,
/// else its first two letters.
pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let text: String = match words.as_slice() {
        [] => "?".to_owned(),
        [one] => one.chars().take(2).collect(),
        [a, b, ..] => a.chars().take(1).chain(b.chars().take(1)).collect(),
    };
    text.to_uppercase()
}

/// One of `count` palette colors for a project, always the same for the same slug.
pub fn avatar_color(slug: &str, count: usize) -> usize {
    let hash = slug.bytes().fold(2_166_136_261_u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(16_777_619)
    });
    usize::try_from(hash).unwrap_or(0) % count.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::messages::{Conversation, Section};

    fn header(id: &str, kind: MessageKind, from: &str, to: &str, date: &str) -> MessageHeader {
        MessageHeader {
            id: id.into(),
            kind,
            from: from.into(),
            to: to.into(),
            date: date.into(),
            subject: "Re: Hook timeouts".into(),
            status: MessageStatus::Open,
            reply_to: String::new(),
            deferred: String::new(),
            thread: "t1".into(),
            cycle: 1,
            round: Some(1),
            needs_user: false,
            tool: "0.3.0".into(),
            summary: String::new(),
        }
    }

    fn message(h: MessageHeader, sections: Vec<Section>) -> Message {
        Message {
            header: h,
            sections,
            body: String::new(),
            text: String::new(),
            notes: Vec::new(),
        }
    }

    fn record() -> ClosingRecord {
        serde_json::from_value(serde_json::json!({
            "thread": "t1", "outcome": "agreement", "subject": "Hook timeouts",
            "projects": ["aurora", "dcp"], "cycle": 1, "rounds": 2,
            "conclusion": "A 300 s cap.", "justification": "", "positions": "",
            "openPoints": "", "decisionNeeded": "1. Accept\n2. Refuse",
            "afterApproval": "", "options": ["Accept", "Refuse"], "userRequests": []
        }))
        .expect("record")
    }

    fn inbox(items: Vec<Message>, waiting: bool) -> ProjectInbox {
        let last = items.last().map(|m| m.header.clone()).expect("items");
        let closing = items.iter().any(|m| m.header.kind.is_closing());
        ProjectInbox {
            project: ProjectId("p1".into()),
            name: "Aurora".into(),
            slug: Some("aurora".into()),
            counterparts: vec!["dcp".into(), "gedo".into()],
            setup: InboxSetup::Ready,
            conversations: vec![Conversation {
                thread: "t1".into(),
                subject: "Hook timeouts".into(),
                projects: vec!["aurora".into(), "dcp".into()],
                opener: Some("dcp".into()),
                cycle: 1,
                round: Some(2),
                next_round: 3,
                messages: u32::try_from(items.len()).expect("small"),
                last,
                waiting_for_user: waiting,
                user_requests: Vec::new(),
                unread: 1,
                record: closing.then(record),
                items,
            }],
        }
    }

    fn sample() -> Vec<Message> {
        let mut a = header(
            "m1",
            MessageKind::Proposal,
            "dcp",
            "aurora",
            "2026-11-01T09:20:00.000Z",
        );
        a.subject = "Hook timeouts".into();
        a.summary = "I propose one limit per hook point.".into();
        a.status = MessageStatus::Done;
        let mut b = header(
            "m2",
            MessageKind::Counter,
            "aurora",
            "dcp",
            "2026-11-01T09:21:00.000Z",
        );
        b.reply_to = "m1".into();
        b.status = MessageStatus::Seen;
        let mut c = header(
            "m3",
            MessageKind::Answer,
            "dcp",
            "aurora",
            "2026-11-02T01:00:00.000Z",
        );
        c.reply_to = "m1".into();
        c.summary = "Then 300 s as the cap.".into();
        let mut d = header(
            "m4",
            MessageKind::Agreement,
            "aurora",
            "dcp",
            "2026-11-02T09:23:00.000Z",
        );
        d.reply_to = "m3".into();
        vec![
            message(a, vec![]),
            message(b, vec![]),
            message(c, vec![]),
            message(d, vec![]),
        ]
    }

    #[test]
    fn bubbles_show_the_summary_or_the_subject_and_their_side() {
        let stream = chat_stream(&inbox(sample(), true), None, 0);
        let bubbles: Vec<&ChatBubble> = stream
            .iter()
            .filter_map(|e| match e {
                ChatEntry::Bubble(b) => Some(b.as_ref()),
                _ => None,
            })
            .collect();
        assert_eq!(bubbles.len(), 4);
        assert_eq!(bubbles[0].text, "I propose one limit per hook point.");
        assert_eq!(bubbles[1].text, "Hook timeouts");
        assert_eq!(
            bubbles.iter().map(|b| b.outgoing).collect::<Vec<_>>(),
            [false, true, false, true]
        );
        assert!(bubbles[0].opens && !bubbles[1].opens);
        assert_eq!(bubbles[1].peer, "dcp");
        assert_eq!(
            bubbles.iter().map(|b| b.mark).collect::<Vec<_>>(),
            [
                DeliveryMark::Handled,
                DeliveryMark::Seen,
                DeliveryMark::Sent,
                DeliveryMark::Sent
            ]
        );
        assert!(bubbles[2].unread && !bubbles[3].unread);
    }

    #[test]
    fn a_reply_quotes_what_it_answers_unless_it_is_right_above() {
        let stream = chat_stream(&inbox(sample(), true), None, 0);
        let quotes: Vec<Option<String>> = stream
            .iter()
            .filter_map(|e| match e {
                ChatEntry::Bubble(b) => Some(b.quote.as_ref().map(|q| q.text.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(
            quotes,
            [
                None,
                None,
                Some("I propose one limit per hook point.".to_owned()),
                None
            ]
        );
    }

    #[test]
    fn days_follow_the_time_zone_and_the_record_follows_the_closing_message() {
        let stream = chat_stream(&inbox(sample(), true), None, -180);
        let keys: Vec<String> = stream
            .iter()
            .map(|e| match e {
                ChatEntry::Day { date } => date.clone(),
                ChatEntry::Bubble(b) => b.message.header.id.clone(),
                ChatEntry::Record(r) => format!("record:{:?}", r.state),
            })
            .collect();
        // 01:00 UTC on the 2nd is still the 1st at UTC-3.
        assert_eq!(
            keys,
            [
                "2026-11-01",
                "m1",
                "m2",
                "m3",
                "2026-11-02",
                "m4",
                "record:Waiting"
            ]
        );
    }

    #[test]
    fn a_decision_closes_the_card_and_is_not_a_bubble() {
        let mut items = sample();
        let mut d = header(
            "m5",
            MessageKind::Decision,
            "aurora",
            "dcp",
            "2026-11-02T10:00:00.000Z",
        );
        d.kind = MessageKind::Decision;
        items.push(message(d, vec![Section::new("Decision", "Accept")]));
        let stream = chat_stream(&inbox(items.clone(), false), None, 0);
        assert!(
            !stream
                .iter()
                .any(|e| matches!(e, ChatEntry::Bubble(b) if b.message.header.id == "m5"))
        );
        assert!(stream.iter().any(|e| matches!(
            e,
            ChatEntry::Record(r) if r.state == RecordState::Decided { choice: "Accept".into(), date: "2026-11-02T10:00:00.000Z".into() }
        )));
        let pending = chat_stream(&inbox(sample(), false), None, 0);
        assert!(
            pending
                .iter()
                .any(|e| matches!(e, ChatEntry::Record(r) if r.state == RecordState::Pending))
        );
    }

    #[test]
    fn the_counterpart_filter_keeps_one_exchange() {
        let mut items = sample();
        let mut g = header(
            "g1",
            MessageKind::Question,
            "gedo",
            "aurora",
            "2026-11-03T10:00:00.000Z",
        );
        g.thread = "t2".into();
        items.push(message(g, vec![]));
        let ib = inbox(items, false);
        assert_eq!(chat_peers(&ib), ["dcp", "gedo"]);
        let only = chat_stream(&ib, Some("gedo"), 0);
        let ids: Vec<&str> = only
            .iter()
            .filter_map(|e| match e {
                ChatEntry::Bubble(b) => Some(b.message.header.id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(ids, ["g1"]);
    }

    #[test]
    fn contacts_show_the_last_message_and_sort_by_it() {
        let busy = inbox(sample(), true);
        let mut quiet = inbox(sample(), false);
        quiet.project = ProjectId("p2".into());
        quiet.name = "Quiet".into();
        quiet.conversations.clear();
        let mut off = quiet.clone();
        off.project = ProjectId("p3".into());
        off.name = "Off".into();
        off.setup = InboxSetup::Unavailable {
            reason: "no network".into(),
        };
        let contacts = chat_contacts(&[off, quiet, busy]);
        assert_eq!(
            contacts.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            ["Aurora", "Quiet", "Off"]
        );
        let last = contacts[0].last.clone().expect("last");
        assert_eq!((last.text.as_str(), last.outgoing), ("Hook timeouts", true));
        assert_eq!((contacts[0].unread, contacts[0].waiting), (1, 1));
        assert!(contacts[1].last.is_none());
    }

    #[test]
    fn dates_read_in_the_users_time_zone() {
        assert_eq!(
            local_time("2026-11-02T01:30:00.000Z", -180),
            Some(("2026-11-01".into(), "22:30".into()))
        );
        assert_eq!(day_label("2026-11-08", "2026-11-08"), "Today");
        assert_eq!(day_label("2026-11-07", "2026-11-08"), "Yesterday");
        assert_eq!(day_label("2026-11-01", "2026-11-08"), "Sunday, 1 November");
        assert_eq!(
            day_label("2025-12-31", "2026-11-08"),
            "Wednesday, 31 December 2025"
        );
        assert_eq!(
            row_time("2026-11-08T12:05:00Z", "2026-11-08", -180),
            "09:05"
        );
        assert_eq!(
            row_time("2026-11-07T12:05:00Z", "2026-11-08", 0),
            "Yesterday"
        );
        assert_eq!(
            row_time("2026-11-04T12:05:00Z", "2026-11-08", 0),
            "Wednesday"
        );
        assert_eq!(row_time("2026-10-30T12:05:00Z", "2026-11-08", 0), "30 Oct");
    }

    #[test]
    fn avatars_without_a_logo_have_initials_and_a_steady_color() {
        assert_eq!(initials("aurora-dev-suite"), "AD");
        assert_eq!(initials("Gedo"), "GE");
        assert_eq!(initials(""), "?");
        assert_eq!(avatar_color("gedo", 6), avatar_color("gedo", 6));
        assert!(avatar_color("any-context", 6) < 6);
        assert_eq!(strip_re("Re: Re: Hook"), "Hook");
    }
}
