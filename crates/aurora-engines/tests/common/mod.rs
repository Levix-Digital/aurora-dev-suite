//! Helpers shared by the adapter tests.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Copies the fake engine into `dir` under the engine's command name, with `mode`.
#[cfg(feature = "fake-engine")]
pub fn install_fake(dir: &Path, command: &str, mode: &str) -> PathBuf {
    let target = dir.join(format!("{command}{}", std::env::consts::EXE_SUFFIX));
    // A hard link avoids "text file busy": a copied binary still has a writable descriptor
    // that a process started at the same time by another test can inherit.
    let source = env!("CARGO_BIN_EXE_aurora-fake-engine");
    if std::fs::hard_link(source, &target).is_err() {
        std::fs::copy(source, &target).expect("copy fake engine");
    }
    let mut mode_file = target.as_os_str().to_owned();
    mode_file.push(".mode");
    std::fs::write(PathBuf::from(mode_file), mode).expect("write mode");
    target
}

/// Creates `dir/name` as a repository (a `.git` directory) and returns it.
pub fn make_repo(dir: &Path, name: &str) -> PathBuf {
    let root = dir.join(name);
    std::fs::create_dir_all(root.join(".git")).expect("create repo");
    root
}

/// A temporary directory under Cargo's target directory, on the same file system as the
/// fake engine so it can be hard-linked.
pub fn engine_dir() -> tempfile::TempDir {
    tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temporary directory")
}
