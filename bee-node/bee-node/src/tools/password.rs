// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::env;

use auth_helper::password::{self, Error as GeneratorError};
use rpassword::prompt_password;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Generator(#[from] GeneratorError),
    #[error("Re-entered password doesn't match")]
    NonMatching,
    #[error("Password verification failed")]
    VerificationFailed,
}

#[derive(Clone, Debug, StructOpt)]
pub struct PasswordTool {}

pub fn exec(_tool: &PasswordTool) -> Result<(), PasswordError> {
    let password = if let Ok(env_password) = env::var("BEE_TOOL_PASSWORD") {
        env_password
    } else {
        let password = prompt_password("Password: ")?;

        if password != prompt_password("Re-enter password: ")? {
            return Err(PasswordError::NonMatching);
        }

        password
    };

    let salt = password::generate_salt();
    let hash = password::password_hash(password.as_bytes(), &salt)?;

    if !password::password_verify(password.as_bytes(), &salt, &hash)? {
        return Err(PasswordError::VerificationFailed);
    }

    println!("Password salt: {}", hex::encode(salt));
    println!("Password hash: {}", hex::encode(hash));

    Ok(())
}
