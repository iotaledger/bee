// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{rejection::CustomRejection, storage::StorageBackend, ApiArgs};

use auth_helper::jwt::{Claims, JsonWebToken, TokenData};
use warp::{
    http::header::{HeaderMap, AUTHORIZATION},
    path::FullPath,
    reject, Filter, Rejection,
};

use std::sync::Arc;

const BEARER: &str = "Bearer ";
pub const API_AUDIENCE_CLAIM: &str = "api";
pub const DASHBOARD_AUDIENCE_CLAIM: &str = "dashboard";

pub(crate) fn check_permission<B: StorageBackend>(
    args: Arc<ApiArgs<B>>,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(warp::filters::path::full())
        .and(warp::header::headers_cloned())
        .and_then(move |path: FullPath, headers| {
            let args = args.clone();
            async move {

                for route in args.rest_api_config.public_routes.iter() {
                    if route.is_match(path.as_str()) {
                        return Ok(());
                    }
                }

                if let Ok(jwt) = validate_jwt(&headers, &args) {
                    let aud_claim = jwt.claims.audience();

                    if aud_claim == API_AUDIENCE_CLAIM {
                        for route in args.rest_api_config.protected_routes.iter() {
                            if route.is_match(path.as_str()) && aud_claim == API_AUDIENCE_CLAIM {
                                return Ok(());
                            }
                        }
                    } else if aud_claim == DASHBOARD_AUDIENCE_CLAIM {
                        // check if the requested route is allowed for the dashboard
                        return Ok(());
                    }
                }

                return Err(reject::custom(CustomRejection::Forbidden));
            }
        })
        .untuple_one()
}

fn extract_jwt(headers: &HeaderMap) -> Result<String, Rejection> {
    let header = headers.get(AUTHORIZATION).ok_or(CustomRejection::Forbidden)?;
    let auth_header = std::str::from_utf8(header.as_bytes()).map_err(|_| CustomRejection::Forbidden)?;

    if !auth_header.starts_with(BEARER) {
        return Err(reject::custom(CustomRejection::Forbidden));
    }

    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}

fn validate_jwt<B: StorageBackend>(headers: &HeaderMap, args: &Arc<ApiArgs<B>>) -> Result<TokenData<Claims>, Rejection> {
    let jwt = JsonWebToken::from(extract_jwt(headers)?);
    jwt.validate(
        args.node_id.to_string(),
        args.rest_api_config.jwt_salt.to_owned(),
        API_AUDIENCE_CLAIM.to_owned(),
        true,
        args.node_key_pair.secret().as_ref(),
    ).map_err(|_| reject::custom(CustomRejection::Forbidden))
}