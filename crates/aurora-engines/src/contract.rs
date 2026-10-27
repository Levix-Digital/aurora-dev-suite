//! The version reply every engine is asked for (contracts/engine-version.schema.json,
//! following Gedo's ADR-0013).

use aurora_core::domain::{Capability, EngineKind};
use serde::Deserialize;

/// The MAJOR version of the version reply this Aurora understands.
pub const SUPPORTED_MAJOR: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionReply {
    pub version: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContractError {
    /// The output is not a JSON object with `schema` and `schema_version`: the engine
    /// does not speak the contract.
    #[error("the reply is not a versioned JSON envelope")]
    NotAnEnvelope,
    #[error("the reply has schema `{found}`, expected `{expected}`")]
    WrongSchema { expected: String, found: String },
    #[error("the reply uses schema version {found}; this Aurora supports {SUPPORTED_MAJOR}")]
    UnsupportedMajor { found: u32 },
    #[error("the reply is invalid: {message}")]
    Invalid { message: String },
}

#[derive(Deserialize)]
struct Envelope {
    schema: String,
    schema_version: String,
}

#[derive(Deserialize)]
struct RawVersion {
    version: String,
    capabilities: Vec<RawCapability>,
}

#[derive(Deserialize)]
struct RawCapability {
    name: String,
    schema_major: u32,
}

pub fn parse_version_reply(kind: EngineKind, bytes: &[u8]) -> Result<VersionReply, ContractError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| ContractError::NotAnEnvelope)?;
    let envelope: Envelope =
        serde_json::from_value(value.clone()).map_err(|_| ContractError::NotAnEnvelope)?;
    if envelope.schema != kind.version_schema() {
        return Err(ContractError::WrongSchema {
            expected: kind.version_schema().to_owned(),
            found: envelope.schema,
        });
    }
    let major = schema_major(&envelope.schema_version)?;
    if major != SUPPORTED_MAJOR {
        return Err(ContractError::UnsupportedMajor { found: major });
    }
    let raw: RawVersion =
        serde_json::from_value(value).map_err(|error| ContractError::Invalid {
            message: error.to_string(),
        })?;
    Ok(VersionReply {
        version: raw.version,
        capabilities: raw
            .capabilities
            .into_iter()
            .map(|c| Capability {
                name: c.name,
                schema_major: c.schema_major,
            })
            .collect(),
    })
}

fn schema_major(schema_version: &str) -> Result<u32, ContractError> {
    schema_version
        .split('.')
        .next()
        .and_then(|major| major.parse().ok())
        .ok_or_else(|| ContractError::Invalid {
            message: format!("schema_version `{schema_version}` is not MAJOR.MINOR"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const READY: &str = r#"{"schema":"gedo.version","schema_version":"1.3","version":"0.5.0",
        "capabilities":[{"name":"radar","schema_major":1}],"extra":"ignored"}"#;

    #[test]
    fn parses_a_reply_and_ignores_unknown_fields() {
        let reply = parse_version_reply(EngineKind::Gedo, READY.as_bytes()).expect("valid");
        insta::assert_debug_snapshot!(reply);
    }

    #[test]
    fn plain_text_is_not_an_envelope() {
        assert_eq!(
            parse_version_reply(EngineKind::Gedo, b"gedo 0.3.0"),
            Err(ContractError::NotAnEnvelope)
        );
        assert_eq!(
            parse_version_reply(EngineKind::Gedo, br#"{"version":"1"}"#),
            Err(ContractError::NotAnEnvelope)
        );
    }

    #[test]
    fn another_engines_schema_is_refused() {
        assert!(matches!(
            parse_version_reply(EngineKind::AnyContext, READY.as_bytes()),
            Err(ContractError::WrongSchema { .. })
        ));
    }

    #[test]
    fn a_newer_major_is_unsupported() {
        let reply = br#"{"schema":"dcp.version","schema_version":"2.0","version":"1.0.0","capabilities":[]}"#;
        assert_eq!(
            parse_version_reply(EngineKind::DevCycle, reply),
            Err(ContractError::UnsupportedMajor { found: 2 })
        );
    }

    #[test]
    fn missing_fields_are_invalid() {
        let reply = br#"{"schema":"actx.version","schema_version":"1.0"}"#;
        assert!(matches!(
            parse_version_reply(EngineKind::AnyContext, reply),
            Err(ContractError::Invalid { .. })
        ));
    }
}
