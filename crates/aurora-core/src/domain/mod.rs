//! Plain data shared by every crate and by the interface.

mod config;
mod cycle;
mod engine;
mod feature;
mod inbox;
mod messages;
mod portfolio;
mod project;
mod project_view;
mod settings;

pub use config::{
    ConfigChange, ConfigDraft, ConfigError, ConfigOutcome, ConfigSetting, ConfigType, ConfigValue,
    FieldError, FieldErrorCode, KeyChange, KeyValue, ProjectConfig, value_text,
};
pub use cycle::{
    CandidateRun, CurrentRelease, CycleError, CycleState, ItemRef, NextOption, NextStep,
    PlanProgress, PullRequestState, UntriagedBug, UpdateSide,
};
pub use engine::{Capability, EngineKind, EngineReport, EngineSettings, EngineStatus};
pub use feature::{AvailabilityState, EnginesOverview, Feature, FeatureAvailability};
pub use inbox::{Inbox, InboxEntry, Urgency};
pub use messages::{
    ClosingRecord, Conversation, ConversationState, ConversationView, Draft, FOLDED_SECTIONS,
    InboxSetup, LEAD_SECTIONS, MAX_BODY_BYTES, MAX_ROUNDS, Message, MessageHeader, MessageKind,
    MessageStatus, MessagesError, MessagesNews, MessagesSnapshot, NEXT_STEP, PollResult,
    ProjectInbox, Section, SetupMemory, StaleInbox, StatusNote, TemplateSection, UserAction,
    active_count, allowed_replies, body as message_body, check_action, conversation_views,
    missing_sections, options as record_options, order_conversations, split_notes,
};
pub use portfolio::{CyclePhase, CycleSummary, WaitingItem, WaitingKind};
pub use project::{
    AddOutcome, Presence, Project, ProjectError, ProjectId, ProjectList, ProjectName,
    RegistryNotice,
};
pub use project_view::{CandidateStage, Gate, GateKind};
pub use settings::{EnginesSettings, SETTINGS_VERSION, Settings, Theme, ThemeChoice};
