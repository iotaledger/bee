// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone)]
pub(crate) enum CustomRejection {
    InvalidCredentials,
    InvalidJWT,
    InternalError,
    NoAuthHeader,
    InvalidAuthHeader,
    Forbidden,
    BadRequest(&'static str),
}

impl warp::reject::Reject for CustomRejection {}
