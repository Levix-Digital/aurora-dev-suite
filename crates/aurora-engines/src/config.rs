//! `ConfigSource` over `dcp config get --json --dir <root>` (schema `dcp.config` MAJOR 1) and
//! `dcp config set --json [--dry-run] --dir <root> --revision <r> --value <key>=<JSON>...`
//! (schema `dcp.config-set` MAJOR 1), specs/007-project-settings/contracts. Aurora never
//! opens the configuration file itself (ADR-0006).

use std::path::Path;
use std::time::{Duration, Instant};

use aurora_core::domain::{
    ConfigChange, ConfigError, ConfigOutcome, ConfigSetting, ConfigType, ConfigValue, EngineReport,
    EngineSettings, FieldError, FieldErrorCode, KeyChange, ProjectConfig,
};
use aurora_core::ports::ConfigSource;
use serde::Deserialize;

use crate::dcp;
use crate::runner::CommandSpec;

/// The schema of the `config get` reply.
pub const CONFIG_SCHEMA: &str = "dcp.config";
/// The schema of the `config set` reply.
pub const CONFIG_SET_SCHEMA: &str = "dcp.config-set";
/// The MAJOR version of both replies this Aurora understands.
pub const SUPPORTED_MAJOR: u32 = 1;

/// Reads and changes settings through the `dcp` binary found by the latest engine probe.
#[derive(Debug, Default, Clone)]
pub struct DcpConfigSource;

impl DcpConfigSource {
    pub fn new() -> Self {
        Self
    }
}

fn spec(
    engine: &EngineReport,
    settings: &EngineSettings,
    args: Vec<String>,
) -> Result<CommandSpec, ConfigError> {
    let Some(program) = &engine.location else {
        return Err(ConfigError::Unavailable {
            hint: "the Dev Cycle Protocol (dcp) was not found".to_owned(),
        });
    };
    let mut spec = CommandSpec::new(program, &[], Duration::from_millis(settings.timeout_ms));
    spec.args = args;
    Ok(spec)
}

/// The arguments of `dcp config set` for `change`.
pub fn set_args(root: &Path, change: &ConfigChange, dry_run: bool) -> Vec<String> {
    let mut args: Vec<String> = ["config", "set", "--json"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    if dry_run {
        args.push("--dry-run".to_owned());
    }
    args.extend([
        "--dir".to_owned(),
        root.to_string_lossy().into_owned(),
        "--revision".to_owned(),
        change.revision.clone(),
    ]);
    for kv in &change.values {
        let json = serde_json::to_string(&kv.value).unwrap_or_else(|_| "null".to_owned());
        args.push("--value".to_owned());
        args.push(format!("{}={json}", kv.key));
    }
    args
}

impl ConfigSource for DcpConfigSource {
    fn read(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<ProjectConfig, ConfigError> {
        let args = vec![
            "config".to_owned(),
            "get".to_owned(),
            "--json".to_owned(),
            "--dir".to_owned(),
            root.to_string_lossy().into_owned(),
        ];
        let spec = spec(engine, settings, args)?;
        let started = Instant::now();
        let result = dcp::run(&spec)
            .and_then(|output| {
                parse_config_reply(&output.stdout, root, &crate::now_rfc3339())
                    .map_err(|error| dcp::with_stderr(error, output.stderr))
            })
            .map_err(ConfigError::from);
        let elapsed_ms = dcp::elapsed_ms(started);
        match &result {
            Ok(config) => tracing::info!(
                root = %root.display(),
                settings = config.settings.len(),
                elapsed_ms,
                "project settings read"
            ),
            Err(error) => {
                tracing::warn!(root = %root.display(), elapsed_ms, %error, "project settings failed");
            }
        }
        result
    }

    fn set(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
        change: &ConfigChange,
        dry_run: bool,
    ) -> Result<ConfigOutcome, ConfigError> {
        let spec = spec(engine, settings, set_args(root, change, dry_run))?;
        let started = Instant::now();
        let result = dcp::run(&spec)
            .map_err(ConfigError::from)
            .and_then(|output| {
                parse_set_reply(&output.stdout, root, dry_run).map_err(|error| match error {
                    ConfigError::Invalid { message, .. } => ConfigError::Invalid {
                        message,
                        stderr: output.stderr,
                    },
                    other => other,
                })
            });
        // Values are never logged: only how many keys and the outcome.
        let elapsed_ms = dcp::elapsed_ms(started);
        let keys = change.values.len();
        match &result {
            Ok(outcome) => tracing::info!(
                root = %root.display(),
                keys,
                dry_run,
                applied = outcome.applied,
                elapsed_ms,
                "project settings change"
            ),
            Err(error) => tracing::warn!(
                root = %root.display(),
                keys,
                dry_run,
                elapsed_ms,
                %error,
                "project settings change failed"
            ),
        }
        result
    }
}

/// Parses a `dcp.config` reply about `root`, read at `read_at`.
pub fn parse_config_reply(
    bytes: &[u8],
    root: &Path,
    read_at: &str,
) -> Result<ProjectConfig, aurora_core::domain::CycleError> {
    let value = dcp::envelope(bytes, CONFIG_SCHEMA, SUPPORTED_MAJOR)?;
    let raw: RawConfig =
        serde_json::from_value(value).map_err(|error| dcp::invalid(error.to_string()))?;
    dcp::check_root(&raw.root, root)?;
    let settings = raw
        .settings
        .into_iter()
        .map(RawSetting::into_setting)
        .collect::<Result<Vec<_>, String>>()
        .map_err(dcp::invalid)?;
    Ok(ProjectConfig {
        revision: raw.revision,
        source: raw.source,
        settings,
        findings: raw.findings,
        read_at: read_at.to_owned(),
    })
}

/// Parses a `dcp.config-set` reply about `root`; refusals become `Rejected` or `Stale`.
pub fn parse_set_reply(
    bytes: &[u8],
    root: &Path,
    dry_run: bool,
) -> Result<ConfigOutcome, ConfigError> {
    let value = dcp::envelope(bytes, CONFIG_SET_SCHEMA, SUPPORTED_MAJOR)?;
    let raw: RawSet =
        serde_json::from_value(value).map_err(|error| dcp::invalid(error.to_string()))?;
    dcp::check_root(&raw.root, root)?;
    if raw.dry_run != dry_run || (dry_run && raw.applied) {
        return Err(dcp::invalid("the reply does not match the dry run asked for").into());
    }
    if !raw.errors.is_empty() {
        if raw.applied {
            return Err(dcp::invalid("the reply has errors but says it was applied").into());
        }
        return Err(ConfigError::from_field_errors(
            raw.errors.into_iter().map(RawError::into_error).collect(),
        ));
    }
    Ok(ConfigOutcome {
        applied: raw.applied,
        revision: raw.revision,
        changes: raw
            .changes
            .into_iter()
            .map(|c| KeyChange {
                key: c.key,
                before: c.before,
                after: c.after,
            })
            .collect(),
        diff: raw.diff,
    })
}

#[derive(Deserialize)]
struct RawConfig {
    root: String,
    revision: String,
    source: Option<String>,
    settings: Vec<RawSetting>,
    findings: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum RawType {
    Enum,
    Boolean,
    Text,
    Integer,
    List,
}

#[derive(Deserialize)]
struct RawSetting {
    key: String,
    group: String,
    description: String,
    #[serde(rename = "type")]
    kind: RawType,
    value: Option<ConfigValue>,
    default: Option<ConfigValue>,
    allowed: Option<Vec<String>>,
    nullable: bool,
    read_only: bool,
    #[serde(default)]
    read_only_reason: Option<String>,
}

impl RawSetting {
    fn into_setting(self) -> Result<ConfigSetting, String> {
        let kind = match self.kind {
            RawType::Enum => ConfigType::Enum,
            RawType::Boolean => ConfigType::Boolean,
            RawType::Text => ConfigType::Text,
            RawType::Integer => ConfigType::Integer,
            RawType::List => ConfigType::List,
        };
        let setting = ConfigSetting {
            key: self.key,
            group: self.group,
            description: self.description,
            kind,
            value: self.value,
            default: self.default,
            allowed: self.allowed,
            nullable: self.nullable,
            read_only: false,
            read_only_reason: self.read_only_reason,
        };
        // The value must fit its own description, or the form cannot show it.
        if let Some(value) = &setting.value
            && let Err(error) = setting.check(Some(value))
        {
            return Err(format!(
                "the setting `{}` has a value that does not fit it: {}",
                setting.key, error.message
            ));
        }
        Ok(ConfigSetting {
            read_only: self.read_only,
            ..setting
        })
    }
}

#[derive(Deserialize)]
struct RawSet {
    root: String,
    dry_run: bool,
    applied: bool,
    revision: String,
    changes: Vec<RawChange>,
    diff: String,
    errors: Vec<RawError>,
}

#[derive(Deserialize)]
struct RawChange {
    key: String,
    before: Option<ConfigValue>,
    after: Option<ConfigValue>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawCode {
    UnknownKey,
    ReadOnly,
    InvalidValue,
    StaleRevision,
    InvalidFile,
}

#[derive(Deserialize)]
struct RawError {
    code: RawCode,
    key: Option<String>,
    message: String,
    #[serde(default)]
    accepted: Vec<String>,
}

impl RawError {
    fn into_error(self) -> FieldError {
        FieldError {
            code: match self.code {
                RawCode::UnknownKey => FieldErrorCode::UnknownKey,
                RawCode::ReadOnly => FieldErrorCode::ReadOnly,
                RawCode::InvalidValue => FieldErrorCode::InvalidValue,
                RawCode::StaleRevision => FieldErrorCode::StaleRevision,
                RawCode::InvalidFile => FieldErrorCode::InvalidFile,
            },
            key: self.key,
            message: self.message,
            accepted: self.accepted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::domain::{CycleError, KeyValue};

    const ROOT: &str = "/repos/gedo";

    fn get_reply(settings: &str) -> String {
        format!(
            r#"{{"schema":"dcp.config","schema_version":"1.0","root":"{ROOT}","revision":"r1",
            "source":".dev-cycle.json","settings":[{settings}],"findings":[]}}"#
        )
    }

    const BRANCHING: &str = r#"{"key":"devops.branching","group":"devops","description":"d",
        "type":"enum","value":"trunk","default":"integration",
        "allowed":["integration","trunk"],"nullable":false,"read_only":false}"#;

    fn parse_get(text: &str) -> Result<ProjectConfig, CycleError> {
        parse_config_reply(text.as_bytes(), Path::new(ROOT), "2026-10-31T00:00:00Z")
    }

    fn set_reply(dry_run: bool, applied: bool, errors: &str) -> String {
        format!(
            r#"{{"schema":"dcp.config-set","schema_version":"1.1","root":"{ROOT}",
            "dry_run":{dry_run},"applied":{applied},"revision":"r2",
            "changes":[{{"key":"devops.branching","before":"trunk","after":"integration"}}],
            "diff":"-a\n+b","errors":[{errors}]}}"#
        )
    }

    #[test]
    fn a_get_reply_reads_into_settings() {
        let config = parse_get(&get_reply(BRANCHING)).expect("valid");
        assert_eq!(config.revision, "r1");
        let setting = &config.settings[0];
        assert_eq!(setting.kind, ConfigType::Enum);
        assert_eq!(setting.value, Some(ConfigValue::Text("trunk".into())));
    }

    #[test]
    fn a_value_that_does_not_fit_its_setting_is_invalid() {
        let wrong = BRANCHING.replace(r#""value":"trunk""#, r#""value":"gitflow""#);
        assert!(matches!(
            parse_get(&get_reply(&wrong)),
            Err(CycleError::Invalid { .. })
        ));
        let unknown_type = BRANCHING.replace(r#""type":"enum""#, r#""type":"map""#);
        assert!(parse_get(&get_reply(&unknown_type)).is_err());
    }

    #[test]
    fn other_schemas_roots_and_majors_are_refused() {
        assert!(parse_get(&get_reply(BRANCHING).replace("dcp.config", "dcp.diagnose")).is_err());
        assert!(parse_get(&get_reply(BRANCHING).replace(ROOT, "/repos/other")).is_err());
        assert!(matches!(
            parse_get(&get_reply(BRANCHING).replace(r#""1.0""#, r#""2.0""#)),
            Err(CycleError::Unsupported { found: 2, .. })
        ));
    }

    #[test]
    fn a_set_reply_reads_into_an_outcome() {
        let outcome = parse_set_reply(set_reply(true, false, "").as_bytes(), Path::new(ROOT), true)
            .expect("valid");
        assert!(!outcome.applied);
        assert_eq!(
            outcome.changes[0].after,
            Some(ConfigValue::Text("integration".into()))
        );
        assert!(
            parse_set_reply(
                set_reply(false, true, "").as_bytes(),
                Path::new(ROOT),
                false
            )
            .expect("valid")
            .applied
        );
    }

    #[test]
    fn a_reply_that_does_not_match_the_request_is_invalid() {
        let applied_dry_run = set_reply(true, true, "");
        assert!(matches!(
            parse_set_reply(applied_dry_run.as_bytes(), Path::new(ROOT), true),
            Err(ConfigError::Invalid { .. })
        ));
        let not_dry = set_reply(false, false, "");
        assert!(matches!(
            parse_set_reply(not_dry.as_bytes(), Path::new(ROOT), true),
            Err(ConfigError::Invalid { .. })
        ));
    }

    #[test]
    fn refusals_become_rejected_or_stale() {
        let refused = set_reply(
            true,
            false,
            r#"{"code":"invalid_value","key":"devops.branching","message":"no","accepted":["trunk"]}"#,
        );
        let Err(ConfigError::Rejected { errors }) =
            parse_set_reply(refused.as_bytes(), Path::new(ROOT), true)
        else {
            panic!("expected a refusal");
        };
        assert_eq!(errors[0].accepted, ["trunk"]);
        let stale = set_reply(
            false,
            false,
            r#"{"code":"stale_revision","key":null,"message":"changed"}"#,
        );
        assert_eq!(
            parse_set_reply(stale.as_bytes(), Path::new(ROOT), false),
            Err(ConfigError::Stale)
        );
    }

    #[test]
    fn set_arguments_carry_json_values_and_the_revision() {
        let change = ConfigChange {
            revision: "r1".into(),
            values: vec![
                KeyValue {
                    key: "devops.release_candidates".into(),
                    value: Some(ConfigValue::Boolean(false)),
                },
                KeyValue {
                    key: "architecture.style".into(),
                    value: None,
                },
                KeyValue {
                    key: "devops.version_files".into(),
                    value: Some(ConfigValue::List(vec!["Cargo.toml".into(), "a b".into()])),
                },
            ],
        };
        assert_eq!(
            set_args(Path::new("/repos/my repo"), &change, true),
            [
                "config",
                "set",
                "--json",
                "--dry-run",
                "--dir",
                "/repos/my repo",
                "--revision",
                "r1",
                "--value",
                "devops.release_candidates=false",
                "--value",
                "architecture.style=null",
                "--value",
                r#"devops.version_files=["Cargo.toml","a b"]"#,
            ]
        );
        assert!(!set_args(Path::new(ROOT), &change, false).contains(&"--dry-run".to_owned()));
    }
}
