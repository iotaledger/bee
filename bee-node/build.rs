// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[derive(Debug)]
enum BuildError {
    GitCommandFailed(std::io::Error),
    GitCommandInvalidOutput(std::string::FromUtf8Error),
}

fn main() -> Result<(), BuildError> {
    match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(output) => println!("cargo:rustc-env=GIT_COMMIT={}", output),
            Err(e) => return Err(BuildError::GitCommandInvalidOutput(e)),
        },
        Err(e) => return Err(BuildError::GitCommandFailed(e)),
    }

    match Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(output) => {
                println!("cargo:rerun-if-changed=../.git/HEAD");
                println!("cargo:rerun-if-changed=../.git/refs/heads/{}", output);
            }
            Err(e) => return Err(BuildError::GitCommandInvalidOutput(e)),
        },
        Err(e) => return Err(BuildError::GitCommandFailed(e)),
    }

    Ok(())
}
