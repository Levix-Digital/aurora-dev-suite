//! `CycleSource` over `dcp diagnose --json --dir <root>` (schema `dcp.diagnose` MAJOR 1,
//! specs/002-dcp-adapter/contracts/dcp-diagnose.schema.json). Nothing else is run or read
//! (ADR-0006).

use std::path::Path;
use std::time::Duration;

use aurora_core::domain::{
    CandidateRun, CurrentRelease, CycleError, CycleState, EngineReport, EngineSettings, ItemRef,
    NextOption, NextStep, PlanProgress, PullRequestState, UntriagedBug,
};
use aurora_core::ports::CycleSource;
use serde::Deserialize;

use crate::dcp;
use crate::runner::CommandSpec;

/// The schema of the reply.
pub const DIAGNOSE_SCHEMA: &str = "dcp.diagnose";
/// The MAJOR version of the reply this Aurora understands.
pub const SUPPORTED_MAJOR: u32 = 1;

/// Reads cycle states from the `dcp` binary found by the latest engine probe.
#[derive(Debug, Default, Clone)]
pub struct DcpCycleSource;

impl DcpCycleSource {
    pub fn new() -> Self {
        Self
    }
}

impl CycleSource for DcpCycleSource {
    fn cycle_state(
        &self,
        engine: &EngineReport,
        settings: &EngineSettings,
        root: &Path,
    ) -> Result<CycleState, CycleError> {
        let Some(program) = &engine.location else {
            return Err(CycleError::Unavailable {
                hint: "the Dev Cycle Protocol (dcp) was not found".to_owned(),
            });
        };
        let timeout = Duration::from_millis(settings.timeout_ms);
        let mut spec = CommandSpec::new(program, &["diagnose", "--json", "--dir"], timeout);
        spec.args.push(root.to_string_lossy().into_owned());

        let started = std::time::Instant::now();
        let result = read(&spec, root);
        let elapsed_ms = dcp::elapsed_ms(started);
        match &result {
            Ok(_) => tracing::info!(root = %root.display(), elapsed_ms, "cycle state read"),
            Err(error) => {
                tracing::warn!(root = %root.display(), elapsed_ms, %error, "cycle state failed");
            }
        }
        result
    }
}

fn read(spec: &CommandSpec, root: &Path) -> Result<CycleState, CycleError> {
    let output = dcp::run(spec)?;
    parse_diagnose_reply(&output.stdout, root, &crate::now_rfc3339())
        .map_err(|error| dcp::with_stderr(error, output.stderr))
}

/// Parses a `dcp.diagnose` reply about `root`, read at `read_at`.
pub fn parse_diagnose_reply(
    bytes: &[u8],
    root: &Path,
    read_at: &str,
) -> Result<CycleState, CycleError> {
    let value = dcp::envelope(bytes, DIAGNOSE_SCHEMA, SUPPORTED_MAJOR)?;
    let raw: RawDiagnose =
        serde_json::from_value(value).map_err(|error| dcp::invalid(error.to_string()))?;
    dcp::check_root(&raw.root, root)?;
    Ok(raw.into_state(read_at))
}

#[derive(Deserialize)]
struct RawDiagnose {
    root: String,
    summary: String,
    next: RawNext,
    release: Option<RawRelease>,
    plans: Vec<RawPlan>,
    release_candidate: Option<RawCandidate>,
    pull_requests: Vec<RawPullRequest>,
    bugs: RawBugs,
    findings: Vec<String>,
}

#[derive(Deserialize)]
struct RawNext {
    ask: bool,
    recommended: Option<String>,
    options: Vec<RawOption>,
}

#[derive(Deserialize)]
struct RawOption {
    key: String,
    title: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
struct RawItem {
    id: String,
    rank: Option<String>,
    title: String,
}

#[derive(Deserialize)]
struct RawUntriaged {
    #[serde(flatten)]
    item: RawItem,
    reason: String,
}

#[derive(Deserialize)]
struct RawRelease {
    version: String,
    name: String,
    open_items: Vec<RawItem>,
}

#[derive(Deserialize)]
struct RawPlan {
    id: String,
    title: String,
    status: String,
    milestones_done: u32,
    milestones_total: u32,
}

#[derive(Deserialize)]
struct RawCandidate {
    run: String,
    version: String,
    not_run: u32,
    failed: u32,
    released: bool,
}

#[derive(Deserialize)]
struct RawPullRequest {
    number: u64,
    title: String,
    summary: String,
    needs_action: bool,
    threads_for_user: u32,
}

#[derive(Deserialize)]
struct RawBugs {
    blocking: Vec<RawItem>,
    untriaged: Vec<RawUntriaged>,
}

impl From<RawItem> for ItemRef {
    fn from(raw: RawItem) -> Self {
        ItemRef {
            id: raw.id,
            rank: raw.rank,
            title: raw.title,
        }
    }
}

impl RawDiagnose {
    fn into_state(self, read_at: &str) -> CycleState {
        CycleState {
            summary: self.summary,
            next: NextStep {
                ask: self.next.ask,
                recommended: self.next.recommended,
                options: self
                    .next
                    .options
                    .into_iter()
                    .map(|o| NextOption {
                        key: o.key,
                        title: o.title,
                        reason: o.reason,
                    })
                    .collect(),
            },
            release: self.release.map(|r| CurrentRelease {
                version: r.version,
                name: r.name,
                open_items: r.open_items.into_iter().map(ItemRef::from).collect(),
            }),
            plans: self
                .plans
                .into_iter()
                .map(|p| PlanProgress {
                    id: p.id,
                    title: p.title,
                    status: p.status,
                    milestones_done: p.milestones_done,
                    milestones_total: p.milestones_total,
                })
                .collect(),
            release_candidate: self.release_candidate.map(|c| CandidateRun {
                run: c.run,
                version: c.version,
                not_run: c.not_run,
                failed: c.failed,
                released: c.released,
            }),
            pull_requests: self
                .pull_requests
                .into_iter()
                .map(|p| PullRequestState {
                    number: p.number,
                    title: p.title,
                    summary: p.summary,
                    needs_action: p.needs_action,
                    threads_for_user: p.threads_for_user,
                })
                .collect(),
            blocking_bugs: self.bugs.blocking.into_iter().map(ItemRef::from).collect(),
            untriaged_bugs: self
                .bugs
                .untriaged
                .into_iter()
                .map(|u| UntriagedBug {
                    item: u.item.into(),
                    reason: u.reason,
                })
                .collect(),
            findings: self.findings,
            read_at: read_at.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::domain::UpdateSide;

    const ROOT: &str = "/repos/gedo";

    fn reply(schema: &str, version: &str) -> String {
        format!(
            r#"{{"schema":"{schema}","schema_version":"{version}","root":"{ROOT}","summary":"s",
            "next":{{"ask":false,"recommended":null,"options":[]}},"release":null,"plans":[],
            "release_candidate":null,"pull_requests":[],"bugs":{{"blocking":[],"untriaged":[]}},
            "findings":[]}}"#
        )
    }

    fn parse(text: &str) -> Result<CycleState, CycleError> {
        parse_diagnose_reply(text.as_bytes(), Path::new(ROOT), "2026-10-30T00:00:00Z")
    }

    #[test]
    fn a_minimal_reply_is_an_empty_state() {
        let state = parse(&reply("dcp.diagnose", "1.0")).expect("valid");
        assert!(state.plans.is_empty() && state.release.is_none());
        assert_eq!(state.read_at, "2026-10-30T00:00:00Z");
    }

    #[test]
    fn plain_text_and_other_schemas_are_invalid() {
        assert!(matches!(
            parse("dcp 2.12.0"),
            Err(CycleError::Invalid { .. })
        ));
        assert!(matches!(
            parse(r#"{"summary":"s"}"#),
            Err(CycleError::Invalid { .. })
        ));
        assert!(matches!(
            parse(&reply("dcp.version", "1.0")),
            Err(CycleError::Invalid { .. })
        ));
        assert!(matches!(
            parse(&reply("dcp.diagnose", "one")),
            Err(CycleError::Invalid { .. })
        ));
    }

    #[test]
    fn another_major_says_which_side_to_update() {
        assert_eq!(
            parse(&reply("dcp.diagnose", "2.0")),
            Err(CycleError::Unsupported {
                found: 2,
                update: UpdateSide::Aurora
            })
        );
        assert_eq!(
            parse(&reply("dcp.diagnose", "0.9")),
            Err(CycleError::Unsupported {
                found: 0,
                update: UpdateSide::Engine
            })
        );
    }

    #[test]
    fn a_reply_about_another_repository_is_invalid() {
        let other = reply("dcp.diagnose", "1.0").replace(ROOT, "/repos/other");
        let Err(CycleError::Invalid { message, .. }) = parse(&other) else {
            panic!("expected invalid");
        };
        assert!(message.contains("/repos/other"), "{message}");
    }

    #[test]
    fn missing_fields_are_invalid() {
        let without_bugs =
            reply("dcp.diagnose", "1.0").replace(r#""bugs":{"blocking":[],"untriaged":[]},"#, "");
        assert!(matches!(
            parse(&without_bugs),
            Err(CycleError::Invalid { .. })
        ));
    }
}
