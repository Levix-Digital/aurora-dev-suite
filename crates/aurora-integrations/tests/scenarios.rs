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
fn reads_reuse_a_recent_fetch_and_writes_never_do() {
    // BUG-008: one command per project must not fetch every inbox again.
    let w = World::new();
    let alpha = url(&w.root, "alpha");
    let beta = w.cx("beta");
    ok(commands::send(
        &beta, "alpha", "question", "Format", None, QUESTION,
    ));
    let mut reader = w.cx("alpha").transport;
    reader.reuse_fetch = Some(aurora_integrations::transport::REUSE_FETCH);
    assert_eq!(reader.read_inbox("alpha", &alpha).unwrap().len(), 1);
    // Another project writes: a read within the interval keeps the cached tip...
    ok(commands::send(
        &beta,
        "alpha",
        "notice",
        "Second",
        None,
        "Just so you know.\n",
    ));
    assert_eq!(reader.read_inbox("alpha", &alpha).unwrap().len(), 1);
    // ...a reader that does not reuse fetches it...
    let mut fresh = reader.clone();
    fresh.reuse_fetch = None;
    assert_eq!(fresh.read_inbox("alpha", &alpha).unwrap().len(), 2);
    // ...and a write starts from the remote tip, then leaves the cache on its own commit.
    reader
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
    // A lock left by a process that died does not block the next fetch for long.
    let lock = w.root.join("cache-alpha").join("alpha.git.lock");
    std::fs::write(&lock, "").unwrap();
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(600);
    std::fs::File::options()
        .write(true)
        .open(&lock)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert!(fresh.fetch("alpha", &alpha).unwrap().1.is_some());
    assert!(!lock.exists());
}

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
        &beta, "alpha", "question", "Format", None, QUESTION,
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
        reuse_fetch: None,
        cached: false,
    };
    stale.fetch("alpha", &alpha).unwrap();
    ok(commands::send(
        &beta,
        "alpha",
        "notice",
        "Second",
        None,
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
        None,
        "### Ask\nx\n",
    ));
    assert_eq!((f.code, f.exit), ("refused", 5));
    assert_eq!(f.missing_sections, ["Context", "Closes when"]);
    assert_eq!(
        refused(commands::send(&a, "beta", "answer", "x", None, ANSWER)).exit,
        5
    );
    assert_eq!(
        refused(commands::send(&a, "alpha", "notice", "x", None, "x")).exit,
        5
    );
    assert_eq!(
        refused(commands::send(&a, "gamma", "notice", "x", None, "x")).code,
        "unknown_project"
    );
    assert!(
        b.transport
            .remote_tip("beta", &url(&w.root, "beta"))
            .unwrap()
            .is_none()
    );

    let q = sent_id(&ok(commands::send(
        &a, "beta", "question", "Format", None, QUESTION,
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
        last = sent_id(&ok(commands::reply(
            me, &last, "counter", None, None, COUNTER,
        )));
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
    let f = refused(commands::reply(&a, &last, "counter", None, None, COUNTER));
    assert_eq!((f.code, f.exit), ("round_limit", 5));
    let ack = sent_id(&ok(commands::reply(&a, &last, "ack", None, None, "Seen.")));
    let close = sent_id(&ok(commands::reply(
        &b,
        &ack,
        "agreement",
        None,
        None,
        AGREEMENT,
    )));
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
    let c2 = sent_id(&ok(commands::reply(
        &a, &close, "counter", None, None, COUNTER,
    )));
    let m = ok(commands::show(&b, &c2));
    assert_eq!(
        (
            m.json["message"]["cycle"].as_i64(),
            m.json["message"]["round"].as_i64()
        ),
        (Some(2), Some(1))
    );
    let close2 = sent_id(&ok(commands::reply(
        &b,
        &c2,
        "agreement",
        None,
        None,
        AGREEMENT,
    )));
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
    assert_eq!(
        decided.json["sent"][0]["summary"],
        "The user decided: Adopt 1.1"
    );
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
        &a, "beta", "question", "Dates", None, QUESTION,
    )));
    let g = ok(commands::guide(&a, &q, "Answer by Friday."));
    assert_eq!(g.json["sent"][0]["to"], "beta");
    assert_eq!(
        g.json["sent"][0]["summary"],
        "The user asks: Answer by Friday."
    );
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
    let r = ok(commands::reply(&b, &q, "answer", None, None, ANSWER));
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
    let bare = Context::open(&fresh, w.root.join("cache-fresh"), TOOL).expect_err("no file");
    assert_eq!((bare.code, bare.repository), ("not_set_up", None));
    git(&fresh, &["remote", "add", "origin", &url(&w.root, "fresh")]);
    // BUG-009: "not set up" carries the origin, the key to the slug others use.
    let not_set_up = Context::open(&fresh, w.root.join("cache-fresh"), TOOL).expect_err("no file");
    assert_eq!(
        not_set_up.repository.as_deref(),
        Some(url(&w.root, "fresh").as_str())
    );
    valid(&not_set_up.to_json());
    // ADJ-005: init says the file is still empty and the next commands.
    let init = ok(commands::init(&fresh, "fresh", "Fresh")).text;
    assert!(init.contains("lists no other project yet"), "{init}");
    assert!(init.contains("dcp-integrations add <slug> --url"), "{init}");
    assert!(init.contains("dcp-integrations validate"), "{init}");
    assert!(init.contains("reach this one as 'fresh'"), "{init}");
    assert_eq!(refused(commands::init(&fresh, "fresh", "")).exit, 2);
    let added = ok(commands::add(
        &fresh,
        "alpha",
        "Alpha",
        &url(&w.root, "alpha"),
    ))
    .text;
    assert!(added.contains("1 project(s) listed"), "{added}");
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
        None,
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

#[test]
fn sync_keeps_copies_that_cached_reads_use_without_git() {
    // IMP-005, IMP-006 (ADR-0011): sync fetches and indexes; --cached reads never fetch.
    let w = World::new();
    let alpha = url(&w.root, "alpha");
    let beta = w.cx("beta");
    ok(commands::send(
        &beta, "alpha", "question", "Format", None, QUESTION,
    ));
    let cx = w.cx("alpha");
    let synced = ok(commands::sync(&cx, std::time::Duration::ZERO));
    assert_eq!(synced.json["changed"], true);
    let inboxes = synced.json["inboxes"].as_array().unwrap();
    assert_eq!(inboxes.len(), 2);
    assert!(
        inboxes.iter().all(|i| i.get("error").is_none()),
        "{inboxes:?}"
    );
    let index = w
        .root
        .join("cache-alpha")
        .join("alpha.git")
        .join("aurora-index.json");
    assert!(index.exists());
    // Nothing new: the second sync changes nothing.
    let again = ok(commands::sync(&cx, std::time::Duration::ZERO));
    assert_eq!(again.json["changed"], false);

    // A message arrives: a cached read keeps the copy, sync brings it.
    ok(commands::send(
        &beta,
        "alpha",
        "notice",
        "Second",
        None,
        "Just so you know.\n",
    ));
    let mut cached = w.cx("alpha");
    cached.transport.cached = true;
    assert_eq!(
        cached.transport.read_inbox("alpha", &alpha).unwrap().len(),
        1
    );
    let check = ok(commands::check(
        &cached,
        false,
        24.0,
        true,
        "dcp-integrations",
    ));
    assert_eq!(check.json["messages"].as_array().unwrap().len(), 1);
    assert_eq!(
        ok(commands::sync(&cx, std::time::Duration::ZERO)).json["changed"],
        true
    );
    assert_eq!(
        cached.transport.read_inbox("alpha", &alpha).unwrap().len(),
        2
    );

    // The index is of a tip: with the objects gone a read of the same tip still answers
    // from it, so no Git ran.
    let objects = w.root.join("cache-alpha").join("alpha.git").join("objects");
    std::fs::rename(&objects, objects.with_extension("moved")).unwrap();
    assert_eq!(
        cached.transport.read_inbox("alpha", &alpha).unwrap().len(),
        2
    );
    std::fs::rename(objects.with_extension("moved"), &objects).unwrap();

    // Marking a message seen is not news for this project's next poll.
    ok(commands::check(&cx, false, 24.0, true, "dcp-integrations"));
    assert_eq!(ok(commands::poll(&cx)).json["changed"], false);
    let id = check.json["messages"][0]["id"].as_str().unwrap().to_owned();
    ok(commands::status(&cx, &id, "seen", None));
    assert_eq!(ok(commands::poll(&cx)).json["changed"], false);

    // A copy never fetched is fetched once, even with --cached.
    let mut fresh = cached.transport.clone();
    fresh.cache = w.root.join("cache-new");
    assert_eq!(fresh.read_inbox("alpha", &alpha).unwrap().len(), 2);
}

#[test]
fn export_gives_conversations_with_their_messages_notes_and_records() {
    // IMP-007: one document holds everything a reader shows for a project.
    let w = World::new();
    let (a, b) = (w.cx("alpha"), w.cx("beta"));
    let q = sent_id(&ok(commands::send(
        &b, "alpha", "question", "Format", None, QUESTION,
    )));
    let answer = sent_id(&ok(commands::reply(&a, &q, "answer", None, None, ANSWER)));
    ok(commands::status(&a, &q, "done", Some("answered")));
    let close = sent_id(&ok(commands::reply(
        &b,
        &answer,
        "agreement",
        None,
        None,
        AGREEMENT,
    )));

    let alpha = ok(commands::export(&a)).json;
    assert_eq!(alpha["project"], "alpha");
    assert_eq!(alpha["items"][1]["slug"], "beta");
    let c = &alpha["conversations"][0];
    assert_eq!(c["thread"], q.as_str());
    let items = c["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "question, answer and agreement");
    // The status note leaves the text and the sections, and stays in the body.
    let first = &items[0];
    assert_eq!(first["notes"][0]["status"], "done");
    assert_eq!(first["notes"][0]["note"], "answered");
    assert!(!first["text"].as_str().unwrap().contains("> done"));
    assert!(first["body"].as_str().unwrap().contains("> done"));
    assert!(
        !first["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["text"].as_str().unwrap().contains("> done"))
    );
    // The agreement to alpha is not read yet; its record is there.
    assert_eq!(c["unread"], 1);
    assert_eq!(c["record"]["outcome"], "agreement");
    assert_eq!(c["record"]["options"][0], "Adopt 1.1");
    assert_eq!(items[2]["message"]["id"], close.as_str());

    // The other side sees the same thread, with alpha's answer not read yet.
    let beta = ok(commands::export(&b)).json;
    assert_eq!(beta["conversations"][0]["thread"], q.as_str());
    assert_eq!(beta["conversations"][0]["unread"], 1);
    assert_eq!(
        beta["conversations"][0]["items"].as_array().unwrap().len(),
        3
    );
}

#[test]
fn a_summary_travels_with_the_message_and_is_never_cut() {
    let w = World::new();
    let (a, b) = (w.cx("alpha"), w.cx("beta"));
    let q = sent_id(&ok(commands::send(
        &a,
        "beta",
        "question",
        "Format",
        Some("I need your export format.\nCan you send it?"),
        QUESTION,
    )));
    let shown = ok(commands::show(&b, &q));
    valid(&shown.json);
    assert_eq!(
        shown.json["message"]["summary"],
        "I need your export format. Can you send it?"
    );
    let r = ok(commands::reply(&b, &q, "answer", None, None, ANSWER));
    valid(&r.json);
    assert_eq!(r.json["sent"][0]["summary"], "");

    let long = "x".repeat(commands::SUMMARY_MAX + 1);
    let f = refused(commands::send(
        &a,
        "beta",
        "notice",
        "Long",
        Some(&long),
        "Just so you know.\n",
    ));
    assert_eq!((f.code, f.exit), ("refused", 5));
    assert!(f.message.contains("at most 280"), "{}", f.message);
    let exact = "y".repeat(commands::SUMMARY_MAX);
    ok(commands::send(
        &a,
        "beta",
        "notice",
        "Exact",
        Some(&exact),
        "Just so you know.\n",
    ));

    let e = ok(commands::export(&b));
    valid(&e.json);
    let summaries: Vec<&str> = e.json["conversations"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["items"].as_array().unwrap())
        .map(|i| i["message"]["summary"].as_str().unwrap())
        .collect();
    assert!(summaries.contains(&"I need your export format. Can you send it?"));
    assert!(summaries.contains(&exact.as_str()));
}
