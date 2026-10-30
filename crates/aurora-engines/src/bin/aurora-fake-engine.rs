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
        "dcp" => &["diagnose"],
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
