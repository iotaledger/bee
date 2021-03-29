// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod mock;

use bee_message::{payload::transaction::TransactionId, MessageId};
use bee_vote::{
    opinion,
    statement::{conflict::Conflict, opinion::Opinion, registry::Registry, timestamp::Timestamp},
};

#[tokio::test]
async fn registry() {
    let registry = Registry::default();

    let node_id = mock::random_id_string();

    println!("Here");
    let tx_id = TransactionId::new(mock::random_id_bytes());
    let msg_id = MessageId::new(mock::random_id_bytes());

    registry
        .write_view(&node_id, |view| {
            view.add_conflict(Conflict {
                id: tx_id,
                opinion: Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 1,
                },
            });
            view.add_conflict(Conflict {
                id: tx_id,
                opinion: Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 2,
                },
            });

            view.add_timestamp(Timestamp {
                id: msg_id,
                opinion: Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 1,
                },
            });
        })
        .await;

    registry
        .read_view(&node_id, |view| {
            let conflict_opinions = view.get_conflict_opinions(tx_id).unwrap();
            let timestamp_opinions = view.get_timestamp_opinions(msg_id).unwrap();

            assert_eq!(conflict_opinions.len(), 2);
            assert_eq!(
                *conflict_opinions.last().unwrap(),
                Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 2
                }
            );
            assert_eq!(conflict_opinions.finalized(2), true);

            assert_eq!(timestamp_opinions.len(), 1);
            assert_eq!(
                *timestamp_opinions.last().unwrap(),
                Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 1
                }
            );
            assert_eq!(timestamp_opinions.finalized(2), false);
        })
        .await
        .unwrap();
}
