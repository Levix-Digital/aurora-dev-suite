//! Which engine capabilities each Aurora feature needs, and whether the engines found
//! provide them (data-model.md, "Feature requirements").

use crate::domain::{
    AvailabilityState, EngineKind, EngineReport, EngineStatus, Feature, FeatureAvailability,
};

/// The release of Aurora this build is.
pub const CURRENT_RELEASE: &str = "v0.1.0";

/// What one feature needs from one engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requirement {
    pub feature: Feature,
    pub engine: EngineKind,
    /// Capability names, all with schema MAJOR 1.
    pub capabilities: &'static [&'static str],
    pub hint: &'static str,
    /// The Aurora release that ships the feature.
    pub release: &'static str,
}

const SCHEMA_MAJOR: u32 = 1;

const DCP_HINT: &str = "Install the Dev Cycle Protocol (dcp) with its JSON contract";

const CATALOG: [Requirement; 6] = [
    Requirement {
        feature: Feature::Portfolio,
        engine: EngineKind::DevCycle,
        capabilities: &["diagnose"],
        hint: DCP_HINT,
        release: "v0.2.0",
    },
    Requirement {
        feature: Feature::ProjectView,
        engine: EngineKind::DevCycle,
        capabilities: &["diagnose"],
        hint: DCP_HINT,
        release: "v0.2.0",
    },
    Requirement {
        feature: Feature::Inbox,
        engine: EngineKind::DevCycle,
        capabilities: &["diagnose"],
        hint: DCP_HINT,
        release: "v0.2.0",
    },
    Requirement {
        feature: Feature::ProjectSettings,
        engine: EngineKind::DevCycle,
        capabilities: &[CONFIG_CAPABILITY],
        hint: DCP_HINT,
        release: "v0.3.0",
    },
    Requirement {
        feature: Feature::Agents,
        engine: EngineKind::Gedo,
        capabilities: &["radar", "scope", "stream"],
        hint: "Install Gedo 0.5 or later",
        release: "v0.4.0",
    },
    Requirement {
        feature: Feature::Knowledge,
        engine: EngineKind::AnyContext,
        capabilities: &["ask", "workspaces"],
        hint: "Update AnyContext to a version with --json output",
        release: "v0.5.0",
    },
];

/// The capability of `dcp` that answers a project's cycle state (`dcp diagnose --json`).
pub const CYCLE_CAPABILITY: &str = "diagnose";

/// The capability of `dcp` that reads and changes a project's settings (`dcp config get`
/// and `dcp config set`, FEAT-027).
pub const CONFIG_CAPABILITY: &str = "config";

/// The `dcp` report when it is ready with the cycle capability at the supported MAJOR;
/// otherwise what to install or update.
pub fn usable_dcp(report: Option<&EngineReport>) -> Result<&EngineReport, String> {
    usable_dcp_for(Feature::ProjectView, report)
}

/// The `dcp` report when it is ready with every capability `feature` needs at the
/// supported MAJOR; otherwise what to install or update.
pub fn usable_dcp_for(
    feature: Feature,
    report: Option<&EngineReport>,
) -> Result<&EngineReport, String> {
    let requirement = requirement(feature);
    match report {
        Some(report)
            if report.kind == EngineKind::DevCycle
                && requirement.engine == EngineKind::DevCycle
                && requirement
                    .capabilities
                    .iter()
                    .all(|name| report.has_capability(name, SCHEMA_MAJOR)) =>
        {
            Ok(report)
        }
        _ => Err(hint_for(requirement, report)),
    }
}

pub fn requirement(feature: Feature) -> Requirement {
    CATALOG
        .into_iter()
        .find(|requirement| requirement.feature == feature)
        .unwrap_or(CATALOG[0])
}

/// The features that depend on an engine.
pub fn features_of(engine: EngineKind) -> Vec<Feature> {
    CATALOG
        .iter()
        .filter(|requirement| requirement.engine == engine)
        .map(|requirement| requirement.feature)
        .collect()
}

/// Whether `feature` can run with the engines in `reports`, in this release.
pub fn availability(feature: Feature, reports: &[EngineReport]) -> FeatureAvailability {
    let requirement = requirement(feature);
    let report = reports
        .iter()
        .find(|report| report.kind == requirement.engine);
    let missing: Vec<String> = requirement
        .capabilities
        .iter()
        .filter(|name| !report.is_some_and(|report| report.has_capability(name, SCHEMA_MAJOR)))
        .map(|name| (*name).to_owned())
        .collect();
    let state = if !missing.is_empty() {
        AvailabilityState::Unavailable {
            engine: requirement.engine,
            missing,
            hint: hint_for(requirement, report),
        }
    } else if release_number(requirement.release) > release_number(CURRENT_RELEASE) {
        AvailabilityState::NotInThisRelease {
            release: requirement.release.to_owned(),
        }
    } else {
        AvailabilityState::Available
    };
    FeatureAvailability { feature, state }
}

pub fn all_availability(reports: &[EngineReport]) -> Vec<FeatureAvailability> {
    Feature::ALL
        .iter()
        .map(|feature| availability(*feature, reports))
        .collect()
}

fn hint_for(requirement: Requirement, report: Option<&EngineReport>) -> String {
    match report.map(|report| &report.status) {
        Some(EngineStatus::Degraded { reason }) => {
            format!("{} is not working: {reason}", requirement.engine)
        }
        Some(EngineStatus::Ready) => format!(
            "{}: this {} version lacks a needed capability",
            requirement.hint, requirement.engine
        ),
        _ => requirement.hint.to_owned(),
    }
}

/// "v1.2.3" as a comparable tuple; anything unparsable counts as zero.
fn release_number(release: &str) -> (u32, u32, u32) {
    let mut parts = release
        .trim_start_matches('v')
        .split('.')
        .map(|part| part.parse().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Capability;

    fn report(kind: EngineKind, status: EngineStatus, capabilities: &[&str]) -> EngineReport {
        EngineReport {
            kind,
            status,
            location: None,
            version: Some("1.0.0".to_owned()),
            capabilities: capabilities.iter().map(|c| Capability::new(c, 1)).collect(),
            last_error: None,
            checked_at: "2026-10-27T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn missing_engine_makes_its_features_unavailable_with_the_hint() {
        let state = availability(Feature::Agents, &[]).state;
        assert_eq!(
            state,
            AvailabilityState::Unavailable {
                engine: EngineKind::Gedo,
                missing: vec!["radar".into(), "scope".into(), "stream".into()],
                hint: "Install Gedo 0.5 or later".into(),
            }
        );
    }

    #[test]
    fn engine_without_contract_provides_nothing() {
        let reports = [report(
            EngineKind::AnyContext,
            EngineStatus::InstalledWithoutContract,
            &["ask", "workspaces"],
        )];
        assert!(matches!(
            availability(Feature::Knowledge, &reports).state,
            AvailabilityState::Unavailable { .. }
        ));
    }

    #[test]
    fn degraded_engine_hint_says_why() {
        let reports = [report(
            EngineKind::DevCycle,
            EngineStatus::Degraded {
                reason: "timed out".into(),
            },
            &[],
        )];
        let AvailabilityState::Unavailable { hint, .. } =
            availability(Feature::Portfolio, &reports).state
        else {
            panic!("expected unavailable");
        };
        assert_eq!(hint, "Dev Cycle Protocol is not working: timed out");
    }

    #[test]
    fn ready_engine_missing_one_capability_lists_only_that_one() {
        let reports = [report(
            EngineKind::Gedo,
            EngineStatus::Ready,
            &["radar", "scope"],
        )];
        let AvailabilityState::Unavailable { missing, .. } =
            availability(Feature::Agents, &reports).state
        else {
            panic!("expected unavailable");
        };
        assert_eq!(missing, vec!["stream".to_owned()]);
    }

    #[test]
    fn capability_with_another_schema_major_does_not_count() {
        let mut gedo = report(EngineKind::Gedo, EngineStatus::Ready, &["radar", "scope"]);
        gedo.capabilities.push(Capability::new("stream", 2));
        assert!(matches!(
            availability(Feature::Agents, &[gedo]).state,
            AvailabilityState::Unavailable { .. }
        ));
    }

    #[test]
    fn capability_of_another_engine_does_not_count() {
        let reports = [report(
            EngineKind::AnyContext,
            EngineStatus::Ready,
            &["radar", "scope", "stream"],
        )];
        assert!(matches!(
            availability(Feature::Agents, &reports).state,
            AvailabilityState::Unavailable { .. }
        ));
    }

    #[test]
    fn satisfied_feature_of_a_later_release_says_when() {
        let reports = [report(
            EngineKind::DevCycle,
            EngineStatus::Ready,
            &["diagnose"],
        )];
        assert_eq!(
            availability(Feature::Portfolio, &reports).state,
            AvailabilityState::NotInThisRelease {
                release: "v0.2.0".into()
            }
        );
    }

    #[test]
    fn every_feature_has_a_requirement() {
        for feature in Feature::ALL {
            assert_eq!(requirement(feature).feature, feature);
        }
        assert_eq!(all_availability(&[]).len(), Feature::ALL.len());
    }

    #[test]
    fn features_are_grouped_by_engine() {
        assert_eq!(
            features_of(EngineKind::DevCycle),
            vec![
                Feature::Portfolio,
                Feature::ProjectView,
                Feature::Inbox,
                Feature::ProjectSettings
            ]
        );
        assert_eq!(
            features_of(EngineKind::AnyContext),
            vec![Feature::Knowledge]
        );
    }

    #[test]
    fn settings_need_the_config_capability() {
        let cycle_only = report(EngineKind::DevCycle, EngineStatus::Ready, &["diagnose"]);
        assert!(usable_dcp(Some(&cycle_only)).is_ok());
        let hint = usable_dcp_for(Feature::ProjectSettings, Some(&cycle_only))
            .expect_err("no config capability");
        assert!(hint.contains("lacks a needed capability"), "{hint}");
        let both = report(
            EngineKind::DevCycle,
            EngineStatus::Ready,
            &["diagnose", "config"],
        );
        assert!(usable_dcp_for(Feature::ProjectSettings, Some(&both)).is_ok());
        assert!(usable_dcp_for(Feature::ProjectSettings, None).is_err());
    }

    #[test]
    fn releases_compare_numerically() {
        assert!(release_number("v0.10.0") > release_number("v0.2.0"));
    }
}
