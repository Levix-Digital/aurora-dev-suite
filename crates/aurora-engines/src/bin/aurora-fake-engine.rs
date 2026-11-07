//! A test double for `dcp`, `gedo` and `actx` (feature `fake-engine`). It answers as the
//! engine its file is named after. The mode comes from a `<file>.mode` file next to the
//! binary, else from `AURORA_FAKE_MODE`, else `ready`:
//! ready, no-contract, unsupported, invalid, huge, hang, crash, echo, orphan (answers, but
//! leaves a child process holding its output open).
//!
//! As `dcp` it also answers `diagnose --json --dir <root>` with the `busy.json` sample about
//! `<root>`, and with these modes: idle (the `idle.json` sample), warn (a warning on
//! standard error, then the reply), other-root (a reply about another repository), old
//! (schema version 0.9), and unsupported and invalid as for the version reply.
//!
//! As `dcp` it answers `config get --json --dir <root>` with the `get.json` sample and
//! `config set --json [--dry-run] --dir <root> ...` with `preview.json` or `applied.json`,
//! writing its arguments to `<file>.args` (one per line); with the modes refused and stale
//! it answers those samples, and unsupported and invalid as above.
//!
//! As `dcp-integrations` it answers `--json [--dir <root>] <command> ...` with the samples
//! in `tests/fixtures/dcp-integrations`, writing its arguments to `<file>.args` and the
//! content of a `--body-file` to `<file>.body`; `poll` exits with 10 (news). With the
//! modes refused and git it answers those error documents, quiet answers `poll` without
//! news, and unsupported and invalid as above.

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("gedo"));
    let engine = exe
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mode = mode_of(&exe);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let contract = args == ["version", "--json"];
    let plain = args == ["--version"];
    let diagnose_root = match args.as_slice() {
        [command, json, dir, root]
            if command == "diagnose" && json == "--json" && dir == "--dir" =>
        {
            Some(root.clone())
        }
        _ => None,
    };

    let config = matches!(
        args.first()
            .zip(args.get(1))
            .map(|(a, b)| (a.as_str(), b.as_str())),
        Some(("config", "get" | "set"))
    );

    let provider = engine == "dcp-integrations" && args.first().is_some_and(|a| a == "--json");

    if args == ["--sleep-forever"] {
        std::thread::sleep(Duration::from_secs(120));
        return;
    }

    match mode.as_str() {
        "hang" => std::thread::sleep(Duration::from_secs(120)),
        "crash" => {
            eprintln!("{engine}: internal error: the fake engine crashed on purpose");
            std::process::exit(101);
        }
        "huge" => {
            let line = "x".repeat(1023) + "\n";
            let mut out = std::io::stdout().lock();
            for _ in 0..2048 {
                if out.write_all(line.as_bytes()).is_err() {
                    break;
                }
            }
        }
        "echo" => println!("{}", json_strings(&args)),
        "orphan" => {
            // The child inherits standard output and error and outlives this process.
            let _ = std::process::Command::new(&exe)
                .arg("--sleep-forever")
                .spawn();
            println!("{engine} 1.2.3");
        }
        _ if provider => provider_reply(&exe, &mode, &args),
        _ if config && engine == "dcp" => config_reply(&exe, &mode, &args),
        _ if diagnose_root.is_some() && engine == "dcp" => {
            diagnose(&mode, diagnose_root.as_deref().unwrap_or_default());
        }
        "no-contract" if contract => {
            eprintln!("error: unrecognized subcommand 'version'");
            std::process::exit(2);
        }
        "invalid" if contract => println!(
            r#"{{"schema":"{engine}.version","schema_version":"1.0","version":7,"capabilities":"none"}}"#
        ),
        "unsupported" if contract => println!(
            r#"{{"schema":"{engine}.version","schema_version":"2.0","version":"9.0.0","capabilities":[]}}"#
        ),
        "ready" if contract => println!(
            r#"{{"schema":"{engine}.version","schema_version":"1.0","version":"1.2.3","capabilities":[{}]}}"#,
            capabilities(&engine)
        ),
        _ if plain => println!("{engine} 1.2.3"),
        _ => {
            eprintln!("error: unexpected arguments {args:?}");
            std::process::exit(2);
        }
    }
}

const BUSY: &str = include_str!("../../tests/fixtures/dcp-diagnose/busy.json");
const IDLE: &str = include_str!("../../tests/fixtures/dcp-diagnose/idle.json");

fn diagnose(mode: &str, root: &str) {
    let sample = if mode == "idle" { IDLE } else { BUSY };
    let mut reply: serde_json::Value = serde_json::from_str(sample).unwrap_or_default();
    if mode != "other-root" {
        reply["root"] = root.into();
    }
    match mode {
        "warn" => eprintln!("dcp: warning: the platform CLI is slow to answer"),
        "unsupported" => reply["schema_version"] = "2.0".into(),
        "old" => reply["schema_version"] = "0.9".into(),
        "invalid" => reply["plans"] = "none".into(),
        _ => {}
    }
    println!("{reply}");
}

const CONFIG_GET: &str = include_str!("../../tests/fixtures/dcp-config/get.json");
const CONFIG_PREVIEW: &str = include_str!("../../tests/fixtures/dcp-config/preview.json");
const CONFIG_APPLIED: &str = include_str!("../../tests/fixtures/dcp-config/applied.json");
const CONFIG_REFUSED: &str = include_str!("../../tests/fixtures/dcp-config/refused.json");
const CONFIG_STALE: &str = include_str!("../../tests/fixtures/dcp-config/stale.json");

fn config_reply(exe: &std::path::Path, mode: &str, args: &[String]) {
    let root = args
        .iter()
        .position(|arg| arg == "--dir")
        .and_then(|at| args.get(at + 1))
        .cloned()
        .unwrap_or_default();
    let sample = if args[1] == "get" {
        CONFIG_GET
    } else {
        let mut args_file = exe.as_os_str().to_owned();
        args_file.push(".args");
        let _ = std::fs::write(PathBuf::from(args_file), args.join("\n"));
        match mode {
            "refused" => CONFIG_REFUSED,
            "stale" => CONFIG_STALE,
            _ if args.iter().any(|arg| arg == "--dry-run") => CONFIG_PREVIEW,
            _ => CONFIG_APPLIED,
        }
    };
    let mut reply: serde_json::Value = serde_json::from_str(sample).unwrap_or_default();
    reply["root"] = root.into();
    match mode {
        "unsupported" => reply["schema_version"] = "2.0".into(),
        "invalid" => {
            reply["settings"] = "none".into();
            reply["changes"] = "none".into();
        }
        _ => {}
    }
    println!("{reply}");
}

const INTEGRATIONS: [(&str, &str); 12] = [
    (
        "export",
        include_str!("../../tests/fixtures/dcp-integrations/export.json"),
    ),
    (
        "projects",
        include_str!("../../tests/fixtures/dcp-integrations/projects.json"),
    ),
    (
        "check",
        include_str!("../../tests/fixtures/dcp-integrations/check.json"),
    ),
    (
        "thread",
        include_str!("../../tests/fixtures/dcp-integrations/thread.json"),
    ),
    (
        "record",
        include_str!("../../tests/fixtures/dcp-integrations/record.json"),
    ),
    (
        "template",
        include_str!("../../tests/fixtures/dcp-integrations/template.json"),
    ),
    (
        "sent",
        include_str!("../../tests/fixtures/dcp-integrations/sent.json"),
    ),
    (
        "status",
        include_str!("../../tests/fixtures/dcp-integrations/status.json"),
    ),
    (
        "poll",
        include_str!("../../tests/fixtures/dcp-integrations/poll-news.json"),
    ),
    (
        "sync",
        include_str!("../../tests/fixtures/dcp-integrations/sync.json"),
    ),
    (
        "refused",
        include_str!("../../tests/fixtures/dcp-integrations/error-refused.json"),
    ),
    (
        "git",
        include_str!("../../tests/fixtures/dcp-integrations/error-git.json"),
    ),
];

fn integrations(name: &str) -> serde_json::Value {
    let text = INTEGRATIONS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, t)| *t)
        .unwrap_or_default();
    serde_json::from_str(text).unwrap_or_default()
}

fn provider_reply(exe: &std::path::Path, mode: &str, args: &[String]) {
    let sibling = |ext: &str, text: &str| {
        let mut file = exe.as_os_str().to_owned();
        file.push(ext);
        let _ = std::fs::write(PathBuf::from(file), text);
    };
    sibling(".args", &args.join("\n"));
    if let Some(body) = args
        .iter()
        .position(|a| a == "--body-file")
        .and_then(|at| args.get(at + 1))
    {
        sibling(".body", &std::fs::read_to_string(body).unwrap_or_default());
    }
    let rest: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    let command = match rest.as_slice() {
        ["--dir", _, command, ..] | [command, ..] => *command,
        [] => "",
    };
    let name = match command {
        "send" | "reply" | "decide" | "guide" => "sent",
        "status" | "defer" => "status",
        other => other,
    };
    let mut reply = match mode {
        "refused" | "git" if !matches!(name, "projects" | "check" | "export" | "template") => {
            integrations(mode)
        }
        _ => integrations(name),
    };
    let mut code = 0;
    match mode {
        "unsupported" => reply["schema_version"] = "2.0".into(),
        "invalid" => reply["conversations"] = "none".into(),
        "quiet" if name == "poll" => {
            reply = serde_json::json!({"schema": "dcp-integrations.poll", "schema_version": "1.0", "project": "alpha", "changed": false});
        }
        "refused" => code = 5,
        "git" => code = 1,
        _ if name == "poll" => code = 10,
        _ => {}
    }
    if reply.is_null() {
        eprintln!("error: unexpected arguments {args:?}");
        std::process::exit(2);
    }
    println!("{reply}");
    std::process::exit(code);
}

fn mode_of(exe: &std::path::Path) -> String {
    let mut mode_file = exe.as_os_str().to_owned();
    mode_file.push(".mode");
    std::fs::read_to_string(PathBuf::from(mode_file))
        .ok()
        .or_else(|| std::env::var("AURORA_FAKE_MODE").ok())
        .map(|mode| mode.trim().to_owned())
        .unwrap_or_else(|| "ready".to_owned())
}

fn capabilities(engine: &str) -> String {
    let names: &[&str] = match engine {
        "dcp" => &["diagnose", "config"],
        "gedo" => &["doctor", "radar", "scope", "stream"],
        "actx" => &["ask", "workspaces"],
        "dcp-integrations" => &["messages"],
        _ => &[],
    };
    names
        .iter()
        .map(|name| format!(r#"{{"name":"{name}","schema_major":1}}"#))
        .collect::<Vec<_>>()
        .join(",")
}

fn json_strings(values: &[String]) -> String {
    let quoted: Vec<String> = values
        .iter()
        .map(|value| format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();
    format!("[{}]", quoted.join(","))
}
