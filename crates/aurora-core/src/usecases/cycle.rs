//! A project's development cycle state (FEAT-016, spec 002 FR-004 to FR-006).

use crate::capabilities;
use crate::domain::{CycleError, CycleState, EngineReport, Presence, Project, Settings};
use crate::ports::CycleSource;

/// Reads `project`'s cycle state through `source`, but only when `dcp` (its latest report)
/// is ready with the cycle capability and the project's folder is present; otherwise
/// nothing runs.
pub fn cycle_state(
    source: &dyn CycleSource,
    dcp: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
) -> Result<CycleState, CycleError> {
    let dcp = capabilities::usable_dcp(dcp).map_err(|hint| CycleError::Unavailable { hint })?;
    if project.presence == Presence::Missing {
        return Err(CycleError::ProjectMissing {
            path: project.root.clone(),
        });
    }
    source.cycle_state(dcp, settings.engines.get(dcp.kind), &project.root)
}

#[cfg(all(test, feature = "fixtures"))]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::domain::{
        Capability, EngineKind, EngineSettings, EngineStatus, ProjectId, ProjectName,
    };
    use crate::fixtures;

    /// Counts its calls and answers the busy sample.
    #[derive(Debug, Default)]
    struct CountingSource {
        calls: AtomicUsize,
    }

    impl CycleSource for CountingSource {
        fn cycle_state(
            &self,
            _engine: &EngineReport,
            _settings: &EngineSettings,
            root: &Path,
        ) -> Result<CycleState, CycleError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if root.ends_with("broken") {
                return Err(CycleError::Failed {
                    exit_code: Some(1),
                    stderr: "boom".into(),
                });
            }
            Ok(fixtures::busy_cycle_state())
        }
    }

    fn dcp(status: EngineStatus, capabilities: &[(&str, u32)]) -> EngineReport {
        EngineReport {
            kind: EngineKind::DevCycle,
            status,
            location: Some(PathBuf::from("/usr/bin/dcp")),
            version: Some("2.12.0".into()),
            capabilities: capabilities
                .iter()
                .map(|(name, major)| Capability::new(name, *major))
                .collect(),
            last_error: None,
            checked_at: fixtures::CHECKED_AT.into(),
        }
    }

    fn ready() -> EngineReport {
        dcp(EngineStatus::Ready, &[("diagnose", 1)])
    }

    fn project(root: &str, presence: Presence) -> Project {
        Project {
            id: ProjectId("p1".into()),
            name: ProjectName::parse("sample").expect("valid name"),
            root: PathBuf::from(root),
            added_at: fixtures::CHECKED_AT.into(),
            presence,
        }
    }

    fn ask(
        source: &CountingSource,
        report: Option<&EngineReport>,
        project: &Project,
    ) -> Result<CycleState, CycleError> {
        cycle_state(source, report, &Settings::default(), project)
    }

    #[test]
    fn without_a_usable_dcp_nothing_runs() {
        let source = CountingSource::default();
        let present = project("/repos/gedo", Presence::Present);
        let reports = [
            None,
            Some(EngineReport::missing(
                EngineKind::DevCycle,
                fixtures::CHECKED_AT,
            )),
            Some(dcp(EngineStatus::InstalledWithoutContract, &[])),
            Some(dcp(
                EngineStatus::Degraded {
                    reason: "timed out".into(),
                },
                &[("diagnose", 1)],
            )),
            Some(dcp(EngineStatus::Ready, &[("metrics", 1)])),
            Some(dcp(EngineStatus::Ready, &[("diagnose", 2)])),
        ];
        for report in &reports {
            let result = ask(&source, report.as_ref(), &present);
            assert!(
                matches!(result, Err(CycleError::Unavailable { .. })),
                "{report:?} gave {result:?}"
            );
        }
        assert_eq!(source.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn the_hint_says_what_to_do() {
        let source = CountingSource::default();
        let present = project("/repos/gedo", Presence::Present);
        let Err(CycleError::Unavailable { hint }) = ask(&source, None, &present) else {
            panic!("expected unavailable");
        };
        assert!(hint.contains("dcp"), "{hint}");
    }

    #[test]
    fn a_missing_project_is_not_asked_for() {
        let source = CountingSource::default();
        let gone = project("/repos/gone", Presence::Missing);
        assert_eq!(
            ask(&source, Some(&ready()), &gone),
            Err(CycleError::ProjectMissing {
                path: PathBuf::from("/repos/gone")
            })
        );
        assert_eq!(source.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_ready_dcp_answers_through_the_source() {
        let source = CountingSource::default();
        let present = project("/repos/gedo", Presence::Present);
        assert_eq!(
            ask(&source, Some(&ready()), &present),
            Ok(fixtures::busy_cycle_state())
        );
        let broken = project("/repos/broken", Presence::Present);
        assert!(matches!(
            ask(&source, Some(&ready()), &broken),
            Err(CycleError::Failed { .. })
        ));
        assert_eq!(source.calls.load(Ordering::SeqCst), 2);
    }
}
