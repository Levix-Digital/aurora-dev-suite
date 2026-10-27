//! The dependency rule of ADR-0001, as pure checks over a dependency list and source texts.

use std::path::PathBuf;

pub const CORE_CRATE: &str = "aurora-core";

/// Crates the core must never depend on, as exact names or prefixes ending in `*`.
const FORBIDDEN_DEPENDENCIES: &[&str] = &[
    "tauri*",
    "leptos*",
    "aurora-engines",
    "aurora-desktop",
    "aurora-ui",
    "tokio",
    "wait-timeout",
    "which",
];

/// Paths the core's sources must never use.
const FORBIDDEN_PATHS: &[&str] = &["std::process", "std::fs", "std::net"];

pub fn forbidden_dependencies(dependencies: &[String]) -> Vec<String> {
    dependencies
        .iter()
        .filter(|name| {
            FORBIDDEN_DEPENDENCIES
                .iter()
                .any(|rule| matches_rule(rule, name))
        })
        .map(|name| format!("{CORE_CRATE} depends on forbidden crate `{name}`"))
        .collect()
}

pub fn forbidden_source_uses(sources: &[(PathBuf, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    for (path, text) in sources {
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or_default();
            for forbidden in FORBIDDEN_PATHS {
                if code.contains(forbidden) {
                    violations.push(format!(
                        "{}:{} uses `{forbidden}`",
                        path.display(),
                        index + 1
                    ));
                }
            }
        }
    }
    violations
}

fn matches_rule(rule: &str, name: &str) -> bool {
    match rule.strip_suffix('*') {
        Some(prefix) => name.starts_with(prefix),
        None => name == rule,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn allows_plain_libraries() {
        assert!(forbidden_dependencies(&names(&["serde", "thiserror"])).is_empty());
    }

    #[test]
    fn rejects_frameworks_and_adapters() {
        let violations = forbidden_dependencies(&names(&[
            "serde",
            "tauri-plugin-dialog",
            "leptos",
            "aurora-engines",
        ]));
        assert_eq!(violations.len(), 3);
        assert!(violations[0].contains("tauri-plugin-dialog"));
    }

    #[test]
    fn rejects_process_use_but_not_in_comments() {
        let sources = vec![
            (
                PathBuf::from("a.rs"),
                "use std::process::Command;\n".to_owned(),
            ),
            (
                PathBuf::from("b.rs"),
                "// std::process is forbidden here\n".to_owned(),
            ),
        ];
        let violations = forbidden_source_uses(&sources);
        assert_eq!(violations, vec!["a.rs:1 uses `std::process`".to_owned()]);
    }
}
