// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::BadRequest, handlers::api::v1::balance_ed25519::balance_ed25519, storage::StorageBackend,
};

use bee_message::prelude::*;
use bee_runtime::resource::ResourceHandle;

use warp::{reject, Rejection, Reply};

pub(crate) async fn balance_bech32<B: StorageBackend>(
    addr: Address,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => balance_ed25519(a, storage).await,
        _ => Err(reject::custom(BadRequest("address type not supported".to_string()))),
    }
}
