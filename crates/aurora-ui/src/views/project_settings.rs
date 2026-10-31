//! The Settings tab of the Project view: a project's Dev Cycle settings read through
//! `dcp config get`, edited, previewed and applied through `dcp config set` (spec 007).

use std::collections::BTreeMap;

use aurora_core::domain::{
    ConfigChange, ConfigDraft, ConfigError, ConfigOutcome, ConfigSetting, ConfigType, ConfigValue,
    Feature, FieldError, ProjectConfig, value_text,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::{FeatureState, StatusLine, Tone, sentence};
use crate::app::AppContext;
use crate::backend::CommandError;

const SAVED: &str = "Saved to .dev-cycle.json. Commit it to share the change.";

/// Where the user is between editing and applying.
#[derive(Debug, Clone, PartialEq)]
enum Step {
    Editing,
    /// A command runs; the buttons wait.
    Busy,
    /// The preview of exactly `change`; Apply sends it unchanged (FR-005).
    Previewing {
        outcome: ConfigOutcome,
        change: ConfigChange,
    },
}

/// The reading of the settings.
#[derive(Debug, Clone, PartialEq)]
enum Loaded {
    Reading,
    Read(Box<ProjectConfig>),
    Failed(CommandError),
}

/// Asks before dropping unsaved edits; true when there are none or the user agrees.
pub fn confirm_discard(ctx: AppContext) -> bool {
    if ctx.unsaved_settings.get_untracked() == 0 {
        return true;
    }
    let agreed = window()
        .confirm_with_message("Discard the settings changes that were not applied?")
        .unwrap_or(false);
    if agreed {
        ctx.unsaved_settings.set(0);
    }
    agreed
}

#[component]
pub fn ProjectSettings() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let loaded = RwSignal::new(Loaded::Reading);
    let draft = RwSignal::new(ConfigDraft::default());
    let input_errors = RwSignal::new(BTreeMap::<String, FieldError>::new());
    let refused = RwSignal::new(None::<ConfigError>);
    let step = RwSignal::new(Step::Editing);
    let stale = RwSignal::new(false);
    let message = RwSignal::new(None::<(Tone, String)>);
    // The number of the latest read; an answer to an earlier one is dropped.
    let latest = StoredValue::new(0_u64);

    Effect::new(move |_| ctx.unsaved_settings.set(draft.with(ConfigDraft::count)));
    on_cleanup(move || ctx.unsaved_settings.set(0));

    // Reads the settings; `keep_edits` rebases the draft on the new reading.
    let read = move |keep_edits: bool| {
        let Some(id) = ctx.project.get_untracked() else {
            return;
        };
        let token = latest.get_value() + 1;
        latest.set_value(token);
        if !keep_edits {
            draft.set(ConfigDraft::default());
            input_errors.set(BTreeMap::new());
        }
        refused.set(None);
        stale.set(false);
        step.set(Step::Editing);
        loaded.set(Loaded::Reading);
        spawn_local(async move {
            let answer = ctx.backend.get_value().project_config(id).await;
            if latest.get_value() != token {
                return;
            }
            match answer {
                Ok(config) => {
                    draft.update(|d| *d = d.rebase(&config));
                    loaded.set(Loaded::Read(Box::new(config)));
                }
                Err(error) => loaded.set(Loaded::Failed(error)),
            }
        });
    };
    Effect::new(move |_| {
        if ctx.project.get().is_some() {
            read(false);
        }
    });

    let config = move || match loaded.get() {
        Loaded::Read(config) => Some(*config),
        _ => None,
    };

    // A command's error: refusals next to the fields, a stale revision as its own state.
    let failed = move |error: CommandError| {
        step.set(Step::Editing);
        match error.config.map(|config| *config) {
            Some(ConfigError::Stale) => {
                stale.set(true);
                message.set(Some((Tone::Warn, ConfigError::Stale.to_string())));
            }
            Some(ConfigError::Rejected { errors }) => {
                message.set(Some((
                    Tone::Error,
                    ConfigError::Rejected {
                        errors: errors.clone(),
                    }
                    .to_string(),
                )));
                refused.set(Some(ConfigError::Rejected { errors }));
            }
            _ => message.set(Some((Tone::Error, error.message))),
        }
    };

    let preview = Callback::new(move |()| {
        let (Some(id), Some(config)) = (ctx.project.get_untracked(), config()) else {
            return;
        };
        let change = match draft.with_untracked(|d| d.change(&config)) {
            Ok(change) => change,
            Err(errors) => {
                refused.set(Some(ConfigError::Rejected { errors }));
                return;
            }
        };
        refused.set(None);
        message.set(None);
        step.set(Step::Busy);
        spawn_local(async move {
            match ctx
                .backend
                .get_value()
                .preview_project_config(id, change.clone())
                .await
            {
                Ok(outcome) if outcome.changes.is_empty() => {
                    step.set(Step::Editing);
                    message.set(Some((
                        Tone::Info,
                        sentence(&ConfigError::NothingToChange.to_string()),
                    )));
                }
                Ok(outcome) => step.set(Step::Previewing { outcome, change }),
                Err(error) => failed(error),
            }
        });
    });

    let apply = Callback::new(move |()| {
        let (Some(id), Step::Previewing { change, .. }) =
            (ctx.project.get_untracked(), step.get_untracked())
        else {
            return;
        };
        step.set(Step::Busy);
        spawn_local(async move {
            match ctx
                .backend
                .get_value()
                .apply_project_config(id, change)
                .await
            {
                Ok(_) => {
                    read(false);
                    message.set(Some((Tone::Ok, SAVED.to_owned())));
                }
                Err(error) => failed(error),
            }
        });
    });

    let discard = Callback::new(move |()| {
        draft.set(ConfigDraft::default());
        input_errors.set(BTreeMap::new());
        refused.set(None);
        message.set(None);
    });

    view! {
        <FeatureState feature=Feature::ProjectSettings />
        <StatusLine message=message />
        {move || stale.get().then(|| view! {
            <p><button class="btn" on:click=move |_| read(true)>"Read the settings again"</button></p>
        })}
        {move || match loaded.get() {
            Loaded::Reading => view! { <p class="muted">"Reading the settings…"</p> }.into_any(),
            Loaded::Failed(error) => view! { <div class="card"><ConfigFailure error=error /></div> }.into_any(),
            Loaded::Read(config) => {
                let config = *config;
                let source = config
                    .source
                    .clone()
                    .unwrap_or_else(|| "defaults (no .dev-cycle.json)".to_owned());
                let findings = config.findings.clone();
                let groups: Vec<(String, Vec<ConfigSetting>)> = config
                    .groups()
                    .into_iter()
                    .map(|(name, members)| (name, members.into_iter().cloned().collect()))
                    .collect();
                let general: Vec<FieldError> = match refused.get() {
                    Some(ConfigError::Rejected { errors }) => {
                        errors.into_iter().filter(|e| e.key.is_none()).collect()
                    }
                    _ => Vec::new(),
                };
                view! {
                    <p class="muted">{format!("Source: {source}. Read at {}", config.read_at)}</p>
                    {(!findings.is_empty()).then(|| view! {
                        <div class="banner warn">
                            <p>"The configuration has problems:"</p>
                            <ul>{findings.into_iter().map(|f| view! { <li>{f}</li> }).collect_view()}</ul>
                        </div>
                    })}
                    {(!general.is_empty()).then(|| view! {
                        <ul class="field-errors">
                            {general.into_iter().map(|e| view! { <li>{sentence(&e.message)}</li> }).collect_view()}
                        </ul>
                    })}
                    <Toolbar draft=draft step=step preview=preview discard=discard apply=apply />
                    {groups.into_iter().map(|(group, members)| {
                        let config = config.clone();
                        view! {
                            <section class="card section settings-group">
                                <h2>{group}</h2>
                                {members.into_iter().map(|setting| view! {
                                    <SettingField
                                        setting=setting
                                        config=config.clone()
                                        draft=draft
                                        input_errors=input_errors
                                        refused=refused
                                        locked=Signal::derive(move || step.with(|s| *s != Step::Editing))
                                    />
                                }).collect_view()}
                            </section>
                        }
                    }).collect_view()}
                }
                .into_any()
            }
        }}
    }
}

/// The pending count and the buttons, or the preview with Apply and Back.
#[component]
fn Toolbar(
    draft: RwSignal<ConfigDraft>,
    step: RwSignal<Step>,
    preview: Callback<()>,
    discard: Callback<()>,
    apply: Callback<()>,
) -> impl IntoView {
    move || {
        match step.get() {
        Step::Previewing { outcome, .. } => view! {
            <section class="card section settings-preview" aria-label="Preview of the changes">
                <h2>"Preview"</h2>
                <p>"Nothing is written until you apply. Aurora does not commit: commit .dev-cycle.json yourself."</p>
                <ul class="items">
                    {outcome.changes.iter().map(|c| view! {
                        <li>
                            <code>{c.key.clone()}</code>": "
                            {value_text(c.before.as_ref())}" → "<strong>{value_text(c.after.as_ref())}</strong>
                        </li>
                    }).collect_view()}
                </ul>
                {(!outcome.diff.is_empty()).then(|| view! {
                    <details open>
                        <summary>"Change to .dev-cycle.json"</summary>
                        <pre>{outcome.diff.clone()}</pre>
                    </details>
                })}
                <div class="actions">
                    <button class="btn primary" on:click=move |_| apply.run(())>"Apply"</button>
                    <button class="btn" on:click=move |_| step.set(Step::Editing)>"Back to editing"</button>
                </div>
            </section>
        }
        .into_any(),
        current => {
            let count = draft.with(ConfigDraft::count);
            let busy = current == Step::Busy;
            let idle = count == 0 || busy;
            view! {
                <div class="settings-toolbar">
                    <span class="muted">
                        {match count {
                            0 => "No changes".to_owned(),
                            1 => "1 change".to_owned(),
                            n => format!("{n} changes"),
                        }}
                    </span>
                    <button class="btn primary" disabled=idle on:click=move |_| preview.run(())>
                        {if busy { "Working…" } else { "Preview changes" }}
                    </button>
                    <button class="btn" disabled=idle on:click=move |_| discard.run(())>"Discard"</button>
                </div>
            }
            .into_any()
        }
    }
    }
}

/// One setting: its description and key, its control, its default and its errors.
#[component]
fn SettingField(
    setting: ConfigSetting,
    config: ProjectConfig,
    draft: RwSignal<ConfigDraft>,
    input_errors: RwSignal<BTreeMap<String, FieldError>>,
    refused: RwSignal<Option<ConfigError>>,
    locked: Signal<bool>,
) -> impl IntoView {
    let id = format!("setting-{}", setting.key.replace('.', "-"));
    let key = setting.key.clone();
    let stored_config = StoredValue::new(config);
    let stored_setting = StoredValue::new(setting.clone());
    let stored_key = StoredValue::new(key.clone());
    let current = move || {
        stored_key.with_value(|key| {
            stored_config.with_value(|config| draft.with(|d| d.value_of(config, key)))
        })
    };
    let edited = move || stored_key.with_value(|key| draft.with(|d| d.is_edited(key)));
    let set_value = move |value: Option<ConfigValue>| {
        let key = stored_key.get_value();
        stored_config.with_value(|config| draft.update(|d| d.set(config, &key, value)));
        input_errors.update(|errors| {
            errors.remove(&key);
        });
    };
    let set_input = move |input: String| match stored_setting.with_value(|s| s.parse_input(&input))
    {
        Ok(value) => set_value(value),
        Err(error) => input_errors.update(|errors| {
            errors.insert(stored_key.get_value(), error);
        }),
    };
    let disabled = move || setting_locked(&stored_setting.get_value(), locked.get());

    let control = match (setting.kind, setting.allowed.clone()) {
        (ConfigType::Boolean, _) => view! {
            <input
                id=id.clone()
                type="checkbox"
                disabled=disabled
                prop:checked={
                                        move || current() == Some(ConfigValue::Boolean(true))
                }
                on:change=move |ev| set_value(Some(ConfigValue::Boolean(event_target_checked(&ev))))
            />
        }
        .into_any(),
        (ConfigType::Enum, Some(allowed)) => {
            let nullable = setting.nullable;
            view! {
                <select
                    id=id.clone()
                    disabled=disabled
                    on:change=move |ev| {
                        let chosen = event_target_value(&ev);
                        set_value((!chosen.is_empty()).then_some(ConfigValue::Text(chosen)));
                    }
                >
                    {nullable.then(|| {
                                                view! { <option value="" selected=move || current().is_none()>"Not set"</option> }
                    })}
                    {allowed.into_iter().map(|option| {
                                                let value = option.clone();
                        view! {
                            <option
                                value=option.clone()
                                selected=move || current() == Some(ConfigValue::Text(value.clone()))
                            >
                                {option.clone()}
                            </option>
                        }
                    }).collect_view()}
                </select>
            }
            .into_any()
        }
        (ConfigType::List, Some(allowed)) => {
            view! {
                <fieldset id=id.clone() class="choices">
                    <legend class="visually-hidden">{setting.description.clone()}</legend>
                    {allowed.clone().into_iter().map(|option| {
                                                let allowed = allowed.clone();
                                                let checked_option = option.clone();
                        let toggled = option.clone();
                        view! {
                            <label>
                                <input
                                    type="checkbox"
                                    disabled=disabled
                                    prop:checked=move || matches!(current(), Some(ConfigValue::List(items)) if items.contains(&checked_option))
                                    on:change={
                                                                                move |ev| {
                                            let on = event_target_checked(&ev);
                                            let mut items = match current() {
                                                Some(ConfigValue::List(items)) => items,
                                                _ => Vec::new(),
                                            };
                                            items.retain(|item| *item != toggled);
                                            if on {
                                                items.push(toggled.clone());
                                            }
                                            // Keep the order of the allowed values.
                                            let ordered = allowed.iter().filter(|a| items.contains(a)).cloned().collect();
                                            set_value(Some(ConfigValue::List(ordered)));
                                        }
                                    }
                                />
                                " "{option}
                            </label>
                            " "
                        }
                    }).collect_view()}
                </fieldset>
            }
            .into_any()
        }
        (ConfigType::List, None) => view! {
            <textarea
                id=id.clone()
                rows="3"
                disabled=disabled
                placeholder="One value per line"
                prop:value=move || match current() {
                    Some(ConfigValue::List(items)) => items.join("\n"),
                    _ => String::new(),
                }
                on:change=move |ev| set_input(event_target_value(&ev))
            ></textarea>
        }
        .into_any(),
        (kind, _) => view! {
            <input
                id=id.clone()
                type="text"
                inputmode=(kind == ConfigType::Integer).then_some("numeric")
                disabled=disabled
                placeholder=setting.nullable.then_some("Not set")
                prop:value=move || current().map(|v| v.to_string()).unwrap_or_default()
                on:change=move |ev| set_input(event_target_value(&ev))
            />
        }
        .into_any(),
    };

    let default_text = (setting.default != setting.value)
        .then(|| format!("Default: {}", value_text(setting.default.as_ref())));
    let errors = {
        let key = key.clone();
        move || {
            let mut errors: Vec<FieldError> =
                input_errors.with(|e| e.get(&key).cloned().into_iter().collect());
            if let Some(refusal) = refused.get() {
                errors.extend(refusal.errors_for(&key).cloned());
            }
            errors
        }
    };

    view! {
        <div class="setting" class:edited=edited>
            <label for=id.clone()>
                <strong>{setting.description.clone()}</strong>
                " "<code class="muted">{setting.key.clone()}</code>
                {move || edited().then(|| view! { " "<span class="badge info">"Edited"</span> })}
            </label>
            {control}
            {setting.read_only.then(|| view! {
                <p class="muted">
                    <span class="badge muted">"Read-only"</span>" "
                    {setting.read_only_reason.clone().unwrap_or_else(|| "This setting cannot be changed.".to_owned())}
                </p>
            })}
            {default_text.map(|text| view! { <p class="muted">{text}</p> })}
            <ul class="field-errors" aria-live="polite">
                {move || errors().into_iter().map(|error| view! {
                    <li>
                        {sentence(&error.message)}
                        {(!error.accepted.is_empty()).then(|| format!(". Accepted: {}", error.accepted.join(", ")))}
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

fn setting_locked(setting: &ConfigSetting, locked: bool) -> bool {
    setting.read_only || locked
}

/// Why a project's settings could not be read, with the engine's error output to open
/// (spec 007 US3, the same shape as `CycleFailure`).
#[component]
pub fn ConfigFailure(error: CommandError) -> impl IntoView {
    let text = match error.config.as_deref() {
        Some(ConfigError::Unavailable { .. }) => "Project settings unavailable.".to_owned(),
        Some(ConfigError::ProjectMissing { .. }) => {
            "Locate the folder to read its settings.".to_owned()
        }
        _ => sentence(&error.message),
    };
    let stderr = error
        .config
        .as_deref()
        .and_then(ConfigError::stderr)
        .map(str::to_owned);
    view! {
        <span class="muted">{text}</span>
        {stderr.map(|stderr| view! {
            <details>
                <summary>"Engine output"</summary>
                <pre>{stderr}</pre>
            </details>
        })}
    }
}
