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
use crate::ports::{ListedProject, MessagesSource};

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
/// found, the project's folder and `slug`.
pub fn setup_command(provider: &std::path::Path, project: &Project, slug: &str) -> String {
    format!(
        "\"{}\" --dir \"{}\" init --project {slug}",
        provider.display(),
        project.root.display(),
    )
}

/// A repository URL in a form that compares equal across its spellings: no scheme, user,
/// trailing slash or `.git`, `host:owner/name` as `host/owner/name`, lowercase.
pub fn repository_key(url: &str) -> String {
    let mut rest = url.trim();
    if let Some((_, after)) = rest.split_once("://") {
        rest = after;
    }
    if let Some((_, after)) = rest.split_once('@') {
        rest = after;
    }
    let rest = rest.trim_end_matches('/');
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    // `git@host:owner/name`: the first colon before any slash separates the host.
    let rest = match (rest.find(':'), rest.find('/')) {
        (Some(colon), Some(slash)) if colon < slash => {
            let port = &rest[colon + 1..slash];
            if port.chars().all(|c| c.is_ascii_digit()) && !port.is_empty() {
                format!("{}{}", &rest[..colon], &rest[slash..])
            } else {
                format!("{}/{}", &rest[..colon], &rest[colon + 1..])
            }
        }
        _ => rest.to_owned(),
    };
    rest.to_lowercase()
}

/// The slug the projects already set up give to `repository`, in the order they are
/// read; the first one wins when they disagree.
pub fn network_slug<'a>(
    repository: &str,
    listed: impl IntoIterator<Item = &'a ListedProject>,
) -> Option<String> {
    let key = repository_key(repository);
    if key.is_empty() {
        return None;
    }
    listed
        .into_iter()
        .find(|l| repository_key(&l.url) == key)
        .map(|l| l.slug.clone())
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

/// One project's inbox as read, before the setup command of a project not set up gets
/// the slug the other projects use for it (BUG-009). Read each project with
/// [`read_inbox`], in any order or in parallel, then [`complete`] them together.
#[derive(Debug, Clone)]
pub struct InboxReading {
    inbox: ProjectInbox,
    project: Project,
    listed: Vec<ListedProject>,
    /// For a project not set up: the provider's location and the repository URL.
    pending: Option<(Option<std::path::PathBuf>, Option<String>)>,
}

/// Reads one project's conversations, or why it has none.
pub fn read_inbox(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
) -> InboxReading {
    let mut reading = InboxReading {
        inbox: ProjectInbox {
            project: project.id.clone(),
            name: project.name.as_str().to_owned(),
            slug: None,
            counterparts: Vec::new(),
            setup: InboxSetup::Ready,
            conversations: Vec::new(),
        },
        project: project.clone(),
        listed: Vec::new(),
        pending: None,
    };
    let read = usable(provider, project).and_then(|provider| {
        source.inbox(provider, settings.engines.get(provider.kind), &project.root)
    });
    let inbox = &mut reading.inbox;
    match read {
        Ok(read) => {
            inbox.slug = Some(read.slug);
            inbox.counterparts = read.counterparts;
            inbox.conversations = read.conversations;
            order_conversations(&mut inbox.conversations);
            reading.listed = read.listed;
        }
        Err(MessagesError::NotSetUp {
            command,
            repository,
        }) => {
            inbox.setup = InboxSetup::NotSetUp { command };
            let location = provider.and_then(|p| p.location.clone());
            reading.pending = Some((location, repository));
        }
        Err(error) => {
            inbox.setup = InboxSetup::Unavailable {
                reason: error.to_string(),
            }
        }
    }
    reading
}

/// The inboxes of `readings`, in their order. A project not set up gets the setup
/// command with the slug the projects set up already give to its repository, or with
/// a slug from its name when none does.
pub fn complete(readings: Vec<InboxReading>) -> Vec<ProjectInbox> {
    let listed: Vec<ListedProject> = readings
        .iter()
        .flat_map(|r| r.listed.iter().cloned())
        .collect();
    readings
        .into_iter()
        .map(|reading| finish(reading, &listed))
        .collect()
}

fn finish(reading: InboxReading, listed: &[ListedProject]) -> ProjectInbox {
    let mut inbox = reading.inbox;
    if let Some((Some(location), repository)) = reading.pending {
        let slug = repository
            .as_deref()
            .and_then(|url| network_slug(url, listed))
            .unwrap_or_else(|| suggested_slug(reading.project.name.as_str()));
        inbox.setup = InboxSetup::NotSetUp {
            command: setup_command(&location, &reading.project, &slug),
        };
    }
    inbox
}

/// One project's conversations, or why it has none. Without the other projects, a
/// project not set up gets a slug from its name: [`overview`] knows better.
pub fn project_inbox(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    project: &Project,
) -> ProjectInbox {
    finish(read_inbox(source, provider, settings, project), &[])
}

/// Every project's inbox, in the registry's order.
pub fn overview(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    projects: &[Project],
) -> Vec<ProjectInbox> {
    complete(
        projects
            .iter()
            .map(|project| read_inbox(source, provider, settings, project))
            .collect(),
    )
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
            match source.poll(provider, settings.engines.get(provider.kind), &project.root) {
                Ok(news) => Some(PollResult {
                    project: project.id.clone(),
                    changed: news.is_some(),
                    summary: news.unwrap_or_default(),
                    set_up: true,
                }),
                // Reported, so a file added later reads as news (IMP-004).
                Err(MessagesError::NotSetUp { .. }) => Some(PollResult {
                    project: project.id.clone(),
                    changed: false,
                    summary: String::new(),
                    set_up: false,
                }),
                Err(_) => None,
            }
        })
        .collect()
}

/// Brings each project's inbox copies up to date (ADR-0011); the projects where
/// something changed, as poll results so they read as news.
pub fn sync(
    source: &dyn MessagesSource,
    provider: Option<&EngineReport>,
    settings: &Settings,
    projects: &[Project],
) -> Vec<PollResult> {
    projects
        .iter()
        .filter_map(|project| {
            let provider = usable(provider, project).ok()?;
            match source.sync(provider, settings.engines.get(provider.kind), &project.root) {
                Ok(true) => Some(PollResult {
                    project: project.id.clone(),
                    changed: true,
                    summary: format!("{}: messages updated.", project.name.as_str()),
                    set_up: true,
                }),
                _ => None,
            }
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
                // The folder is joined with the platform's separator.
                command: format!(
                    "\"/usr/local/bin/dcp-integrations\" --dir \"{}\" init --project gedo",
                    Path::new("/repos").join("gedo").display()
                )
            }
        );
        assert!(matches!(list[2].setup, InboxSetup::Unavailable { .. }));
        let none = overview(&source, None, &Settings::default(), &projects[..1]);
        assert!(
            matches!(none[0].setup, InboxSetup::Unavailable { ref reason } if reason.contains("dcp-integrations"))
        );
    }

    #[test]
    fn a_project_not_set_up_gets_the_slug_the_network_uses() {
        // BUG-009: a clone of Gedo in the folder levix-gedo is suggested `gedo`, the slug
        // the projects set up give to its repository; one outside the network keeps the
        // slug from its name.
        let source = FixtureMessages::sample();
        let p = provider();
        let settings = Settings::default();
        let projects = [
            project("levix-gedo", Presence::Present),
            project("aurora-dev-suite", Presence::Present),
            project("DocMapper", Presence::Present),
        ];
        let slug_in = |inbox: &ProjectInbox| match &inbox.setup {
            InboxSetup::NotSetUp { command } => command.rsplit(' ').next().map(str::to_owned),
            _ => None,
        };
        let list = overview(&source, Some(&p), &settings, &projects);
        assert_eq!(slug_in(&list[0]).as_deref(), Some("gedo"));
        assert_eq!(list[1].setup, InboxSetup::Ready);
        assert_eq!(slug_in(&list[2]).as_deref(), Some("docmapper"));
        // Read one at a time, in any order, then completed together: the same result.
        let readings = projects
            .iter()
            .rev()
            .map(|project| read_inbox(&source, Some(&p), &settings, project))
            .collect();
        let reversed = complete(readings);
        assert_eq!(slug_in(&reversed[2]).as_deref(), Some("gedo"));
        // Alone, nothing tells the network's slug: the folder's name.
        let alone = project_inbox(&source, Some(&p), &settings, &projects[0]);
        assert_eq!(slug_in(&alone).as_deref(), Some("levix-gedo"));
    }

    #[test]
    fn repository_urls_compare_across_spellings() {
        let key = repository_key("https://github.com/Levix-Digital/ai-code-agent-skills.git");
        for same in [
            "https://github.com/Levix-Digital/ai-code-agent-skills",
            "https://github.com/levix-digital/ai-code-agent-skills/",
            "git@github.com:Levix-Digital/ai-code-agent-skills.git",
            "ssh://git@github.com/Levix-Digital/ai-code-agent-skills.git",
            "ssh://git@github.com:22/Levix-Digital/ai-code-agent-skills",
            " https://user@github.com/Levix-Digital/ai-code-agent-skills.git\n",
        ] {
            assert_eq!(repository_key(same), key, "{same}");
        }
        assert_eq!(key, "github.com/levix-digital/ai-code-agent-skills");
        assert_ne!(repository_key("https://github.com/Levix-Digital/gedo"), key);
        let listed = [ListedProject {
            slug: "dev-cycle-protocol".into(),
            url: "https://github.com/Levix-Digital/ai-code-agent-skills.git".into(),
        }];
        assert_eq!(
            network_slug("git@github.com:Levix-Digital/ai-code-agent-skills", &listed).as_deref(),
            Some("dev-cycle-protocol")
        );
        assert_eq!(network_slug("", &listed), None);
        assert_eq!(
            network_slug("https://github.com/Levix-Digital/gedo", &listed),
            None
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
        assert!(first[0].changed && !first[0].summary.is_empty() && first[0].set_up);
        let second = poll(&source, Some(&p), &Settings::default(), &projects);
        assert!(!second[0].changed);
        assert!(
            template(&source, Some(&p), &Settings::default(), &MessageKind::Ack)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn poll_reports_a_project_not_set_up() {
        // IMP-004: reported, not skipped, so the desktop sees when the file appears.
        let source = FixtureMessages::sample();
        let p = provider();
        let results = poll(
            &source,
            Some(&p),
            &Settings::default(),
            &[project("gedo", Presence::Present)],
        );
        assert_eq!(results.len(), 1);
        assert!(!results[0].set_up && !results[0].changed);
    }
}
