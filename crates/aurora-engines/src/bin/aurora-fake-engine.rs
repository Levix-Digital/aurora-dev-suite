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
