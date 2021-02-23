// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// https://tools.ietf.org/html/rfc7519#section-4.1

use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Identifies the principal that issued the JWT. The processing of this claim is generally application specific.
    iss: String,
    /// Identifies the principal that is the subject of the JWT. The claims in a JWT are normally statements about the
    /// subject. The subject value MUST either be scoped to be locally unique in the context of the issuer or be
    /// globally unique. The processing of this claim is generally application specific.
    sub: String,
    /// Identifies the recipients that the JWT is intended for. Each principal intended to process the JWT MUST identify
    /// itself with a value in the audience claim. If the principal processing the claim does not identify itself with a
    /// value in the "aud" claim when this claim is present, then the JWT MUST be rejected. The interpretation of
    /// audience values is generally application specific.
    aud: String,
    /// Identifies the expiration time on or after which the JWT MUST NOT be accepted for processing. The processing of
    /// the "exp" claim requires that the current date/time MUST be before the expiration date/time listed in the "exp"
    /// claim. Implementers MAY provide for some small leeway, usually no more than a few minutes, to account for clock
    /// skew.
    exp: u64,
    /// Identifies the time before which the JWT MUST NOT be accepted for processing. The processing of the "nbf" claim
    /// requires that the current date/time MUST be after or equal to the not-before date/time listed in the "nbf"
    /// claim. Implementers MAY provide for some small leeway, usually no more than a few minutes, to account for clock
    /// skew.
    nbf: u64,
    /// Identifies the time at which the JWT was issued. This claim can be used to determine the age of the JWT.
    iat: u64,
}

impl Claims {
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
