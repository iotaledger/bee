// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use auth_helper::jwt::{Claims, JsonWebToken, TokenData};
use axum::{
    async_trait,
    extract::{Extension, FromRequest, OriginalUri, RequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::Uri,
};
use lazy_static::lazy_static;
use regex::RegexSet;
use serde::{Deserialize, Serialize};

use crate::endpoints::{config::route_to_regex, error::ApiError, storage::StorageBackend, ApiArgsFullNode};

pub const API_AUDIENCE_CLAIM: &str = "api";
pub const DASHBOARD_AUDIENCE_CLAIM: &str = "dashboard";
const API_JWT_HINT: &str = "\"aud\":\"api\"";
#[cfg(feature = "dashboard")]
const DASHBOARD_JWT_HINT: &str = "\"aud\":\"dashboard\"";

lazy_static! {
    static ref DASHBOARD_ROUTES: RegexSet = {
        let routes = vec![
            "/api/v2/info",
            "/api/v2/messages/*",
            "/api/v2/outputs/*",
            "/api/v2/addresses/*",
            "/api/v2/milestones/*",
            "/api/v2/peers*",
        ];
        // Panic: unwrapping is fine because all strings in `routes` can be turned into valid regular expressions.
        RegexSet::new(routes.iter().map(|r| route_to_regex(r)).collect::<Vec<_>>()).unwrap()
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth<S> {
    phantom: PhantomData<S>,
}

#[async_trait]
impl<B, S> FromRequest<B> for Auth<S>
where
    B: Send + Sync,
    S: StorageBackend,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let OriginalUri(uri) = OriginalUri::from_request(req).await.map_err(|_| ApiError::Forbidden)?;

        let Extension(args) = Extension::<ApiArgsFullNode<S>>::from_request(req)
            .await
            .map_err(|_| ApiError::Forbidden)?;

        // Check if the requested endpoint is open for public use.
        if args.rest_api_config.public_routes().is_match(&uri.to_string()) {
            return Ok(Auth { phantom: PhantomData });
        }

        // Extract the token from the authorization header
        let jwt = {
            let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| ApiError::Forbidden)?;
            JsonWebToken::from(bearer.token().to_string())
        };

        validate_jwt(uri, jwt, args).await?;

        Ok(Auth { phantom: PhantomData })
    }
}

async fn validate_jwt<B: StorageBackend>(
    uri: Uri,
    jwt: JsonWebToken,
    args: ApiArgsFullNode<B>,
) -> Result<(), ApiError> {
    // Decode the JWT payload to find out how to validate it. The `aud` claim will indicate if it's an API
    // JWT or a Dashboard JWT.
    let jwt_payload = {
        let jwt_string = jwt.to_string();
        // Every JWT consists of 3 parts: 1) header, 2) payload, 3) signature.
        // The different parts are separated by `.`.
        let split = jwt_string.split('.').collect::<Vec<_>>();
        // If there are less or more then 3 parts the given JWT is invalid.
        if split.len() != 3 {
            return Err(ApiError::Forbidden);
        }

        // Get the base64 encoded payload.
        let encoded_payload = split[1];
        // Base64-decode the payload to bytes.
        let decoded_payload_bytes = base64::decode(encoded_payload).map_err(|_| ApiError::Forbidden)?;
        // Create a UTF-8 string from the decoded bytes.
        String::from_utf8(decoded_payload_bytes).map_err(|_| ApiError::Forbidden)?
    };

    if jwt_payload.contains(API_JWT_HINT) {
        if validate_api_jwt(&jwt, &args).is_ok() && args.rest_api_config.protected_routes().is_match(&uri.to_string()) {
            return Ok(());
        }
    } else {
        #[cfg(feature = "dashboard")]
        if jwt_payload.contains(DASHBOARD_JWT_HINT)
            && validate_dashboard_jwt(&jwt, &args).is_ok()
            && DASHBOARD_ROUTES.is_match(&uri.to_string())
        {
            return Ok(());
        }
    }

    Err(ApiError::Forbidden)
}

fn validate_api_jwt<B: StorageBackend>(
    jwt: &JsonWebToken,
    args: &ApiArgsFullNode<B>,
) -> Result<TokenData<Claims>, ApiError> {
    jwt.validate(
        args.node_id.to_string(),
        args.rest_api_config.jwt_salt().to_owned(),
        API_AUDIENCE_CLAIM.to_owned(),
        false,
        args.node_keypair.secret().as_ref(),
    )
    .map_err(|_| ApiError::Forbidden)
}

#[cfg(feature = "dashboard")]
fn validate_dashboard_jwt<B: StorageBackend>(
    jwt: &JsonWebToken,
    args: &ApiArgsFullNode<B>,
) -> Result<TokenData<Claims>, ApiError> {
    jwt.validate(
        args.node_id.to_string(),
        args.dashboard_username.to_owned(),
        DASHBOARD_AUDIENCE_CLAIM.to_owned(),
        true,
        args.node_keypair.secret().as_ref(),
    )
    .map_err(|_| ApiError::Forbidden)
}
