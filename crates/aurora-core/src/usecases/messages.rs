//! Messages between projects (spec 008): the overview of every project's conversations,
//! one conversation, a closing record, templates, the user's actions and the poll.
//! Nothing runs without a usable `dcp-integrations` or for a project whose folder is
//! missing.

use crate::capabilities;
use crate::domain::{
    ClosingRecord, EngineReport, Feature, InboxSetup, Message, MessageHeader, MessageKind,
    MessagesError, PollResult, Presence, Project, ProjectInbox, Settings, TemplateSection,
    UserAction, check_action, order_conversations,
};
use crate::ports::MessagesSource;

/// The command that sets up a project for messages.
pub const SETUP_COMMAND: &str = "dcp-integrations init --project <slug>";

/// A slug suggested from the project's name: lowercase letters, digits and dashes.
pub fn suggested_slug(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "project".to_owned()
    } else {
        slug.to_owned()
    }
}

/// The setup command for one project, runnable from any folder: the provider Aurora
/// found, the project's folder and a slug suggested from its name.
pub fn setup_command(provider: &std::path::Path, project: &Project) -> String {
    format!(
        "\"{}\" --dir \"{}\" init --project {}",
        provider.display(),
        project.root.display(),
        suggested_slug(project.name.as_str())
    )
}

fn usable<'a>(
    provider: Option<&'a EngineReport>,
    project: &Project,
) -> Result<&'a EngineReport, MessagesError> {
    let provider = capabilities::usable_for(Feature::Messages, provider)
        .map_err(|hint| MessagesError::ProviderMissing { hint })?;
    if project.presence == Presence::Missing {
        return Err(MessagesError::ProjectMissing);
    }
    Ok(provider)
}

/// One project's conversations, or why it has none.
pub fn project_inbox(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
) -> ProjectInbox {
    let mut inbox = ProjectInbox {
        project: project.id.clone(),
        name: project.name.as_str().to_owned(),
        slug: None,
        counterparts: Vec::new(),
        setup: InboxSetup::Ready,
        conversations: Vec::new(),
    };
    let read = usable(provider, project).and_then(|provider| {
        source.inbox(provider, settings.engines.get(provider.kind), &project.root)
    });
    match read {
        Ok(read) => {
            inbox.slug = Some(read.slug);
            inbox.counterparts = read.counterparts;
            inbox.conversations = read.conversations;
            order_conversations(&mut inbox.conversations);
        }
        Err(MessagesError::NotSetUp { command }) => {
            inbox.setup = InboxSetup::NotSetUp {
                command: provider
                    .and_then(|p| p.location.as_deref())
                    .map_or(command, |location| setup_command(location, project)),
            }
        }
        Err(error) => {
            inbox.setup = InboxSetup::Unavailable {
                reason: error.to_string(),
            }
        }
    }
    inbox
}

/// Every project's inbox, in the registry's order.
pub fn overview(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    projects: &[Project],
) -> Vec<ProjectInbox> {
    projects
        .iter()
        .map(|project| project_inbox(source, provider, settings, project))
        .collect()
}

pub fn conversation(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
    thread: &str,
) -> Result<Vec<Message>, MessagesError> {
    let provider = usable(provider, project)?;
    source.thread(
        provider,
        settings.engines.get(provider.kind),
        &project.root,
        thread,
    )
}

pub fn record(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
    thread: &str,
) -> Result<ClosingRecord, MessagesError> {
    let provider = usable(provider, project)?;
    source.record(
        provider,
        settings.engines.get(provider.kind),
        &project.root,
        thread,
    )
}

/// The sections of a kind's template; empty for the free kinds.
pub fn template(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    kind: &MessageKind,
) -> Result<Vec<TemplateSection>, MessagesError> {
    let provider = capabilities::usable_for(Feature::Messages, provider)
        .map_err(|hint| MessagesError::ProviderMissing { hint })?;
    if matches!(
        kind,
        MessageKind::Ack | MessageKind::Notice | MessageKind::Decision | MessageKind::Guidance
    ) {
        return Ok(Vec::new());
    }
    source.template(provider, settings.engines.get(provider.kind), kind)
}

/// Checks `action` and runs it in `project`'s name.
pub fn act(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
    action: &UserAction,
) -> Result<Vec<MessageHeader>, MessagesError> {
    let usable_provider = usable(provider, project)?;
    let kind = match action {
        UserAction::Send { draft } => Some(&draft.kind),
        UserAction::Reply { kind, .. } => Some(kind),
        _ => None,
    };
    let sections = match kind {
        Some(kind) => template(source, provider, settings, kind)?,
        None => Vec::new(),
    };
    check_action(action, &sections, None)?;
    source.act(
        usable_provider,
        settings.engines.get(usable_provider.kind),
        &project.root,
        action,
    )
}

/// Polls every present project; a project that cannot be polled is left out.
pub fn poll(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    projects: &[Project],
) -> Vec<PollResult> {
    projects
        .iter()
        .filter_map(|project| {
            let provider = usable(provider, project).ok()?;
            let news = source
                .poll(provider, settings.engines.get(provider.kind), &project.root)
                .ok()?;
            Some(PollResult {
                project: project.id.clone(),
                changed: news.is_some(),
                summary: news.unwrap_or_default(),
            })
        })
        .collect()
}

#[cfg(all(test, feature = "fixtures"))]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::domain::{Draft, EngineKind, ProjectId, ProjectName, Section};
    use crate::fixtures::{self, FixtureMessages};

    fn project(name: &str, presence: Presence) -> Project {
        Project {
            id: ProjectId(name.into()),
            name: ProjectName::parse(name).unwrap_or_else(|_| unreachable!("valid sample")),
            root: Path::new("/repos").join(name),
            added_at: fixtures::CHECKED_AT.into(),
            presence,
        }
    }

    fn provider() -> EngineReport {
        fixtures::ready_report(EngineKind::Integrations, &["messages"])
    }

    #[test]
    fn overview_orders_and_reports_each_setup() {
        let source = FixtureMessages::sample();
        let p = provider();
        let projects = [
            project("aurora-dev-suite", Presence::Present),
            project("gedo", Presence::Present),
            project("levix-platform", Presence::Missing),
        ];
        let list = overview(&source, Some(&p), &Settings::default(), &projects);
        assert_eq!(list[0].setup, InboxSetup::Ready);
        assert!(list[0].conversations[0].waiting_for_user);
        assert_eq!(list[0].waiting_count(), 1);
        assert_eq!(
            list[1].setup,
            InboxSetup::NotSetUp {
                command:
                    "\"/usr/local/bin/dcp-integrations\" --dir \"/repos/gedo\" init --project gedo"
                        .into()
            }
        );
        assert!(matches!(list[2].setup, InboxSetup::Unavailable { .. }));
        let none = overview(&source, None, &Settings::default(), &projects[..1]);
        assert!(
            matches!(none[0].setup, InboxSetup::Unavailable { ref reason } if reason.contains("dcp-integrations"))
        );
    }

    #[test]
    fn the_suggested_slug_comes_from_the_name() {
        assert_eq!(suggested_slug("DocMapper"), "docmapper");
        assert_eq!(
            suggested_slug("Levix Digital  Website!"),
            "levix-digital-website"
        );
        assert_eq!(
            suggested_slug("ai-code-agent-skills"),
            "ai-code-agent-skills"
        );
        assert_eq!(suggested_slug("..."), "project");
    }

    #[test]
    fn act_checks_before_running_and_runs_nothing_without_a_provider() {
        let source = FixtureMessages::sample();
        let p = provider();
        let aurora = project("aurora-dev-suite", Presence::Present);
        let send = UserAction::Send {
            draft: Draft {
                to: "gedo".into(),
                kind: MessageKind::Question,
                subject: "Dates".into(),
                sections: vec![Section::new("Ask", "When?")],
            },
        };
        let Err(MessagesError::Refused {
            missing_sections, ..
        }) = act(&source, Some(&p), &Settings::default(), &aurora, &send)
        else {
            panic!("refused");
        };
        assert_eq!(missing_sections, ["Context", "Closes when"]);
        assert!(matches!(
            act(&source, None, &Settings::default(), &aurora, &send),
            Err(MessagesError::ProviderMissing { .. })
        ));
        assert_eq!(source.calls(), 0, "nothing reached the provider");
    }

    #[test]
    fn decide_and_revise_change_the_waiting_record() {
        let source = FixtureMessages::sample();
        let p = provider();
        let settings = Settings::default();
        let aurora = project("aurora-dev-suite", Presence::Present);
        let inbox = project_inbox(&source, Some(&p), &settings, &aurora);
        let thread = inbox.conversations[0].thread.clone();
        let record = record(&source, Some(&p), &settings, &aurora, &thread).unwrap();
        assert!(!record.options.is_empty());
        let revise = UserAction::Revise {
            thread: thread.clone(),
            text: "Keep the old dates.".into(),
        };
        let sent = act(&source, Some(&p), &settings, &aurora, &revise).unwrap();
        assert_eq!(sent[0].kind, MessageKind::Guidance);
        let after = project_inbox(&source, Some(&p), &settings, &aurora);
        let c = after
            .conversations
            .iter()
            .find(|c| c.thread == thread)
            .unwrap();
        assert!(!c.waiting_for_user && c.cycle == 2);
        let messages = conversation(&source, Some(&p), &settings, &aurora, &thread).unwrap();
        assert_eq!(messages.last().unwrap().header.kind, MessageKind::Guidance);
    }

    #[test]
    fn poll_reports_news_once_and_skips_missing_projects() {
        let source = FixtureMessages::sample();
        let p = provider();
        let projects = [
            project("aurora-dev-suite", Presence::Present),
            project("levix-platform", Presence::Missing),
        ];
        let first = poll(&source, Some(&p), &Settings::default(), &projects);
        assert_eq!(first.len(), 1);
        assert!(first[0].changed && !first[0].summary.is_empty());
        let second = poll(&source, Some(&p), &Settings::default(), &projects);
        assert!(!second[0].changed);
        assert!(
            template(&source, Some(&p), &Settings::default(), &MessageKind::Ack)
                .unwrap()
                .is_empty()
        );
    }
}
