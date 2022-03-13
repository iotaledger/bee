// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod add_peer;
pub mod info;
pub mod message;
pub mod message_children;
pub mod message_metadata;
pub mod message_raw;
pub mod milestone;
pub mod milestone_utxo_changes;
pub mod output;
pub mod peer;
pub mod peers;
pub mod receipts;
pub mod receipts_at;
pub mod remove_peer;
pub mod submit_message;
pub mod tips;
pub mod transaction_included_message;
pub mod treasury;

use crate::endpoints::{storage::StorageBackend};

use axum::Router;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .nest("v2",
              add_peer::filter()
                  .merge(info::filter())
                  .merge(message::filter())
                  .merge(message_children::filter())
                  .merge(message_metadata::filter())
                  .merge(message_raw::filter())
                  .merge(milestone::filter())
                  .merge(milestone_utxo_changes::filter())
                  .merge(output::filter())
                  .merge(peer::filter( ))
                  .merge(peers::filter())
                  .merge(receipts::filter())
                  .merge(receipts_at::filter())
                  .merge(remove_peer::filter())
                  .merge(submit_message::filter())
                  .merge(tips::filter())
                  .merge(treasury::filter())
                  .merge(transaction_included_message::filter())
        )
}
