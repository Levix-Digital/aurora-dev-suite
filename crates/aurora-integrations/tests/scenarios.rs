//! Scenarios of the provider on local bare repositories: the transport (T006), every
//! command of tool 0.3.0 (T007) and the `--json` replies and written files against the
//! contracts (T009).

#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers outside #[test] functions

use std::path::{Path, PathBuf};
use std::process::Command;

use aurora_integrations::commands::{self, Context, Failure, NEWS, Reply};
use aurora_integrations::registry::{self, Registry};
use aurora_integrations::transport::{Identity, Transport, TransportError};
use serde_json::Value;

const TOOL: &str = "9.9.9-test";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

fn contracts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../specs/008-messages/contracts")
}

fn problems(schema: &str, value: &Value) -> Vec<String> {
    let text = std::fs::read_to_string(contracts().join(schema)).expect("contract");
    let schema: Value = serde_json::from_str(&text).expect("contract is JSON");
    let validator = jsonschema::draft202012::new(&schema).expect("the contract is a valid schema");
    validator
        .iter_errors(value)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect()
}

fn valid(reply: &Value) {
    let p = problems("dcp-integrations.schema.json", reply);
    assert!(p.is_empty(), "{p:#?}\n{reply:#}");
}

/// Two projects, `alpha` and `beta`, each with a bare "remote" and its own cache.
struct World {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl World {
    fn new() -> World {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        for p in ["alpha", "beta"] {
            std::fs::create_dir_all(root.join(format!("{p}.git"))).unwrap();
            git(
                &root.join(format!("{p}.git")),
                &["init", "--quiet", "--bare"],
            );
            let dir = root.join(p);
            std::fs::create_dir_all(&dir).unwrap();
            git(&dir, &["init", "--quiet"]);
            git(&dir, &["config", "user.name", &format!("{p} maintainer")]);
            git(&dir, &["config", "user.email", &format!("{p}@example.org")]);
            let other = if p == "alpha" { "beta" } else { "alpha" };
            let mut r = Registry::new(p, &p.to_uppercase(), &url(&root, p));
            r.add(other, &other.to_uppercase(), &url(&root, other))
                .unwrap();
            registry::save(&dir, &r).unwrap();
            let written: Value =
                serde_json::from_str(&std::fs::read_to_string(registry::path(&dir)).unwrap())
                    .unwrap();
            assert!(problems("aurora-integrations.schema.json", &written).is_empty());
        }
        World { _tmp: tmp, root }
    }

    fn cx(&self, p: &str) -> Context {
        Context::open(
            &self.root.join(p),
            self.root.join(format!("cache-{p}")),
            TOOL,
        )
        .unwrap()
    }

    fn remote(&self, p: &str) -> PathBuf {
        self.root.join(format!("{p}.git"))
    }
}

fn url(root: &Path, p: &str) -> String {
    root.join(format!("{p}.git")).to_string_lossy().into_owned()
}

fn ok(r: Result<Reply, Failure>) -> Reply {
    let r = r.unwrap_or_else(|f| panic!("{f:?}"));
    valid(&r.json);
    r
}

fn refused(r: Result<Reply, Failure>) -> Failure {
    let f = r.expect_err("refused");
    valid(&f.to_json());
    f
}

fn sent_id(r: &Reply) -> String {
    r.json["sent"][0]["id"].as_str().unwrap().to_owned()
}

const QUESTION: &str = "### Ask\nShould both sides adopt format 1.0?\n\n### Context\nThe draft contract.\n\n### Closes when\nBoth sides record the answer.\n";
const ANSWER: &str = "### Answer\nAgree.\n\n### Next step\nAlpha records it.\n";
const COUNTER: &str = "### Answer\nCounter.\n\n### Per point\n1. Use 1.1 instead.\n\n### Next step\nThe other side answers.\n";
const AGREEMENT: &str = "### Answer\nAgree.\n\n### Conclusion\nAdopt 1.1.\n\n### Justification\nIt keeps the dates.\n\n### Decision needed\n1. Adopt 1.1\n2. Keep 1.0\n\n### After approval\nBoth record it.\n";

#[test]
fn transport_reads_writes_and_checks_the_format() {
    let w = World::new();
    let cx = w.cx("alpha");
    let t = &cx.transport;
    let alpha = url(&w.root, "alpha");
    // A missing branch is an empty inbox, and ls-remote answers without fetching.
    assert_eq!(t.remote_tip("alpha", &alpha).unwrap(), None);
    assert!(t.read_inbox("alpha", &alpha).unwrap().is_empty());
    // The first write creates the branch with README.md and inbox.json 1.0.
    let beta = w.cx("beta");
    ok(commands::send(
        &beta, "alpha", "question", "Format", QUESTION,
    ));
    let tip = t
        .remote_tip("alpha", &alpha)
        .unwrap()
        .expect("branch exists");
    let files = git(
        &w.remote("alpha"),
        &["ls-tree", "-r", "--name-only", "aurora/inbox"],
    );
    assert!(files.contains("README.md") && files.contains("inbox.json"));
    let inbox: Value = serde_json::from_str(&git(
        &w.remote("alpha"),
        &["show", "aurora/inbox:inbox.json"],
    ))
    .unwrap();
    assert!(
        problems("aurora-inbox.schema.json", &inbox).is_empty(),
        "{inbox}"
    );
    // The commit identity is the sending project's.
    let author = git(
        &w.remote("alpha"),
        &["log", "-1", "--format=%an <%ae>|%cn", &tip],
    );
    assert_eq!(author, "beta maintainer <beta@example.org>|beta maintainer");
    assert_eq!(t.read_inbox("alpha", &alpha).unwrap().len(), 1);

    // A write that loses a race is rebuilt on the new tip: a stale cache keeps both files.
    let stale = Transport {
        cache: w.root.join("cache-stale"),
        identity: Identity {
            name: "x".into(),
            email: "x@example.org".into(),
        },
        tool: TOOL.into(),
    };
    stale.fetch("alpha", &alpha).unwrap();
    ok(commands::send(
        &beta,
        "alpha",
        "notice",
        "Second",
        "Just so you know.\n",
    ));
    stale
        .write_files(
            "alpha",
            &alpha,
            &[("messages/extra.md".into(), "x".into())],
            "extra",
        )
        .unwrap();
    let files = git(
        &w.remote("alpha"),
        &["ls-tree", "-r", "--name-only", "aurora/inbox"],
    );
    assert_eq!(
        files.lines().filter(|l| l.starts_with("messages/")).count(),
        3
    );

    // MAJOR 2 is refused, for reading and for writing.
    let v2 = Transport {
        cache: w.root.join("cache-v2"),
        ..stale.clone()
    };
    v2.write_files(
        "alpha",
        &alpha,
        &[(
            "inbox.json".into(),
            r#"{"schema":"aurora.inbox","schema_version":"2.0"}"#.into(),
        )],
        "v2",
    )
    .unwrap();
    assert!(matches!(
        t.read_inbox("alpha", &alpha),
        Err(TransportError::UnsupportedInbox { .. })
    ));
    let f = refused(commands::list(&cx, false, false));
    assert_eq!((f.code, f.exit), ("unsupported_inbox", 1));
    assert!(
        v2.write_files("alpha", &alpha, &[("a".into(), "b".into())], "x")
            .is_err()
    );

    // An unreachable remote names the project and keeps Git's message.
    let gone = Transport {
        cache: w.root.join("cache-gone"),
        ..stale
    };
    let e = gone
        .read_inbox("gamma", &url(&w.root, "gamma"))
        .unwrap_err();
    assert!(matches!(e, TransportError::Unreachable { .. }) && e.stderr().is_some());
}

#[test]
fn a_conversation_from_question_to_decision() {
    let w = World::new();
    let (a, b) = (w.cx("alpha"), w.cx("beta"));

    // poll: nothing, then news with exit code 10, then nothing again.
    assert_eq!(ok(commands::poll(&b)).code, 0);
    // Refusals write nothing.
    let f = refused(commands::send(
        &a,
        "beta",
        "question",
        "Format",
        "### Ask\nx\n",
    ));
    assert_eq!((f.code, f.exit), ("refused", 5));
    assert_eq!(f.missing_sections, ["Context", "Closes when"]);
    assert_eq!(
        refused(commands::send(&a, "beta", "answer", "x", ANSWER)).exit,
        5
    );
    assert_eq!(
        refused(commands::send(&a, "alpha", "notice", "x", "x")).exit,
        5
    );
    assert_eq!(
        refused(commands::send(&a, "gamma", "notice", "x", "x")).code,
        "unknown_project"
    );
    assert!(
        b.transport
            .remote_tip("beta", &url(&w.root, "beta"))
            .unwrap()
            .is_none()
    );

    let q = sent_id(&ok(commands::send(
        &a, "beta", "question", "Format", QUESTION,
    )));
    let news = ok(commands::poll(&b));
    assert_eq!(news.code, NEWS);
    assert!(news.text.contains("1 new message"), "{}", news.text);
    assert_eq!(ok(commands::poll(&b)).code, 0);

    // check lists it with the guide; show marks it seen.
    let check = ok(commands::check(&b, false, 24.0, false, "dcp-integrations"));
    assert!(check.text.contains(&q) && check.text.contains("How to handle them"));
    let shown = ok(commands::show(&b, &q));
    assert_eq!(shown.json["message"]["status"], "seen");
    assert_eq!(refused(commands::show(&b, "nope")).exit, 4);

    // Three rounds of counters, then only a closing kind (or ack) is accepted.
    let mut last = q.clone();
    let mut turn = [&b, &a].into_iter().cycle();
    for _ in 0..5 {
        let me = turn.next().unwrap();
        last = sent_id(&ok(commands::reply(me, &last, "counter", None, COUNTER)));
    }
    let r = ok(commands::list(&a, false, false));
    let rounds: Vec<i64> = r.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["round"].as_i64().unwrap())
        .collect();
    assert_eq!(
        rounds,
        [1, 2, 3],
        "alpha received the counters of rounds 1 to 3"
    );
    let f = refused(commands::reply(&a, &last, "counter", None, COUNTER));
    assert_eq!((f.code, f.exit), ("round_limit", 5));
    let ack = sent_id(&ok(commands::reply(&a, &last, "ack", None, "Seen.")));
    let close = sent_id(&ok(commands::reply(&b, &ack, "agreement", None, AGREEMENT)));
    assert!(close.contains("-beta-"));

    // The opener sees it waiting for the user and presents the record.
    let check = ok(commands::check(&a, true, 24.0, false, "dcp-integrations"));
    assert_eq!(check.json["waiting_for_user"].as_array().unwrap().len(), 1);
    let record = ok(commands::record(&a, &q));
    assert_eq!(
        record.json["options"],
        serde_json::json!(["Adopt 1.1", "Keep 1.0"])
    );
    assert_eq!(record.json["rounds"], 3);
    assert!(record.text.contains("Conclusion"));
    assert_eq!(
        refused(commands::record(&a, "20990101-000000-alpha-none")).exit,
        4
    );

    // The user asks for changes: cycle 2 opens three new rounds.
    let guided = ok(commands::guide(&a, &q, "Keep the old dates too."));
    assert_eq!(guided.json["sent"][0]["kind"], "guidance");
    assert_eq!(guided.json["sent"][0]["cycle"], 2);
    assert_eq!(guided.json["closed"], serde_json::json!([close]));
    assert!(
        ok(commands::check(&a, true, 24.0, false, "x")).json["waiting_for_user"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let c2 = sent_id(&ok(commands::reply(&a, &close, "counter", None, COUNTER)));
    let m = ok(commands::show(&b, &c2));
    assert_eq!(
        (
            m.json["message"]["cycle"].as_i64(),
            m.json["message"]["round"].as_i64()
        ),
        (Some(2), Some(1))
    );
    let close2 = sent_id(&ok(commands::reply(&b, &c2, "agreement", None, AGREEMENT)));
    let record = ok(commands::record(&a, &q));
    assert_eq!(
        (
            record.json["cycle"].as_i64(),
            record.json["rounds"].as_i64()
        ),
        (Some(2), Some(1))
    );
    assert_eq!(
        record.json["user_requests"],
        serde_json::json!(["Keep the old dates too."])
    );

    // The decision goes to the other side and closes the record.
    let decided = ok(commands::decide(&a, &q, "Adopt 1.1", Some("Both sides.")));
    assert_eq!(decided.json["sent"][0]["kind"], "decision");
    assert_eq!(decided.json["closed"], serde_json::json!([close2]));
    let check = ok(commands::check(&a, true, 24.0, true, "x"));
    assert!(
        check.json["waiting_for_user"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // status and defer.
    let d = sent_id(&decided);
    let s = ok(commands::defer(&b, &d, "belongs to the next release"));
    assert_eq!(s.json["changed"]["deferred"], "belongs to the next release");
    ok(commands::status(&b, &d, "done", Some("recorded")));
    assert_eq!(refused(commands::status(&b, &d, "closed", None)).exit, 2);

    // list --sent from alpha, projects, template, version.
    let sent = ok(commands::list(&a, true, false));
    assert!(
        sent.json["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["from"] == "alpha")
    );
    assert_eq!(
        ok(commands::projects(&a)).json["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        ok(commands::template("impasse"))
            .text
            .contains("### Positions")
    );
    assert_eq!(refused(commands::template("ack")).exit, 5);
    assert_eq!(ok(commands::version(TOOL)).json["version"], TOOL);

    // Every written message carries the provider's version.
    let tree = git(
        &w.remote("beta"),
        &["show", &format!("aurora/inbox:messages/{q}.md")],
    );
    assert!(tree.contains(&format!("tool: {TOOL}")), "{tree}");
}

#[test]
fn guide_on_a_thread_and_unanswered_sent_messages() {
    let w = World::new();
    let (a, b) = (w.cx("alpha"), w.cx("beta"));
    let q = sent_id(&ok(commands::send(
        &a, "beta", "question", "Dates", QUESTION,
    )));
    let g = ok(commands::guide(&a, &q, "Answer by Friday."));
    assert_eq!(g.json["sent"][0]["to"], "beta");
    assert_eq!(refused(commands::guide(&a, &q, "  ")).exit, 5);
    assert_eq!(
        refused(commands::guide(&a, "20990101-000000-alpha-x", "x")).exit,
        4
    );
    // With a zero threshold, the question is reported as sent and unread.
    let check = ok(commands::check(&a, false, 0.0, false, "x"));
    assert_eq!(check.json["unanswered_sent"].as_array().unwrap().len(), 2);
    assert!(check.text.contains("Sent and still unread"));
    let convs = check.json["conversations"].as_array().unwrap();
    assert_eq!(convs.len(), 1);
    assert_eq!(
        (convs[0]["thread"].as_str(), convs[0]["cycle"].as_i64()),
        (Some(q.as_str()), Some(2))
    );
    assert_eq!(convs[0]["opener"], "alpha");
    assert_eq!(
        convs[0]["user_requests"],
        serde_json::json!(["Answer by Friday."])
    );
    let t = ok(commands::thread(&a, &q));
    let kinds: Vec<&str> = t.json["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["message"]["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["question", "guidance"]);
    assert_eq!(
        refused(commands::thread(&a, "20990101-000000-alpha-x")).exit,
        4
    );
    let quiet = ok(commands::check(&a, true, 24.0, true, "x"));
    assert!(quiet.text.is_empty());
    // A reply in cycle 2 starts at round 1.
    let r = ok(commands::reply(&b, &q, "answer", None, ANSWER));
    assert_eq!(
        (
            r.json["sent"][0]["cycle"].as_i64(),
            r.json["sent"][0]["round"].as_i64()
        ),
        (Some(2), Some(1))
    );
}

#[test]
fn init_add_validate_and_watch_once() {
    let w = World::new();
    let fresh = w.root.join("fresh");
    std::fs::create_dir_all(&fresh).unwrap();
    git(&fresh, &["init", "--quiet"]);
    assert_eq!(
        refused(commands::init(&fresh, "fresh", "")).exit,
        3,
        "no origin remote"
    );
    git(&fresh, &["remote", "add", "origin", &url(&w.root, "fresh")]);
    ok(commands::init(&fresh, "fresh", "Fresh"));
    assert_eq!(refused(commands::init(&fresh, "fresh", "")).exit, 2);
    ok(commands::add(
        &fresh,
        "alpha",
        "Alpha",
        &url(&w.root, "alpha"),
    ));
    assert_eq!(refused(commands::add(&fresh, "alpha", "", "u")).exit, 2);
    assert!(
        ok(commands::validate(&fresh))
            .text
            .contains("1 counterpart")
    );
    assert_eq!(refused(commands::validate(&w.root)).code, "not_set_up");

    // watch: one pass runs the command in the project folder only when there is news.
    ok(commands::send(
        &w.cx("beta"),
        "alpha",
        "notice",
        "Hello",
        "Hi.\n",
    ));
    let marker = w.root.join("alpha").join("ran.txt");
    let exec = if cfg!(windows) {
        "echo %AURORA_PROJECT%> ran.txt"
    } else {
        "echo \"$AURORA_PROJECT\" > ran.txt"
    };
    let targets = vec![("alpha".to_owned(), w.root.join("alpha"))];
    let cache = w.root.join("cache-watch");
    let lines = commands::watch_once(&targets, &cache, TOOL, exec);
    assert!(lines[0].contains("1 new message"), "{lines:?}");
    assert_eq!(std::fs::read_to_string(&marker).unwrap().trim(), "alpha");
    std::fs::remove_file(&marker).unwrap();
    assert!(commands::watch_once(&targets, &cache, TOOL, exec).is_empty());
    assert!(!marker.exists());
    let wrong = vec![("beta".to_owned(), w.root.join("alpha"))];
    assert!(commands::watch_once(&wrong, &cache, TOOL, exec)[0].contains("not beta"));
}

#[test]
fn the_binary_answers_json_and_exit_codes() {
    let w = World::new();
    let bin = env!("CARGO_BIN_EXE_dcp-integrations");
    let run = |args: &[&str]| {
        let out = Command::new(bin)
            .args(args)
            .env("AURORA_INBOX_CACHE", w.root.join("cache-bin"))
            .output()
            .unwrap();
        let json: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        (
            out.status.code().unwrap(),
            json,
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    let (code, json, _) = run(&["version", "--json"]);
    assert_eq!(code, 0);
    valid(&json);
    for args in [&["--help"][..], &["init", "--help"], &["help"]] {
        let out = Command::new(bin).args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{args:?}");
        assert!(String::from_utf8_lossy(&out.stdout).starts_with("usage:"));
    }
    let (code, json, _) = run(&["--json", "frobnicate"]);
    assert_eq!((code, json["code"].as_str()), (2, Some("invalid")));
    let (code, _, err) = run(&["--dir", &w.root.to_string_lossy(), "list"]);
    assert!(code == 3 && err.contains("init --project"), "{err}");

    let alpha = w.root.join("alpha").to_string_lossy().into_owned();
    let beta = w.root.join("beta").to_string_lossy().into_owned();
    let body = w.root.join("body.md");
    std::fs::write(&body, QUESTION).unwrap();
    let body = body.to_string_lossy().into_owned();
    let (code, json, err) = run(&[
        "--dir",
        &alpha,
        "--json",
        "send",
        "--to",
        "beta",
        "--kind",
        "question",
        "--subject",
        "Binary",
        "--body-file",
        &body,
    ]);
    assert_eq!(code, 0, "{err}");
    valid(&json);
    let (code, json, _) = run(&["--dir", &beta, "--json", "poll"]);
    assert_eq!((code, json["changed"].as_bool()), (NEWS, Some(true)));
    let (code, _, _) = run(&["--dir", &beta, "poll"]);
    assert_eq!(code, 0);
    let (code, json, _) = run(&["--dir", &beta, "--json", "show", "missing-id"]);
    assert_eq!((code, json["code"].as_str()), (4, Some("unknown_id")));
    let (code, _, _) = run(&["--dir", &beta, "decide", "x"]);
    assert_eq!(code, 2);
}
