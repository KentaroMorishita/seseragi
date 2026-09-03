//! Shared distribution metadata for every official Seseragi tool.

use serde::Serialize;

mod unicode_version;
pub use unicode_version::{UNICODE_VERSION, UNICODE_VERSION_TUPLE};

pub const TOOLCHAIN_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT_SHA: &str = env!("SESERAGI_COMMIT_SHA");
pub const BUILD_CHANNEL: &str = env!("SESERAGI_BUILD_CHANNEL");
pub const BUILD_TARGET: &str = env!("SESERAGI_BUILD_TARGET");
const BUILD_DIRTY: &str = env!("SESERAGI_BUILD_DIRTY");
const RELEASE_TAG: &str = env!("SESERAGI_RELEASE_TAG");

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub unicode_version: &'static str,
    pub commit: &'static str,
    pub channel: &'static str,
    pub target: &'static str,
    pub dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_tag: Option<&'static str>,
}

pub fn build_metadata(name: &'static str) -> BuildMetadata {
    BuildMetadata {
        name,
        version: TOOLCHAIN_VERSION,
        unicode_version: UNICODE_VERSION,
        commit: COMMIT_SHA,
        channel: BUILD_CHANNEL,
        target: BUILD_TARGET,
        dirty: BUILD_DIRTY == "true",
        release_tag: (!RELEASE_TAG.is_empty()).then_some(RELEASE_TAG),
    }
}

pub fn format_human_version(name: &'static str) -> String {
    let metadata = build_metadata(name);
    let dirty = if metadata.dirty { ", dirty" } else { "" };
    format!(
        "{} {} ({}, commit {}, target {}{})",
        metadata.name, metadata.version, metadata.channel, metadata.commit, metadata.target, dirty
    )
}

pub fn classify_channel(version: &str, tag: Option<&str>, dirty: bool) -> &'static str {
    let expected_tag = format!("v{version}");
    if !dirty && tag == Some(expected_tag.as_str()) {
        "release"
    } else {
        "development"
    }
}

#[cfg(test)]
mod tests {
    use super::classify_channel;

    #[test]
    fn a_release_requires_the_matching_clean_tag() {
        assert_eq!(classify_channel("0.4.0", Some("v0.4.0"), false), "release");
        assert_eq!(
            classify_channel("0.4.0", Some("v0.4.0"), true),
            "development"
        );
        assert_eq!(
            classify_channel("0.4.0", Some("v0.3.0"), false),
            "development"
        );
        assert_eq!(classify_channel("0.4.0", None, false), "development");
    }
}
