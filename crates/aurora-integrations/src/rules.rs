//! The conversation rules of ADR-0009: kinds, templates, cycles and rounds, what waits for
//! the user, and the closing record.

use serde::Serialize;

use crate::format::{self, Kind, Message, Section, Status};

pub const MAX_ROUNDS: u32 = 3;

/// Kinds that open a thread.
pub const OPENING: [Kind; 3] = [Kind::Question, Kind::Proposal, Kind::Notice];
/// Kinds a reply may have.
pub const REPLIES: [Kind; 6] = [
    Kind::Answer,
    Kind::Counter,
    Kind::Agreement,
    Kind::Impasse,
    Kind::Question,
    Kind::Ack,
];

pub fn is_closing(kind: &Kind) -> bool {
    matches!(kind, Kind::Agreement | Kind::Impasse)
}

/// Kinds that never use up a round.
pub fn outside_rounds(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Ack | Kind::Notice | Kind::Decision | Kind::Guidance
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemplateSection {
    pub name: &'static str,
    pub hint: &'static str,
    pub required: bool,
}

const fn s(name: &'static str, hint: &'static str, required: bool) -> TemplateSection {
    TemplateSection {
        name,
        hint,
        required,
    }
}

const ASK: &str = "One or two sentences: what the receiver should do or answer.";
const CONTEXT: &str = "Only what the receiver needs. Cite items with the owner's prefix (DCP FEAT-018, Aurora FEAT-031) and pin code with a commit or a tag, never \"latest\".";
const PROPOSAL: &str = "What the sender suggests, numbered, so the answer can refer to each point.";
const CONSTRAINTS: &str =
    "What the sender cannot change (decided by its user, released, a standing rule).";
const CLOSES: &str = "The answer or artifact that ends the conversation, checkable by both sides.";
const AUTHORITY: &str = "What the projects may settle in the rounds, and what goes to the user.";
const ANSWER: &str = "Agree | counter | question | impasse, in one line.";
const PER_POINT: &str = "1. agreed / changed to ... / refused, because ...";
const OPEN_POINTS: &str = "What is still undecided, or \"none\".";
const NEXT_STEP: &str = "Who does what, and the record each project keeps (backlog id, PR, ADR).";
const CONCLUSION: &str =
    "For the user: what was agreed, or what remains open, in two or three sentences.";
const JUSTIFICATION: &str = "For the user: why this is the right outcome.";
const POSITIONS: &str = "For the user: each project's position and recommendation.";
const DECISION: &str = "For the user: the options, numbered, with the recommended one marked.";
const AFTER: &str = "For the user: what each project does once the user decides.";

/// The skeleton of a kind, `None` for the free kinds.
pub fn template(kind: &Kind) -> Option<Vec<TemplateSection>> {
    let reply = |per_point_required: bool| {
        vec![
            s("Answer", ANSWER, true),
            s("Per point", PER_POINT, per_point_required),
            s("Open points", OPEN_POINTS, false),
            s("Next step", NEXT_STEP, true),
        ]
    };
    let closing = |impasse: bool| {
        let mut v = vec![
            s("Answer", ANSWER, true),
            s("Per point", PER_POINT, false),
            s("Open points", OPEN_POINTS, false),
            s("Next step", NEXT_STEP, false),
            s("Conclusion", CONCLUSION, true),
            s("Justification", JUSTIFICATION, true),
        ];
        if impasse {
            v.push(s("Positions", POSITIONS, true));
        }
        v.push(s("Decision needed", DECISION, true));
        v.push(s("After approval", AFTER, true));
        v
    };
    Some(match kind {
        Kind::Question => vec![
            s("Ask", ASK, true),
            s("Context", CONTEXT, true),
            s("Constraints", CONSTRAINTS, false),
            s("Closes when", CLOSES, true),
            s("Authority", AUTHORITY, false),
        ],
        Kind::Proposal => vec![
            s("Ask", ASK, true),
            s("Context", CONTEXT, true),
            s("Proposal", PROPOSAL, true),
            s("Constraints", CONSTRAINTS, false),
            s("Closes when", CLOSES, true),
            s("Authority", AUTHORITY, false),
        ],
        Kind::Answer => reply(false),
        Kind::Counter => reply(true),
        Kind::Agreement => closing(false),
        Kind::Impasse => closing(true),
        _ => return None,
    })
}

/// The skeleton as Markdown, as `template <kind>` prints it.
pub fn skeleton(sections: &[TemplateSection]) -> String {
    let parts: Vec<String> = sections
        .iter()
        .map(|s| format!("### {}\n{}", s.name, s.hint))
        .collect();
    parts.join("\n\n") + "\n"
}

/// The required sections a body leaves missing or empty.
pub fn missing_sections(kind: &Kind, body: &str) -> Vec<String> {
    let Some(template) = template(kind) else {
        return Vec::new();
    };
    let have = format::sections(body);
    template
        .iter()
        .filter(|t| t.required)
        .filter(|t| format::section(&have, t.name).is_none_or(str::is_empty))
        .map(|t| t.name.to_owned())
        .collect()
}

/// The current cycle (1 + the user's requests) and the messages after the last request.
/// `messages` must be in date order.
pub fn current_cycle(messages: &[Message]) -> (u32, &[Message]) {
    let cuts: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.kind == Kind::Guidance)
        .map(|(i, _)| i)
        .collect();
    let cycle = u32::try_from(cuts.len()).unwrap_or(u32::MAX - 1) + 1;
    match cuts.last() {
        Some(&last) => (cycle, &messages[last + 1..]),
        None => (cycle, messages),
    }
}

/// The round the next message of the thread would be in.
pub fn next_round(messages: &[Message]) -> u32 {
    let (_, current) = current_cycle(messages);
    let counted = current.iter().filter(|m| !outside_rounds(&m.kind)).count();
    u32::try_from(counted / 2).unwrap_or(u32::MAX - 1) + 1
}

/// Why a reply of `kind` is refused at this point of the thread, if it is.
pub fn reply_refusal(kind: &Kind, messages: &[Message]) -> Option<String> {
    if outside_rounds(kind) || is_closing(kind) {
        return None;
    }
    let (cycle, _) = current_cycle(messages);
    (next_round(messages) > MAX_ROUNDS).then(|| {
        format!(
            "the thread has used its {MAX_ROUNDS} rounds in cycle {cycle}: close it with an \
             agreement or an impasse, which go to the user"
        )
    })
}

/// The project that opened a thread, read from its id (`date-time-project-slug`), among
/// the known projects (longest name first, so `gedo` never shadows `gedo-x`).
pub fn opener<'a>(thread: &str, projects: &[&'a str]) -> Option<&'a str> {
    let rest = thread.splitn(3, '-').nth(2)?;
    let mut sorted = projects.to_vec();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.len()));
    sorted
        .into_iter()
        .find(|p| rest.strip_prefix(*p).is_some_and(|r| r.starts_with('-')))
}

/// The closing records `me` must present to the user: received, not done, not followed by
/// a decision or a request of the user, in a thread `me` opened.
pub fn waiting_for_user<'a>(
    me: &str,
    received: &'a [Message],
    projects: &[&str],
) -> Vec<&'a Message> {
    received
        .iter()
        .filter(|m| m.needs_user && m.status != Status::Done)
        .filter(|m| opener(m.thread_id(), projects) == Some(me))
        .filter(|m| {
            !received.iter().any(|x| {
                x.thread_id() == m.thread_id()
                    && x.date > m.date
                    && matches!(x.kind, Kind::Decision | Kind::Guidance)
            })
        })
        .collect()
}

/// The report the user reads (ADR-0009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    pub thread: String,
    pub subject: String,
    pub projects: Vec<String>,
    pub rounds: u32,
    pub cycle: u32,
    pub outcome: Kind,
    pub conclusion: String,
    pub justification: String,
    pub positions: String,
    pub open_points: String,
    pub decision_needed: String,
    pub options: Vec<String>,
    pub after_approval: String,
    pub user_requests: Vec<String>,
    pub messages: Vec<String>,
}

/// The closing record of a thread from its messages in date order, `None` without an
/// agreement or impasse.
pub fn record(thread: &str, messages: &[Message]) -> Option<Record> {
    let close = messages.iter().rev().find(|m| is_closing(&m.kind))?;
    let before: Vec<Message> = messages
        .iter()
        .filter(|m| m.date <= close.date)
        .cloned()
        .collect();
    let (cycle, last) = current_cycle(&before);
    let rounds = last.iter().filter_map(|m| m.round).max().unwrap_or(1);
    let sections = close.sections();
    let get = |name: &str| {
        format::section(&sections, name)
            .unwrap_or_default()
            .to_owned()
    };
    let first = messages
        .iter()
        .find(|m| m.id == thread)
        .unwrap_or(&messages[0]);
    let mut projects: Vec<String> = messages
        .iter()
        .flat_map(|m| [m.from.clone(), m.to.clone()])
        .filter(|p| !p.is_empty())
        .collect();
    projects.sort();
    projects.dedup();
    let decision_needed = get("Decision needed");
    Some(Record {
        thread: thread.to_owned(),
        subject: strip_re(&first.subject).to_owned(),
        projects,
        rounds,
        cycle,
        outcome: close.kind.clone(),
        conclusion: get("Conclusion"),
        justification: get("Justification"),
        positions: get("Positions"),
        open_points: get("Open points"),
        options: numbered_options(&decision_needed),
        decision_needed,
        after_approval: get("After approval"),
        user_requests: messages
            .iter()
            .filter(|m| m.kind == Kind::Guidance)
            .map(|m| {
                format::section(&m.sections(), "User request")
                    .map(str::to_owned)
                    .unwrap_or_else(|| m.body.clone())
            })
            .collect(),
        messages: messages.iter().map(|m| m.id.clone()).collect(),
    })
}

/// The record as text, as `record <thread>` prints it.
pub fn record_text(r: &Record) -> String {
    let cycle = if r.cycle > 1 {
        format!(
            ", cycle {} after {} request(s) from the user",
            r.cycle,
            r.cycle - 1
        )
    } else {
        String::new()
    };
    let mut lines = vec![
        format!(
            "Discussed with: {} ({} round(s){cycle})",
            r.projects.join(", "),
            r.rounds
        ),
        format!("Subject: {}", r.subject),
        format!("Outcome: {}", r.outcome),
        String::new(),
        "Conclusion:".into(),
        r.conclusion.clone(),
        String::new(),
        "Justification:".into(),
        r.justification.clone(),
    ];
    if !r.positions.is_empty() {
        lines.extend([String::new(), "Positions:".into(), r.positions.clone()]);
    }
    if !r.open_points.is_empty() && !r.open_points.eq_ignore_ascii_case("none") {
        lines.extend([String::new(), "Open points:".into(), r.open_points.clone()]);
    }
    lines.extend([
        String::new(),
        "Decision needed:".into(),
        r.decision_needed.clone(),
        String::new(),
        "After approval:".into(),
        r.after_approval.clone(),
    ]);
    lines.join("\n")
}

/// The numbered lines (`1.` or `1)`) of a text, without their numbers.
pub fn numbered_options(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let digits = line.chars().take_while(char::is_ascii_digit).count();
            if digits == 0 {
                return None;
            }
            let rest = &line[digits..];
            rest.strip_prefix('.')
                .or_else(|| rest.strip_prefix(')'))
                .map(|r| r.trim().to_owned())
                .filter(|r| !r.is_empty())
        })
        .collect()
}

/// A subject without its leading `Re:`.
pub fn strip_re(subject: &str) -> &str {
    let mut s = subject.trim();
    while let Some(rest) = s.strip_prefix("Re:") {
        s = rest.trim_start();
    }
    s
}

/// Section names of a body, for messages that must show their sections as fields.
pub fn section_list(body: &str) -> Vec<Section> {
    format::sections(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(id: &str, kind: Kind, from: &str, to: &str, date: &str, thread: &str) -> Message {
        Message {
            id: id.into(),
            kind,
            from: from.into(),
            to: to.into(),
            date: date.into(),
            subject: "Re: Re: Test".into(),
            status: Status::Open,
            reply_to: String::new(),
            deferred: String::new(),
            thread: thread.into(),
            cycle: 1,
            round: None,
            needs_user: false,
            tool: String::new(),
            body: String::new(),
        }
    }

    const T: &str = "20261106-120000-alpha-test";

    fn thread(kinds: &[Kind]) -> Vec<Message> {
        kinds
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let (from, to) = if i % 2 == 0 {
                    ("alpha", "beta")
                } else {
                    ("beta", "alpha")
                };
                let id = if i == 0 {
                    T.to_owned()
                } else {
                    format!("{T}-{i}")
                };
                msg(
                    &id,
                    k.clone(),
                    from,
                    to,
                    &format!("2026-11-06T12:00:{i:02}.000Z"),
                    T,
                )
            })
            .collect()
    }

    #[test]
    fn templates_and_required_sections() {
        assert!(template(&Kind::Ack).is_none());
        let missing = missing_sections(&Kind::Proposal, "### Ask\nX\n### Context\n\n");
        assert_eq!(missing, ["Context", "Proposal", "Closes when"]);
        assert!(missing_sections(&Kind::Ack, "").is_empty());
        let impasse: Vec<_> = template(&Kind::Impasse)
            .unwrap()
            .iter()
            .map(|s| s.name)
            .collect();
        assert!(impasse.contains(&"Positions"));
        assert!(skeleton(&template(&Kind::Question).unwrap()).starts_with("### Ask\n"));
    }

    #[test]
    fn three_rounds_then_only_closing() {
        use Kind::*;
        let six = thread(&[Proposal, Counter, Counter, Counter, Counter, Counter]);
        assert_eq!(next_round(&six[..1]), 1);
        assert_eq!(next_round(&six[..2]), 2);
        assert_eq!(next_round(&six[..5]), 3);
        assert_eq!(next_round(&six), 4);
        assert!(reply_refusal(&Counter, &six).is_some());
        assert!(reply_refusal(&Agreement, &six).is_none());
        assert!(reply_refusal(&Ack, &six).is_none());
    }

    #[test]
    fn acks_do_not_count_and_guidance_restarts_the_rounds() {
        use Kind::*;
        let t = thread(&[
            Proposal, Ack, Counter, Ack, Counter, Counter, Counter, Guidance, Counter,
        ]);
        let (cycle, rest) = current_cycle(&t);
        assert_eq!((cycle, rest.len()), (2, 1));
        assert_eq!(next_round(&t), 1);
        assert!(reply_refusal(&Counter, &t).is_none());
    }

    #[test]
    fn opener_prefers_the_longest_project_name() {
        assert_eq!(
            opener("20261106-120000-gedo-x-test", &["gedo", "gedo-x"]),
            Some("gedo-x")
        );
        assert_eq!(
            opener("20261106-120000-gedo-test", &["gedo", "gedo-x"]),
            Some("gedo")
        );
        assert_eq!(opener("garbage", &["gedo"]), None);
    }

    #[test]
    fn waiting_only_for_the_opener_until_decided() {
        let mut close = msg(
            "c",
            Kind::Agreement,
            "beta",
            "alpha",
            "2026-11-06T12:00:05.000Z",
            T,
        );
        close.needs_user = true;
        let received = vec![close.clone()];
        assert_eq!(
            waiting_for_user("alpha", &received, &["alpha", "beta"]).len(),
            1
        );
        assert!(waiting_for_user("beta", &received, &["alpha", "beta"]).is_empty());
        let guided = vec![
            close.clone(),
            msg(
                "g",
                Kind::Guidance,
                "beta",
                "alpha",
                "2026-11-06T12:00:06.000Z",
                T,
            ),
        ];
        assert!(waiting_for_user("alpha", &guided, &["alpha", "beta"]).is_empty());
        let done = vec![Message {
            status: Status::Done,
            ..close
        }];
        assert!(waiting_for_user("alpha", &done, &["alpha", "beta"]).is_empty());
    }

    #[test]
    fn record_reads_the_report_and_the_options() {
        use Kind::*;
        let mut t = thread(&[Proposal, Counter, Guidance, Counter, Agreement]);
        t[0].subject = "Test".into();
        t[1].round = Some(1);
        t[3].round = Some(1);
        t[4].round = Some(1);
        t[2].body = "### User request\nUse option 2.\n".into();
        t[4].body = "### Answer\nAgree\n### Conclusion\nC.\n### Justification\nJ.\n### Open points\nnone\n### Decision needed\n1. Ship now (recommended)\n2) Wait\n### After approval\nA.\n".into();
        let r = record(T, &t).unwrap();
        assert_eq!((r.cycle, r.rounds, r.subject.as_str()), (2, 1, "Test"));
        assert_eq!(r.options, ["Ship now (recommended)", "Wait"]);
        assert_eq!(r.user_requests, ["Use option 2."]);
        assert_eq!(r.projects, ["alpha", "beta"]);
        let text = record_text(&r);
        assert!(text.starts_with(
            "Discussed with: alpha, beta (1 round(s), cycle 2 after 1 request(s) from the user)"
        ));
        assert!(!text.contains("Open points"));
        assert!(record(T, &t[..4]).is_none());
    }

    #[test]
    fn strip_re_removes_every_prefix() {
        assert_eq!(strip_re("Re: Re:Re: Subject"), "Subject");
    }
}
