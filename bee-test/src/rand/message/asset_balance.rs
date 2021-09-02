// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes_array, number::rand_number};

use bee_message::output::{AssetBalance, AssetId};

/// Generates a random [`AssetId`].
pub fn rand_asset_id() -> AssetId {
    AssetId::new(rand_bytes_array())
}

/// Generates a random [`AssetBalance`].
pub fn rand_asset_balance() -> AssetBalance {
    AssetBalance::new(rand_asset_id(), rand_number())
}
