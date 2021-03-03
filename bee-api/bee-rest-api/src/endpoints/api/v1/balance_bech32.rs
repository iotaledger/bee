// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::api::v1::balance_ed25519::balance_ed25519, rejection::CustomRejection, storage::StorageBackend,
};

use bee_message::address::Address;
use bee_runtime::resource::ResourceHandle;

use warp::{reject, Rejection, Reply};

pub(crate) async fn balance_bech32<B: StorageBackend>(
    addr: Address,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => balance_ed25519(a, storage).await,
        _ => Err(reject::custom(CustomRejection::BadRequest(
            "address type not supported".to_string(),
        ))),
    }
}
