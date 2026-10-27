//! Sample data and in-memory implementations of the ports, for tests and for the
//! interface preview (FR-017, FR-018). Repositories exist only under `/repos/<name>`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::domain::{
    AddOutcome, Capability, EngineKind, EngineReport, EngineSettings, EngineStatus,
    EnginesOverview, Presence, Project, ProjectError, ProjectId, ProjectList, ProjectName,
    RegistryNotice, Settings,
};
use crate::ports::{EngineProbe, ProjectRegistry, SettingsError, SettingsStore};
use crate::usecases;

pub const CHECKED_AT: &str = "2026-10-27T18:00:00Z";
const REPOS: &str = "/repos";

/// The root of the sample repository that contains `path`, if any.
fn sample_repository_root(path: &Path) -> Option<PathBuf> {
    let relative = path.strip_prefix(REPOS).ok()?;
    let first = relative.components().next()?;
    Some(Path::new(REPOS).join(first))
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Debug, Default)]
struct RegistryState {
    projects: Vec<Project>,
    next_id: u32,
}

/// An in-memory `ProjectRegistry`.
#[derive(Debug, Default)]
pub struct MemoryRegistry {
    state: Mutex<RegistryState>,
    notice: Option<RegistryNotice>,
}

impl MemoryRegistry {
    pub fn with_notice(notice: RegistryNotice) -> Self {
        Self {
            notice: Some(notice),
            ..Self::default()
        }
    }

    fn insert(&self, root: PathBuf, name: ProjectName, presence: Presence) -> Project {
        let mut state = lock(&self.state);
        state.next_id += 1;
        let project = Project {
            id: ProjectId(format!("sample-{}", state.next_id)),
            name,
            root,
            added_at: CHECKED_AT.to_owned(),
            presence,
        };
        state.projects.push(project.clone());
        project
    }
}

impl ProjectRegistry for MemoryRegistry {
    fn load(&self) -> ProjectList {
        ProjectList {
            projects: lock(&self.state).projects.clone(),
            notice: self.notice.clone(),
        }
    }

    fn add(&self, path: &Path, name: Option<ProjectName>) -> Result<AddOutcome, ProjectError> {
        let root = sample_repository_root(path).ok_or_else(|| ProjectError::NotARepository {
            path: path.to_owned(),
        })?;
        if let Some(existing) = lock(&self.state).projects.iter().find(|p| p.root == root) {
            return Err(ProjectError::AlreadyRegistered {
                name: existing.name.to_string(),
            });
        }
        let default_name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = match name {
            Some(name) => name,
            None => ProjectName::parse(&default_name)?,
        };
        let used_repository_root = root != path;
        let project = self.insert(root, name, Presence::Present);
        Ok(AddOutcome {
            project,
            used_repository_root,
        })
    }

    fn rename(&self, id: &ProjectId, name: ProjectName) -> Result<Project, ProjectError> {
        let mut state = lock(&self.state);
        let project = find_mut(&mut state.projects, id)?;
        project.name = name;
        Ok(project.clone())
    }

    fn remove(&self, id: &ProjectId) -> Result<(), ProjectError> {
        let mut state = lock(&self.state);
        let before = state.projects.len();
        state.projects.retain(|p| &p.id != id);
        if state.projects.len() == before {
            return Err(unknown(id));
        }
        Ok(())
    }

    fn relocate(&self, id: &ProjectId, path: &Path) -> Result<Project, ProjectError> {
        let root = sample_repository_root(path).ok_or_else(|| ProjectError::NotARepository {
            path: path.to_owned(),
        })?;
        let mut state = lock(&self.state);
        let project = find_mut(&mut state.projects, id)?;
        project.root = root;
        project.presence = Presence::Present;
        Ok(project.clone())
    }
}

fn find_mut<'a>(
    projects: &'a mut [Project],
    id: &ProjectId,
) -> Result<&'a mut Project, ProjectError> {
    projects
        .iter_mut()
        .find(|p| &p.id == id)
        .ok_or_else(|| unknown(id))
}

fn unknown(id: &ProjectId) -> ProjectError {
    ProjectError::UnknownProject { id: id.0.clone() }
}

/// Four sample projects; `levix-platform` is missing from disk.
pub fn sample_registry() -> MemoryRegistry {
    let registry = MemoryRegistry::default();
    for name in ["gedo", "any-context", "ai-code-agent-skills"] {
        let parsed = ProjectName::parse(name).unwrap_or_else(|_| unreachable!("valid sample"));
        registry.insert(Path::new(REPOS).join(name), parsed, Presence::Present);
    }
    let platform =
        ProjectName::parse("levix-platform").unwrap_or_else(|_| unreachable!("valid sample"));
    registry.insert(
        PathBuf::from("/old/levix-platform"),
        platform,
        Presence::Missing,
    );
    registry
}

/// An in-memory `SettingsStore`.
#[derive(Debug, Default)]
pub struct MemorySettings {
    settings: Mutex<Settings>,
}

impl SettingsStore for MemorySettings {
    fn load(&self) -> Settings {
        lock(&self.settings).clone()
    }

    fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        *lock(&self.settings) = settings.clone();
        Ok(())
    }
}

/// An `EngineProbe` that answers fixed reports.
#[derive(Debug, Default)]
pub struct FixtureProbe {
    reports: BTreeMap<EngineKind, EngineReport>,
}

impl FixtureProbe {
    pub fn new(reports: impl IntoIterator<Item = EngineReport>) -> Self {
        Self {
            reports: reports.into_iter().map(|r| (r.kind, r)).collect(),
        }
    }

    /// Gedo ready, AnyContext without the contract, the Dev Cycle missing.
    pub fn sample() -> Self {
        Self::new(sample_reports())
    }
}

impl EngineProbe for FixtureProbe {
    fn probe(&self, kind: EngineKind, _settings: &EngineSettings) -> EngineReport {
        self.reports
            .get(&kind)
            .cloned()
            .unwrap_or_else(|| EngineReport::missing(kind, CHECKED_AT))
    }
}

pub fn ready_report(kind: EngineKind, capabilities: &[&str]) -> EngineReport {
    EngineReport {
        kind,
        status: EngineStatus::Ready,
        location: Some(PathBuf::from(format!("/usr/local/bin/{}", kind.command()))),
        version: Some("0.5.0".to_owned()),
        capabilities: capabilities.iter().map(|c| Capability::new(c, 1)).collect(),
        last_error: None,
        checked_at: CHECKED_AT.to_owned(),
    }
}

pub fn degraded_report(kind: EngineKind, reason: &str) -> EngineReport {
    EngineReport {
        kind,
        status: EngineStatus::Degraded {
            reason: reason.to_owned(),
        },
        location: Some(PathBuf::from(format!("/usr/local/bin/{}", kind.command()))),
        version: None,
        capabilities: Vec::new(),
        last_error: Some(format!("{}: {reason}", kind.command())),
        checked_at: CHECKED_AT.to_owned(),
    }
}

pub fn sample_reports() -> Vec<EngineReport> {
    let any_context = EngineReport {
        kind: EngineKind::AnyContext,
        status: EngineStatus::InstalledWithoutContract,
        location: Some(PathBuf::from("/home/levi/.local/bin/actx")),
        version: Some("0.38.0".to_owned()),
        capabilities: Vec::new(),
        last_error: None,
        checked_at: CHECKED_AT.to_owned(),
    };
    vec![
        EngineReport::missing(EngineKind::DevCycle, CHECKED_AT),
        ready_report(
            EngineKind::Gedo,
            &["doctor", "radar", "scope", "stream", "check-contracts"],
        ),
        any_context,
    ]
}

/// The overview of the sample engines.
pub fn sample_overview() -> EnginesOverview {
    usecases::engines::overview(sample_reports())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_registry_has_a_missing_project() {
        let list = sample_registry().load();
        assert_eq!(list.projects.len(), 4);
        assert_eq!(
            list.projects
                .iter()
                .filter(|p| p.presence == Presence::Missing)
                .count(),
            1
        );
    }

    #[test]
    fn sample_reports_cover_ready_without_contract_and_missing() {
        let statuses: Vec<_> = sample_reports().into_iter().map(|r| r.status).collect();
        assert!(statuses.contains(&EngineStatus::Ready));
        assert!(statuses.contains(&EngineStatus::InstalledWithoutContract));
        assert!(statuses.contains(&EngineStatus::Missing));
    }

    #[test]
    fn degraded_report_keeps_the_error() {
        let report = degraded_report(EngineKind::Gedo, "timed out after 10 s");
        assert!(matches!(report.status, EngineStatus::Degraded { .. }));
        assert!(report.last_error.is_some());
    }

    #[test]
    fn unknown_engines_are_missing() {
        let probe = FixtureProbe::default();
        let report = probe.probe(EngineKind::Gedo, &EngineSettings::default());
        assert_eq!(report.status, EngineStatus::Missing);
    }
}
