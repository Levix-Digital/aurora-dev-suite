//! Every project's cycle summary read in parallel through the loader (spec 003 FR-006,
//! spec 005 FR-001): the row state of each project and the reads that fill it, shared by
//! the Portfolio and the Inbox.

use std::collections::HashMap;

use aurora_core::domain::CycleSummary;
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::loader::{Loader, Request};
use crate::backend::{Backend, CommandError};

/// What a view knows of one project's cycle state.
#[derive(Debug, Clone, PartialEq)]
pub enum RowState {
    Reading,
    Read {
        summary: CycleSummary,
        refreshing: bool,
    },
    Failed {
        error: CommandError,
        refreshing: bool,
    },
}

impl RowState {
    /// The state while a new read runs: the last answer stays, marked as refreshing.
    pub fn refreshing(self) -> Self {
        match self {
            RowState::Read { summary, .. } => RowState::Read {
                summary,
                refreshing: true,
            },
            RowState::Failed { error, .. } => RowState::Failed {
                error,
                refreshing: true,
            },
            RowState::Reading => RowState::Reading,
        }
    }

    /// A read of this project runs, first or again.
    pub const fn is_reading(&self) -> bool {
        matches!(
            self,
            RowState::Reading
                | RowState::Read {
                    refreshing: true,
                    ..
                }
                | RowState::Failed {
                    refreshing: true,
                    ..
                }
        )
    }
}

/// The row state of each project id.
pub type Rows = RwSignal<HashMap<String, RowState>>;

/// Asks for `ids` and runs the reads the loader allows now.
pub fn read(backend: Backend, loader: StoredValue<Loader>, rows: Rows, ids: Vec<String>) {
    rows.update(|rows| {
        for id in &ids {
            let next = rows
                .remove(id)
                .map_or(RowState::Reading, RowState::refreshing);
            rows.insert(id.clone(), next);
        }
    });
    let started = loader
        .try_update_value(|l| l.request(ids))
        .unwrap_or_default();
    run(backend, loader, rows, started);
}

/// Runs each read; its answer is applied when the loader still wants it, and every
/// finished read lets the next ones start.
fn run(backend: Backend, loader: StoredValue<Loader>, rows: Rows, requests: Vec<Request>) {
    for request in requests {
        let backend = backend.clone();
        spawn_local(async move {
            let answer = backend.project_cycle_state(request.id.clone()).await;
            let Some((wanted, next)) = loader.try_update_value(|l| l.finish(&request)) else {
                return;
            };
            if wanted {
                let state = match answer {
                    Ok(state) => RowState::Read {
                        summary: CycleSummary::of(&state),
                        refreshing: false,
                    },
                    Err(error) => RowState::Failed {
                        error,
                        refreshing: false,
                    },
                };
                rows.update(|rows| {
                    rows.insert(request.id.clone(), state);
                });
            }
            run(backend, loader, rows, next);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::domain::CyclePhase;

    fn summary() -> CycleSummary {
        CycleSummary {
            phase: CyclePhase::Idle,
            release: None,
            waiting: Vec::new(),
            has_findings: false,
        }
    }

    #[test]
    fn a_refresh_keeps_the_last_answer() {
        let read = RowState::Read {
            summary: summary(),
            refreshing: false,
        };
        assert!(!read.is_reading());
        let again = read.refreshing();
        assert!(again.is_reading());
        assert!(matches!(again, RowState::Read { .. }));
        assert!(RowState::Reading.refreshing().is_reading());
    }
}
