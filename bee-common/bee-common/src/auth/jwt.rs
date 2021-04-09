// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides JSON Web Token utilities.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
pub use jsonwebtoken::{
    errors::{Error, ErrorKind},
    TokenData,
};
use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};

/// Represents registered JSON Web Token Claims.
/// https://tools.ietf.org/html/rfc7519#section-4.1
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer.
    /// Identifies the principal that issued the JWT. The processing of this claim is generally application specific.
    iss: String,
    /// Subject.
    /// Identifies the principal that is the subject of the JWT. The claims in a JWT are normally statements about the
    /// subject. The subject value MUST either be scoped to be locally unique in the context of the issuer or be
    /// globally unique. The processing of this claim is generally application specific.
    sub: String,
    /// Audience.
    /// Identifies the recipients that the JWT is intended for. Each principal intended to process the JWT MUST
    /// identify itself with a value in the audience claim. If the principal processing the claim does not identify
    /// itself with a value in the "aud" claim when this claim is present, then the JWT MUST be rejected. The
    /// interpretation of audience values is generally application specific.
    aud: String,
    /// Expiration Time.
    /// Identifies the expiration time on or after which the JWT MUST NOT be accepted for processing. The processing of
    /// the "exp" claim requires that the current date/time MUST be before the expiration date/time listed in the "exp"
    /// claim. Implementers MAY provide for some small leeway, usually no more than a few minutes, to account for clock
    /// skew.
    exp: u64,
    /// Not Before.
    /// Identifies the time before which the JWT MUST NOT be accepted for processing. The processing of the "nbf" claim
    /// requires that the current date/time MUST be after or equal to the not-before date/time listed in the "nbf"
    /// claim. Implementers MAY provide for some small leeway, usually no more than a few minutes, to account for clock
    /// skew.
    nbf: u64,
    /// Issued At.
    /// Identifies the time at which the JWT was issued. This claim can be used to determine the age of the JWT.
    iat: u64,
}

impl Claims {
    /// Creates a new set of claims.
    pub fn new(iss: String, sub: String, aud: String, exp: u64, nbf: u64) -> Self {
        Self {
            iss,
            sub,
            aud,
            exp,
            nbf,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_secs() as u64,
        }
    }
}

/// Represents a JSON Web Token.
/// https://tools.ietf.org/html/rfc7519
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
    /// Creates a new JSON Web Token.
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

    /// Validates a JSON Web Token.
    pub fn validate(
        &self,
        issuer: String,
        subject: String,
        audience: String,
        secret: &[u8],
    ) -> Result<TokenData<Claims>, Error> {
        let mut validation = Validation {
            iss: Some(issuer),
            sub: Some(subject),
            ..Default::default()
        };
        validation.set_audience(&[audience]);

        decode::<Claims>(&self.0, &DecodingKey::from_secret(secret), &validation)
    }
}
