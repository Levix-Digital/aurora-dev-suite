//! What the Inbox shows (FEAT-017, spec 005 FR-002 to FR-004): every project's waiting
//! items in one list, the most urgent first. The items are the Portfolio's, so both views
//! always agree.

use serde::{Deserialize, Serialize};

use super::{CycleSummary, WaitingItem};

/// How soon a waiting item needs the user, the most urgent first (research R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Urgency {
    /// A blocking bug or a release candidate with failed tests.
    BlocksRelease,
    /// A release candidate to test or to release.
    Acceptance,
    /// A pull request that needs the user's action.
    Review,
    /// A bug without a severity or a release.
    Triage,
}

impl Urgency {
    pub const ALL: [Urgency; 4] = [
        Urgency::BlocksRelease,
        Urgency::Acceptance,
        Urgency::Review,
        Urgency::Triage,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Urgency::BlocksRelease => "Blocks a release",
            Urgency::Acceptance => "Waits for your acceptance",
            Urgency::Review => "Waits for your review",
            Urgency::Triage => "To triage",
        }
    }
}

/// One waiting item and the project it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxEntry {
    pub project_id: String,
    pub project_name: String,
    pub item: WaitingItem,
}

/// Every waiting item of the projects read so far, in the Inbox's order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inbox {
    pub entries: Vec<InboxEntry>,
}

impl Inbox {
    /// `projects` in the registry's order, each with its id, name and cycle summary; the
    /// entries by urgency, then project, then the item's order in its project (research R3).
    pub fn of<'a>(
        projects: impl IntoIterator<Item = (&'a str, &'a str, &'a CycleSummary)>,
    ) -> Self {
        let mut entries: Vec<InboxEntry> = projects
            .into_iter()
            .flat_map(|(id, name, summary)| {
                summary.waiting.iter().map(move |item| InboxEntry {
                    project_id: id.to_owned(),
                    project_name: name.to_owned(),
                    item: item.clone(),
                })
            })
            .collect();
        entries.sort_by_key(|entry| entry.item.urgency);
        Self { entries }
    }

    /// The entries of each urgency, in urgency order, without the empty ones.
    pub fn groups(&self) -> Vec<(Urgency, Vec<&InboxEntry>)> {
        Urgency::ALL
            .into_iter()
            .map(|urgency| {
                let entries = self
                    .entries
                    .iter()
                    .filter(|entry| entry.item.urgency == urgency)
                    .collect();
                (urgency, entries)
            })
            .filter(|(_, entries): &(Urgency, Vec<&InboxEntry>)| !entries.is_empty())
            .collect()
    }

    /// How many projects have at least one entry.
    pub fn project_count(&self) -> usize {
        let mut ids: Vec<&str> = self
            .entries
            .iter()
            .map(|entry| entry.project_id.as_str())
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CyclePhase, WaitingKind};

    fn item(key: &str, urgency: Urgency) -> WaitingItem {
        WaitingItem {
            kind: WaitingKind::BlockingBug,
            key: key.to_owned(),
            title: format!("title of {key}"),
            detail: None,
            urgency,
        }
    }

    fn summary(items: Vec<WaitingItem>) -> CycleSummary {
        CycleSummary {
            phase: CyclePhase::Idle,
            release: None,
            waiting: items,
            has_findings: false,
        }
    }

    fn keys(inbox: &Inbox) -> Vec<String> {
        inbox
            .entries
            .iter()
            .map(|e| format!("{}/{}", e.project_id, e.item.key))
            .collect()
    }

    #[test]
    fn entries_follow_urgency_then_project_then_item() {
        let a = summary(vec![
            item("a1", Urgency::Triage),
            item("a2", Urgency::Review),
            item("a3", Urgency::BlocksRelease),
        ]);
        let b = summary(vec![
            item("b1", Urgency::BlocksRelease),
            item("b2", Urgency::Acceptance),
            item("b3", Urgency::BlocksRelease),
        ]);
        let inbox = Inbox::of([("a", "A", &a), ("b", "B", &b)]);
        assert_eq!(
            keys(&inbox),
            ["a/a3", "b/b1", "b/b3", "b/b2", "a/a2", "a/a1"]
        );
        assert_eq!(inbox.entries[0].project_name, "A");
    }

    #[test]
    fn groups_skip_empty_levels() {
        let a = summary(vec![
            item("a1", Urgency::Triage),
            item("a2", Urgency::BlocksRelease),
        ]);
        let inbox = Inbox::of([("a", "A", &a)]);
        let groups: Vec<_> = inbox
            .groups()
            .into_iter()
            .map(|(urgency, entries)| (urgency, entries.len()))
            .collect();
        assert_eq!(groups, [(Urgency::BlocksRelease, 1), (Urgency::Triage, 1)]);
    }

    #[test]
    fn projects_without_items_are_not_counted() {
        let a = summary(vec![
            item("a1", Urgency::Review),
            item("a2", Urgency::Review),
        ]);
        let empty = summary(Vec::new());
        let inbox = Inbox::of([("a", "A", &a), ("e", "E", &empty)]);
        assert_eq!(inbox.project_count(), 1);
        assert_eq!(Inbox::of([("e", "E", &empty)]).project_count(), 0);
        assert!(Inbox::default().groups().is_empty());
    }

    #[test]
    fn every_urgency_has_a_label_in_order() {
        let mut sorted = Urgency::ALL;
        sorted.sort();
        assert_eq!(sorted, Urgency::ALL);
        for urgency in Urgency::ALL {
            assert!(!urgency.label().is_empty());
        }
    }

    #[cfg(feature = "fixtures")]
    #[test]
    fn the_samples_read_as_the_quickstart_says() {
        use crate::fixtures;

        let busy = CycleSummary::of(&fixtures::busy_cycle_state());
        let idle = CycleSummary::of(&fixtures::idle_cycle_state());
        let candidate = CycleSummary::of(&fixtures::candidate_cycle_state());
        let inbox = Inbox::of([
            ("gedo", "gedo", &busy),
            ("any-context", "any-context", &idle),
            ("ai-code-agent-skills", "ai-code-agent-skills", &candidate),
        ]);
        assert_eq!(
            keys(&inbox),
            [
                "gedo/bug:BUG-012",
                "ai-code-agent-skills/rc:v2.9.0-rc.1",
                "gedo/pr:41",
                "gedo/triage:BUG-015",
            ]
        );
        assert_eq!(inbox.project_count(), 2);
        let details: Vec<_> = inbox
            .entries
            .iter()
            .map(|e| e.item.detail.as_deref().unwrap_or_default())
            .collect();
        assert_eq!(
            details,
            [
                "Severity high",
                "4 not run, 1 failed",
                "2 threads open, CI green. 1 review thread for you",
                "no severity",
            ]
        );
    }
}
