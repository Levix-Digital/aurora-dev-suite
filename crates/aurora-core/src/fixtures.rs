//! Sample data and in-memory implementations of the ports, for tests and for the
//! interface preview (FR-017, FR-018). Repositories exist only under `/repos/<name>`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::domain::{
    AddOutcome, CandidateRun, Capability, ConfigChange, ConfigError, ConfigOutcome, ConfigSetting,
    ConfigType, ConfigValue, CurrentRelease, CycleError, CycleState, EngineKind, EngineReport,
    EngineSettings, EngineStatus, EnginesOverview, FieldError, FieldErrorCode, ItemRef, KeyChange,
    NextOption, NextStep, PlanProgress, Presence, Project, ProjectConfig, ProjectError, ProjectId,
    ProjectList, ProjectName, PullRequestState, RegistryNotice, Settings, UntriagedBug, UpdateSide,
};
use crate::ports::{
    ConfigSource, CycleSource, EngineProbe, ProjectRegistry, SettingsError, SettingsStore,
};
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

fn config_setting(
    key: &str,
    description: &str,
    kind: ConfigType,
    value: Option<ConfigValue>,
    default: Option<ConfigValue>,
) -> ConfigSetting {
    ConfigSetting {
        key: key.to_owned(),
        group: key.split('.').next().unwrap_or(key).to_owned(),
        description: description.to_owned(),
        kind,
        value,
        default,
        allowed: None,
        nullable: false,
        read_only: false,
        read_only_reason: None,
    }
}

fn text(value: &str) -> Option<ConfigValue> {
    Some(ConfigValue::Text(value.to_owned()))
}

fn list(values: &[&str]) -> Option<ConfigValue> {
    Some(ConfigValue::List(
        values.iter().map(|value| (*value).to_owned()).collect(),
    ))
}

fn allowed(values: &[&str]) -> Option<Vec<String>> {
    Some(values.iter().map(|value| (*value).to_owned()).collect())
}

/// The sample settings of a project, as `dcp config get` would describe them: every type,
/// an unset value, a list with allowed values and a read-only setting.
pub fn sample_project_config() -> ProjectConfig {
    use ConfigType::{Boolean, Enum, Integer, List, Text};
    let severities = ["critical", "high", "medium", "low"];
    ProjectConfig {
        revision: "sha256:4f1c2a".to_owned(),
        source: Some(".dev-cycle.json".to_owned()),
        settings: vec![
            ConfigSetting {
                allowed: allowed(&["integration", "trunk", "release-branches"]),
                ..config_setting(
                    "devops.branching",
                    "Branching model: where work merges and how releases are cut",
                    Enum,
                    text("trunk"),
                    text("integration"),
                )
            },
            config_setting(
                "devops.integration_branch",
                "Integration branch of the integration model",
                Text,
                text("dev"),
                text("dev"),
            ),
            config_setting(
                "devops.production_branch",
                "Production branch",
                Text,
                text("main"),
                text("main"),
            ),
            config_setting(
                "devops.release_candidates",
                "Release candidates (vX.Y.Z-rc.N) for manual acceptance before a release",
                Boolean,
                Some(ConfigValue::Boolean(true)),
                Some(ConfigValue::Boolean(true)),
            ),
            config_setting(
                "devops.version_files",
                "Files whose version is bumped at a release",
                List,
                list(&["Cargo.toml"]),
                list(&[]),
            ),
            ConfigSetting {
                allowed: allowed(&["cli", "service", "library", "desktop", "none"]),
                nullable: true,
                ..config_setting(
                    "devops.observability",
                    "Logging and telemetry profile required in plans",
                    Enum,
                    None,
                    None,
                )
            },
            ConfigSetting {
                allowed: allowed(&[
                    "hexagonal",
                    "layered",
                    "mvc",
                    "pipeline",
                    "modular-monolith",
                    "other",
                ]),
                nullable: true,
                ..config_setting(
                    "architecture.style",
                    "Architecture style the design review checks",
                    Enum,
                    text("hexagonal"),
                    None,
                )
            },
            ConfigSetting {
                nullable: true,
                ..config_setting(
                    "architecture.core_path",
                    "Folder of the core, which must not import adapters",
                    Text,
                    text("crates/gedo-core"),
                    None,
                )
            },
            ConfigSetting {
                allowed: allowed(&severities),
                ..config_setting(
                    "backlog.blocking_severities",
                    "Bug severities that block new work",
                    List,
                    list(&["critical", "high", "medium"]),
                    list(&["critical", "high", "medium"]),
                )
            },
            ConfigSetting {
                allowed: allowed(&["next_release", "off"]),
                ..config_setting(
                    "backlog.bug_deadline",
                    "Latest release for a bug that does not block",
                    Enum,
                    text("next_release"),
                    text("next_release"),
                )
            },
            config_setting(
                "docs.user_doc",
                "The user documentation file",
                Text,
                text("README.md"),
                text("README.md"),
            ),
            config_setting(
                "docs.readme_max_lines",
                "README line limit",
                Integer,
                Some(ConfigValue::Integer(400)),
                Some(ConfigValue::Integer(400)),
            ),
            ConfigSetting {
                read_only: true,
                read_only_reason: Some(
                    "The user's approval gates are mandatory in the Dev Cycle Protocol".to_owned(),
                ),
                ..config_setting(
                    "principles.gates",
                    "Mandatory gates",
                    List,
                    list(&[
                        "product baseline",
                        "plan",
                        "release candidate acceptance",
                        "release gate",
                    ]),
                    list(&[
                        "product baseline",
                        "plan",
                        "release candidate acceptance",
                        "release gate",
                    ]),
                )
            },
        ],
        findings: Vec::new(),
        read_at: CHECKED_AT.to_owned(),
    }
}

/// A `ConfigSource` that keeps the sample settings of each present sample repository in
/// memory, checks a change as `dcp` would (including that the integration branch differs
/// from the production branch), and applies it with a new revision. `slow` times out.
#[derive(Debug, Default)]
pub struct FixtureConfigSource {
    configs: Mutex<BTreeMap<PathBuf, ProjectConfig>>,
}

impl FixtureConfigSource {
    fn with_config<T>(
        &self,
        root: &Path,
        settings: &EngineSettings,
        work: impl FnOnce(&mut ProjectConfig) -> Result<T, ConfigError>,
    ) -> Result<T, ConfigError> {
        if root.ends_with("slow") {
            return Err(ConfigError::TimedOut {
                timeout_ms: settings.timeout_ms,
                stderr: String::new(),
            });
        }
        let mut configs = lock(&self.configs);
        let config = configs
            .entry(root.to_owned())
            .or_insert_with(sample_project_config);
        work(config)
    }
}

/// The refusals `dcp` would give: the core's checks, then the rules across keys.
fn refusals(config: &ProjectConfig, change: &ConfigChange) -> Vec<FieldError> {
    if change.revision != config.revision {
        return vec![FieldError {
            code: FieldErrorCode::StaleRevision,
            key: None,
            message: "the configuration changed since it was read".to_owned(),
            accepted: Vec::new(),
        }];
    }
    let mut errors: Vec<FieldError> = change
        .values
        .iter()
        .filter_map(|kv| match config.setting(&kv.key) {
            None => Some(FieldError {
                code: FieldErrorCode::UnknownKey,
                key: Some(kv.key.clone()),
                message: "unknown setting".to_owned(),
                accepted: Vec::new(),
            }),
            Some(setting) => setting.check(kv.value.as_ref()).err(),
        })
        .collect();
    let after = |key: &str| {
        change
            .values
            .iter()
            .find(|kv| kv.key == key)
            .map(|kv| kv.value.clone())
            .or_else(|| config.setting(key).map(|s| s.value.clone()))
            .flatten()
    };
    if errors.is_empty() && after("devops.integration_branch") == after("devops.production_branch")
    {
        errors.push(FieldError {
            code: FieldErrorCode::InvalidValue,
            key: Some("devops.integration_branch".to_owned()),
            message: "the integration branch must differ from the production branch".to_owned(),
            accepted: Vec::new(),
        });
    }
    errors
}

impl ConfigSource for FixtureConfigSource {
    fn read(
        &self,
        _engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<ProjectConfig, ConfigError> {
        self.with_config(root, settings, |config| Ok(config.clone()))
    }

    fn set(
        &self,
        _engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        change: &ConfigChange,
        dry_run: bool,
    ) -> Result<ConfigOutcome, ConfigError> {
        self.with_config(root, settings, |config| {
            let errors = refusals(config, change);
            if !errors.is_empty() {
                return Err(ConfigError::from_field_errors(errors));
            }
            let changes: Vec<KeyChange> = change
                .values
                .iter()
                .filter_map(|kv| {
                    let before = config.setting(&kv.key)?.value.clone();
                    (before != kv.value).then(|| KeyChange {
                        key: kv.key.clone(),
                        before,
                        after: kv.value.clone(),
                    })
                })
                .collect();
            let diff = sample_diff(&changes);
            let applied = !dry_run && !changes.is_empty();
            if applied {
                for kv in &change.values {
                    if let Some(setting) = config.settings.iter_mut().find(|s| s.key == kv.key) {
                        setting.value = kv.value.clone();
                    }
                }
                config.revision = format!("{}+", config.revision);
            }
            Ok(ConfigOutcome {
                applied,
                revision: config.revision.clone(),
                changes,
                diff,
            })
        })
    }
}

/// A diff shaped like the one `dcp` returns, for the preview.
fn sample_diff(changes: &[KeyChange]) -> String {
    if changes.is_empty() {
        return String::new();
    }
    let mut diff = String::from("--- a/.dev-cycle.json\n+++ b/.dev-cycle.json\n");
    for change in changes {
        let leaf = change.key.rsplit('.').next().unwrap_or(&change.key);
        let json = |value: &Option<ConfigValue>| match value {
            None => "null".to_owned(),
            Some(ConfigValue::Text(text)) => format!("\"{text}\""),
            Some(ConfigValue::List(items)) => format!(
                "[{}]",
                items
                    .iter()
                    .map(|item| format!("\"{item}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Some(ConfigValue::Boolean(flag)) => flag.to_string(),
            Some(ConfigValue::Integer(number)) => number.to_string(),
        };
        diff.push_str(&format!(
            "@@ {} @@\n-    \"{leaf}\": {},\n+    \"{leaf}\": {},\n",
            change.key,
            json(&change.before),
            json(&change.after)
        ));
    }
    diff
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
    fn the_sample_settings_preview_refuse_and_apply_like_dcp() {
        use crate::domain::{ConfigDraft, KeyValue};

        let source = FixtureConfigSource::default();
        let dcp = ready_report(EngineKind::DevCycle, &["diagnose", "config"]);
        let settings = EngineSettings::default();
        let root = Path::new(REPOS).join("gedo");
        let config = source.read(&dcp, &settings, &root).expect("read");
        let mut draft = ConfigDraft::default();
        draft.set(&config, "devops.branching", text("integration"));
        draft.set(
            &config,
            "devops.release_candidates",
            Some(ConfigValue::Boolean(false)),
        );
        let change = draft.change(&config).expect("valid");

        let preview = source
            .set(&dcp, &settings, &root, &change, true)
            .expect("preview");
        assert!(!preview.applied);
        assert_eq!(preview.changes.len(), 2);
        assert!(
            preview
                .diff
                .contains("+    \"branching\": \"integration\",")
        );
        assert!(preview.diff.contains("+    \"release_candidates\": false,"));
        assert_eq!(source.read(&dcp, &settings, &root).expect("read"), config);

        let applied = source
            .set(&dcp, &settings, &root, &change, false)
            .expect("apply");
        assert!(applied.applied);
        let after = source.read(&dcp, &settings, &root).expect("read");
        assert_eq!(after.revision, applied.revision);
        assert_eq!(
            after
                .setting("devops.branching")
                .and_then(|s| s.value.clone()),
            text("integration")
        );

        assert_eq!(
            source.set(&dcp, &settings, &root, &change, true),
            Err(ConfigError::Stale)
        );
        let same_branch = ConfigChange {
            revision: after.revision.clone(),
            values: vec![KeyValue {
                key: "devops.integration_branch".into(),
                value: text("main"),
            }],
        };
        let Err(ConfigError::Rejected { errors }) =
            source.set(&dcp, &settings, &root, &same_branch, true)
        else {
            panic!("expected a refusal");
        };
        assert_eq!(errors[0].key.as_deref(), Some("devops.integration_branch"));
        assert!(matches!(
            source.read(&dcp, &settings, Path::new("/repos/slow")),
            Err(ConfigError::TimedOut { .. })
        ));
    }

    #[test]
    fn unknown_engines_are_missing() {
        let probe = FixtureProbe::default();
        let report = probe.probe(EngineKind::Gedo, &EngineSettings::default());
        assert_eq!(report.status, EngineStatus::Missing);
    }
}
