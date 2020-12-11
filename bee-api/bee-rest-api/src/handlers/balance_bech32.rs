// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::BadRequest, handlers::balance_ed25519::balance_ed25519, storage::Backend,
};
use bee_common::node::ResHandle;
use bee_message::prelude::*;
use warp::{reject, Rejection, Reply};

pub(crate) async fn balance_bech32<B: Backend>(
    addr: Address,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => balance_ed25519(a, storage).await,
        _ => Err(reject::custom(BadRequest("address type not supported".to_string()))),
    }
}
