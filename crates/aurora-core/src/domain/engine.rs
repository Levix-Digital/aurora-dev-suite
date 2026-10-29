use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The three Levix engines Aurora reads, each through its Rust binary (ADR-0006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EngineKind {
    DevCycle,
    Gedo,
    AnyContext,
}

impl EngineKind {
    pub const ALL: [EngineKind; 3] = [
        EngineKind::DevCycle,
        EngineKind::Gedo,
        EngineKind::AnyContext,
    ];

    /// The command name of the engine's binary.
    pub const fn command(self) -> &'static str {
        match self {
            EngineKind::DevCycle => "dcp",
            EngineKind::Gedo => "gedo",
            EngineKind::AnyContext => "actx",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            EngineKind::DevCycle => "Dev Cycle Protocol",
            EngineKind::Gedo => "Gedo",
            EngineKind::AnyContext => "AnyContext",
        }
    }

    /// The `schema` value of the engine's version reply (contracts/engine-version.schema.json).
    pub const fn version_schema(self) -> &'static str {
        match self {
            EngineKind::DevCycle => "dcp.version",
            EngineKind::Gedo => "gedo.version",
            EngineKind::AnyContext => "actx.version",
        }
    }
}

impl fmt::Display for EngineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

/// A named ability an engine reports, with the MAJOR version of its JSON schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub name: String,
    pub schema_major: u32,
}

impl Capability {
    pub fn new(name: &str, schema_major: u32) -> Self {
        Self {
            name: name.to_owned(),
            schema_major,
        }
    }
}

/// What a probe found out about one engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum EngineStatus {
    /// Answered the version contract with a supported MAJOR.
    Ready,
    /// Found and answered `--version`, but not the version contract.
    InstalledWithoutContract,
    /// Found, but the request failed; `reason` says why and what to do.
    Degraded { reason: String },
    /// Not found on the search path nor at the configured location.
    Missing,
}

/// The result of probing one engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineReport {
    pub kind: EngineKind,
    pub status: EngineStatus,
    pub location: Option<PathBuf>,
    pub version: Option<String>,
    pub capabilities: Vec<Capability>,
    /// Standard error and the reason of the last failed request.
    pub last_error: Option<String>,
    /// RFC 3339 date-time.
    pub checked_at: String,
}

impl EngineReport {
    pub fn missing(kind: EngineKind, checked_at: &str) -> Self {
        Self {
            kind,
            status: EngineStatus::Missing,
            location: None,
            version: None,
            capabilities: Vec::new(),
            last_error: None,
            checked_at: checked_at.to_owned(),
        }
    }

    pub fn has_capability(&self, name: &str, schema_major: u32) -> bool {
        self.status == EngineStatus::Ready
            && self
                .capabilities
                .iter()
                .any(|c| c.name == name && c.schema_major == schema_major)
    }
}

/// Where to find one engine and how long to wait for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineSettings {
    /// A path that wins over the search path.
    pub path: Option<PathBuf>,
    pub timeout_ms: u64,
}

impl EngineSettings {
    pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            path: None,
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }
}
