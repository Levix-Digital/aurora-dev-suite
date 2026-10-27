mod common;

use std::fs;

use aurora_core::domain::{
    Presence, ProjectError, ProjectName, RegistryNotice, Settings, ThemeChoice,
};
use aurora_core::ports::{ProjectRegistry, SettingsStore};
use aurora_engines::registry::{PROJECTS_FILE, write_atomic};
use aurora_engines::{TomlRegistry, TomlSettings};

#[test]
fn projects_survive_a_restart() {
    let config = tempfile::tempdir().unwrap();
    let repos = tempfile::tempdir().unwrap();
    let gedo = common::make_repo(repos.path(), "gedo");
    let id = {
        let registry = TomlRegistry::open(config.path());
        let added = registry.add(&gedo, None).unwrap();
        assert_eq!(added.project.name.as_str(), "gedo");
        assert!(!added.used_repository_root);
        added.project.id
    };
    let reopened = TomlRegistry::open(config.path()).load();
    assert_eq!(reopened.notice, None);
    assert_eq!(reopened.projects.len(), 1);
    assert_eq!(reopened.projects[0].id, id);
    assert_eq!(reopened.projects[0].presence, Presence::Present);
}

#[test]
fn a_sub_folder_registers_the_repository_root() {
    let config = tempfile::tempdir().unwrap();
    let repos = tempfile::tempdir().unwrap();
    let root = common::make_repo(repos.path(), "any-context");
    let inner = root.join("src").join("deep");
    fs::create_dir_all(&inner).unwrap();
    let added = TomlRegistry::open(config.path()).add(&inner, None).unwrap();
    assert!(added.used_repository_root);
    assert_eq!(added.project.root, dunce::canonicalize(&root).unwrap());
}

#[test]
fn a_git_file_marks_a_worktree_root() {
    let config = tempfile::tempdir().unwrap();
    let repos = tempfile::tempdir().unwrap();
    let worktree = repos.path().join("wt");
    fs::create_dir_all(&worktree).unwrap();
    fs::write(
        worktree.join(".git"),
        "gitdir: /elsewhere/.git/worktrees/wt\n",
    )
    .unwrap();
    assert!(
        TomlRegistry::open(config.path())
            .add(&worktree, None)
            .is_ok()
    );
}

#[test]
fn folders_outside_a_repository_and_missing_folders_are_refused() {
    let config = tempfile::tempdir().unwrap();
    let plain = tempfile::tempdir().unwrap();
    let registry = TomlRegistry::open(config.path());
    assert!(matches!(
        registry.add(plain.path(), None),
        Err(ProjectError::NotARepository { .. })
    ));
    assert!(matches!(
        registry.add(&plain.path().join("nope"), None),
        Err(ProjectError::NotFound { .. })
    ));
    assert!(registry.load().projects.is_empty());
}

#[test]
fn the_same_repository_through_another_path_is_a_duplicate() {
    let config = tempfile::tempdir().unwrap();
    let repos = tempfile::tempdir().unwrap();
    let root = common::make_repo(repos.path(), "gedo");
    fs::create_dir_all(root.join("docs")).unwrap();
    let registry = TomlRegistry::open(config.path());
    registry.add(&root, None).unwrap();
    let again = registry.add(&root.join("docs").join(".."), None);
    assert!(matches!(again, Err(ProjectError::AlreadyRegistered { name }) if name == "gedo"));
    if cfg!(windows) {
        let upper = std::path::PathBuf::from(root.to_string_lossy().to_uppercase());
        assert!(matches!(
            registry.add(&upper, None),
            Err(ProjectError::AlreadyRegistered { .. })
        ));
    }
}

#[cfg(unix)]
#[test]
fn the_same_repository_through_a_link_is_a_duplicate() {
    let config = tempfile::tempdir().unwrap();
    let repos = tempfile::tempdir().unwrap();
    let root = common::make_repo(repos.path(), "gedo");
    let link = repos.path().join("gedo-link");
    std::os::unix::fs::symlink(&root, &link).unwrap();
    let registry = TomlRegistry::open(config.path());
    registry.add(&root, None).unwrap();
    assert!(matches!(
        registry.add(&link, None),
        Err(ProjectError::AlreadyRegistered { .. })
    ));
}

#[test]
fn rename_remove_and_relocate_are_kept() {
    let config = tempfile::tempdir().unwrap();
    let repos = tempfile::tempdir().unwrap();
    let gedo = common::make_repo(repos.path(), "gedo");
    let skills = common::make_repo(repos.path(), "skills");
    let registry = TomlRegistry::open(config.path());
    let first = registry.add(&gedo, None).unwrap().project;
    let second = registry.add(&skills, None).unwrap().project;

    registry
        .rename(&first.id, ProjectName::parse("Gedo").unwrap())
        .unwrap();
    registry.remove(&second.id).unwrap();

    fs::rename(&gedo, repos.path().join("gedo-moved")).unwrap();
    let missing = TomlRegistry::open(config.path()).load();
    assert_eq!(missing.projects[0].presence, Presence::Missing);

    let reopened = TomlRegistry::open(config.path());
    let moved = reopened
        .relocate(&first.id, &repos.path().join("gedo-moved"))
        .unwrap();
    assert_eq!(moved.presence, Presence::Present);
    assert_eq!(moved.name.as_str(), "Gedo");
    assert!(
        skills.exists(),
        "removing a project never touches the repository"
    );

    let last = TomlRegistry::open(config.path()).load();
    assert_eq!(last.projects.len(), 1);
    assert_eq!(last.projects[0].root, moved.root);
}

#[test]
fn a_damaged_file_is_moved_aside_and_the_list_starts_empty() {
    let config = tempfile::tempdir().unwrap();
    fs::write(config.path().join(PROJECTS_FILE), "this is [not toml").unwrap();
    let list = TomlRegistry::open(config.path()).load();
    assert!(list.projects.is_empty());
    let Some(RegistryNotice::Recovered { moved_to }) = list.notice else {
        panic!("expected a recovered notice, got {:?}", list.notice);
    };
    assert!(moved_to.exists());
    assert!(!config.path().join(PROJECTS_FILE).exists());
}

#[test]
fn a_newer_file_is_left_untouched_and_read_only() {
    let config = tempfile::tempdir().unwrap();
    let path = config.path().join(PROJECTS_FILE);
    fs::write(&path, "version = 2\n").unwrap();
    let list = TomlRegistry::open(config.path()).load();
    assert!(matches!(list.notice, Some(RegistryNotice::ReadOnly { .. })));
    assert_eq!(fs::read_to_string(&path).unwrap(), "version = 2\n");
}

#[cfg(unix)]
#[test]
fn a_read_only_directory_keeps_changes_for_the_session() {
    use std::os::unix::fs::PermissionsExt;
    if is_root() {
        return; // root ignores permissions, so the case cannot be produced.
    }
    let config = tempfile::tempdir().unwrap();
    let repos = tempfile::tempdir().unwrap();
    let gedo = common::make_repo(repos.path(), "gedo");
    fs::set_permissions(config.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let registry = TomlRegistry::open(config.path());
    let list = registry.load();
    assert!(matches!(list.notice, Some(RegistryNotice::ReadOnly { .. })));
    registry.add(&gedo, None).unwrap();
    assert_eq!(registry.load().projects.len(), 1);
    fs::set_permissions(config.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(!config.path().join(PROJECTS_FILE).exists());
}

#[cfg(unix)]
fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim() == "0")
        .unwrap_or(false)
}

#[test]
fn a_failed_atomic_write_keeps_the_old_file() {
    let config = tempfile::tempdir().unwrap();
    let path = config.path().join(PROJECTS_FILE);
    write_atomic(&path, "version = 1\n").unwrap();
    // A directory where the temporary file should go makes the next write fail.
    fs::create_dir(config.path().join(format!(".{PROJECTS_FILE}.tmp"))).unwrap();
    assert!(write_atomic(&path, "version = 1\n# changed\n").is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "version = 1\n");
}

#[test]
fn settings_are_kept_and_unreadable_settings_fall_back_to_defaults() {
    let config = tempfile::tempdir().unwrap();
    let store = TomlSettings::open(config.path());
    assert_eq!(store.load(), Settings::default());
    let mut settings = Settings {
        theme: ThemeChoice::Dark,
        ..Settings::default()
    };
    settings.engines.gedo.timeout_ms = 3_000;
    store.save(&settings).unwrap();
    assert_eq!(TomlSettings::open(config.path()).load(), settings);

    fs::write(config.path().join("settings.toml"), "theme = 42").unwrap();
    assert_eq!(
        TomlSettings::open(config.path()).load(),
        Settings::default()
    );
}
