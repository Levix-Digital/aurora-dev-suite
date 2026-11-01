# Data Model: Project settings

The core types live in `aurora-core` (`domain/config.rs`) and come only from `dcp config get` and `dcp config set` (contracts/); nothing is stored by Aurora (ADR-0004).

## Values

- `ConfigType`: `Enum`, `Boolean`, `Text`, `Integer`, `List`.
- `ConfigValue` (serialized untagged): `Boolean(bool)`, `Integer(i64)`, `Text(String)`, `List(Vec<String>)`. An unset value is `None` (JSON `null`).

## Setting and project settings

`ConfigSetting { key, group, description, kind, set, value: Option<ConfigValue>, default: Option<ConfigValue>, allowed: Option<Vec<String>>, nullable, read_only, read_only_reason: Option<String> }`. `set` says whether the key is written in the file; otherwise `value` is a default or detected value. `ConfigType` has `Other` for a type this Aurora does not know, and `ConfigValue` has `Json` for any other JSON value (structured read-only settings); both are shown as they are and never edited (`editable()` is false, as for `read_only`).

- `check(&Option<ConfigValue>) -> Result<(), FieldError>`: not editable, type, allowed values (FR-006). `None` removes the key from the file and is accepted for any editable setting.
- `parse_input(&str) -> Result<Option<ConfigValue>, FieldError>`: the text of a text, integer or free list field (one value per line, empty lines dropped; empty text is `None` when nullable, refused otherwise).

`ProjectConfig { revision, source: Option<String>, settings: Vec<ConfigSetting>, findings: Vec<String>, read_at }`

- `groups()`: `(group, settings)` in the order of each group's first setting.
- `setting(key)`.

## Draft, change and outcome

`ConfigDraft { edits: BTreeMap<String, Option<ConfigValue>> }`: the user's edits over one `ProjectConfig`; `None` removes the key. Only what the user changed is kept: a value equal to the effective one is no edit, and removing a key that is not set is no edit, so the form never writes a default the user did not choose. `reset(config, key)` removes a set key; `is_reset(key)` says so, and `value_of` then shows the default.

- `set(config, key, value)`: records the edit, or drops it when the value equals the current one.
- `count()`, `is_edited(key)`, `value_of(config, key)` (the edit, else the current value).
- `change(config) -> Result<ConfigChange, Vec<FieldError>>`: the edits in the order of the settings, checked with `ConfigSetting::check`; unknown keys refused.

`ConfigChange { revision, values: Vec<KeyValue> }`, `KeyValue { key, value: Option<ConfigValue> }`.

`ConfigOutcome { applied, revision, changes: Vec<KeyChange>, diff }`, `KeyChange { key, before, after }` (values `Option<ConfigValue>`).

## Errors

`FieldError { code: FieldErrorCode, key: Option<String>, message, accepted: Vec<String> }`; `FieldErrorCode`: `UnknownKey`, `ReadOnly`, `InvalidValue`, `StaleRevision`, `InvalidFile`.

`ConfigError` (serialized with `kind`): `Unavailable { hint }`, `ProjectMissing { path }`, `TimedOut`, `TooLarge`, `Failed`, `Invalid`, `Unsupported` (as `CycleError`, converted with `From<CycleError>`), plus:

- `Rejected { errors }`: `dcp` or the core refused values; nothing written.
- `Stale`: the file changed since it was read (an error with code `stale_revision`).
- `NothingToChange`: the change is empty, or `dcp` found every value equal to the file.

`ConfigError::from_field_errors(errors)` gives `Stale` when any error is `StaleRevision`, else `Rejected`.

## Port

`ConfigSource` (`ports.rs`):

- `read(engine, settings, root) -> Result<ProjectConfig, ConfigError>`
- `set(engine, settings, root, change, dry_run) -> Result<ConfigOutcome, ConfigError>`: a reply with errors becomes `Err(ConfigError::from_field_errors(..))`.

## Capability and feature

`Feature::ProjectSettings` ("Project settings") needs `dcp` capability `config` (schema MAJOR 1), release v0.3.0. `capabilities::usable_dcp_for(feature, report)` generalizes `usable_dcp`.

## View state (interface only)

The Project view's tab (`Cycle` or `Settings`) in `AppContext`. The Settings tab: `Loading`, `Loaded { config, draft, input_errors, field_errors, step }` with `step` `Editing`, `Previewing(ConfigOutcome, ConfigChange)`, `Applying`, `Applied(ConfigOutcome)`; or `Failed(CommandError)`.
