//! A project's Dev Cycle settings: read, preview and apply (FEAT-027, spec 007 FR-002 to
//! FR-006).

use crate::capabilities;
use crate::domain::{
    ConfigChange, ConfigError, ConfigOutcome, EngineReport, Feature, Presence, Project,
    ProjectConfig, Settings,
};
use crate::ports::ConfigSource;

/// Reads `project`'s settings through `source`, only when `dcp` (its latest report) has the
/// `config` capability and the project's folder is present; otherwise nothing runs.
pub fn read(
    source: &dyn ConfigSource,
    dcp: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
) -> Result<ProjectConfig, ConfigError> {
    let dcp = usable(dcp, project)?;
    source.read(dcp, settings.engines.get(dcp.kind), &project.root)
}

/// Shows what `change` would do; nothing is written.
pub fn preview(
    source: &dyn ConfigSource,
    dcp: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
    change: &ConfigChange,
) -> Result<ConfigOutcome, ConfigError> {
    set(source, dcp, settings, project, change, true)
}

/// Writes `change` through `dcp`, which refuses it when the file changed since it was read.
pub fn apply(
    source: &dyn ConfigSource,
    dcp: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
    change: &ConfigChange,
) -> Result<ConfigOutcome, ConfigError> {
    let outcome = set(source, dcp, settings, project, change, false)?;
    if outcome.applied {
        Ok(outcome)
    } else {
        Err(ConfigError::NothingToChange)
    }
}

fn set(
    source: &dyn ConfigSource,
    dcp: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
    change: &ConfigChange,
    dry_run: bool,
) -> Result<ConfigOutcome, ConfigError> {
    let dcp = usable(dcp, project)?;
    if change.values.is_empty() {
        return Err(ConfigError::NothingToChange);
    }
    source.set(
        dcp,
        settings.engines.get(dcp.kind),
        &project.root,
        change,
        dry_run,
    )
}

fn usable<'a>(
    dcp: Option<&'a EngineReport>,
    project: &Project,
) -> Result<&'a EngineReport, ConfigError> {
    let dcp = capabilities::usable_dcp_for(Feature::ProjectSettings, dcp)
        .map_err(|hint| ConfigError::Unavailable { hint })?;
    if project.presence == Presence::Missing {
        return Err(ConfigError::ProjectMissing {
            path: project.root.clone(),
        });
    }
    Ok(dcp)
}

#[cfg(all(test, feature = "fixtures"))]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use super::*;
    use crate::domain::{
        ConfigValue, EngineKind, EngineSettings, EngineStatus, KeyValue, ProjectId, ProjectName,
    };
    use crate::fixtures;

    /// Records each call and answers from the sample settings.
    #[derive(Debug, Default)]
    struct RecordingSource {
        calls: Mutex<Vec<String>>,
    }

    impl ConfigSource for RecordingSource {
        fn read(
            &self,
            _engine: &EngineReport,
            _settings: &EngineSettings,
            _root: &Path,
        ) -> Result<ProjectConfig, ConfigError> {
            self.calls.lock().expect("lock").push("read".into());
            Ok(fixtures::sample_project_config())
        }

        fn set(
            &self,
            _engine: &EngineReport,
            _settings: &EngineSettings,
            _root: &Path,
            change: &ConfigChange,
            dry_run: bool,
        ) -> Result<ConfigOutcome, ConfigError> {
            self.calls.lock().expect("lock").push(format!(
                "set dry_run={dry_run} keys={}",
                change.values.len()
            ));
            Ok(ConfigOutcome {
                applied: !dry_run && change.revision != "unchanged",
                revision: "r2".into(),
                changes: vec![],
                diff: String::new(),
            })
        }
    }

    fn project(presence: Presence) -> Project {
        Project {
            id: ProjectId("p1".into()),
            name: ProjectName::parse("gedo").expect("valid name"),
            root: PathBuf::from("/repos/gedo"),
            added_at: fixtures::CHECKED_AT.into(),
            presence,
        }
    }

    fn change(revision: &str) -> ConfigChange {
        ConfigChange {
            revision: revision.into(),
            values: vec![KeyValue {
                key: "devops.branching".into(),
                value: Some(ConfigValue::Text("integration".into())),
            }],
        }
    }

    fn with_config() -> EngineReport {
        fixtures::ready_report(EngineKind::DevCycle, &["diagnose", "config"])
    }

    #[test]
    fn without_the_config_capability_nothing_runs() {
        let source = RecordingSource::default();
        let present = project(Presence::Present);
        let reports = [
            None,
            Some(fixtures::ready_report(EngineKind::DevCycle, &["diagnose"])),
            Some(fixtures::degraded_report(EngineKind::DevCycle, "timed out")),
            Some(EngineReport::missing(
                EngineKind::DevCycle,
                fixtures::CHECKED_AT,
            )),
        ];
        for report in &reports {
            let settings = Settings::default();
            assert!(matches!(
                read(&source, report.as_ref(), &settings, &present),
                Err(ConfigError::Unavailable { .. })
            ));
            assert!(matches!(
                apply(&source, report.as_ref(), &settings, &present, &change("r1")),
                Err(ConfigError::Unavailable { .. })
            ));
        }
        assert!(source.calls.lock().expect("lock").is_empty());
        assert!(
            reports
                .iter()
                .flatten()
                .any(|r| r.status == EngineStatus::Ready)
        );
    }

    #[test]
    fn a_missing_project_and_an_empty_change_run_nothing() {
        let source = RecordingSource::default();
        let dcp = with_config();
        let settings = Settings::default();
        assert!(matches!(
            read(&source, Some(&dcp), &settings, &project(Presence::Missing)),
            Err(ConfigError::ProjectMissing { .. })
        ));
        let empty = ConfigChange {
            revision: "r1".into(),
            values: vec![],
        };
        assert_eq!(
            preview(
                &source,
                Some(&dcp),
                &settings,
                &project(Presence::Present),
                &empty
            ),
            Err(ConfigError::NothingToChange)
        );
        assert!(source.calls.lock().expect("lock").is_empty());
    }

    #[test]
    fn preview_is_a_dry_run_and_apply_is_not() {
        let source = RecordingSource::default();
        let dcp = with_config();
        let settings = Settings::default();
        let present = project(Presence::Present);
        assert!(read(&source, Some(&dcp), &settings, &present).is_ok());
        let previewed =
            preview(&source, Some(&dcp), &settings, &present, &change("r1")).expect("preview");
        assert!(!previewed.applied);
        let applied =
            apply(&source, Some(&dcp), &settings, &present, &change("r1")).expect("apply");
        assert!(applied.applied);
        assert_eq!(
            *source.calls.lock().expect("lock"),
            [
                "read",
                "set dry_run=true keys=1",
                "set dry_run=false keys=1"
            ]
        );
    }

    #[test]
    fn an_apply_that_wrote_nothing_says_so() {
        let source = RecordingSource::default();
        assert_eq!(
            apply(
                &source,
                Some(&with_config()),
                &Settings::default(),
                &project(Presence::Present),
                &change("unchanged")
            ),
            Err(ConfigError::NothingToChange)
        );
    }
}
