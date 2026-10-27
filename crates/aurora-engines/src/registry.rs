//! The project registry and the settings over TOML files in the user's configuration
//! directory, with atomic writes (ADR-0004, research R8).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use aurora_core::domain::{
    AddOutcome, Presence, Project, ProjectError, ProjectId, ProjectList, ProjectName,
    RegistryNotice, SETTINGS_VERSION, Settings,
};
use aurora_core::ports::{ProjectRegistry, SettingsError, SettingsStore};
use serde::{Deserialize, Serialize};

use crate::repo;

pub const PROJECTS_FILE: &str = "projects.toml";
pub const SETTINGS_FILE: &str = "settings.toml";
const REGISTRY_VERSION: u32 = 1;

/// Aurora's configuration directory, `None` when the system has no home directory.
pub fn config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("digital", "Levix", "Aurora")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistryDocument {
    version: u32,
    #[serde(default)]
    projects: Vec<StoredProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredProject {
    id: String,
    name: ProjectName,
    root: PathBuf,
    added_at: String,
}

impl StoredProject {
    fn to_project(&self) -> Project {
        let presence = if repo::is_repository_root(&self.root) {
            Presence::Present
        } else {
            Presence::Missing
        };
        Project {
            id: ProjectId(self.id.clone()),
            name: self.name.clone(),
            root: self.root.clone(),
            added_at: self.added_at.clone(),
            presence,
        }
    }
}

#[derive(Debug)]
struct RegistryState {
    projects: Vec<StoredProject>,
    notice: Option<RegistryNotice>,
}

/// `ProjectRegistry` over `projects.toml`. The file is read once; every change is
/// written back at once unless the registry is read-only for this session.
#[derive(Debug)]
pub struct TomlRegistry {
    path: PathBuf,
    state: Mutex<RegistryState>,
}

impl TomlRegistry {
    pub fn open(dir: &Path) -> Self {
        let path = dir.join(PROJECTS_FILE);
        let (projects, mut notice) = read_registry(&path);
        if notice.is_none()
            && let Err(reason) = check_writable(dir)
        {
            notice = Some(RegistryNotice::ReadOnly { reason });
        }
        Self {
            path,
            state: Mutex::new(RegistryState { projects, notice }),
        }
    }

    fn with_state<T>(&self, change: impl FnOnce(&mut RegistryState) -> T) -> T {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        change(&mut state)
    }

    /// Writes the list, unless it is read-only. A failed write turns the registry
    /// read-only for the session, so the change is kept in memory and the user told.
    fn persist(&self, state: &mut RegistryState) {
        if matches!(state.notice, Some(RegistryNotice::ReadOnly { .. })) {
            return;
        }
        let document = RegistryDocument {
            version: REGISTRY_VERSION,
            projects: state.projects.clone(),
        };
        let result = toml::to_string_pretty(&document)
            .map_err(|error| error.to_string())
            .and_then(|text| write_atomic(&self.path, &text).map_err(|e| e.to_string()));
        if let Err(reason) = result {
            tracing::warn!(path = %self.path.display(), %reason, "projects not saved");
            state.notice = Some(RegistryNotice::ReadOnly { reason });
        }
    }
}

impl ProjectRegistry for TomlRegistry {
    fn load(&self) -> ProjectList {
        self.with_state(|state| ProjectList {
            projects: state
                .projects
                .iter()
                .map(StoredProject::to_project)
                .collect(),
            notice: state.notice.clone(),
        })
    }

    fn add(&self, path: &Path, name: Option<ProjectName>) -> Result<AddOutcome, ProjectError> {
        let (root, used_repository_root) = repo::repository_root(path)?;
        self.with_state(|state| {
            if let Some(existing) = find_by_root(&state.projects, &root) {
                return Err(ProjectError::AlreadyRegistered {
                    name: existing.name.to_string(),
                });
            }
            let name = match name {
                Some(name) => name,
                None => default_name(&root)?,
            };
            let stored = StoredProject {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                root,
                added_at: crate::now_rfc3339(),
            };
            state.projects.push(stored.clone());
            self.persist(state);
            Ok(AddOutcome {
                project: stored.to_project(),
                used_repository_root,
            })
        })
    }

    fn rename(&self, id: &ProjectId, name: ProjectName) -> Result<Project, ProjectError> {
        self.with_state(|state| {
            let stored = find_by_id(&mut state.projects, id)?;
            stored.name = name;
            let project = stored.to_project();
            self.persist(state);
            Ok(project)
        })
    }

    fn remove(&self, id: &ProjectId) -> Result<(), ProjectError> {
        self.with_state(|state| {
            let before = state.projects.len();
            state.projects.retain(|stored| stored.id != id.0);
            if state.projects.len() == before {
                return Err(unknown(id));
            }
            self.persist(state);
            Ok(())
        })
    }

    fn relocate(&self, id: &ProjectId, path: &Path) -> Result<Project, ProjectError> {
        let (root, _) = repo::repository_root(path)?;
        self.with_state(|state| {
            if let Some(other) = find_by_root(&state.projects, &root)
                && other.id != id.0
            {
                return Err(ProjectError::AlreadyRegistered {
                    name: other.name.to_string(),
                });
            }
            let stored = find_by_id(&mut state.projects, id)?;
            stored.root = root;
            let project = stored.to_project();
            self.persist(state);
            Ok(project)
        })
    }
}

fn find_by_root<'a>(projects: &'a [StoredProject], root: &Path) -> Option<&'a StoredProject> {
    projects.iter().find(|stored| {
        let known = repo::canonicalize(&stored.root).unwrap_or_else(|| stored.root.clone());
        repo::same_path(&known, root)
    })
}

fn find_by_id<'a>(
    projects: &'a mut [StoredProject],
    id: &ProjectId,
) -> Result<&'a mut StoredProject, ProjectError> {
    projects
        .iter_mut()
        .find(|stored| stored.id == id.0)
        .ok_or_else(|| unknown(id))
}

fn unknown(id: &ProjectId) -> ProjectError {
    ProjectError::UnknownProject { id: id.0.clone() }
}

fn default_name(root: &Path) -> Result<ProjectName, ProjectError> {
    let folder = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string());
    let truncated: String = folder.chars().take(ProjectName::MAX_CHARS).collect();
    ProjectName::parse(&truncated)
}

/// Reads `projects.toml`. A missing file is an empty list; a file that cannot be read is
/// moved aside (FR-010); a newer version is kept untouched and the list is read-only.
fn read_registry(path: &Path) -> (Vec<StoredProject>, Option<RegistryNotice>) {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (Vec::new(), None),
        Err(error) => return recover(path, &error.to_string()),
    };
    match toml::from_str::<RegistryDocument>(&text) {
        Ok(document) if document.version > REGISTRY_VERSION => (
            Vec::new(),
            Some(RegistryNotice::ReadOnly {
                reason: format!(
                    "{} was written by a newer Aurora (version {}); update Aurora to change it",
                    path.display(),
                    document.version
                ),
            }),
        ),
        Ok(document) => (document.projects, None),
        Err(error) => recover(path, &error.to_string()),
    }
}

fn recover(path: &Path, reason: &str) -> (Vec<StoredProject>, Option<RegistryNotice>) {
    let stamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let moved_to = path.with_file_name(format!("{PROJECTS_FILE}.damaged-{stamp}"));
    tracing::warn!(path = %path.display(), %reason, "project list unreadable");
    match fs::rename(path, &moved_to) {
        Ok(()) => (Vec::new(), Some(RegistryNotice::Recovered { moved_to })),
        Err(error) => (
            Vec::new(),
            Some(RegistryNotice::ReadOnly {
                reason: format!(
                    "{} cannot be read ({reason}) nor moved aside ({error})",
                    path.display()
                ),
            }),
        ),
    }
}

/// Checks that `dir` exists (creating it) and accepts new files.
fn check_writable(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let probe = dir.join(".aurora-write-check");
    fs::write(&probe, b"").map_err(|error| format!("{}: {error}", dir.display()))?;
    let _ = fs::remove_file(&probe);
    Ok(())
}

/// Writes `text` to a temporary file next to `path`, then renames it over `path`.
pub fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporary = path.with_file_name(format!(".{file_name}.tmp"));
    let written = (|| {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written
}

/// `SettingsStore` over `settings.toml`. Unreadable or newer settings fall back to the
/// defaults without touching the file.
#[derive(Debug)]
pub struct TomlSettings {
    path: PathBuf,
}

impl TomlSettings {
    pub fn open(dir: &Path) -> Self {
        Self {
            path: dir.join(SETTINGS_FILE),
        }
    }
}

impl SettingsStore for TomlSettings {
    fn load(&self) -> Settings {
        let Ok(text) = fs::read_to_string(&self.path) else {
            return Settings::default();
        };
        match toml::from_str::<Settings>(&text) {
            Ok(settings) if settings.version <= SETTINGS_VERSION => settings,
            Ok(_) => Settings::default(),
            Err(error) => {
                tracing::warn!(path = %self.path.display(), %error, "settings unreadable, using defaults");
                Settings::default()
            }
        }
    }

    fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        let text = toml::to_string_pretty(settings).map_err(|error| SettingsError {
            message: error.to_string(),
        })?;
        write_atomic(&self.path, &text).map_err(|error| SettingsError {
            message: format!("{}: {error}", self.path.display()),
        })
    }
}
