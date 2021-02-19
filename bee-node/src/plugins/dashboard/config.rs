// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_SESSION_TIMEOUT: u64 = 86400;
const DEFAULT_USER: &str = "";
const DEFAULT_PASSWORD_HASH: &str = "";
const DEFAULT_PASSWORD_SALT: &str = "";
const DEFAULT_PORT: u16 = 8081;

#[derive(Default, Deserialize)]
pub struct DashboardAuthConfigBuilder {
    session_timeout: Option<u64>,
    user: Option<String>,
    password_hash: Option<String>,
    password_salt: Option<String>,
}

impl DashboardAuthConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> DashboardAuthConfig {
        DashboardAuthConfig {
            session_timeout: self.session_timeout.unwrap_or(DEFAULT_SESSION_TIMEOUT),
            user: self.user.unwrap_or_else(|| DEFAULT_USER.to_owned()),
            password_hash: self.password_hash.unwrap_or_else(|| DEFAULT_PASSWORD_HASH.to_owned()),
            password_salt: self.password_salt.unwrap_or_else(|| DEFAULT_PASSWORD_SALT.to_owned()),
        }
    }
}

#[derive(Clone)]
pub struct DashboardAuthConfig {
    session_timeout: u64,
    user: String,
    password_hash: String,
    password_salt: String,
}

impl DashboardAuthConfig {
    pub fn build() -> DashboardAuthConfigBuilder {
        DashboardAuthConfigBuilder::new()
    }

    pub fn session_timeout(&self) -> u64 {
        self.session_timeout
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn password_salt(&self) -> &str {
        &self.password_salt
    }
}

#[derive(Default, Deserialize)]
pub struct DashboardConfigBuilder {
    port: Option<u16>,
    auth: Option<DashboardAuthConfigBuilder>,
}

impl DashboardConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> DashboardConfig {
        DashboardConfig {
            port: self.port.unwrap_or(DEFAULT_PORT),
            auth: self.auth.unwrap_or_default().finish(),
        }
    }
}

#[derive(Clone)]
pub struct DashboardConfig {
    port: u16,
    auth: DashboardAuthConfig,
}

impl DashboardConfig {
    pub fn build() -> DashboardConfigBuilder {
        DashboardConfigBuilder::new()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn auth(&self) -> &DashboardAuthConfig {
        &self.auth
    }
}
