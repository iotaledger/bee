// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::auth::password::{self, Error as GeneratorError};

use rpassword::read_password_from_tty;
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
    let password = read_password_from_tty(Some("Password: "))?;
    let password_reenter = read_password_from_tty(Some("Re-enter password: "))?;
    if password != password_reenter {
        return Err(PasswordError::NonMatching);
    }
    let salt = password::generate_salt();
    let hash = password::password_hash(password.as_bytes(), &salt)?;

    if !password::password_verify(password.as_bytes(), &salt, &hash)? {
        return Err(PasswordError::VerificationFailed);
    }

    println!("Password salt: {}", hex::encode(salt));
    println!("Password hash: {}", hex::encode(hash));

    Ok(())
}
