//! What the Project view shows of a project's cycle state beyond the Portfolio's summary
//! (FEAT-015, spec 004 FR-005, FR-006): the release candidate's stage, the gates that wait
//! for the user, the next step in order and the progress of each plan.

use serde::{Deserialize, Serialize};

use super::{CandidateRun, CycleState, NextOption, NextStep, PlanProgress};

/// Where a release candidate stands (research R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CandidateStage {
    TestsToRun,
    FailuresToReview,
    ReadyForRelease,
    Released,
}

impl CandidateRun {
    /// The first stage that applies, in the order of data-model.md.
    pub const fn stage(&self) -> CandidateStage {
        if self.released {
            CandidateStage::Released
        } else if self.not_run > 0 {
            CandidateStage::TestsToRun
        } else if self.failed > 0 {
            CandidateStage::FailuresToReview
        } else {
            CandidateStage::ReadyForRelease
        }
    }
}

/// The kinds of decisions that wait for the user (research R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GateKind {
    Decision,
    CandidateAcceptance,
    Release,
}

impl GateKind {
    pub const fn label(self) -> &'static str {
        match self {
            GateKind::Decision => "Decision",
            GateKind::CandidateAcceptance => "Release candidate acceptance",
            GateKind::Release => "Release gate",
        }
    }
}

/// One decision that waits for the user in a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gate {
    pub kind: GateKind,
    pub detail: String,
}

impl Gate {
    /// The gates of `state`: the choice the protocol asks for, then the release
    /// candidate's acceptance or its release gate.
    pub fn all(state: &CycleState) -> Vec<Gate> {
        let decision = state.next.ask.then(|| {
            let recommended = state.next.recommended_option();
            Gate {
                kind: GateKind::Decision,
                detail: recommended.map_or_else(
                    || "Choose how to go on".to_owned(),
                    |option| format!("Choose how to go on: {}", option.title),
                ),
            }
        });
        let candidate = state
            .release_candidate
            .as_ref()
            .and_then(|rc| match rc.stage() {
                CandidateStage::TestsToRun | CandidateStage::FailuresToReview => Some(Gate {
                    kind: GateKind::CandidateAcceptance,
                    detail: format!("{}: {} not run, {} failed", rc.run, rc.not_run, rc.failed),
                }),
                CandidateStage::ReadyForRelease => Some(Gate {
                    kind: GateKind::Release,
                    detail: format!(
                        "{} passed its manual tests and waits for your approval",
                        rc.version
                    ),
                }),
                CandidateStage::Released => None,
            });
        decision.into_iter().chain(candidate).collect()
    }
}

impl NextStep {
    /// The option the protocol recommends, when it is among the options.
    pub fn recommended_option(&self) -> Option<&NextOption> {
        let key = self.recommended.as_deref()?;
        self.options.iter().find(|option| option.key == key)
    }

    /// The options with the recommended one first, each with whether it is recommended.
    pub fn ordered(&self) -> Vec<(&NextOption, bool)> {
        let recommended = self.recommended_option().map(|option| option.key.as_str());
        let is_recommended = |option: &NextOption| Some(option.key.as_str()) == recommended;
        let first = self.options.iter().filter(|o| is_recommended(o));
        let rest = self.options.iter().filter(|o| !is_recommended(o));
        first
            .map(|option| (option, true))
            .chain(rest.map(|option| (option, false)))
            .collect()
    }
}

impl PlanProgress {
    /// Milestones done out of the total, from 0 to 1; 0 without milestones.
    pub fn fraction(&self) -> f64 {
        if self.milestones_total == 0 {
            0.0
        } else {
            (f64::from(self.milestones_done) / f64::from(self.milestones_total)).min(1.0)
        }
    }

    /// For example "3 of 5 milestones".
    pub fn progress_label(&self) -> String {
        match self.milestones_total {
            0 => "No milestones".to_owned(),
            1 => format!("{} of 1 milestone", self.milestones_done),
            total => format!("{} of {total} milestones", self.milestones_done),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(not_run: u32, failed: u32, released: bool) -> CandidateRun {
        CandidateRun {
            run: "v1.0.0-rc.1".to_owned(),
            version: "v1.0.0".to_owned(),
            not_run,
            failed,
            released,
        }
    }

    fn option(key: &str) -> NextOption {
        NextOption {
            key: key.to_owned(),
            title: format!("title of {key}"),
            reason: None,
        }
    }

    fn next(ask: bool, recommended: Option<&str>, keys: &[&str]) -> NextStep {
        NextStep {
            ask,
            recommended: recommended.map(str::to_owned),
            options: keys.iter().map(|key| option(key)).collect(),
        }
    }

    fn state(next: NextStep, release_candidate: Option<CandidateRun>) -> CycleState {
        CycleState {
            summary: String::new(),
            next,
            release: None,
            plans: Vec::new(),
            release_candidate,
            pull_requests: Vec::new(),
            blocking_bugs: Vec::new(),
            untriaged_bugs: Vec::new(),
            findings: Vec::new(),
            read_at: "2026-10-30T12:00:00Z".to_owned(),
        }
    }

    fn plan(done: u32, total: u32) -> PlanProgress {
        PlanProgress {
            id: "p".to_owned(),
            title: "P".to_owned(),
            status: "in_progress".to_owned(),
            milestones_done: done,
            milestones_total: total,
        }
    }

    fn kinds(state: &CycleState) -> Vec<GateKind> {
        Gate::all(state).into_iter().map(|gate| gate.kind).collect()
    }

    #[test]
    fn the_stage_follows_the_precedence() {
        assert_eq!(candidate(1, 1, true).stage(), CandidateStage::Released);
        assert_eq!(candidate(1, 1, false).stage(), CandidateStage::TestsToRun);
        assert_eq!(
            candidate(0, 1, false).stage(),
            CandidateStage::FailuresToReview
        );
        assert_eq!(
            candidate(0, 0, false).stage(),
            CandidateStage::ReadyForRelease
        );
    }

    #[test]
    fn a_candidate_is_one_gate_until_released() {
        let none = next(false, None, &[]);
        let gates = Gate::all(&state(none.clone(), Some(candidate(4, 1, false))));
        assert_eq!(gates.len(), 1);
        assert_eq!(gates[0].kind, GateKind::CandidateAcceptance);
        assert_eq!(gates[0].detail, "v1.0.0-rc.1: 4 not run, 1 failed");

        let gates = Gate::all(&state(none.clone(), Some(candidate(0, 0, false))));
        assert_eq!(gates[0].kind, GateKind::Release);
        assert_eq!(
            gates[0].detail,
            "v1.0.0 passed its manual tests and waits for your approval"
        );

        assert!(Gate::all(&state(none, Some(candidate(0, 0, true)))).is_empty());
    }

    #[test]
    fn a_decision_names_the_recommended_option() {
        let asked = state(next(true, Some("bugs"), &["pr:1", "bugs"]), None);
        let gates = Gate::all(&asked);
        assert_eq!(kinds(&asked), [GateKind::Decision]);
        assert_eq!(gates[0].detail, "Choose how to go on: title of bugs");

        let unknown = state(next(true, Some("gone"), &["bugs"]), None);
        assert_eq!(Gate::all(&unknown)[0].detail, "Choose how to go on");

        let not_asked = state(next(false, Some("bugs"), &["bugs"]), None);
        assert!(Gate::all(&not_asked).is_empty());
    }

    #[test]
    fn the_decision_comes_before_the_candidate() {
        let both = state(next(true, None, &[]), Some(candidate(0, 2, false)));
        assert_eq!(
            kinds(&both),
            [GateKind::Decision, GateKind::CandidateAcceptance]
        );
    }

    #[test]
    fn the_recommended_option_comes_first() {
        let step = next(true, Some("c"), &["a", "b", "c"]);
        let order: Vec<_> = step
            .ordered()
            .into_iter()
            .map(|(option, recommended)| (option.key.as_str(), recommended))
            .collect();
        assert_eq!(order, [("c", true), ("a", false), ("b", false)]);

        let unknown = next(true, Some("z"), &["a", "b"]);
        assert!(
            unknown
                .ordered()
                .iter()
                .all(|(_, recommended)| !recommended)
        );
        assert_eq!(unknown.ordered()[0].0.key, "a");
    }

    #[test]
    fn plan_progress_reads_as_a_fraction() {
        assert!((plan(3, 5).fraction() - 0.6).abs() < f64::EPSILON);
        assert!(plan(0, 0).fraction().abs() < f64::EPSILON);
        assert!((plan(7, 5).fraction() - 1.0).abs() < f64::EPSILON);
        assert_eq!(plan(3, 5).progress_label(), "3 of 5 milestones");
        assert_eq!(plan(0, 1).progress_label(), "0 of 1 milestone");
        assert_eq!(plan(0, 0).progress_label(), "No milestones");
    }

    #[test]
    fn every_gate_has_a_label() {
        for kind in [
            GateKind::Decision,
            GateKind::CandidateAcceptance,
            GateKind::Release,
        ] {
            assert!(!kind.label().is_empty());
        }
    }

    #[cfg(feature = "fixtures")]
    #[test]
    fn the_samples_read_as_the_quickstart_says() {
        use crate::fixtures;

        let busy = fixtures::busy_cycle_state();
        assert_eq!(kinds(&busy), [GateKind::Decision]);
        assert_eq!(
            Gate::all(&busy)[0].detail,
            "Choose how to go on: Fix the blocking bug first"
        );
        assert_eq!(busy.next.ordered()[0].0.key, "bugs");
        assert_eq!(busy.plans[0].progress_label(), "3 of 5 milestones");

        assert!(Gate::all(&fixtures::idle_cycle_state()).is_empty());

        let candidate = fixtures::candidate_cycle_state();
        assert_eq!(
            kinds(&candidate),
            [GateKind::Decision, GateKind::CandidateAcceptance]
        );
    }
}
