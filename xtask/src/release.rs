//! The release tag rule: a release tag is `vX.Y.Z` (final) or `vX.Y.Z-rc.N` (release
//! candidate), and `X.Y.Z` is the workspace version, so the packages built from the tag carry
//! the version the tag names.

/// The kind of a release tag that matches the workspace version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    Final,
    Candidate,
}

impl TagKind {
    pub fn label(self) -> &'static str {
        match self {
            TagKind::Final => "final release",
            TagKind::Candidate => "release candidate",
        }
    }
}

/// Checks `tag` against the workspace `version` (`X.Y.Z`, without `v`).
pub fn check_tag(tag: &str, version: &str) -> Result<TagKind, String> {
    let malformed = || format!("'{tag}' is not a release tag like v{version} or v{version}-rc.1");
    let rest = tag.strip_prefix('v').ok_or_else(malformed)?;
    let (tagged, kind) = match rest.split_once('-') {
        None => (rest, TagKind::Final),
        Some((tagged, suffix)) => {
            let number = suffix.strip_prefix("rc.").ok_or_else(malformed)?;
            if !is_number(number) {
                return Err(malformed());
            }
            (tagged, TagKind::Candidate)
        }
    };
    let parts: Vec<&str> = tagged.split('.').collect();
    if parts.len() != 3 || !parts.iter().all(|part| is_number(part)) {
        return Err(malformed());
    }
    if tagged != version {
        return Err(format!(
            "'{tag}' names version {tagged}, but the workspace version is {version}; bump the version in Cargo.toml first"
        ));
    }
    Ok(kind)
}

fn is_number(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_final_tag_of_the_workspace_version_is_accepted() {
        assert_eq!(check_tag("v0.2.0", "0.2.0"), Ok(TagKind::Final));
    }

    #[test]
    fn a_candidate_tag_of_the_workspace_version_is_accepted() {
        assert_eq!(check_tag("v0.2.0-rc.1", "0.2.0"), Ok(TagKind::Candidate));
        assert_eq!(check_tag("v0.2.0-rc.12", "0.2.0"), Ok(TagKind::Candidate));
    }

    #[test]
    fn another_version_is_refused_naming_both_versions() {
        let error = check_tag("v0.3.0-rc.1", "0.2.0").unwrap_err();
        assert!(error.contains("0.3.0"), "{error}");
        assert!(error.contains("0.2.0"), "{error}");
    }

    #[test]
    fn malformed_tags_are_refused_naming_the_tag() {
        for tag in [
            "",
            "0.2.0",
            "v0.2",
            "v0.2.0.1",
            "v0.2.x",
            "v0.2.0-rc",
            "v0.2.0-rc.",
            "v0.2.0-rc.a",
            "v0.2.0-beta.1",
            "v0.2.0-rc.1-extra",
        ] {
            let error = check_tag(tag, "0.2.0").unwrap_err();
            assert!(error.contains(&format!("'{tag}'")), "{tag}: {error}");
            assert!(error.contains("v0.2.0-rc.1"), "{tag}: {error}");
        }
    }
}
