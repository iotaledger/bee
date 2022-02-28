// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone)]
pub(crate) enum CustomRejection {
    InvalidCredentials,
    InvalidJwt,
    InternalError,
    Forbidden,
    BadRequest(&'static str),
}

impl warp::reject::Reject for CustomRejection {}
