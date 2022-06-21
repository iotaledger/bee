// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use auth_helper::jwt::{ClaimsBuilder, JsonWebToken};
use bee_rest_api::endpoints::auth::API_AUDIENCE_CLAIM;
use structopt::StructOpt;
use thiserror::Error;

use crate::{Local, NodeConfig, NodeStorageBackend};

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
    let claims = ClaimsBuilder::new(
        local.peer_id().to_string(),
        node_config.rest_api.jwt_salt().to_owned(),
        API_AUDIENCE_CLAIM.to_owned(),
    )
    .build()?;
    let jwt = JsonWebToken::new(claims, local.keypair().secret().as_ref())?;

    println!("{jwt}");

    Ok(())
}
