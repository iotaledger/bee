// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{config::route_to_regex, rejection::CustomRejection, storage::StorageBackend, ApiArgs};

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
        RegexSet::new(routes.iter().map(|r| route_to_regex(&r)).collect::<Vec<String>>()).unwrap()
    };
}

pub(crate) fn check_permission<B: StorageBackend>(
    args: Arc<ApiArgs<B>>,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(warp::filters::path::full())
        .and(warp::header::headers_cloned())
        .and_then(move |path: FullPath, headers| {
            let args = args.clone();
            async move {
                if args.rest_api_config.public_routes.is_match(path.as_str()) {
                    return Ok(());
                }

                let jwt = extract_jwt(&headers)?;
                // Decode the JWT payload to find out how to validate it.
                let jwt_payload = {
                    let jwt_string = jwt.to_string();
                    let split = jwt_string.split(".").collect::<Vec<&str>>();
                    if split.len() < 3 {
                        return Err(reject::custom(CustomRejection::Forbidden));
                    }
                    let encoded = split[1];
                    let decoded = base64::decode(encoded).map_err(|_| reject::custom(CustomRejection::Forbidden))?;
                    String::from_utf8(decoded.clone()).map_err(|_| CustomRejection::Forbidden)?
                };

                if jwt_payload.contains(&format!("\"aud\":\"{}\"", API_AUDIENCE_CLAIM)) {
                    if let Ok(_) = validate_api_jwt(&jwt, &args) {
                        if args.rest_api_config.protected_routes.is_match(path.as_str()) {
                            return Ok(());
                        }
                    }
                } else if jwt_payload.contains(&format!("\"aud\":\"{}\"", DASHBOARD_AUDIENCE_CLAIM)) {
                    if let Ok(_) = validate_dashboard_jwt(&jwt, &args) {
                        if DASHBOARD_ROUTES.is_match(path.as_str()) {
                            return Ok(());
                        }
                    }
                }

                return Err(reject::custom(CustomRejection::Forbidden));
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
    args: &Arc<ApiArgs<B>>,
) -> Result<TokenData<Claims>, Rejection> {
    jwt.validate(
        args.node_id.to_string(),
        args.rest_api_config.jwt_salt.to_owned(),
        API_AUDIENCE_CLAIM.to_owned(),
        false,
        args.node_key_pair.secret().as_ref(),
    )
    .map_err(|_| reject::custom(CustomRejection::Forbidden))
}

fn validate_dashboard_jwt<B: StorageBackend>(
    jwt: &JsonWebToken,
    args: &Arc<ApiArgs<B>>,
) -> Result<TokenData<Claims>, Rejection> {
    jwt.validate(
        args.node_id.to_string(),
        args.rest_api_config.jwt_salt.to_owned(), // Dashboard SALT MISSING
        DASHBOARD_AUDIENCE_CLAIM.to_owned(),
        true,
        args.node_key_pair.secret().as_ref(),
    )
    .map_err(|_| reject::custom(CustomRejection::Forbidden))
}
