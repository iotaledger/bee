// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Local, NodeConfig, NodeStorageBackend};

use bee_rest_api::endpoints::permission::DASHBOARD_AUDIENCE_CLAIM;

use auth_helper::jwt::JsonWebToken;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JwtApiError {
    #[error("{0}")]
    GeneratorError(#[from] auth_helper::jwt::Error),
}

#[derive(Clone, Debug, StructOpt)]
pub struct JwtApiTool {}

pub fn exec<B: NodeStorageBackend>(
    _tool: &JwtApiTool,
    local: &Local,
    node_config: &NodeConfig<B>,
) -> Result<(), JwtApiError> {
    let jwt = JsonWebToken::new(
        local.peer_id().to_string(),
        node_config.rest_api_config.jwt_salt().to_owned(),
        API_AUDIENCE_CLAIM.to_owned(),
        10000, // JWT tokens shouldn't expire.
        local.keypair().secret().as_ref(),
    )?;

    println!("Generated JWT: {}", jwt);

    Ok(())
}
