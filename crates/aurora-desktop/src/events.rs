//! Probing the three engines in parallel and reporting each result as it arrives
//! (research R4, SC-003).

use aurora_core::domain::{EngineKind, EngineReport, EnginesOverview, Settings};
use aurora_core::ports::EngineProbe;
use aurora_core::usecases::engines;

pub const ENGINE_REPORT: &str = "engine-report";
pub const ENGINES_READY: &str = "engines-ready";

/// Probes every engine on its own thread, calls `on_report` as each one finishes, and
/// returns the overview once all have. A hanging engine delays only its own report.
pub fn probe_all(
    probe: &dyn EngineProbe,
    settings: &Settings,
    on_report: &(dyn Fn(&EngineReport) + Sync),
) -> EnginesOverview {
    let reports = std::thread::scope(|scope| {
        let handles: Vec<_> = EngineKind::ALL
            .into_iter()
            .map(|kind| {
                scope.spawn(move || {
                    let report = engines::engine_status(probe, kind, settings);
                    on_report(&report);
                    report
                })
            })
            .collect();
        EngineKind::ALL
            .into_iter()
            .zip(handles)
            .map(|(kind, handle)| {
                handle.join().unwrap_or_else(|_| {
                    tracing::error!(engine = kind.command(), "engine probe panicked");
                    EngineReport::missing(kind, "")
                })
            })
            .collect()
    });
    engines::overview(reports)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use aurora_core::fixtures::FixtureProbe;

    use super::*;

    #[test]
    fn every_engine_is_reported_once_and_the_overview_covers_all() {
        let seen = Mutex::new(Vec::new());
        let overview = probe_all(&FixtureProbe::sample(), &Settings::default(), &|report| {
            seen.lock().expect("lock").push(report.kind);
        });
        let mut seen = seen.into_inner().expect("lock");
        seen.sort();
        assert_eq!(seen, EngineKind::ALL.to_vec());
        assert_eq!(overview.reports.len(), 3);
        assert!(!overview.features.is_empty());
    }
}
