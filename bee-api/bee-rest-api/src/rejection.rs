// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use warp::reject::Reject;

#[derive(Debug, Clone)]
pub(crate) enum CustomRejection {
    Forbidden,
    BadRequest(String),
    NotFound(String),
    ServiceUnavailable(String),
}

impl Reject for CustomRejection {}
