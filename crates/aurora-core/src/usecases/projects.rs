//! Registering the projects the user follows (US2, FR-005 to FR-010).

use std::path::Path;

use crate::domain::{AddOutcome, Project, ProjectError, ProjectId, ProjectList, ProjectName};
use crate::ports::ProjectRegistry;

pub fn list(registry: &dyn ProjectRegistry) -> ProjectList {
    registry.load()
}

/// Adds the repository that contains `path`; `name` defaults to the root folder's name.
pub fn add(
    registry: &dyn ProjectRegistry,
    path: &Path,
    name: Option<&str>,
) -> Result<AddOutcome, ProjectError> {
    let name = name.map(ProjectName::parse).transpose()?;
    registry.add(path, name)
}

pub fn rename(
    registry: &dyn ProjectRegistry,
    id: &ProjectId,
    name: &str,
) -> Result<Project, ProjectError> {
    registry.rename(id, ProjectName::parse(name)?)
}

/// Forgets a project; nothing on disk outside Aurora's own files changes.
pub fn remove(registry: &dyn ProjectRegistry, id: &ProjectId) -> Result<(), ProjectError> {
    registry.remove(id)
}

/// Points a missing project at the folder where its repository is now.
pub fn relocate(
    registry: &dyn ProjectRegistry,
    id: &ProjectId,
    path: &Path,
) -> Result<Project, ProjectError> {
    registry.relocate(id, path)
}

#[cfg(all(test, feature = "fixtures"))]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::domain::{Presence, RegistryNotice};
    use crate::fixtures::{self, MemoryRegistry};

    #[test]
    fn add_uses_the_root_folder_name_by_default() {
        let registry = MemoryRegistry::default();
        let outcome = add(&registry, Path::new("/repos/gedo"), None).expect("added");
        assert_eq!(outcome.project.name.as_str(), "gedo");
        assert_eq!(list(&registry).projects.len(), 1);
    }

    #[test]
    fn add_from_a_sub_folder_registers_the_root() {
        let registry = MemoryRegistry::default();
        let outcome = add(&registry, Path::new("/repos/gedo/crates/core"), None).expect("added");
        assert_eq!(outcome.project.root, PathBuf::from("/repos/gedo"));
        assert!(outcome.used_repository_root);
    }

    #[test]
    fn add_refuses_a_folder_outside_any_repository() {
        let registry = MemoryRegistry::default();
        assert!(matches!(
            add(&registry, Path::new("/tmp/notes"), None),
            Err(ProjectError::NotARepository { .. })
        ));
    }

    #[test]
    fn add_refuses_a_duplicate() {
        let registry = MemoryRegistry::default();
        add(&registry, Path::new("/repos/gedo"), Some("Gedo")).expect("added");
        assert_eq!(
            add(&registry, Path::new("/repos/gedo/src"), None).map(|o| o.project.id),
            Err(ProjectError::AlreadyRegistered {
                name: "Gedo".to_owned()
            })
        );
    }

    #[test]
    fn add_refuses_an_invalid_name_before_touching_the_registry() {
        let registry = MemoryRegistry::default();
        assert!(matches!(
            add(&registry, Path::new("/repos/gedo"), Some("  ")),
            Err(ProjectError::InvalidName { .. })
        ));
        assert!(list(&registry).projects.is_empty());
    }

    #[test]
    fn rename_and_remove() {
        let registry = MemoryRegistry::default();
        let id = add(&registry, Path::new("/repos/gedo"), None)
            .expect("added")
            .project
            .id;
        let renamed = rename(&registry, &id, "Gedo engine").expect("renamed");
        assert_eq!(renamed.name.as_str(), "Gedo engine");
        remove(&registry, &id).expect("removed");
        assert!(list(&registry).projects.is_empty());
        assert!(matches!(
            remove(&registry, &id),
            Err(ProjectError::UnknownProject { .. })
        ));
    }

    #[test]
    fn relocate_a_missing_project() {
        let registry = fixtures::sample_registry();
        let missing = list(&registry)
            .projects
            .into_iter()
            .find(|p| p.presence == Presence::Missing)
            .expect("the samples include a missing project");
        let moved = relocate(&registry, &missing.id, Path::new("/repos/levix-platform"))
            .expect("relocated");
        assert_eq!(moved.presence, Presence::Present);
    }

    #[test]
    fn notices_reach_the_caller() {
        let registry = MemoryRegistry::with_notice(RegistryNotice::ReadOnly {
            reason: "read-only directory".to_owned(),
        });
        assert!(matches!(
            list(&registry).notice,
            Some(RegistryNotice::ReadOnly { .. })
        ));
    }
}
