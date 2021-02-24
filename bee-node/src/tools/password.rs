// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use argon2::{self, Config};
use rand::Rng;
use rpassword::read_password_from_tty;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Argon(#[from] argon2::Error),
    #[error("Re-entered password doesn't match")]
    NonMatching,
}

#[derive(Debug, StructOpt)]
pub struct PasswordTool {}

pub fn exec(_tool: &PasswordTool) -> Result<(), PasswordError> {
    let password = read_password_from_tty(Some("Password: "))?;
    let password_reenter = read_password_from_tty(Some("Re-enter password: "))?;
    if password != password_reenter {
        return Err(PasswordError::NonMatching);
    }
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let hash = argon2::hash_raw(password.as_bytes(), &salt, &Config::default())?;

    println!("Password salt: {}", hex::encode(salt));
    println!("Password hash: {}", hex::encode(hash));

    Ok(())
}
