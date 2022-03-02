// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, process::Command};

#[allow(dead_code)]
#[derive(Debug)]
enum BuildError {
    DashboardDownload,
    DashboardInvalidArchive,
    DashboardInvalidChecksum,
    DashboardRequest(Option<u16>, String),
    GitBranch,
    GitCommit,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::DashboardDownload => write!(f, "failed to download latest release archive"),
            Self::DashboardInvalidArchive => write!(f, "failed to open or extract release archive"),
            Self::DashboardInvalidChecksum => write!(
                f,
                "checksum of downloaded archive did not match that specified in the release"
            ),
            Self::DashboardRequest(Some(code), url) => write!(f, "failed request to `{}`: status code {}", url, code),
            Self::DashboardRequest(_, url) => write!(f, "failed request to `{}`", url),
            Self::GitBranch => write!(f, "failed to retrieve git branch name"),
            Self::GitCommit => write!(f, "failed to retrieve git commit"),
        }
    }
}

fn main() -> Result<(), BuildError> {
    parse_git_information()?;

    #[cfg(feature = "dashboard")]
    {
        let should_fetch = std::env::var("FETCH_DASHBOARD").map(|val| val == "1").unwrap_or(false);

        let dashboard_dir = std::env::var("OUT_DIR").unwrap() + "/dashboard";
        println!("cargo:rustc-env=DASHBOARD_DIR={}", dashboard_dir);

        // Rebuild if FETCH_DASHBOARD environment variable has changed.
        println!("cargo:rerun-if-env-changed=FETCH_DASHBOARD");
        // Rebuild if DASHBOARD_DIR has changed to a different path.
        println!("cargo:rerun-if-env-changed=DASHBOARD_DIR");

        let dashboard_dir = std::path::Path::new(&dashboard_dir);

        if should_fetch {
            if dashboard_dir.exists() {
                // If the path already exists, we are re-downloading: remove the old files.
                std::fs::remove_dir_all(dashboard_dir).expect("could not remove existing dashboard");
            }

            dashboard::fetch(dashboard_dir)?;
        }
    }

    Ok(())
}

fn parse_git_information() -> Result<(), BuildError> {
    match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(output) => {
            println!(
                "cargo:rustc-env=GIT_COMMIT={}",
                String::from_utf8(output.stdout).unwrap()
            );
        }
        Err(_) => return Err(BuildError::GitCommit),
    }

    match Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        Ok(output) => {
            println!("cargo:rerun-if-changed=../.git/HEAD");
            println!(
                "cargo:rerun-if-changed=../.git/refs/heads/{}",
                String::from_utf8(output.stdout).unwrap(),
            );
        }
        Err(_) => return Err(BuildError::GitBranch),
    }

    Ok(())
}

#[cfg(feature = "dashboard")]
mod dashboard {
    use super::*;

    use sha2::{Digest, Sha256};
    use zip::ZipArchive;

    use std::{
        io::{self, BufReader, Cursor},
        path::Path,
    };

    const RELEASE_URL: &str =
        "https://github.com/iotaledger/node-dashboard/releases/download/v1.0.0/node-dashboard-bee-1.0.0.zip";
    const RELEASE_CHECKSUM: &str = "d61bce7d70b51ea88536b22c5341ccdc3c4651bf8e02e093a771312f3f046c30";

    pub(super) fn fetch<P: AsRef<Path>>(dashboard_dir: P) -> Result<(), BuildError> {
        println!("downloading latest dashboard release from {}", RELEASE_URL);

        let client = reqwest::blocking::Client::builder()
            .user_agent("bee-fetch-dashboard")
            .build()
            .expect("could not create client");

        let mut archive = tempfile::tempfile().expect("could not create temp file");

        let response = client
            .get(RELEASE_URL)
            .send()
            .and_then(|resp| resp.error_for_status())
            .map_err(|e| BuildError::DashboardRequest(e.status().map(|code| code.as_u16()), RELEASE_URL.to_string()))?;

        let content = response.bytes().map_err(|_| BuildError::DashboardDownload)?;

        let mut sha256 = Sha256::new();
        sha256.update(&content);
        let checksum = format!("{:x}", sha256.finalize());

        if checksum != RELEASE_CHECKSUM {
            return Err(BuildError::DashboardInvalidChecksum);
        }

        println!("checksum ok");

        let mut content = Cursor::new(content);
        io::copy(&mut content, &mut archive).expect("copying failed");
        let mut archive = ZipArchive::new(BufReader::new(archive)).map_err(|_| BuildError::DashboardInvalidArchive)?;

        println!("extracting release archive to {}", dashboard_dir.as_ref().display());
        archive
            .extract(dashboard_dir)
            .map_err(|_| BuildError::DashboardInvalidArchive)?;

        Ok(())
    }
}
