//! A test double for `dcp`, `gedo` and `actx` (feature `fake-engine`). It answers as the
//! engine its file is named after. The mode comes from a `<file>.mode` file next to the
//! binary, else from `AURORA_FAKE_MODE`, else `ready`:
//! ready, no-contract, unsupported, invalid, huge, hang, crash, echo.

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
