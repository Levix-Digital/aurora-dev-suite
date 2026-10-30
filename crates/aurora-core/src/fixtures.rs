//! Sample data and in-memory implementations of the ports, for tests and for the
//! interface preview (FR-017, FR-018). Repositories exist only under `/repos/<name>`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::domain::{
    AddOutcome, CandidateRun, Capability, CurrentRelease, CycleError, CycleState, EngineKind,
    EngineReport, EngineSettings, EngineStatus, EnginesOverview, ItemRef, NextOption, NextStep,
    PlanProgress, Presence, Project, ProjectError, ProjectId, ProjectList, ProjectName,
    PullRequestState, RegistryNotice, Settings, UntriagedBug, UpdateSide,
};
use crate::ports::{CycleSource, EngineProbe, ProjectRegistry, SettingsError, SettingsStore};
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

fn item(id: &str, rank: Option<&str>, title: &str) -> ItemRef {
    ItemRef {
        id: id.to_owned(),
        rank: rank.map(str::to_owned),
        title: title.to_owned(),
    }
}

fn option(key: &str, title: &str, reason: Option<&str>) -> NextOption {
    NextOption {
        key: key.to_owned(),
        title: title.to_owned(),
        reason: reason.map(str::to_owned),
    }
}

/// A project with a release under way, a plan, a pull request waiting for the user and
/// bugs; the same data as `busy.json` of the `dcp.diagnose` samples.
pub fn busy_cycle_state() -> CycleState {
    CycleState {
        summary: "Project state: current release v0.4.0 Command line, 1 blocking bug, 1 plan in progress.".to_owned(),
        next: NextStep {
            ask: true,
            recommended: Some("bugs".to_owned()),
            options: vec![
                option("bugs", "Fix the blocking bug first", Some("BUG-012 blocks the release")),
                option("plan:command-line", "Resume plan command-line", Some("3 of 5 milestones done")),
                option("pr:41", "Answer the review of pull request 41", None),
            ],
        },
        release: Some(CurrentRelease {
            version: "v0.4.0".to_owned(),
            name: "Command line".to_owned(),
            open_items: vec![
                item("FEAT-031", Some("high"), "gedo radar --json"),
                item("FEAT-034", Some("medium"), "gedo doctor --json"),
            ],
        }),
        plans: vec![PlanProgress {
            id: "command-line".to_owned(),
            title: "Command line".to_owned(),
            status: "in_progress".to_owned(),
            milestones_done: 3,
            milestones_total: 5,
        }],
        release_candidate: None,
        pull_requests: vec![PullRequestState {
            number: 41,
            title: "feat: gedo radar --json".to_owned(),
            summary: "2 threads open, CI green".to_owned(),
            needs_action: true,
            threads_for_user: 1,
        }],
        blocking_bugs: vec![item(
            "BUG-012",
            Some("high"),
            "Merge falls back to text on renamed files",
        )],
        untriaged_bugs: vec![UntriagedBug {
            item: item("BUG-015", None, "Slow radar on large repositories"),
            reason: "no severity".to_owned(),
        }],
        findings: Vec::new(),
        read_at: CHECKED_AT.to_owned(),
    }
}

/// A project with nothing in progress; `idle.json` of the samples.
pub fn idle_cycle_state() -> CycleState {
    CycleState {
        summary: "Project state: no roadmap, no blocking bugs, no plans in progress.".to_owned(),
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
        read_at: CHECKED_AT.to_owned(),
    }
}

/// A release candidate waiting for manual tests; `release-candidate.json` of the samples.
pub fn candidate_cycle_state() -> CycleState {
    CycleState {
        summary: "Project state: release candidate v2.9.0-rc.1 waits for manual tests.".to_owned(),
        next: NextStep {
            ask: true,
            recommended: Some("rc:v2.9.0-rc.1".to_owned()),
            options: vec![option(
                "rc:v2.9.0-rc.1",
                "Run the manual tests of v2.9.0-rc.1",
                Some("4 not run, 1 failed"),
            )],
        },
        release: Some(CurrentRelease {
            version: "v2.9.0".to_owned(),
            name: "Baseline and zero friction".to_owned(),
            open_items: Vec::new(),
        }),
        plans: Vec::new(),
        release_candidate: Some(CandidateRun {
            run: "v2.9.0-rc.1".to_owned(),
            version: "v2.9.0".to_owned(),
            not_run: 4,
            failed: 1,
            released: false,
        }),
        pull_requests: Vec::new(),
        blocking_bugs: Vec::new(),
        untriaged_bugs: Vec::new(),
        findings: vec![
            "pull requests: the platform could not be reached, so they were not read".to_owned(),
        ],
        read_at: CHECKED_AT.to_owned(),
    }
}

/// A `CycleSource` that answers per sample repository root: `gedo` busy, `any-context`
/// idle, `ai-code-agent-skills` a release candidate, and `slow`, `broken` and `future`
/// the failures timed out, invalid and unsupported.
#[derive(Debug, Default)]
pub struct FixtureCycleSource;

impl CycleSource for FixtureCycleSource {
    fn cycle_state(
        &self,
        _engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<CycleState, CycleError> {
        let name = root
            .strip_prefix(REPOS)
            .ok()
            .map(|relative| relative.to_string_lossy().into_owned())
            .unwrap_or_default();
        match name.as_str() {
            "gedo" => Ok(busy_cycle_state()),
            "any-context" => Ok(idle_cycle_state()),
            "ai-code-agent-skills" => Ok(candidate_cycle_state()),
            "slow" => Err(CycleError::TimedOut {
                timeout_ms: settings.timeout_ms,
                stderr: String::new(),
            }),
            "future" => Err(CycleError::Unsupported {
                found: 2,
                update: UpdateSide::Aurora,
            }),
            _ => Err(CycleError::Invalid {
                message: "the reply is not a versioned JSON envelope".to_owned(),
                stderr: "dcp: warning: sample engine without an answer for this project".to_owned(),
            }),
        }
    }
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
    fn every_sample_project_has_a_cycle_answer() {
        let source = FixtureCycleSource;
        let dcp = ready_report(EngineKind::DevCycle, &["diagnose"]);
        let settings = EngineSettings::default();
        let answer = |name: &str| source.cycle_state(&dcp, &settings, &Path::new(REPOS).join(name));
        assert_eq!(answer("gedo"), Ok(busy_cycle_state()));
        assert_eq!(answer("any-context"), Ok(idle_cycle_state()));
        assert_eq!(answer("ai-code-agent-skills"), Ok(candidate_cycle_state()));
        assert!(matches!(answer("slow"), Err(CycleError::TimedOut { .. })));
        assert!(matches!(
            answer("future"),
            Err(CycleError::Unsupported { .. })
        ));
        assert!(matches!(answer("broken"), Err(CycleError::Invalid { .. })));
    }

    #[test]
    fn unknown_engines_are_missing() {
        let probe = FixtureProbe::default();
        let report = probe.probe(EngineKind::Gedo, &EngineSettings::default());
        assert_eq!(report.status, EngineStatus::Missing);
    }
}
