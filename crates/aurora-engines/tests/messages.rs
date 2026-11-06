//! `ProviderMessagesSource` against the fake `dcp-integrations` (T014), and the provider
//! samples against the contract (T013).

mod common;

use std::path::{Path, PathBuf};

use aurora_core::domain::{
    Draft, EngineKind, EngineReport, EngineSettings, MessageKind, MessagesError, Section,
    UserAction,
};
use aurora_core::fixtures;
use aurora_core::ports::MessagesSource;
use aurora_engines::ProviderMessagesSource;
use aurora_engines::messages::parse;
use serde_json::Value;

fn sample(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dcp-integrations")
        .join(format!("{name}.json"));
    serde_json::from_str(&std::fs::read_to_string(&path).expect("sample")).expect("JSON")
}

#[test]
fn samples_follow_the_contract() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/008-messages/contracts/dcp-integrations.schema.json");
    let schema: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("contract")).expect("JSON");
    let validator = jsonschema::draft202012::new(&schema).expect("valid schema");
    for name in [
        "projects",
        "check",
        "thread",
        "record",
        "template",
        "sent",
        "status",
        "poll-news",
        "error-refused",
        "error-git",
    ] {
        let problems: Vec<String> = validator
            .iter_errors(&sample(name))
            .map(|e| format!("{}: {e}", e.instance_path()))
            .collect();
        assert!(problems.is_empty(), "{name}: {problems:#?}");
    }
}

#[test]
fn replies_are_checked_before_they_are_read() {
    let bytes = |v: &Value| serde_json::to_vec(v).expect("JSON");
    let mut check = sample("check");
    assert!(parse::<Value>(&bytes(&check), "check").is_ok());
    assert!(matches!(
        parse::<Value>(&bytes(&check), "record"),
        Err(MessagesError::InvalidReply { .. })
    ));
    check["schema_version"] = "2.0".into();
    assert!(matches!(
        parse::<Value>(&bytes(&check), "check"),
        Err(MessagesError::Unsupported { found }) if found == "2.0"
    ));
    assert!(matches!(
        parse::<Value>(b"not json", "check"),
        Err(MessagesError::InvalidReply { .. })
    ));
    let Err(MessagesError::Refused {
        missing_sections, ..
    }) = parse::<Value>(&bytes(&sample("error-refused")), "sent")
    else {
        panic!("refused");
    };
    assert_eq!(missing_sections, ["Ask", "Context", "Closes when"]);
    assert!(matches!(
        parse::<Value>(&bytes(&sample("error-git")), "sent"),
        Err(MessagesError::Git { stderr, .. }) if stderr.contains("denied")
    ));
}

#[cfg(feature = "fake-engine")]
mod fake {
    use super::*;

    struct Setup {
        _dir: tempfile::TempDir,
        provider: PathBuf,
        root: PathBuf,
    }

    fn setup(mode: &str) -> Setup {
        let dir = common::engine_dir();
        let provider = common::install_fake(dir.path(), "dcp-integrations", mode);
        let root = common::make_repo(dir.path(), "alpha");
        Setup {
            _dir: dir,
            provider,
            root,
        }
    }

    impl Setup {
        fn report(&self) -> EngineReport {
            let mut report = fixtures::ready_report(EngineKind::Integrations, &["messages"]);
            report.location = Some(self.provider.clone());
            report
        }

        fn sibling(&self, ext: &str) -> String {
            let mut file = self.provider.as_os_str().to_owned();
            file.push(ext);
            std::fs::read_to_string(PathBuf::from(file)).unwrap_or_default()
        }

        fn args(&self) -> Vec<String> {
            self.sibling(".args").lines().map(str::to_owned).collect()
        }

        fn act(
            &self,
            action: &UserAction,
        ) -> Result<Vec<aurora_core::domain::MessageHeader>, MessagesError> {
            ProviderMessagesSource.act(
                &self.report(),
                &EngineSettings::default(),
                &self.root,
                action,
            )
        }
    }

    fn send() -> UserAction {
        UserAction::Send {
            draft: Draft {
                to: "beta".into(),
                kind: MessageKind::Question,
                subject: "Format".into(),
                sections: vec![Section::new("Ask", "Should both sides adopt 1.0?")],
            },
        }
    }

    #[test]
    fn reads_the_inbox_thread_record_and_template() {
        let s = setup("ready");
        let settings = EngineSettings::default();
        let source = ProviderMessagesSource;
        let inbox = source
            .inbox(&s.report(), &settings, &s.root)
            .expect("inbox");
        assert_eq!(inbox.slug, "alpha");
        assert_eq!(inbox.counterparts, ["beta"]);
        assert!(inbox.conversations[0].waiting_for_user);
        assert_eq!(inbox.conversations[0].last.kind, MessageKind::Agreement);
        let root = s.root.to_string_lossy().into_owned();
        assert_eq!(s.args(), ["--json", "--dir", root.as_str(), "check"]);
        let thread = &inbox.conversations[0].thread;
        let messages = source
            .thread(&s.report(), &settings, &s.root, thread)
            .expect("thread");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].sections[0].name, "Ask");
        let record = source
            .record(&s.report(), &settings, &s.root, thread)
            .expect("record");
        assert_eq!(record.options, ["Adopt 1.0 (recommended)", "Wait"]);
        let template = source
            .template(&s.report(), &settings, &MessageKind::Question)
            .expect("template");
        assert!(
            template
                .iter()
                .any(|t| t.name == "Closes when" && t.required)
        );
        assert_eq!(s.args(), ["--json", "template", "question"]);
    }

    #[test]
    fn actions_pass_their_arguments_and_remove_the_body_file() {
        let s = setup("ready");
        let sent = s.act(&send()).expect("sent");
        assert_eq!(
            sent[0].kind,
            MessageKind::Agreement,
            "the sample's sent message"
        );
        let args = s.args();
        let at = args
            .iter()
            .position(|a| a == "--body-file")
            .expect("body file");
        assert!(!Path::new(&args[at + 1]).exists(), "removed after the run");
        assert_eq!(
            s.sibling(".body"),
            "### Ask\nShould both sides adopt 1.0?\n"
        );
        assert_eq!(
            &args[3..9],
            ["send", "--to", "beta", "--kind", "question", "--subject"]
        );

        let decide = UserAction::Decide {
            thread: "t".into(),
            choice: "Adopt 1.0".into(),
            note: Some("Both.".into()),
        };
        s.act(&decide).expect("decided");
        assert_eq!(
            &s.args()[3..],
            ["decide", "t", "--choice", "Adopt 1.0", "--note", "Both."]
        );
        s.act(&UserAction::Revise {
            thread: "t".into(),
            text: "Change it.".into(),
        })
        .expect("revised");
        assert_eq!(&s.args()[3..], ["decide", "t", "--revise", "Change it."]);
        s.act(&UserAction::Guide {
            thread: "t".into(),
            text: "Hurry.".into(),
        })
        .expect("guided");
        assert_eq!(&s.args()[3..], ["guide", "t", "--text", "Hurry."]);
        let seen = s
            .act(&UserAction::MarkSeen { id: "m".into() })
            .expect("seen");
        assert_eq!(seen.len(), 1);
        assert_eq!(&s.args()[3..], ["status", "m", "seen"]);
    }

    #[test]
    fn poll_reads_the_exit_code() {
        let s = setup("ready");
        let news = ProviderMessagesSource
            .poll(&s.report(), &EngineSettings::default(), &s.root)
            .expect("poll");
        assert!(news.is_some_and(|n| n.contains("new message")));
        let quiet = setup("quiet");
        assert_eq!(
            ProviderMessagesSource
                .poll(&quiet.report(), &EngineSettings::default(), &quiet.root)
                .expect("poll"),
            None
        );
    }

    #[test]
    fn failures_are_typed() {
        assert!(matches!(
            setup("refused").act(&send()),
            Err(MessagesError::Refused { missing_sections, .. }) if missing_sections.len() == 3
        ));
        assert!(matches!(
            setup("git").act(&send()),
            Err(MessagesError::Git { .. })
        ));
        let s = setup("unsupported");
        assert!(matches!(
            ProviderMessagesSource.inbox(&s.report(), &EngineSettings::default(), &s.root),
            Err(MessagesError::Unsupported { .. })
        ));
        let s = setup("invalid");
        assert!(matches!(
            ProviderMessagesSource.inbox(&s.report(), &EngineSettings::default(), &s.root),
            Err(MessagesError::InvalidReply { .. })
        ));
        let s = setup("hang");
        let settings = EngineSettings {
            path: None,
            timeout_ms: 300,
        };
        assert!(matches!(
            ProviderMessagesSource.inbox(&s.report(), &settings, &s.root),
            Err(MessagesError::Timeout { .. })
        ));
        let mut missing = s.report();
        missing.location = None;
        assert!(matches!(
            ProviderMessagesSource.inbox(&missing, &settings, &s.root),
            Err(MessagesError::ProviderMissing { .. })
        ));
    }
}
