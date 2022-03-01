// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use zip::ZipArchive;

use std::{
    env, fmt,
    io::{self, BufReader, Cursor},
    path::Path,
    process::Command,
};

const ASSET_PREFIX: &str = "node-dashboard-bee-";
const RELEASES_URL: &str = "https://api.github.com/repos/iotaledger/node-dashboard/releases/latest";

#[derive(Debug)]
enum BuildError {
    DashboardDecode,
    DashboardDownload,
    DashboardInvalidArchive,
    DashboardNoSuitableRelease,
    DashboardRequest(Option<u16>, String),
    GitBranch,
    GitCommit,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::DashboardDecode => write!(f, "failed to decode response from JSON"),
            Self::DashboardDownload => write!(f, "failed to download latest release archive"),
            Self::DashboardInvalidArchive => write!(f, "failed to open or extract release archive"),
            Self::DashboardNoSuitableRelease => write!(f, "no release assets found for latest version"),
            Self::DashboardRequest(Some(code), url) => write!(f, "failed request to `{}`: status code {}", url, code),
            Self::DashboardRequest(_, url) => write!(f, "failed request to `{}`", url),
            Self::GitBranch => write!(f, "failed to retrieve git branch name"),
            Self::GitCommit => write!(f, "failed to retrieve git commit"),
        }
    }
}

#[derive(Deserialize, Debug)]
struct LatestReleaseAssets {
    assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[tokio::main]
async fn main() -> Result<(), BuildError> {
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

    let should_fetch = env::var("FETCH_DASHBOARD").map(|val| val == "1").unwrap_or(false);
    let dashboard_dir = env::var("OUT_DIR").unwrap() + "/dashboard";

    println!("cargo:rustc-env=DASHBOARD_DIR={}", dashboard_dir);

    if should_fetch || !Path::new(&dashboard_dir).exists() {
        fetch_dashboard(dashboard_dir).await?;
    }

    Ok(())
}

async fn fetch_dashboard<P: AsRef<Path>>(dashboard_dir: P) -> Result<(), BuildError> {
    println!("fetching latest dashboard release");

    let client = reqwest::Client::builder()
        .user_agent("bee-fetch-dashboard")
        .build()
        .expect("could not create client");

    let response = client
        .get(RELEASES_URL)
        .send()
        .await
        .and_then(|resp| resp.error_for_status())
        .map_err(|e| BuildError::DashboardRequest(e.status().map(|code| code.as_u16()), RELEASES_URL.to_string()))?;

    let assets = response
        .json::<LatestReleaseAssets>()
        .await
        .map_err(|_| BuildError::DashboardDecode)?
        .assets;

    let release_asset = assets
        .iter()
        .find(|asset| asset.name.starts_with(ASSET_PREFIX))
        .ok_or(BuildError::DashboardNoSuitableRelease)?;

    println!(
        "downloading latest dashboard release `{}` from {}",
        release_asset.name, release_asset.browser_download_url
    );

    let mut archive = tempfile::tempfile().expect("could not create temp file");

    let response = client
        .get(&release_asset.browser_download_url)
        .send()
        .await
        .and_then(|resp| resp.error_for_status())
        .map_err(|e| {
            BuildError::DashboardRequest(
                e.status().map(|code| code.as_u16()),
                release_asset.browser_download_url.clone(),
            )
        })?;

    let mut content = Cursor::new(response.bytes().await.map_err(|_| BuildError::DashboardDownload)?);
    io::copy(&mut content, &mut archive).expect("copying failed");
    let mut archive = ZipArchive::new(BufReader::new(archive)).map_err(|_| BuildError::DashboardInvalidArchive)?;

    println!("extracting release archive to {}", dashboard_dir.as_ref().display());
    archive.extract(dashboard_dir).map_err(|_| BuildError::DashboardInvalidArchive)?;

    Ok(())
}