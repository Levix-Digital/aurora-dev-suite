//! A project's Dev Cycle settings, read with `dcp config get` and changed with
//! `dcp config set` (FEAT-027, spec 007). Aurora never reads or writes the file itself.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{CycleError, UpdateSide};

/// The kind of value a setting holds, which decides its control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigType {
    Enum,
    Boolean,
    Text,
    Integer,
    List,
}

/// A setting's value; an unset value is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
    List(Vec<String>),
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Boolean(true) => f.write_str("on"),
            ConfigValue::Boolean(false) => f.write_str("off"),
            ConfigValue::Integer(number) => write!(f, "{number}"),
            ConfigValue::Text(text) => f.write_str(text),
            ConfigValue::List(items) if items.is_empty() => f.write_str("(empty)"),
            ConfigValue::List(items) => f.write_str(&items.join(", ")),
        }
    }
}

/// A value as shown to the user: "Not set" for `None`.
pub fn value_text(value: Option<&ConfigValue>) -> String {
    value.map_or_else(|| "Not set".to_owned(), ToString::to_string)
}

/// Why `dcp` or the core refused part of a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldErrorCode {
    UnknownKey,
    ReadOnly,
    InvalidValue,
    StaleRevision,
    InvalidFile,
}

/// One refusal, about a key or (without one) about the whole change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    pub code: FieldErrorCode,
    pub key: Option<String>,
    pub message: String,
    /// The accepted values, when the value was outside them.
    pub accepted: Vec<String>,
}

impl FieldError {
    fn about(key: &str, code: FieldErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            key: Some(key.to_owned()),
            message: message.into(),
            accepted: Vec::new(),
        }
    }
}

/// One setting of the project, as `dcp config get` describes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSetting {
    /// Dotted path in the file, for example `devops.branching`.
    pub key: String,
    pub group: String,
    pub description: String,
    pub kind: ConfigType,
    pub value: Option<ConfigValue>,
    pub default: Option<ConfigValue>,
    /// The accepted values of an enum, or of each item of a list.
    pub allowed: Option<Vec<String>>,
    pub nullable: bool,
    pub read_only: bool,
    pub read_only_reason: Option<String>,
}

impl ConfigSetting {
    /// Whether `value` may be sent for this setting (spec 007 FR-006).
    pub fn check(&self, value: Option<&ConfigValue>) -> Result<(), FieldError> {
        let key = self.key.as_str();
        if self.read_only {
            return Err(FieldError::about(
                key,
                FieldErrorCode::ReadOnly,
                self.read_only_reason
                    .clone()
                    .unwrap_or_else(|| "this setting cannot be changed".to_owned()),
            ));
        }
        let Some(value) = value else {
            return if self.nullable {
                Ok(())
            } else {
                Err(FieldError::about(
                    key,
                    FieldErrorCode::InvalidValue,
                    "a value is required",
                ))
            };
        };
        let outside = |item: &str| {
            self.allowed
                .as_ref()
                .is_some_and(|allowed| !allowed.iter().any(|a| a == item))
        };
        let refused = |message: String| FieldError {
            accepted: self.allowed.clone().unwrap_or_default(),
            ..FieldError::about(key, FieldErrorCode::InvalidValue, message)
        };
        match (self.kind, value) {
            (ConfigType::Boolean, ConfigValue::Boolean(_))
            | (ConfigType::Integer, ConfigValue::Integer(_)) => Ok(()),
            (ConfigType::Enum | ConfigType::Text, ConfigValue::Text(text)) => {
                if outside(text) {
                    Err(refused(format!(
                        "`{text}` is not one of the accepted values"
                    )))
                } else {
                    Ok(())
                }
            }
            (ConfigType::List, ConfigValue::List(items)) => {
                match items.iter().find(|item| outside(item)) {
                    Some(item) => Err(refused(format!(
                        "`{item}` is not one of the accepted values"
                    ))),
                    None => Ok(()),
                }
            }
            _ => Err(FieldError::about(
                key,
                FieldErrorCode::InvalidValue,
                format!("expected {}", self.kind.expected()),
            )),
        }
    }

    /// The value of a text, integer or free list field; empty text is "not set" when the
    /// setting allows it. Lists take one value per line, empty lines dropped.
    pub fn parse_input(&self, input: &str) -> Result<Option<ConfigValue>, FieldError> {
        let trimmed = input.trim();
        let value = match self.kind {
            ConfigType::List => Some(ConfigValue::List(
                input
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_owned)
                    .collect(),
            )),
            _ if trimmed.is_empty() && self.nullable => None,
            ConfigType::Integer => Some(ConfigValue::Integer(trimmed.parse().map_err(|_| {
                FieldError::about(
                    &self.key,
                    FieldErrorCode::InvalidValue,
                    "expected a whole number",
                )
            })?)),
            ConfigType::Boolean => match trimmed {
                "true" | "on" => Some(ConfigValue::Boolean(true)),
                "false" | "off" => Some(ConfigValue::Boolean(false)),
                _ => {
                    return Err(FieldError::about(
                        &self.key,
                        FieldErrorCode::InvalidValue,
                        "expected on or off",
                    ));
                }
            },
            ConfigType::Enum | ConfigType::Text => Some(ConfigValue::Text(trimmed.to_owned())),
        };
        self.check(value.as_ref())?;
        Ok(value)
    }
}

impl ConfigType {
    const fn expected(self) -> &'static str {
        match self {
            ConfigType::Enum => "one of the accepted values",
            ConfigType::Boolean => "on or off",
            ConfigType::Text => "text",
            ConfigType::Integer => "a whole number",
            ConfigType::List => "a list of values",
        }
    }
}

/// A project's settings, read through `dcp config get --json` (ADR-0006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    /// Opaque token of the file read; a change must send it back.
    pub revision: String,
    /// The file read, relative to the project root, or `None` when only defaults apply.
    pub source: Option<String>,
    pub settings: Vec<ConfigSetting>,
    /// Configuration problems the engine found.
    pub findings: Vec<String>,
    /// RFC 3339 date-time.
    pub read_at: String,
}

impl ProjectConfig {
    pub fn setting(&self, key: &str) -> Option<&ConfigSetting> {
        self.settings.iter().find(|setting| setting.key == key)
    }

    /// The settings by group, groups in the order of their first setting.
    pub fn groups(&self) -> Vec<(String, Vec<&ConfigSetting>)> {
        let mut groups: Vec<(String, Vec<&ConfigSetting>)> = Vec::new();
        for setting in &self.settings {
            match groups.iter_mut().find(|(name, _)| *name == setting.group) {
                Some((_, members)) => members.push(setting),
                None => groups.push((setting.group.clone(), vec![setting])),
            }
        }
        groups
    }
}

/// The user's edits over one reading of the settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDraft {
    edits: BTreeMap<String, Option<ConfigValue>>,
}

impl ConfigDraft {
    /// Records `value` for `key`; a value equal to the current one is no edit.
    pub fn set(&mut self, config: &ProjectConfig, key: &str, value: Option<ConfigValue>) {
        let current = config.setting(key).map(|setting| &setting.value);
        if current == Some(&value) {
            self.edits.remove(key);
        } else {
            self.edits.insert(key.to_owned(), value);
        }
    }

    pub fn count(&self) -> usize {
        self.edits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    pub fn is_edited(&self, key: &str) -> bool {
        self.edits.contains_key(key)
    }

    /// The edited value, else the current one.
    pub fn value_of(&self, config: &ProjectConfig, key: &str) -> Option<ConfigValue> {
        match self.edits.get(key) {
            Some(value) => value.clone(),
            None => config
                .setting(key)
                .and_then(|setting| setting.value.clone()),
        }
    }

    /// The same edits over a new reading of the settings: edits equal to the new values
    /// are dropped, the others kept.
    pub fn rebase(&self, config: &ProjectConfig) -> ConfigDraft {
        let mut draft = ConfigDraft::default();
        for (key, value) in &self.edits {
            draft.set(config, key, value.clone());
        }
        draft
    }

    /// The change to send, in the order of the settings, after the core's checks.
    pub fn change(&self, config: &ProjectConfig) -> Result<ConfigChange, Vec<FieldError>> {
        let mut errors: Vec<FieldError> = self
            .edits
            .keys()
            .filter(|key| config.setting(key).is_none())
            .map(|key| FieldError::about(key, FieldErrorCode::UnknownKey, "unknown setting"))
            .collect();
        let mut values = Vec::new();
        for setting in &config.settings {
            let Some(value) = self.edits.get(&setting.key) else {
                continue;
            };
            match setting.check(value.as_ref()) {
                Ok(()) => values.push(KeyValue {
                    key: setting.key.clone(),
                    value: value.clone(),
                }),
                Err(error) => errors.push(error),
            }
        }
        if errors.is_empty() {
            Ok(ConfigChange {
                revision: config.revision.clone(),
                values,
            })
        } else {
            Err(errors)
        }
    }
}

/// One key and its new value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValue {
    pub key: String,
    pub value: Option<ConfigValue>,
}

/// What is sent to `dcp config set`: the revision read and the edited keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigChange {
    pub revision: String,
    pub values: Vec<KeyValue>,
}

/// One key's value before and after.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyChange {
    pub key: String,
    pub before: Option<ConfigValue>,
    pub after: Option<ConfigValue>,
}

/// The answer of `dcp config set`, previewed or applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigOutcome {
    pub applied: bool,
    /// The file's revision after the command.
    pub revision: String,
    pub changes: Vec<KeyChange>,
    /// Unified diff of the file.
    pub diff: String,
}

/// Why a project's settings could not be read or changed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ConfigError {
    #[error("project settings are unavailable: {hint}")]
    Unavailable { hint: String },
    #[error("the project's folder {path} is missing")]
    ProjectMissing { path: PathBuf },
    #[error("the Dev Cycle Protocol did not answer within {} s", *timeout_ms as f64 / 1000.0)]
    TimedOut { timeout_ms: u64, stderr: String },
    #[error("the Dev Cycle Protocol's reply is larger than {limit} bytes")]
    TooLarge { limit: usize, stderr: String },
    #[error("the Dev Cycle Protocol failed{}", exit_code.map(|code| format!(" with exit code {code}")).unwrap_or_default())]
    Failed {
        exit_code: Option<i32>,
        stderr: String,
    },
    #[error("the Dev Cycle Protocol's reply is invalid: {message}")]
    Invalid { message: String, stderr: String },
    #[error("the Dev Cycle Protocol's reply uses schema version {found}: {}", update.advice())]
    Unsupported { found: u32, update: UpdateSide },
    #[error("{}", rejected_text(errors))]
    Rejected { errors: Vec<FieldError> },
    #[error("the settings changed on disk since they were read; read them again")]
    Stale,
    #[error("nothing to change: every value is already the one in the file")]
    NothingToChange,
}

fn rejected_text(errors: &[FieldError]) -> String {
    match errors {
        [one] => format!("the change was refused: {}", one.message),
        _ => format!("the change was refused: {} problems", errors.len()),
    }
}

impl ConfigError {
    /// `Stale` when the file changed meanwhile, else `Rejected` with the errors.
    pub fn from_field_errors(errors: Vec<FieldError>) -> Self {
        if errors
            .iter()
            .any(|error| error.code == FieldErrorCode::StaleRevision)
        {
            ConfigError::Stale
        } else {
            ConfigError::Rejected { errors }
        }
    }

    /// The engine's error output kept for the user, if any.
    pub fn stderr(&self) -> Option<&str> {
        match self {
            ConfigError::TimedOut { stderr, .. }
            | ConfigError::TooLarge { stderr, .. }
            | ConfigError::Failed { stderr, .. }
            | ConfigError::Invalid { stderr, .. } => Some(stderr).filter(|s| !s.is_empty()),
            _ => None,
        }
        .map(String::as_str)
    }

    /// The refusals about one key.
    pub fn errors_for<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a FieldError> + 'a {
        let errors: &[FieldError] = match self {
            ConfigError::Rejected { errors } => errors,
            _ => &[],
        };
        errors
            .iter()
            .filter(move |error| error.key.as_deref() == Some(key))
    }
}

impl From<CycleError> for ConfigError {
    fn from(error: CycleError) -> Self {
        match error {
            CycleError::Unavailable { hint } => ConfigError::Unavailable { hint },
            CycleError::ProjectMissing { path } => ConfigError::ProjectMissing { path },
            CycleError::TimedOut { timeout_ms, stderr } => {
                ConfigError::TimedOut { timeout_ms, stderr }
            }
            CycleError::TooLarge { limit, stderr } => ConfigError::TooLarge { limit, stderr },
            CycleError::Failed { exit_code, stderr } => ConfigError::Failed { exit_code, stderr },
            CycleError::Invalid { message, stderr } => ConfigError::Invalid { message, stderr },
            CycleError::Unsupported { found, update } => ConfigError::Unsupported { found, update },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting(key: &str, kind: ConfigType, value: Option<ConfigValue>) -> ConfigSetting {
        ConfigSetting {
            key: key.to_owned(),
            group: key.split('.').next().unwrap_or(key).to_owned(),
            description: String::new(),
            kind,
            value,
            default: None,
            allowed: None,
            nullable: false,
            read_only: false,
            read_only_reason: None,
        }
    }

    fn text(value: &str) -> Option<ConfigValue> {
        Some(ConfigValue::Text(value.to_owned()))
    }

    fn branching() -> ConfigSetting {
        ConfigSetting {
            allowed: Some(vec![
                "integration".into(),
                "trunk".into(),
                "release-branches".into(),
            ]),
            ..setting("devops.branching", ConfigType::Enum, text("trunk"))
        }
    }

    fn config() -> ProjectConfig {
        ProjectConfig {
            revision: "r1".into(),
            source: Some(".dev-cycle.json".into()),
            settings: vec![
                branching(),
                setting(
                    "devops.release_candidates",
                    ConfigType::Boolean,
                    Some(ConfigValue::Boolean(true)),
                ),
                ConfigSetting {
                    nullable: true,
                    ..setting("architecture.style", ConfigType::Text, None)
                },
                setting(
                    "docs.readme_max_lines",
                    ConfigType::Integer,
                    Some(ConfigValue::Integer(400)),
                ),
                ConfigSetting {
                    allowed: Some(vec!["critical".into(), "high".into(), "low".into()]),
                    ..setting(
                        "backlog.blocking_severities",
                        ConfigType::List,
                        Some(ConfigValue::List(vec!["critical".into()])),
                    )
                },
                ConfigSetting {
                    read_only: true,
                    read_only_reason: Some("mandatory gate".into()),
                    ..setting(
                        "principles.gates",
                        ConfigType::List,
                        Some(ConfigValue::List(vec![])),
                    )
                },
            ],
            findings: vec![],
            read_at: "2026-10-31T00:00:00Z".into(),
        }
    }

    #[test]
    fn checks_types_allowed_values_null_and_read_only() {
        let config = config();
        let get = |key| config.setting(key).expect("known");
        assert!(
            get("devops.branching")
                .check(text("integration").as_ref())
                .is_ok()
        );
        let outside = get("devops.branching")
            .check(text("gitflow").as_ref())
            .expect_err("outside");
        assert_eq!(outside.code, FieldErrorCode::InvalidValue);
        assert_eq!(outside.accepted.len(), 3);
        assert!(get("devops.branching").check(None).is_err());
        assert!(get("architecture.style").check(None).is_ok());
        assert!(
            get("devops.release_candidates")
                .check(text("yes").as_ref())
                .is_err()
        );
        let list = get("backlog.blocking_severities");
        assert!(
            list.check(Some(&ConfigValue::List(vec!["high".into()])))
                .is_ok()
        );
        assert!(
            list.check(Some(&ConfigValue::List(vec!["urgent".into()])))
                .is_err()
        );
        let read_only = get("principles.gates")
            .check(Some(&ConfigValue::List(vec![])))
            .expect_err("read-only");
        assert_eq!(read_only.code, FieldErrorCode::ReadOnly);
        assert_eq!(read_only.message, "mandatory gate");
    }

    #[test]
    fn parses_the_input_of_each_type() {
        let config = config();
        let get = |key| config.setting(key).expect("known");
        assert_eq!(
            get("docs.readme_max_lines").parse_input(" 250 "),
            Ok(Some(ConfigValue::Integer(250)))
        );
        assert_eq!(
            get("docs.readme_max_lines")
                .parse_input("abc")
                .expect_err("not a number")
                .message,
            "expected a whole number"
        );
        assert_eq!(get("architecture.style").parse_input("  "), Ok(None));
        assert_eq!(
            get("architecture.style").parse_input("layered"),
            Ok(text("layered"))
        );
        assert_eq!(
            get("backlog.blocking_severities").parse_input("critical\n\n high \n"),
            Ok(Some(ConfigValue::List(vec![
                "critical".into(),
                "high".into()
            ])))
        );
        assert!(get("devops.branching").parse_input("gitflow").is_err());
    }

    #[test]
    fn an_edit_back_to_the_current_value_is_no_edit() {
        let config = config();
        let mut draft = ConfigDraft::default();
        draft.set(&config, "devops.branching", text("integration"));
        assert!(draft.is_edited("devops.branching"));
        assert_eq!(
            draft.value_of(&config, "devops.branching"),
            text("integration")
        );
        draft.set(&config, "devops.branching", text("trunk"));
        assert!(draft.is_empty());
        assert_eq!(draft.value_of(&config, "devops.branching"), text("trunk"));
    }

    #[test]
    fn a_rebase_keeps_the_edits_that_still_change_something() {
        let config = config();
        let mut draft = ConfigDraft::default();
        draft.set(&config, "devops.branching", text("integration"));
        draft.set(
            &config,
            "docs.readme_max_lines",
            Some(ConfigValue::Integer(300)),
        );
        let mut newer = config.clone();
        newer.revision = "r2".into();
        newer.settings[0].value = text("integration");
        let rebased = draft.rebase(&newer);
        assert_eq!(rebased.count(), 1);
        assert!(rebased.is_edited("docs.readme_max_lines"));
        assert_eq!(rebased.change(&newer).expect("valid").revision, "r2");
    }

    #[test]
    fn the_change_follows_the_settings_order_and_carries_the_revision() {
        let config = config();
        let mut draft = ConfigDraft::default();
        draft.set(
            &config,
            "docs.readme_max_lines",
            Some(ConfigValue::Integer(300)),
        );
        draft.set(&config, "devops.branching", text("integration"));
        let change = draft.change(&config).expect("valid");
        assert_eq!(change.revision, "r1");
        let keys: Vec<&str> = change.values.iter().map(|v| v.key.as_str()).collect();
        assert_eq!(keys, ["devops.branching", "docs.readme_max_lines"]);
        assert_eq!(draft.count(), 2);
    }

    #[test]
    fn the_change_refuses_unknown_read_only_and_invalid_values() {
        let config = config();
        let mut draft = ConfigDraft::default();
        draft.set(&config, "nope", text("x"));
        draft.set(
            &config,
            "principles.gates",
            Some(ConfigValue::List(vec!["x".into()])),
        );
        draft.set(&config, "devops.branching", text("gitflow"));
        let errors = draft.change(&config).expect_err("refused");
        let codes: Vec<FieldErrorCode> = errors.iter().map(|e| e.code).collect();
        assert_eq!(
            codes,
            [
                FieldErrorCode::UnknownKey,
                FieldErrorCode::InvalidValue,
                FieldErrorCode::ReadOnly
            ]
        );
    }

    #[test]
    fn groups_keep_the_order_of_their_first_setting() {
        let mut config = config();
        config.settings.push(setting(
            "devops.production_branch",
            ConfigType::Text,
            text("main"),
        ));
        let groups: Vec<(String, usize)> = config
            .groups()
            .into_iter()
            .map(|(name, members)| (name, members.len()))
            .collect();
        assert_eq!(
            groups,
            [
                ("devops".to_owned(), 3),
                ("architecture".to_owned(), 1),
                ("docs".to_owned(), 1),
                ("backlog".to_owned(), 1),
                ("principles".to_owned(), 1)
            ]
        );
    }

    #[test]
    fn a_stale_revision_wins_over_other_refusals() {
        let refusal = |code| FieldError {
            code,
            key: None,
            message: "m".into(),
            accepted: vec![],
        };
        assert_eq!(
            ConfigError::from_field_errors(vec![
                refusal(FieldErrorCode::InvalidValue),
                refusal(FieldErrorCode::StaleRevision)
            ]),
            ConfigError::Stale
        );
        let rejected = ConfigError::from_field_errors(vec![FieldError {
            key: Some("devops.branching".into()),
            ..refusal(FieldErrorCode::InvalidValue)
        }]);
        assert_eq!(rejected.errors_for("devops.branching").count(), 1);
        assert_eq!(rejected.errors_for("other").count(), 0);
        assert_eq!(rejected.to_string(), "the change was refused: m");
    }

    #[test]
    fn values_read_as_the_user_sees_them() {
        assert_eq!(value_text(None), "Not set");
        assert_eq!(value_text(Some(&ConfigValue::Boolean(false))), "off");
        assert_eq!(
            value_text(Some(&ConfigValue::List(vec!["a".into(), "b".into()]))),
            "a, b"
        );
        assert_eq!(value_text(Some(&ConfigValue::List(vec![]))), "(empty)");
    }

    #[test]
    fn values_and_errors_serialize_as_the_contract_and_the_interface_expect() {
        let json = serde_json::to_value(KeyValue {
            key: "k".into(),
            value: Some(ConfigValue::List(vec!["a".into()])),
        })
        .expect("serializes");
        assert_eq!(json["value"], serde_json::json!(["a"]));
        let back: Option<ConfigValue> = serde_json::from_str("400").expect("parses");
        assert_eq!(back, Some(ConfigValue::Integer(400)));
        let error = serde_json::to_value(ConfigError::Stale).expect("serializes");
        assert_eq!(error["kind"], "stale");
        let from_cycle = ConfigError::from(CycleError::Failed {
            exit_code: Some(2),
            stderr: "boom".into(),
        });
        assert_eq!(from_cycle.stderr(), Some("boom"));
    }
}
