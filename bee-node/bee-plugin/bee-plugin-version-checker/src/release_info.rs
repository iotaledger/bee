// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use semver::Version;
use serde::Deserialize;

use std::cmp;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ReleaseInfoBuilder {
    html_url: String,
    tag_name: String,
}

impl ReleaseInfoBuilder {
    /// Attempts to build a [`ReleaseInfo`].
    ///
    /// Returns:
    ///  - `None` if there is an error parsing a version from the release tag.
    ///  - `None` if this is a pre-release.
    ///  - `Some` otherwise.
    pub(crate) fn build(self) -> Option<ReleaseInfo> {
        let mut version = self.tag_name.clone();
        version.retain(|c| c != 'v');

        match Version::parse(&version) {
            Err(e) => {
                log::warn!("error parsing version from tag {}: {}", self.tag_name, e);
                None
            }
            Ok(version) => version.pre.is_empty().then(|| ReleaseInfo {
                html_url: self.html_url,
                version,
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ReleaseInfo {
    pub html_url: String,
    pub version: Version,
}

impl Ord for ReleaseInfo {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for ReleaseInfo {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
