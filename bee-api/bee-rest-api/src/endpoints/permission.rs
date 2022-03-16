// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{config::route_to_regex, rejection::CustomRejection, storage::StorageBackend, ApiArgsFullNode};

use auth_helper::jwt::{Claims, JsonWebToken, TokenData};
use base64;

use lazy_static::lazy_static;
use regex::RegexSet;
use warp::{
    http::header::{HeaderMap, AUTHORIZATION},
    path::FullPath,
    reject, Filter, Rejection,
};

use std::sync::Arc;

/// Bearer for JWT. Please note the whitespace " " is important for correct parsing.
const BEARER: &str = "Bearer ";
pub const API_AUDIENCE_CLAIM: &str = "api";
pub const DASHBOARD_AUDIENCE_CLAIM: &str = "dashboard";

lazy_static! {
    static ref DASHBOARD_ROUTES: RegexSet = {
        let routes = vec![
            "/api/v1/info",
            "/api/v1/messages/*",
            "/api/v1/outputs/*",
            "/api/v1/addresses/*",
            "/api/v1/milestones/*",
            "/api/v1/peers*",
        ];
        RegexSet::new(routes.iter().map(|r| route_to_regex(r)).collect::<Vec<String>>()).unwrap()
    };
}

pub(crate) fn check_permission<B: StorageBackend>(
    args: Arc<ApiArgsFullNode<B>>,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(warp::filters::path::full())
        .and(warp::header::headers_cloned())
        .and_then(move |path: FullPath, headers| {
            let args = args.clone();
            async move {
                // Check if the requested endpoint is open for public use.
                if args.rest_api_config.public_routes.is_match(path.as_str()) {
                    return Ok(());
                }

                // Else the requested endpoint is protected by JWT / or not even exposed.
                let jwt = extract_jwt(&headers)?;
                // Decode the JWT payload to find out how to validate it. The `aud` claim will indicate if it's an API
                // JWT or a Dashboard JWT.
                let jwt_payload = {
                    let jwt_string = jwt.to_string();
                    // Every JWT consists of 3 parts: 1) header, 2) payload, 3) signature.
                    // The different parts are separated by `.`.
                    let split = jwt_string.split('.').collect::<Vec<&str>>();
                    // If there are less or more then 3 parts the given JWT is invalid.
                    if split.len() != 3 {
                        return Err(reject::custom(CustomRejection::Forbidden));
                    }

                    // Get the base64 encoded payload.
                    let encoded_payload = split[1];
                    // Base64-decode the payload to bytes.
                    let decoded_payload_bytes =
                        base64::decode(encoded_payload).map_err(|_| reject::custom(CustomRejection::Forbidden))?;
                    // Create a UTF-8 string from the decoded bytes.
                    String::from_utf8(decoded_payload_bytes).map_err(|_| CustomRejection::Forbidden)?
                };

                if jwt_payload.contains(&format!("\"aud\":\"{}\"", API_AUDIENCE_CLAIM)) {
                    if validate_api_jwt(&jwt, &args).is_ok()
                        && args.rest_api_config.protected_routes.is_match(path.as_str())
                    {
                        return Ok(());
                    }
                } else {
                    #[cfg(feature = "dashboard")]
                    if jwt_payload.contains(&format!("\"aud\":\"{}\"", DASHBOARD_AUDIENCE_CLAIM))
                        && validate_dashboard_jwt(&jwt, &args).is_ok()
                        && DASHBOARD_ROUTES.is_match(path.as_str())
                    {
                        return Ok(());
                    }
                }

                Err(reject::custom(CustomRejection::Forbidden))
            }
        })
        .untuple_one()
}

fn extract_jwt(headers: &HeaderMap) -> Result<JsonWebToken, Rejection> {
    let header = headers.get(AUTHORIZATION).ok_or(CustomRejection::Forbidden)?;
    let auth_header = std::str::from_utf8(header.as_bytes()).map_err(|_| CustomRejection::Forbidden)?;

    if !auth_header.starts_with(BEARER) {
        return Err(reject::custom(CustomRejection::Forbidden));
    }

    Ok(JsonWebToken::from(auth_header.trim_start_matches(BEARER).to_owned()))
}

fn validate_api_jwt<B: StorageBackend>(
    jwt: &JsonWebToken,
    args: &Arc<ApiArgsFullNode<B>>,
) -> Result<TokenData<Claims>, Rejection> {
    jwt.validate(
        args.node_id.to_string(),
        args.rest_api_config.jwt_salt.to_owned(),
        API_AUDIENCE_CLAIM.to_owned(),
        false,
        args.node_keypair.secret().as_ref(),
    )
    .map_err(|_| reject::custom(CustomRejection::Forbidden))
}

#[cfg(feature = "dashboard")]
fn validate_dashboard_jwt<B: StorageBackend>(
    jwt: &JsonWebToken,
    args: &Arc<ApiArgsFullNode<B>>,
) -> Result<TokenData<Claims>, Rejection> {
    jwt.validate(
        args.node_id.to_string(),
        args.dashboard_username.to_owned(),
        DASHBOARD_AUDIENCE_CLAIM.to_owned(),
        true,
        args.node_keypair.secret().as_ref(),
    )
    .map_err(|_| reject::custom(CustomRejection::Forbidden))
}
