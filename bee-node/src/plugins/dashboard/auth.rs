// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{config::DashboardAuthConfig, rejection::CustomRejection};

use bee_common::auth::{jwt::JsonWebToken, password};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use warp::{reject, Rejection, Reply};

pub(crate) const AUDIENCE_CLAIM: &str = "dashboard";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub jwt: String,
}

pub(crate) async fn auth(
    node_id: String,
    config: DashboardAuthConfig,
    body: JsonValue,
) -> Result<impl Reply, Rejection> {
    let jwt_json = &body["jwt"];

    if !jwt_json.is_null() {
        let jwt = JsonWebToken::from(
            jwt_json
                .as_str()
                .ok_or_else(|| reject::custom(CustomRejection::InvalidJwt))?
                .to_owned(),
        );
        return match jwt.validate(node_id, config.user().to_owned(), AUDIENCE_CLAIM.to_owned(), b"secret") {
            Ok(_) => Ok(warp::reply::json(&AuthResponse { jwt: jwt.to_string() })),
            Err(_) => Err(reject::custom(CustomRejection::InvalidJwt)),
        };
    }

    let user_json = &body["user"];

    let user = if user_json.is_null() {
        return Err(reject::custom(CustomRejection::BadRequest("No user provided")));
    } else {
        user_json
            .as_str()
            .ok_or_else(|| reject::custom(CustomRejection::BadRequest("Invalid user provided")))?
    };

    if user != config.user() {
        return Err(reject::custom(CustomRejection::InvalidCredentials));
    }

    let password_json = &body["password"];

    let password = if password_json.is_null() {
        return Err(reject::custom(CustomRejection::BadRequest("No password provided")));
    } else {
        password_json
            .as_str()
            .ok_or_else(|| reject::custom(CustomRejection::BadRequest("Invalid password provided")))?
    };

    if !password::password_verify(
        password.as_bytes(),
        &hex::decode(config.password_salt()).unwrap(),
        &hex::decode(config.password_hash()).unwrap(),
    )
    .map_err(|_| reject::custom(CustomRejection::InternalError))?
    {
        return Err(reject::custom(CustomRejection::InvalidCredentials));
    }

    let jwt = JsonWebToken::new(
        node_id,
        config.user().to_owned(),
        AUDIENCE_CLAIM.to_owned(),
        config.session_timeout(),
        b"secret",
    )
    .map_err(|_| reject::custom(CustomRejection::InternalError))?;

    Ok(warp::reply::json(&AuthResponse { jwt: jwt.to_string() }))
}
