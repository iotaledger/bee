// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::handlers::spawn_static_topic_handler, storage::StorageBackend};

use bee_common::packable::Packable;
use bee_protocol::workers::event::IndexationMessage;
use bee_runtime::node::Node;

use librumqttd::LinkTx;

pub(crate) fn spawn<N>(node: &mut N, messages_indexation_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    spawn_static_topic_handler(
        node,
        messages_indexation_tx,
        "messages/indexation/[index]",
        |event: IndexationMessage| {
            (
                format!("messages/indexation/{}", event.index.to_string()),
                event.message.pack_new(),
            )
        },
    );
}
