//! `dcp-integrations`: messages between projects over each repository's orphan
//! `aurora/inbox` branch (ADR-0007, ADR-0009, ADR-0010).
//!
//! `format` reads and writes message files, `rules` holds the conversation rules,
//! `registry` the project's `.aurora/integrations.json`, `transport` the Git
//! plumbing on the inbox branch and `commands` every command of the binary.

pub mod commands;
pub mod format;
pub mod registry;
pub mod rules;
pub mod transport;

/// The provider's version, the workspace version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
