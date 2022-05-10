// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod info;
pub mod messages;
pub mod messages_children;
pub mod messages_metadata;
pub mod messages_submit;
pub mod milestones_by_milestone_id;
pub mod milestones_by_milestone_index;
pub mod outputs;
pub mod peers;
pub mod peers_add;
pub mod peers_all;
pub mod peers_remove;
pub mod receipts;
pub mod receipts_at;
pub mod tips;
pub mod transactions_included_message;
pub mod treasury;
pub mod utxo_changes_by_milestone_id;
pub mod utxo_changes_by_milestone_index;

use axum::Router;

use crate::endpoints::storage::StorageBackend;

pub(crate) const MAX_RESPONSE_RESULTS: usize = 1000;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().nest(
        "/v2",
        info::filter::<B>()
            .merge(messages::filter::<B>())
            .merge(messages_children::filter::<B>())
            .merge(messages_metadata::filter::<B>())
            .merge(messages_submit::filter::<B>())
            .merge(milestones_by_milestone_id::filter::<B>())
            .merge(milestones_by_milestone_index::filter::<B>())
            .merge(outputs::filter::<B>())
            .merge(peers::filter::<B>())
            .merge(peers_add::filter::<B>())
            .merge(peers_all::filter::<B>())
            .merge(peers_remove::filter::<B>())
            .merge(receipts::filter::<B>())
            .merge(receipts_at::filter::<B>())
            .merge(tips::filter::<B>())
            .merge(transactions_included_message::filter::<B>())
            .merge(treasury::filter::<B>())
            .merge(utxo_changes_by_milestone_id::filter::<B>())
            .merge(utxo_changes_by_milestone_index::filter::<B>()),
    )
}
