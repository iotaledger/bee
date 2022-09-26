// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, process::Command};

#[derive(Debug)]
enum BuildError {
    GitBranch,
    GitCommit,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::GitBranch => write!(f, "failed to retrieve git branch name"),
            Self::GitCommit => write!(f, "failed to retrieve git commit"),
        }
    }
}

fn main() -> Result<(), BuildError> {
    match Command::new("git").args(["rev-parse", "HEAD"]).output() {
        Ok(output) => {
            println!(
                "cargo:rustc-env=GIT_COMMIT={}",
                String::from_utf8(output.stdout).unwrap()
            );
        }
        Err(_) => return Err(BuildError::GitCommit),
    }

    match Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output() {
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
