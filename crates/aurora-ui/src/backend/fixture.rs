//! The preview backend: every command answered by the core's fixtures, with the engine
//! events simulated after a short delay (FR-017, FR-018).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use aurora_core::domain::{
    AddOutcome, ConfigChange, ConfigOutcome, CycleState, EngineKind, EngineReport, EnginesOverview,
    MessageHeader, MessageKind, MessagesSnapshot, PollResult, Project, ProjectConfig, ProjectError,
    ProjectId, ProjectList, Settings, TemplateSection, UserAction,
};
use aurora_core::fixtures::{
    self, FixtureConfigSource, FixtureCycleSource, FixtureMessages, MemoryRegistry, MemorySettings,
};
use aurora_core::usecases::{config, cycle, engines, messages, projects, settings};

use super::{CommandError, OnNews, OnReady, OnReport, Reply};

/// The simulated background poll of the preview fires once, this long after the view
/// subscribes.
const POLL_DELAY: Duration = Duration::from_secs(4);

const EVENT_DELAY: Duration = Duration::from_millis(350);

#[derive(Default)]
struct Subscribers {
    on_report: Option<OnReport>,
    on_ready: Option<OnReady>,
}

pub struct FixtureBackend {
    registry: MemoryRegistry,
    settings: MemorySettings,
    config: FixtureConfigSource,
    messages: FixtureMessages,
    /// What the worker would keep (IMP-007).
    snapshot: Mutex<MessagesSnapshot>,
    refreshes: Mutex<u32>,
    picks: Mutex<u32>,
    subscribers: Mutex<Subscribers>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl FixtureBackend {
    pub fn sample() -> Self {
        Self {
            registry: fixtures::sample_registry(),
            settings: MemorySettings::default(),
            config: FixtureConfigSource::default(),
            messages: FixtureMessages::sample(),
            snapshot: Mutex::new(MessagesSnapshot::default()),
            refreshes: Mutex::new(0),
            picks: Mutex::new(0),
            subscribers: Mutex::new(Subscribers::default()),
        }
    }

    pub fn list_projects(&self) -> ProjectList {
        projects::list(&self.registry)
    }

    pub fn add_project(&self, path: &Path, name: Option<&str>) -> Reply<AddOutcome> {
        Ok(projects::add(&self.registry, path, name)?)
    }

    /// There is no folder dialog in a browser: each pick adds the next sample repository.
    pub fn pick_and_add_project(&self) -> Reply<Option<AddOutcome>> {
        let mut picks = lock(&self.picks);
        *picks += 1;
        let path = PathBuf::from(format!("/repos/sample-{}", *picks));
        Ok(Some(projects::add(&self.registry, &path, None)?))
    }

    pub fn rename_project(&self, id: &str, name: &str) -> Reply<Project> {
        Ok(projects::rename(
            &self.registry,
            &ProjectId(id.to_owned()),
            name,
        )?)
    }

    /// There is no file dialog in a browser: the pick takes a sample logo.
    pub fn pick_project_logo(&self, id: &str) -> Reply<Option<Project>> {
        const SAMPLE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" fill="#0E7490"/><path d="M14 44 L32 14 L50 44 Z" fill="none" stroke="#F1F5F9" stroke-width="5" stroke-linejoin="round"/><circle cx="32" cy="36" r="5" fill="#00E5A3"/></svg>"##;
        Ok(Some(projects::set_logo(
            &self.registry,
            &ProjectId(id.to_owned()),
            "sample-logo.svg",
            SAMPLE,
        )?))
    }

    pub fn clear_project_logo(&self, id: &str) -> Reply<Project> {
        Ok(projects::clear_logo(
            &self.registry,
            &ProjectId(id.to_owned()),
        )?)
    }

    pub fn remove_project(&self, id: &str) -> Reply<()> {
        Ok(projects::remove(&self.registry, &ProjectId(id.to_owned()))?)
    }

    pub fn relocate_project(&self, id: &str, path: &Path) -> Reply<Project> {
        Ok(projects::relocate(
            &self.registry,
            &ProjectId(id.to_owned()),
            path,
        )?)
    }

    /// The sample engines; every other refresh shows Gedo hanging, so the preview covers
    /// ready, without contract, degraded and missing.
    pub fn engines_overview(&self) -> EnginesOverview {
        engines::overview(self.reports(*lock(&self.refreshes)))
    }

    fn reports(&self, refresh: u32) -> Vec<EngineReport> {
        let mut reports = fixtures::sample_reports();
        if refresh % 2 == 1 {
            for report in &mut reports {
                if report.kind == EngineKind::Gedo {
                    *report = fixtures::degraded_report(EngineKind::Gedo, "no answer after 10 s");
                }
            }
        }
        reports
    }

    pub fn subscribe_engines(&self, on_report: OnReport, on_ready: OnReady) {
        *lock(&self.subscribers) = Subscribers {
            on_report: Some(on_report),
            on_ready: Some(on_ready),
        };
    }

    pub fn refresh_engines(&self) {
        let refresh = {
            let mut refreshes = lock(&self.refreshes);
            *refreshes += 1;
            *refreshes
        };
        let reports = self.reports(refresh);
        let overview = engines::overview(reports.clone());
        let subscribers = lock(&self.subscribers);
        let mut delay = EVENT_DELAY;
        if let Some(on_report) = &subscribers.on_report {
            for report in reports {
                let on_report = on_report.clone();
                leptos::prelude::set_timeout(move || on_report(report), delay);
                delay += EVENT_DELAY;
            }
        }
        if let Some(on_ready) = subscribers.on_ready.clone() {
            leptos::prelude::set_timeout(move || on_ready(overview), delay);
        }
    }

    pub fn get_settings(&self) -> Settings {
        settings::get(&self.settings)
    }

    pub fn save_settings(&self, new: Settings) -> Reply<Settings> {
        settings::save(&self.settings, new).map_err(|error| CommandError::new("storage", error))
    }

    /// The sample cycle states. The preview takes `dcp` as ready with the contract, so the
    /// views can be built before it ships (ADR-0006); the Engines view still shows the
    /// sample engines.
    pub fn project_cycle_state(&self, id: &str) -> Reply<CycleState> {
        let project = self
            .list_projects()
            .projects
            .into_iter()
            .find(|project| project.id.0 == id)
            .ok_or_else(|| ProjectError::UnknownProject { id: id.to_owned() })?;
        let dcp = fixtures::ready_report(EngineKind::DevCycle, &["diagnose"]);
        Ok(cycle::cycle_state(
            &FixtureCycleSource,
            Some(&dcp),
            &self.get_settings(),
            &project,
        )?)
    }

    fn project(&self, id: &str) -> Reply<Project> {
        Ok(self
            .list_projects()
            .projects
            .into_iter()
            .find(|project| project.id.0 == id)
            .ok_or_else(|| ProjectError::UnknownProject { id: id.to_owned() })?)
    }

    /// The preview takes `dcp` as ready with the `config` capability, as for the cycle
    /// state; the sample settings live in memory, so a change applied here stays until the
    /// page reloads.
    pub fn project_config(&self, id: &str) -> Reply<ProjectConfig> {
        let project = self.project(id)?;
        Ok(config::read(
            &self.config,
            Some(&config_dcp()),
            &self.get_settings(),
            &project,
        )?)
    }

    pub fn set_project_config(
        &self,
        id: &str,
        change: &ConfigChange,
        dry_run: bool,
    ) -> Reply<ConfigOutcome> {
        let project = self.project(id)?;
        let settings = self.get_settings();
        let dcp = config_dcp();
        Ok(if dry_run {
            config::preview(&self.config, Some(&dcp), &settings, &project, change)?
        } else {
            config::apply(&self.config, Some(&dcp), &settings, &project, change)?
        })
    }
}

/// The preview takes `dcp-integrations` as ready, as for `dcp`; the sample conversations
/// live in memory and change with each action until the page reloads.
impl FixtureBackend {
    fn provider() -> EngineReport {
        fixtures::ready_report(EngineKind::Integrations, &["messages"])
    }

    /// The preview's background reading fires once, a little after the view subscribes:
    /// a new snapshot when the sample changed.
    pub fn subscribe_messages(self: &Arc<Self>, on_news: OnNews) {
        let backend = Arc::clone(self);
        leptos::prelude::set_timeout(
            move || {
                let before = lock(&backend.snapshot).clone();
                backend.messages_refresh();
                let after = backend.messages_snapshot();
                if after.version != before.version {
                    on_news(after.news(&before));
                }
            },
            POLL_DELAY,
        );
    }

    /// Reads the sample like the worker does and keeps the result.
    pub fn messages_refresh(&self) -> Vec<PollResult> {
        let read = messages::complete(
            self.list_projects()
                .projects
                .iter()
                .map(|project| {
                    messages::export_inbox(
                        &self.messages,
                        Some(&Self::provider()),
                        &self.get_settings(),
                        project,
                    )
                })
                .collect(),
        );
        let mut current = lock(&self.snapshot);
        match current.next(read, "2026-11-07T15:00:00Z") {
            Some(next) => {
                let changes = next.changes_from(&current);
                *current = next;
                changes
            }
            None => Vec::new(),
        }
    }

    /// The kept snapshot; the first call reads the sample.
    pub fn messages_snapshot(&self) -> MessagesSnapshot {
        let empty = lock(&self.snapshot).version == 0;
        if empty {
            self.messages_refresh();
        }
        lock(&self.snapshot).clone()
    }

    pub fn messages_template(&self, kind: &MessageKind) -> Reply<Vec<TemplateSection>> {
        Ok(messages::template(
            &self.messages,
            Some(&Self::provider()),
            &self.get_settings(),
            kind,
        )?)
    }

    pub fn messages_act(&self, id: &str, action: &UserAction) -> Reply<Vec<MessageHeader>> {
        let project = self.project(id)?;
        let sent = messages::act(
            &self.messages,
            Some(&Self::provider()),
            &self.get_settings(),
            &project,
            action,
        )?;
        // Like the worker: a reading follows each write.
        self.messages_refresh();
        Ok(sent)
    }
}

fn config_dcp() -> EngineReport {
    fixtures::ready_report(EngineKind::DevCycle, &["diagnose", "config"])
}

impl From<aurora_core::domain::ProjectError> for CommandError {
    fn from(error: aurora_core::domain::ProjectError) -> Self {
        Self::new(&serde_json_kind(&error), error)
    }
}

/// The `kind` tag the desktop sends for a project error.
fn serde_json_kind(error: &aurora_core::domain::ProjectError) -> String {
    use aurora_core::domain::ProjectError as E;
    match error {
        E::NotFound { .. } => "notFound",
        E::NotARepository { .. } => "notARepository",
        E::AlreadyRegistered { .. } => "alreadyRegistered",
        E::InvalidName { .. } => "invalidName",
        E::UnknownProject { .. } => "unknownProject",
        E::Storage { .. } => "storage",
        E::InvalidLogo { .. } => "invalidLogo",
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use aurora_core::domain::{EngineStatus, Presence};

    use super::*;

    /// The commands of contracts/desktop-commands.md that this backend answers.
    const COMMANDS: &[&str] = &[
        "list_projects",
        "add_project",
        "pick_and_add_project",
        "rename_project",
        "remove_project",
        "relocate_project",
        "engines_overview",
        "refresh_engines",
        "get_settings",
        "save_settings",
        "project_cycle_state",
        "project_config",
        "preview_project_config",
        "apply_project_config",
        "messages_snapshot",
        "messages_refresh",
        "messages_template",
        "messages_act",
    ];

    const CONTRACTS: [&str; 4] = [
        include_str!("../../../../specs/001-foundations/contracts/desktop-commands.md"),
        include_str!("../../../../specs/002-dcp-adapter/contracts/desktop-commands.md"),
        include_str!("../../../../specs/007-project-settings/contracts/desktop-commands.md"),
        include_str!("../../../../specs/008-messages/contracts/desktop-commands.md"),
    ];

    #[test]
    fn answers_every_command_of_the_contract() {
        let documented: Vec<&str> = CONTRACTS
            .iter()
            .flat_map(|contract| contract.lines())
            .filter_map(|line| line.strip_prefix("| `"))
            .filter_map(|rest| rest.split('`').next())
            .filter(|name| !name.contains('-'))
            .collect();
        assert_eq!(documented, COMMANDS);
    }

    #[test]
    fn every_sample_project_answers_its_cycle_state() {
        let backend = FixtureBackend::sample();
        for project in backend.list_projects().projects {
            let answer = backend.project_cycle_state(&project.id.0);
            match project.presence {
                Presence::Present => assert!(answer.is_ok(), "{}: {answer:?}", project.name),
                Presence::Missing => {
                    let error = answer.expect_err("missing project");
                    assert_eq!(error.kind, "cycle");
                    assert!(matches!(
                        error.detail,
                        Some(aurora_core::domain::CycleError::ProjectMissing { .. })
                    ));
                }
            }
        }
        let unknown = backend.project_cycle_state("nope").expect_err("unknown");
        assert_eq!(unknown.kind, "unknownProject");
    }

    #[test]
    fn the_sample_covers_every_project_and_engine_state() {
        let backend = FixtureBackend::sample();
        let list = backend.list_projects();
        assert!(
            list.projects
                .iter()
                .any(|p| p.presence == Presence::Present)
        );
        assert!(
            list.projects
                .iter()
                .any(|p| p.presence == Presence::Missing)
        );

        let mut statuses: Vec<EngineStatus> =
            backend.reports(0).into_iter().map(|r| r.status).collect();
        statuses.extend(backend.reports(1).into_iter().map(|r| r.status));
        assert!(statuses.contains(&EngineStatus::Ready));
        assert!(statuses.contains(&EngineStatus::InstalledWithoutContract));
        assert!(statuses.contains(&EngineStatus::Missing));
        assert!(
            statuses
                .iter()
                .any(|s| matches!(s, EngineStatus::Degraded { .. }))
        );
    }

    #[test]
    fn settings_preview_refuse_and_apply_on_the_sample() {
        use aurora_core::domain::{ConfigDraft, ConfigError, ConfigValue};

        let backend = FixtureBackend::sample();
        let projects = backend.list_projects().projects;
        let gedo = &projects[0].id.0;
        let config = backend.project_config(gedo).expect("settings");
        let mut draft = ConfigDraft::default();
        draft.set(
            &config,
            "devops.branching",
            Some(ConfigValue::Text("integration".into())),
        );
        let change = draft.change(&config).expect("valid");
        let preview = backend
            .set_project_config(gedo, &change, true)
            .expect("preview");
        assert!(!preview.applied);
        let applied = backend
            .set_project_config(gedo, &change, false)
            .expect("apply");
        assert!(applied.applied);
        let stale = backend
            .set_project_config(gedo, &change, false)
            .expect_err("stale");
        assert_eq!(stale.kind, "config");
        assert_eq!(stale.config.as_deref(), Some(&ConfigError::Stale));
        let missing = projects
            .iter()
            .find(|p| p.presence == Presence::Missing)
            .expect("a missing sample");
        assert!(matches!(
            backend
                .project_config(&missing.id.0)
                .expect_err("missing")
                .config
                .as_deref(),
            Some(ConfigError::ProjectMissing { .. })
        ));
    }

    #[test]
    fn messages_decide_revise_and_refresh_on_the_sample() {
        use aurora_core::domain::{InboxSetup, MessageKind, UserAction};

        let backend = FixtureBackend::sample();
        let snapshot = backend.messages_snapshot();
        assert_eq!(snapshot.version, 1);
        assert!(
            snapshot
                .inboxes
                .iter()
                .any(|i| matches!(i.setup, InboxSetup::NotSetUp { .. }))
        );
        let views = snapshot.views();
        let waiting = views
            .iter()
            .find(|v| v.conversation.waiting_for_user)
            .expect("a record waits in the preview");
        let record = waiting.conversation.record.clone().expect("record");
        assert_eq!(record.options.len(), 2);
        let (project, thread) = (
            waiting.project.0.clone(),
            waiting.conversation.thread.clone(),
        );
        let decided = backend
            .messages_act(
                &project,
                &UserAction::Decide {
                    thread: thread.clone(),
                    choice: record.options[0].clone(),
                    note: None,
                },
            )
            .expect("decided");
        assert_eq!(decided[0].kind, MessageKind::Decision);
        let after = backend.messages_snapshot();
        assert!(
            after.version > snapshot.version,
            "a reading followed the write"
        );
        assert!(
            after
                .views()
                .iter()
                .all(|v| !v.conversation.waiting_for_user)
        );
        let refused = backend
            .messages_act(
                &project,
                &UserAction::Guide {
                    thread,
                    text: " ".into(),
                },
            )
            .expect_err("empty request");
        assert_eq!(refused.kind, "messages");
        assert!(
            backend.messages_refresh().is_empty(),
            "nothing changed since"
        );
        assert!(
            !backend
                .messages_template(&MessageKind::Question)
                .expect("template")
                .is_empty()
        );
    }

    #[test]
    fn project_commands_work_on_the_sample() {
        let backend = FixtureBackend::sample();
        let picked = backend
            .pick_and_add_project()
            .expect("pick")
            .expect("added");
        assert_eq!(picked.project.name.as_str(), "sample-1");
        let renamed = backend
            .rename_project(&picked.project.id.0, "Sample")
            .expect("rename");
        assert_eq!(renamed.name.as_str(), "Sample");
        backend
            .remove_project(&picked.project.id.0)
            .expect("remove");
        let refused = backend.add_project(Path::new("/tmp/not-a-repo"), None);
        assert_eq!(
            refused.map_err(|e| e.kind),
            Err("notARepository".to_owned())
        );
    }
}
