// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, io, path::Path};

use sha2::{Digest, Sha256};
use zip::ZipArchive;

const RELEASE_URL: &str =
    "https://github.com/iotaledger/node-dashboard/releases/download/v2.0.0-alpha7/node-dashboard-bee-2.0.0-alpha7.zip";
const RELEASE_CHECKSUM: &str = "72e5ccf934ada48b04dca4e1f28bfd811e9cca526f3e30edb5ed77d2cdac984e";

#[derive(Debug)]
enum BuildError {
    InvalidArchive,
    InvalidChecksum,
    Request(Option<u16>, String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            BuildError::InvalidArchive => write!(f, "failed to open or extract release archive"),
            BuildError::InvalidChecksum => write!(
                f,
                "checksum of downloaded archive did not match that specified in the release"
            ),
            BuildError::Request(Some(code), url) => write!(f, "failed request to `{}`: status code {}", url, code),
            BuildError::Request(_, url) => write!(f, "failed request to `{}`", url),
        }
    }
}

fn main() -> Result<(), BuildError> {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DASHBOARD");

    let dashboard_dir = std::env::var("OUT_DIR").unwrap() + "/dashboard";
    println!("cargo:rustc-env=DASHBOARD_DIR={}", dashboard_dir);
    let dashboard_dir = std::path::Path::new(&dashboard_dir);

    // Rebuild if FETCH_DASHBOARD environment variable has changed.
    println!("cargo:rerun-if-env-changed=FETCH_DASHBOARD");
    // Rebuild if DASHBOARD_DIR has changed to a different path.
    println!("cargo:rerun-if-env-changed=DASHBOARD_DIR");

    let should_fetch = std::env::var("FETCH_DASHBOARD").map(|val| val == "1").unwrap_or(false);

    if should_fetch || !dashboard_dir.exists() {
        if dashboard_dir.exists() {
            // If the path already exists, we are re-downloading: remove the old files.
            std::fs::remove_dir_all(dashboard_dir).expect("could not remove existing dashboard");
        }

        fetch(dashboard_dir)?;
    }

    Ok(())
}

fn fetch<P: AsRef<Path>>(dashboard_dir: P) -> Result<(), BuildError> {
    println!("downloading latest dashboard release from {}", RELEASE_URL);

    let client = reqwest::blocking::Client::builder()
        .user_agent("bee-fetch-dashboard")
        .build()
        .expect("could not create client");

    let mut tmp_file = tempfile::NamedTempFile::new().expect("could not create temp file");

    client
        .get(RELEASE_URL)
        .send()
        .and_then(|resp| resp.error_for_status())
        .map_err(|e| BuildError::Request(e.status().map(|code| code.as_u16()), RELEASE_URL.to_string()))?
        .copy_to(&mut tmp_file)
        .expect("copying failed");

    let mut hasher = Sha256::new();
    io::copy(&mut tmp_file.reopen().expect("could not open temp file"), &mut hasher).expect("io error");
    let checksum = format!("{:x}", hasher.finalize());

    println!("checksum: {}\noriginal: {}", checksum, RELEASE_CHECKSUM);

    if checksum != RELEASE_CHECKSUM {
        return Err(BuildError::InvalidChecksum);
    }

    println!("checksum ok");

    let mut archive = ZipArchive::new(tmp_file).map_err(|_| BuildError::InvalidArchive)?;

    println!("extracting release archive to {}", dashboard_dir.as_ref().display());
    archive.extract(dashboard_dir).map_err(|_| BuildError::InvalidArchive)?;

    Ok(())
}
