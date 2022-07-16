// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod blocks;
pub mod blocks_metadata;
pub mod blocks_submit;
pub mod info;
pub mod milestones_by_id;
pub mod milestones_by_index;
pub mod outputs;
pub mod outputs_metadata;
pub mod peers;
pub mod peers_add;
pub mod peers_all;
pub mod peers_remove;
pub mod receipts;
pub mod receipts_at;
pub mod tips;
pub mod transactions_included_block;
pub mod treasury;
pub mod utxo_changes_by_id;
pub mod utxo_changes_by_index;
pub mod white_flag;

use axum::Router;

use crate::storage::StorageBackend;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().nest(
        "/v2",
        info::filter::<B>()
            .merge(blocks::filter::<B>())
            .merge(blocks_metadata::filter::<B>())
            .merge(blocks_submit::filter::<B>())
            .merge(milestones_by_id::filter::<B>())
            .merge(milestones_by_index::filter::<B>())
            .merge(outputs::filter::<B>())
            .merge(outputs_metadata::filter::<B>())
            .merge(peers::filter::<B>())
            .merge(peers_add::filter::<B>())
            .merge(peers_all::filter::<B>())
            .merge(peers_remove::filter::<B>())
            .merge(receipts::filter::<B>())
            .merge(receipts_at::filter::<B>())
            .merge(tips::filter::<B>())
            .merge(transactions_included_block::filter::<B>())
            .merge(treasury::filter::<B>())
            .merge(utxo_changes_by_id::filter::<B>())
            .merge(utxo_changes_by_index::filter::<B>())
            .merge(white_flag::filter::<B>()),
    )
}
