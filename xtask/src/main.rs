//! Repository automation. `cargo xtask arch-check` enforces the dependency rule of
//! ADR-0001: the core depends on no framework, adapter or process code.
//! `cargo xtask release-check <tag>` checks a release tag against the workspace version.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod arch;
mod release;

fn main() -> ExitCode {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("arch-check") => arch_check(),
        Some("release-check") => release_check(std::env::args().nth(2).as_deref()),
        _ => {
            eprintln!("usage: cargo xtask arch-check | cargo xtask release-check <tag>");
            ExitCode::from(2)
        }
    }
}

fn arch_check() -> ExitCode {
    let metadata = match cargo_metadata::MetadataCommand::new().exec() {
        Ok(metadata) => metadata,
        Err(error) => {
            eprintln!("cargo metadata failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let Some(core) = metadata
        .workspace_packages()
        .into_iter()
        .find(|package| package.name.as_str() == arch::CORE_CRATE)
    else {
        eprintln!("{} is not in the workspace", arch::CORE_CRATE);
        return ExitCode::FAILURE;
    };
    let normal_dependencies: Vec<String> = core
        .dependencies
        .iter()
        .filter(|dependency| dependency.kind == cargo_metadata::DependencyKind::Normal)
        .map(|dependency| dependency.name.clone())
        .collect();
    let core_src = core
        .manifest_path
        .parent()
        .map(|dir| dir.join("src").into_std_path_buf());
    let sources = core_src.map(|dir| read_sources(&dir)).unwrap_or_default();

    let mut violations = arch::forbidden_dependencies(&normal_dependencies);
    violations.extend(arch::forbidden_source_uses(&sources));
    if violations.is_empty() {
        println!(
            "arch-check: {} respects the dependency rule",
            arch::CORE_CRATE
        );
        ExitCode::SUCCESS
    } else {
        for violation in &violations {
            eprintln!("arch-check: {violation}");
        }
        ExitCode::FAILURE
    }
}

fn release_check(tag: Option<&str>) -> ExitCode {
    let Some(tag) = tag else {
        eprintln!("usage: cargo xtask release-check <tag>");
        return ExitCode::from(2);
    };
    let metadata = match cargo_metadata::MetadataCommand::new().no_deps().exec() {
        Ok(metadata) => metadata,
        Err(error) => {
            eprintln!("cargo metadata failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let Some(desktop) = metadata
        .workspace_packages()
        .into_iter()
        .find(|package| package.name.as_str() == "aurora-desktop")
    else {
        eprintln!("aurora-desktop is not in the workspace");
        return ExitCode::FAILURE;
    };
    let version = desktop.version.to_string();
    match release::check_tag(tag, &version) {
        Ok(kind) => {
            println!("release-check: {tag} is a {} of {version}", kind.label());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("release-check: {message}");
            ExitCode::FAILURE
        }
    }
}

fn read_sources(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut sources = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return sources;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources.extend(read_sources(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs")
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            sources.push((path, text));
        }
    }
    sources
}
