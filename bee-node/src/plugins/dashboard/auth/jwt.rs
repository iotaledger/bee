// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::auth::claims::Claims;

use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct JsonWebToken(String);

impl From<String> for JsonWebToken {
    fn from(inner: String) -> Self {
        JsonWebToken(inner)
    }
}

impl std::fmt::Display for JsonWebToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl JsonWebToken {
    pub fn new(
        issuer: String,
        subject: String,
        audience: String,
        session_timeout: u64,
        secret: &[u8],
    ) -> Result<Self, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_secs() as u64;
        let claims = Claims::new(issuer, subject, audience, now + session_timeout, now);
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))?;

        Ok(Self(token))
    }

    pub fn validate(&self, issuer: String, subject: String, audience: String, secret: &[u8]) -> bool {
        let mut validation = Validation {
            leeway: 60,
            iss: Some(issuer),
            sub: Some(subject),
            ..Default::default()
        };
        validation.set_audience(&[audience]);

        decode::<Claims>(&self.0, &DecodingKey::from_secret(secret), &validation).is_ok()
    }
}
