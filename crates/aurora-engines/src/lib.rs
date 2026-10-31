//! The driven adapters of Aurora Dev Suite: the engine command runner and probe, and the
//! project registry and settings files (ADR-0003, ADR-0004, ADR-0006).

pub mod config;
pub mod contract;
pub mod cycle;
pub mod dcp;
pub mod probe;
pub mod registry;
pub mod repo;
pub mod runner;

pub use config::DcpConfigSource;
pub use cycle::DcpCycleSource;
pub use probe::CliProbe;
pub use registry::{TomlRegistry, TomlSettings};

/// The current time as RFC 3339, for `added_at` and `checked_at`.
pub(crate) fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}
