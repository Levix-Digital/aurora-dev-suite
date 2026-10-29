//! Repository root discovery and path canonicalization, without running Git (research R8).

use std::path::{Path, PathBuf};

use aurora_core::domain::ProjectError;

/// The canonical root of the repository containing `path`, and whether it differs from
/// `path` itself.
pub fn repository_root(path: &Path) -> Result<(PathBuf, bool), ProjectError> {
    let canonical = canonicalize(path).ok_or_else(|| ProjectError::NotFound {
        path: path.to_owned(),
    })?;
    if !canonical.is_dir() {
        return Err(ProjectError::NotFound {
            path: path.to_owned(),
        });
    }
    let root = canonical
        .ancestors()
        .find(|candidate| is_repository_root(candidate))
        .map(Path::to_path_buf)
        .ok_or_else(|| ProjectError::NotARepository {
            path: path.to_owned(),
        })?;
    let moved = root != canonical;
    Ok((root, moved))
}

/// Whether `path` holds a `.git` directory, or a `.git` file as in worktrees and
/// submodules.
pub fn is_repository_root(path: &Path) -> bool {
    let marker = path.join(".git");
    marker.is_dir() || marker.is_file()
}

/// The canonical form of an existing path, without `\\?\` prefixes on Windows.
pub fn canonicalize(path: &Path) -> Option<PathBuf> {
    dunce::canonicalize(path).ok()
}

/// Whether two canonical paths name the same folder: case-insensitive on Windows.
pub fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    } else {
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_paths_compare_exactly_and_windows_paths_ignore_case() {
        let a = Path::new("/src/Gedo");
        let b = Path::new("/src/gedo");
        assert_eq!(same_path(a, b), cfg!(windows));
        assert!(same_path(a, a));
    }
}
