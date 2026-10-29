//! Which engines are installed and what they enable (US3, FR-011 to FR-016).

use crate::capabilities;
use crate::domain::{EngineKind, EngineReport, EnginesOverview, Settings};
use crate::ports::EngineProbe;

/// Probes one engine with the user's settings for it.
pub fn engine_status(
    probe: &dyn EngineProbe,
    kind: EngineKind,
    settings: &Settings,
) -> EngineReport {
    probe.probe(kind, settings.engines.get(kind))
}

/// The engines in a stable order, and every feature's availability.
pub fn overview(mut reports: Vec<EngineReport>) -> EnginesOverview {
    reports.sort_by_key(|report| report.kind);
    let features = capabilities::all_availability(&reports);
    EnginesOverview { reports, features }
}

#[cfg(all(test, feature = "fixtures"))]
mod tests {
    use super::*;
    use crate::domain::{AvailabilityState, EngineStatus, Feature};
    use crate::fixtures::FixtureProbe;

    #[test]
    fn overview_sorts_engines_and_evaluates_every_feature() {
        let probe = FixtureProbe::sample();
        let settings = Settings::default();
        let reports = vec![
            engine_status(&probe, EngineKind::AnyContext, &settings),
            engine_status(&probe, EngineKind::DevCycle, &settings),
            engine_status(&probe, EngineKind::Gedo, &settings),
        ];
        let overview = overview(reports);
        let kinds: Vec<_> = overview.reports.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, EngineKind::ALL.to_vec());
        assert_eq!(overview.features.len(), Feature::ALL.len());
    }

    #[test]
    fn sample_engines_cover_every_status() {
        let probe = FixtureProbe::sample();
        let settings = Settings::default();
        let statuses: Vec<EngineStatus> = EngineKind::ALL
            .iter()
            .map(|kind| engine_status(&probe, *kind, &settings).status)
            .collect();
        assert!(statuses.contains(&EngineStatus::Ready));
        assert!(statuses.contains(&EngineStatus::InstalledWithoutContract));
        assert!(statuses.contains(&EngineStatus::Missing));
    }

    #[test]
    fn ready_gedo_sample_enables_agents_in_a_later_release() {
        let probe = FixtureProbe::sample();
        let report = engine_status(&probe, EngineKind::Gedo, &Settings::default());
        let overview = overview(vec![report]);
        let agents = overview
            .features
            .iter()
            .find(|f| f.feature == Feature::Agents)
            .expect("agents listed");
        assert!(matches!(
            agents.state,
            AvailabilityState::NotInThisRelease { .. }
        ));
    }
}
