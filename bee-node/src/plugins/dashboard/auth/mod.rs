// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod claims;
pub mod jwt;

use jwt::JsonWebToken;

use crate::plugins::dashboard::{config::DashboardAuthConfig, rejection::CustomRejection};

use argon2::{self, Config};
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
                .ok_or_else(|| reject::custom(CustomRejection::InvalidJWT))?
                .to_owned(),
        );
        if jwt.validate(node_id.clone(), config.user().to_owned(), AUDIENCE_CLAIM.to_owned()) {
            return Ok(warp::reply::json(&AuthResponse { jwt: jwt.to_string() }));
        } else {
            return Err(reject::custom(CustomRejection::InvalidJWT));
        }
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

    let hash = hex::encode(
        argon2::hash_raw(
            password.as_bytes(),
            // Unwrap is fine, salt comes from the config and has already been verified
            &hex::decode(config.password_salt()).unwrap(),
            &Config::default(),
        )
        .map_err(|_| reject::custom(CustomRejection::InternalError))?,
    );

    if hash != config.password_hash() {
        return Err(reject::custom(CustomRejection::InvalidCredentials));
    }

    let jwt = JsonWebToken::new(
        node_id,
        config.user().to_owned(),
        AUDIENCE_CLAIM.to_owned(),
        config.session_timeout(),
    )
    .map_err(|_| reject::custom(CustomRejection::InternalError))?;

    Ok(warp::reply::json(&AuthResponse { jwt: jwt.to_string() }))
}
