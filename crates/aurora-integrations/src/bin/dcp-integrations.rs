//! The `dcp-integrations` command line (specs/008-messages/contracts/cli.md).

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use aurora_integrations::VERSION;
use aurora_integrations::commands::{self, Context, Failure, Reply};
use aurora_integrations::transport;

const USAGE: &str =
    "usage: dcp-integrations [--dir <root>] [--json] [--verbose] <command> [arguments]

commands:
  version | init --project <slug> [--name <n>] | add <slug> --url <u> [--name <n>] | validate
  projects | poll | sync [--max-age S] | check [--no-sent] [--stale-hours H] | export
  list [--sent] [--all] | show <id>
  template <kind> | send --to <slug> --kind <kind> --subject <s> --body-file <f>
  reply <id> --kind <kind> [--subject <s>] --body-file <f> | thread <thread> | record <thread>
  decide <thread> (--choice <text> [--note <text>] | --revise <text>) | guide <thread> --text <text>
  status <id> open|seen|done [--note <text>] | defer <id> --reason <text>
  watch --target <slug>=<dir> [--target ...] --exec <cmd> [--interval S] [--once]

The reading commands (projects, check, export, list, show, thread, record) take --cached: they
read the local caches and fetch only an inbox never fetched before; sync updates them.";

/// The arguments after the global options: positionals and `--name value` options.
struct Args {
    positional: Vec<String>,
    options: Vec<(String, Option<String>)>,
}

const FLAGS: &[&str] = &[
    "--no-sent",
    "--sent",
    "--all",
    "--once",
    "--quiet",
    "--cached",
];

impl Args {
    fn parse(raw: &[String]) -> Result<Args, Failure> {
        let mut positional = Vec::new();
        let mut options = Vec::new();
        let mut it = raw.iter();
        while let Some(a) = it.next() {
            if let Some((name, value)) = a.split_once('=').filter(|_| a.starts_with("--")) {
                options.push((name.to_owned(), Some(value.to_owned())));
            } else if FLAGS.contains(&a.as_str()) {
                options.push((a.clone(), None));
            } else if a.starts_with("--") {
                let value = it
                    .next()
                    .ok_or_else(|| usage(format!("{a} needs a value")))?;
                options.push((a.clone(), Some(value.clone())));
            } else {
                positional.push(a.clone());
            }
        }
        Ok(Args {
            positional,
            options,
        })
    }

    fn flag(&self, name: &str) -> bool {
        self.options.iter().any(|(n, _)| n == name)
    }

    fn opt(&self, name: &str) -> Option<&str> {
        self.options
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .and_then(|(_, v)| v.as_deref())
    }

    fn all(&self, name: &str) -> Vec<&str> {
        self.options
            .iter()
            .filter(|(n, _)| n == name)
            .filter_map(|(_, v)| v.as_deref())
            .collect()
    }

    fn need(&self, name: &str) -> Result<&str, Failure> {
        self.opt(name)
            .ok_or_else(|| usage(format!("{name} is required")))
    }

    fn arg(&self, i: usize, what: &str) -> Result<&str, Failure> {
        self.positional
            .get(i)
            .map(String::as_str)
            .ok_or_else(|| usage(format!("<{what}> is required")))
    }
}

fn usage(message: String) -> Failure {
    Failure::new("invalid", 2, format!("{message}\n{USAGE}"))
}

fn body_file(path: &str) -> Result<String, Failure> {
    std::fs::read_to_string(path)
        .map_err(|e| Failure::new("io", 1, format!("cannot read {path}: {e}")))
}

/// The repository root above `dir`: the nearest folder with `.git` or `.aurora`.
fn root_of(dir: &Path) -> PathBuf {
    let start = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    start
        .ancestors()
        .find(|p| p.join(".aurora").is_dir() || p.join(".git").exists())
        .map(Path::to_path_buf)
        .unwrap_or(start)
}

fn run(root: &Path, command: &str, a: &Args) -> Result<Reply, Failure> {
    let cache = transport::default_cache();
    let cx = || Context::open(root, cache.clone(), VERSION);
    // Read-only commands reuse a fetch of the last seconds: Aurora runs one per project
    // and each reads every counterpart's inbox (BUG-008).
    // With --cached they never fetch an inbox fetched before: sync keeps the caches
    // current (ADR-0011).
    let reading = || {
        cx().map(|mut c| {
            c.transport.reuse_fetch = Some(transport::REUSE_FETCH);
            c.transport.cached = a.flag("--cached");
            c
        })
    };
    match command {
        "version" => commands::version(VERSION),
        "template" => commands::template(a.arg(1, "kind")?),
        "init" => commands::init(root, a.need("--project")?, a.opt("--name").unwrap_or("")),
        "add" => commands::add(
            root,
            a.arg(1, "slug")?,
            a.opt("--name").unwrap_or(""),
            a.need("--url")?,
        ),
        "validate" => commands::validate(root),
        "projects" => commands::projects(&cx()?),
        "poll" => commands::poll(&cx()?),
        "sync" => {
            let max_age = match a.opt("--max-age") {
                Some(s) => Duration::from_secs(
                    s.parse()
                        .map_err(|_| usage(format!("--max-age {s} is not a number of seconds")))?,
                ),
                None => transport::SYNC_REUSE,
            };
            commands::sync(&cx()?, max_age)
        }
        "check" => {
            let hours = match a.opt("--stale-hours") {
                Some(h) => h
                    .parse()
                    .map_err(|_| usage(format!("--stale-hours {h} is not a number")))?,
                None => 24.0,
            };
            commands::check(
                &reading()?,
                a.flag("--no-sent"),
                hours,
                a.flag("--quiet"),
                "dcp-integrations",
            )
        }
        "list" => commands::list(&reading()?, a.flag("--sent"), a.flag("--all")),
        "show" => commands::show(&reading()?, a.arg(1, "id")?),
        "send" => commands::send(
            &cx()?,
            a.need("--to")?,
            a.need("--kind")?,
            a.need("--subject")?,
            &body_file(a.need("--body-file")?)?,
        ),
        "reply" => commands::reply(
            &cx()?,
            a.arg(1, "id")?,
            a.need("--kind")?,
            a.opt("--subject"),
            &body_file(a.need("--body-file")?)?,
        ),
        "thread" => commands::thread(&reading()?, a.arg(1, "thread")?),
        "export" => commands::export(&reading()?),
        "record" => commands::record(&reading()?, a.arg(1, "thread")?),
        "decide" => {
            let thread = a.arg(1, "thread")?;
            match (a.opt("--choice"), a.opt("--revise")) {
                (Some(choice), None) => commands::decide(&cx()?, thread, choice, a.opt("--note")),
                (None, Some(text)) => commands::guide(&cx()?, thread, text),
                _ => Err(usage("decide takes either --choice or --revise".into())),
            }
        }
        "guide" => commands::guide(&cx()?, a.arg(1, "thread")?, a.need("--text")?),
        "status" => commands::status(
            &cx()?,
            a.arg(1, "id")?,
            a.arg(2, "status")?,
            a.opt("--note"),
        ),
        "defer" => commands::defer(&cx()?, a.arg(1, "id")?, a.need("--reason")?),
        other => Err(usage(format!("unknown command '{other}'"))),
    }
}

fn watch(a: &Args) -> Result<(), Failure> {
    let exec = a.need("--exec")?;
    let mut targets = Vec::new();
    for t in a.all("--target") {
        let (slug, dir) = t
            .split_once('=')
            .ok_or_else(|| usage(format!("--target {t} must be <slug>=<dir>")))?;
        targets.push((slug.to_owned(), root_of(Path::new(dir))));
    }
    if targets.is_empty() {
        return Err(usage("watch needs at least one --target".into()));
    }
    let interval = a
        .opt("--interval")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60)
        .max(5);
    let cache = transport::default_cache();
    loop {
        for line in commands::watch_once(&targets, &cache, VERSION, exec) {
            println!("{line}");
        }
        if a.flag("--once") {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(interval));
    }
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut json = false;
    let mut verbose = false;
    let mut dir: Option<PathBuf> = None;
    let mut rest = Vec::new();
    let mut it = raw.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true,
            "--verbose" => verbose = true,
            "--dir" => dir = it.next().map(PathBuf::from),
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "help" if rest.is_empty() => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => rest.push(a),
        }
    }
    let result = Args::parse(&rest).and_then(|args| {
        let Some(command) = args.positional.first().cloned() else {
            return Err(usage("a command is required".into()));
        };
        if command == "watch" {
            return watch(&args).map(|()| None);
        }
        let root = root_of(&dir.clone().unwrap_or_else(|| PathBuf::from(".")));
        run(&root, &command, &args).map(Some)
    });
    match result {
        Ok(None) => ExitCode::SUCCESS,
        Ok(Some(reply)) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&reply.json).unwrap_or_default()
                );
            } else if !reply.text.is_empty() {
                println!("{}", reply.text);
            }
            ExitCode::from(u8::try_from(reply.code).unwrap_or(1))
        }
        Err(f) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&f.to_json()).unwrap_or_default()
                );
            }
            eprintln!("dcp-integrations: {}", f.message);
            if verbose && let Some(stderr) = &f.git_stderr {
                eprintln!("{stderr}");
            }
            ExitCode::from(u8::try_from(f.exit).unwrap_or(1))
        }
    }
}
