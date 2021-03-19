// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod white_flag;

use warp::{self, Filter, Rejection, Reply};

use std::net::IpAddr;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("debug"))
}

pub(crate) fn filter(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    white_flag::filter(public_routes, allowed_ips)
}
