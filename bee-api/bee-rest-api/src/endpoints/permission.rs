// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::rejection::CustomRejection;

use warp::{reject, Filter, Rejection};

use std::net::{IpAddr, SocketAddr};

pub fn has_permission(
    route: &'static str,
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::addr::remote()
        .and_then(move |addr: Option<SocketAddr>| {
            let route = route.to_owned();
            let public_routes = public_routes.clone();
            let allowed_ips = allowed_ips.clone();
            async move {
                if let Some(v) = addr {
                    if allowed_ips.contains(&v.ip()) || public_routes.contains(&route) {
                        return Ok(());
                    }
                }
                Err(reject::custom(CustomRejection::Forbidden))
            }
        })
        .untuple_one()
}
