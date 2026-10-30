//! What the Portfolio shows of a project's cycle state (FEAT-014, spec 003 FR-002 to
//! FR-005): its phase and what waits for the user. The Project view and the Inbox use the
//! same rules.

use serde::{Deserialize, Serialize};

use super::{CandidateRun, CandidateStage, CurrentRelease, CycleState, Urgency};

/// Where a project stands in its development cycle, derived from its cycle state
/// (research R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CyclePhase {
    ReleaseCandidate,
    Implementation,
    Review,
    Planning,
    Idle,
}

impl CyclePhase {
    /// The first phase that applies, in the order of data-model.md.
    pub fn of(state: &CycleState) -> Self {
        if state
            .release_candidate
            .as_ref()
            .is_some_and(|rc| !rc.released)
        {
            CyclePhase::ReleaseCandidate
        } else if !state.plans.is_empty() {
            CyclePhase::Implementation
        } else if !state.pull_requests.is_empty() {
            CyclePhase::Review
        } else if state
            .release
            .as_ref()
            .is_some_and(|release| !release.open_items.is_empty())
        {
            CyclePhase::Planning
        } else {
            CyclePhase::Idle
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            CyclePhase::ReleaseCandidate => "Release candidate",
            CyclePhase::Implementation => "Implementation",
            CyclePhase::Review => "Review",
            CyclePhase::Planning => "Planning",
            CyclePhase::Idle => "Idle",
        }
    }
}

/// The kinds of things that wait for the user (research R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WaitingKind {
    ReleaseCandidate,
    PullRequest,
    BlockingBug,
    UntriagedBug,
}

impl WaitingKind {
    pub const ALL: [WaitingKind; 4] = [
        WaitingKind::ReleaseCandidate,
        WaitingKind::PullRequest,
        WaitingKind::BlockingBug,
        WaitingKind::UntriagedBug,
    ];

    /// One item of this kind, for example "Blocking bug".
    pub const fn name(self) -> &'static str {
        match self {
            WaitingKind::ReleaseCandidate => "Release candidate",
            WaitingKind::PullRequest => "Pull request",
            WaitingKind::BlockingBug => "Blocking bug",
            WaitingKind::UntriagedBug => "Untriaged bug",
        }
    }

    /// `count` items of this kind, for example "2 blocking bugs".
    pub fn label(self, count: usize) -> String {
        let (one, many) = match self {
            WaitingKind::ReleaseCandidate => ("release candidate", "release candidates"),
            WaitingKind::PullRequest => ("pull request", "pull requests"),
            WaitingKind::BlockingBug => ("blocking bug", "blocking bugs"),
            WaitingKind::UntriagedBug => ("untriaged bug", "untriaged bugs"),
        };
        format!("{count} {}", if count == 1 { one } else { many })
    }
}

/// One thing that waits for the user in a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitingItem {
    pub kind: WaitingKind,
    /// Stable within the project, for example `rc:<run>`, `pr:<number>`, `bug:<id>`.
    pub key: String,
    pub title: String,
    /// More about the item, for example a bug's severity or a candidate's test counts.
    pub detail: Option<String>,
    /// How soon it needs the user (spec 005 FR-003).
    pub urgency: Urgency,
}

impl WaitingItem {
    /// Everything that waits for the user in `state`: the release candidate not released,
    /// each pull request that needs action, each blocking bug and each untriaged bug.
    pub fn all(state: &CycleState) -> Vec<WaitingItem> {
        let candidate = state
            .release_candidate
            .as_ref()
            .filter(|rc| !rc.released)
            .map(|rc| WaitingItem {
                kind: WaitingKind::ReleaseCandidate,
                key: format!("rc:{}", rc.run),
                title: candidate_title(rc),
                detail: Some(candidate_detail(rc)),
                urgency: match rc.stage() {
                    CandidateStage::FailuresToReview => Urgency::BlocksRelease,
                    _ => Urgency::Acceptance,
                },
            });
        let pull_requests = state
            .pull_requests
            .iter()
            .filter(|pr| pr.needs_action)
            .map(|pr| WaitingItem {
                kind: WaitingKind::PullRequest,
                key: format!("pr:{}", pr.number),
                title: format!("#{} {}", pr.number, pr.title),
                detail: pull_request_detail(&pr.summary, pr.threads_for_user),
                urgency: Urgency::Review,
            });
        let blocking = state.blocking_bugs.iter().map(|bug| WaitingItem {
            kind: WaitingKind::BlockingBug,
            key: format!("bug:{}", bug.id),
            title: format!("{} {}", bug.id, bug.title),
            detail: bug.rank.as_ref().map(|rank| format!("Severity {rank}")),
            urgency: Urgency::BlocksRelease,
        });
        let untriaged = state.untriaged_bugs.iter().map(|bug| WaitingItem {
            kind: WaitingKind::UntriagedBug,
            key: format!("triage:{}", bug.item.id),
            title: format!("{} {}", bug.item.id, bug.item.title),
            detail: Some(bug.reason.clone()).filter(|reason| !reason.is_empty()),
            urgency: Urgency::Triage,
        });
        candidate
            .into_iter()
            .chain(pull_requests)
            .chain(blocking)
            .chain(untriaged)
            .collect()
    }
}

fn candidate_title(rc: &CandidateRun) -> String {
    match rc.stage() {
        CandidateStage::TestsToRun => format!("Run the manual tests of {}", rc.run),
        CandidateStage::FailuresToReview => format!("Look at the failed tests of {}", rc.run),
        CandidateStage::ReadyForRelease | CandidateStage::Released => {
            format!("Approve the release of {}", rc.version)
        }
    }
}

fn candidate_detail(rc: &CandidateRun) -> String {
    match rc.stage() {
        CandidateStage::ReadyForRelease | CandidateStage::Released => {
            "Passed its manual tests".to_owned()
        }
        _ => format!("{} not run, {} failed", rc.not_run, rc.failed),
    }
}

/// The pull request's summary, then the review threads that wait for the user.
fn pull_request_detail(summary: &str, threads_for_user: u32) -> Option<String> {
    let threads = match threads_for_user {
        0 => None,
        1 => Some("1 review thread for you".to_owned()),
        n => Some(format!("{n} review threads for you")),
    };
    match (summary.is_empty(), threads) {
        (true, threads) => threads,
        (false, None) => Some(summary.to_owned()),
        (false, Some(threads)) => Some(format!("{summary}. {threads}")),
    }
}

/// What the Portfolio shows of one project's cycle state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleSummary {
    pub phase: CyclePhase,
    pub release: Option<CurrentRelease>,
    pub waiting: Vec<WaitingItem>,
    /// The engine could not read part of the project.
    pub has_findings: bool,
}

impl CycleSummary {
    pub fn of(state: &CycleState) -> Self {
        Self {
            phase: CyclePhase::of(state),
            release: state.release.clone(),
            waiting: WaitingItem::all(state),
            has_findings: !state.findings.is_empty(),
        }
    }

    /// How many items of each kind wait, in the order of `WaitingKind::ALL`, without the
    /// kinds that have none.
    pub fn waiting_by_kind(&self) -> Vec<(WaitingKind, usize)> {
        WaitingKind::ALL
            .into_iter()
            .map(|kind| {
                let count = self.waiting.iter().filter(|item| item.kind == kind).count();
                (kind, count)
            })
            .filter(|(_, count)| *count > 0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ItemRef, NextStep, PlanProgress, PullRequestState, UntriagedBug};

    fn empty() -> CycleState {
        CycleState {
            summary: String::new(),
            next: NextStep {
                ask: false,
                recommended: None,
                options: Vec::new(),
            },
            release: None,
            plans: Vec::new(),
            release_candidate: None,
            pull_requests: Vec::new(),
            blocking_bugs: Vec::new(),
            untriaged_bugs: Vec::new(),
            findings: Vec::new(),
            read_at: "2026-10-30T12:00:00Z".to_owned(),
        }
    }

    fn item(id: &str) -> ItemRef {
        ItemRef {
            id: id.to_owned(),
            rank: None,
            title: format!("title of {id}"),
        }
    }

    fn candidate(not_run: u32, failed: u32, released: bool) -> CandidateRun {
        CandidateRun {
            run: "v1.0.0-rc.1".to_owned(),
            version: "v1.0.0".to_owned(),
            not_run,
            failed,
            released,
        }
    }

    fn pull_request(number: u64, needs_action: bool) -> PullRequestState {
        PullRequestState {
            number,
            title: "feat: something".to_owned(),
            summary: String::new(),
            needs_action,
            threads_for_user: 0,
        }
    }

    fn plan() -> PlanProgress {
        PlanProgress {
            id: "p".to_owned(),
            title: "P".to_owned(),
            status: "in_progress".to_owned(),
            milestones_done: 1,
            milestones_total: 2,
        }
    }

    fn release(open: &[&str]) -> CurrentRelease {
        CurrentRelease {
            version: "v1.0.0".to_owned(),
            name: "One".to_owned(),
            open_items: open.iter().map(|id| item(id)).collect(),
        }
    }

    #[test]
    fn the_phase_follows_the_precedence() {
        let mut state = empty();
        assert_eq!(CyclePhase::of(&state), CyclePhase::Idle);
        state.release = Some(release(&[]));
        assert_eq!(CyclePhase::of(&state), CyclePhase::Idle);
        state.release = Some(release(&["FEAT-001"]));
        assert_eq!(CyclePhase::of(&state), CyclePhase::Planning);
        state.pull_requests.push(pull_request(1, false));
        assert_eq!(CyclePhase::of(&state), CyclePhase::Review);
        state.plans.push(plan());
        assert_eq!(CyclePhase::of(&state), CyclePhase::Implementation);
        state.release_candidate = Some(candidate(0, 0, true));
        assert_eq!(CyclePhase::of(&state), CyclePhase::Implementation);
        state.release_candidate = Some(candidate(1, 0, false));
        assert_eq!(CyclePhase::of(&state), CyclePhase::ReleaseCandidate);
    }

    #[test]
    fn every_phase_has_a_label() {
        for phase in [
            CyclePhase::ReleaseCandidate,
            CyclePhase::Implementation,
            CyclePhase::Review,
            CyclePhase::Planning,
            CyclePhase::Idle,
        ] {
            assert!(!phase.label().is_empty());
        }
    }

    #[test]
    fn a_candidate_waits_until_released() {
        let mut state = empty();
        let title = |state: &CycleState| WaitingItem::all(state).first().map(|i| i.title.clone());
        state.release_candidate = Some(candidate(2, 1, false));
        assert_eq!(
            title(&state).as_deref(),
            Some("Run the manual tests of v1.0.0-rc.1")
        );
        state.release_candidate = Some(candidate(0, 1, false));
        assert_eq!(
            title(&state).as_deref(),
            Some("Look at the failed tests of v1.0.0-rc.1")
        );
        state.release_candidate = Some(candidate(0, 0, false));
        assert_eq!(
            title(&state).as_deref(),
            Some("Approve the release of v1.0.0")
        );
        state.release_candidate = Some(candidate(0, 0, true));
        assert!(WaitingItem::all(&state).is_empty());
    }

    #[test]
    fn only_pull_requests_that_need_action_wait() {
        let mut state = empty();
        state.pull_requests = vec![pull_request(7, false), pull_request(8, true)];
        let waiting = WaitingItem::all(&state);
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0].key, "pr:8");
        assert_eq!(waiting[0].title, "#8 feat: something");
    }

    #[test]
    fn bugs_wait_with_their_ids() {
        let mut state = empty();
        state.blocking_bugs = vec![item("BUG-1"), item("BUG-2")];
        state.untriaged_bugs = vec![UntriagedBug {
            item: item("BUG-3"),
            reason: "no severity".to_owned(),
        }];
        let keys: Vec<_> = WaitingItem::all(&state)
            .into_iter()
            .map(|i| i.key)
            .collect();
        assert_eq!(keys, ["bug:BUG-1", "bug:BUG-2", "triage:BUG-3"]);
        let summary = CycleSummary::of(&state);
        assert_eq!(
            summary.waiting_by_kind(),
            [
                (WaitingKind::BlockingBug, 2),
                (WaitingKind::UntriagedBug, 1)
            ]
        );
    }

    #[test]
    fn each_item_has_its_urgency_and_detail() {
        let urgency_and_detail = |state: &CycleState| -> Vec<(Urgency, Option<String>)> {
            WaitingItem::all(state)
                .into_iter()
                .map(|i| (i.urgency, i.detail))
                .collect()
        };
        let mut state = empty();
        state.release_candidate = Some(candidate(2, 1, false));
        assert_eq!(
            urgency_and_detail(&state),
            [(Urgency::Acceptance, Some("2 not run, 1 failed".to_owned()))]
        );
        state.release_candidate = Some(candidate(0, 1, false));
        assert_eq!(
            urgency_and_detail(&state),
            [(
                Urgency::BlocksRelease,
                Some("0 not run, 1 failed".to_owned())
            )]
        );
        state.release_candidate = Some(candidate(0, 0, false));
        assert_eq!(
            urgency_and_detail(&state),
            [(
                Urgency::Acceptance,
                Some("Passed its manual tests".to_owned())
            )]
        );

        let mut state = empty();
        let mut ranked = item("BUG-1");
        ranked.rank = Some("high".to_owned());
        state.blocking_bugs = vec![ranked, item("BUG-2")];
        state.untriaged_bugs = vec![UntriagedBug {
            item: item("BUG-3"),
            reason: "no severity".to_owned(),
        }];
        assert_eq!(
            urgency_and_detail(&state),
            [
                (Urgency::BlocksRelease, Some("Severity high".to_owned())),
                (Urgency::BlocksRelease, None),
                (Urgency::Triage, Some("no severity".to_owned())),
            ]
        );
    }

    #[test]
    fn a_pull_request_details_its_summary_and_threads() {
        let detail = |summary: &str, threads: u32| {
            let mut state = empty();
            let mut pr = pull_request(1, true);
            pr.summary = summary.to_owned();
            pr.threads_for_user = threads;
            state.pull_requests = vec![pr];
            let item = WaitingItem::all(&state).remove(0);
            assert_eq!(item.urgency, Urgency::Review);
            item.detail
        };
        assert_eq!(detail("", 0), None);
        assert_eq!(detail("CI green", 0).as_deref(), Some("CI green"));
        assert_eq!(detail("", 1).as_deref(), Some("1 review thread for you"));
        assert_eq!(
            detail("CI green", 2).as_deref(),
            Some("CI green. 2 review threads for you")
        );
    }

    #[test]
    fn kind_labels_count_in_the_plural() {
        assert_eq!(WaitingKind::BlockingBug.label(1), "1 blocking bug");
        assert_eq!(WaitingKind::PullRequest.label(2), "2 pull requests");
        assert_eq!(WaitingKind::UntriagedBug.name(), "Untriaged bug");
    }

    #[test]
    fn the_summary_flags_findings() {
        let mut state = empty();
        assert!(!CycleSummary::of(&state).has_findings);
        state.findings.push("pull requests: offline".to_owned());
        assert!(CycleSummary::of(&state).has_findings);
    }

    #[cfg(feature = "fixtures")]
    #[test]
    fn the_samples_read_as_the_quickstart_says() {
        use crate::fixtures;

        let busy = CycleSummary::of(&fixtures::busy_cycle_state());
        assert_eq!(busy.phase, CyclePhase::Implementation);
        assert_eq!(busy.waiting.len(), 3);
        assert_eq!(busy.release.map(|r| r.version).as_deref(), Some("v0.4.0"));

        let idle = CycleSummary::of(&fixtures::idle_cycle_state());
        assert_eq!(idle.phase, CyclePhase::Idle);
        assert!(idle.waiting.is_empty());

        let candidate = CycleSummary::of(&fixtures::candidate_cycle_state());
        assert_eq!(candidate.phase, CyclePhase::ReleaseCandidate);
        assert_eq!(candidate.waiting.len(), 1);
        assert!(candidate.has_findings);
    }
}
