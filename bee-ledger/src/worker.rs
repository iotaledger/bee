// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error, event::MilestoneConfirmed, merkle_hasher::MerkleHasher, metadata::WhiteFlagMetadata,
    storage::Backend, white_flag::visit_dfs,
};

use bee_common::{
    event::Bus,
    node::{Node, ResHandle},
    shutdown_stream::ShutdownStream,
    worker::Worker,
};
use bee_message::{payload::Payload, MessageId};
use bee_protocol::{config::ProtocolCoordinatorConfig, tangle::MsTangle, MilestoneIndex, StorageWorker, TangleWorker};
use bee_storage::access::BatchBuilder;

use async_trait::async_trait;
use blake2::Blake2b;
use futures::stream::StreamExt;
use log::{error, info};

use std::{any::TypeId, convert::Infallible, sync::Arc};

// TODO refactor errors

pub enum LedgerWorkerEvent {
    Confirm(MessageId),
    // GetBalance(Address, oneshot::Sender<u64>),
}

pub(crate) struct LedgerWorker {
    // pub(crate) tx: flume::Sender<LedgerWorkerEvent>,
}

async fn confirm<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &ResHandle<N::Backend>,
    message_id: MessageId,
    index: &mut MilestoneIndex,
    _coo_config: &ProtocolCoordinatorConfig,
    bus: &Arc<Bus<'static>>,
) -> Result<(), Error>
where
    N::Backend: Backend,
{
    let message = tangle.get(&message_id).await.ok_or(Error::MilestoneMessageNotFound)?;

    let milestone = match message.payload() {
        Some(Payload::Milestone(milestone)) => milestone,
        _ => return Err(Error::NoMilestonePayload),
    };

    if milestone.essence().index() != index.0 + 1 {
        error!(
            "Tried to confirm {} on top of {}.",
            milestone.essence().index(),
            index.0
        );
        return Err(Error::NonContiguousMilestone);
    }

    let mut metadata = WhiteFlagMetadata::new(
        MilestoneIndex(milestone.essence().index()),
        // TODO useful ?
        milestone.essence().timestamp(),
    );

    let batch = N::Backend::batch_begin();

    if let Err(e) = visit_dfs::<N>(tangle, storage, message_id, &mut metadata).await {
        error!(
            "Error occured while traversing to confirm {}: {:?}.",
            milestone.essence().index(),
            e
        );
        return Err(e);
    };

    let merkle_proof = MerkleHasher::<Blake2b>::new().digest(&metadata.messages_included);

    if !merkle_proof.eq(&milestone.essence().merkle_proof()) {
        error!(
            "The computed merkle proof on milestone {}, {}, does not match the one provided by the coordinator, {}.",
            milestone.essence().index(),
            hex::encode(merkle_proof),
            hex::encode(milestone.essence().merkle_proof())
        );
        return Err(Error::MerkleProofMismatch);
    }

    if metadata.num_messages_referenced
        != metadata.num_messages_excluded_no_transaction
            + metadata.num_messages_excluded_conflicting
            + metadata.messages_included.len()
    {
        error!(
            "Invalid messages count at {}: referenced ({}) != no transaction ({}) + conflicting ({}) + included ({}).",
            milestone.essence().index(),
            metadata.num_messages_referenced,
            metadata.num_messages_excluded_no_transaction,
            metadata.num_messages_excluded_conflicting,
            metadata.messages_included.len()
        );
        return Err(Error::InvalidMessagesCount);
    }

    // TODO unwrap
    storage.batch_commit(batch, true).await.unwrap();

    // TODO update meta only when sure everything is fine

    *index = MilestoneIndex(milestone.essence().index());

    info!(
        "Confirmed milestone {}: referenced {}, zero value {}, conflicting {}, included {}.",
        milestone.essence().index(),
        metadata.num_messages_referenced,
        metadata.num_messages_excluded_no_transaction,
        metadata.num_messages_excluded_conflicting,
        metadata.messages_included.len()
    );

    bus.dispatch(MilestoneConfirmed {
        index: milestone.essence().index().into(),
        timestamp: milestone.essence().timestamp(),
        messages_referenced: metadata.num_messages_referenced,
        messages_excluded_no_transaction: metadata.num_messages_excluded_no_transaction,
        messages_excluded_conflicting: metadata.num_messages_excluded_conflicting,
        messages_included: metadata.messages_included.len(),
    });

    Ok(())
}

// fn get_balance(state: &LedgerState, address: Address, sender: oneshot::Sender<u64>) {
//     if let Err(e) = sender.send(state.get_or_zero(&address)) {
//         warn!("Failed to send balance: {:?}.", e);
//     }
// }

#[async_trait]
impl<N: Node> Worker<N> for LedgerWorker
where
    N::Backend: Backend,
{
    type Config = (MilestoneIndex, ProtocolCoordinatorConfig, Arc<Bus<'static>>);
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<StorageWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (_tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut index = config.0;
            let coo_config = config.1;
            let bus = config.2;

            while let Some(event) = receiver.next().await {
                match event {
                    LedgerWorkerEvent::Confirm(message_id) => {
                        if confirm::<N>(&tangle, &storage, message_id, &mut index, &coo_config, &bus)
                            .await
                            .is_err()
                        {
                            panic!(
                                "Error while confirming milestone from message {}, aborting.",
                                message_id
                            );
                        }
                    } // LedgerWorkerEvent::GetBalance(address, sender) => get_balance(&state, address, sender),
                }
            }

            info!("Stopped.");
        });

        Ok(Self {})
    }
}

// #[cfg(test)]
// mod tests {
//
//     use super::*;
//
//     use bee_test::field::rand_trits_field;
//
//     use async_std::task::{block_on, spawn};
//     use futures::sink::SinkExt;
//     use rand::Rng;
//
//     #[test]
//     fn get_balances() {
//         let mut rng = rand::thread_rng();
//         let mut state = HashMap::new();
//         let (mut tx, rx) = flume::unbounded();
//         let (_shutdown_tx, shutdown_rx) = oneshot::channel();
//
//         for _ in 0..100 {
//             state.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
//         }
//
//         spawn(LedgerStateWorker::new(state.clone(), ShutdownStream::new(shutdown_rx, rx)).run());
//
//         for (address, balance) in state {
//             let (get_balance_tx, get_balance_rx) = oneshot::channel();
//             block_on(tx.send(LedgerStateWorkerEvent::GetBalance(address, get_balance_tx))).unwrap();
//             let value = block_on(get_balance_rx).unwrap().unwrap();
//             assert_eq!(balance, value)
//         }
//     }
//
//     #[test]
//     fn get_balances_not_found() {
//         let mut rng = rand::thread_rng();
//         let mut state = HashMap::new();
//         let (mut tx, rx) = flume::unbounded();
//         let (_shutdown_tx, shutdown_rx) = oneshot::channel();
//
//         for _ in 0..100 {
//             state.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
//         }
//
//         spawn(LedgerStateWorker::new(state.clone(), ShutdownStream::new(shutdown_rx, rx)).run());
//
//         for _ in 0..100 {
//             let (get_balance_tx, get_balance_rx) = oneshot::channel();
//             block_on(tx.send(LedgerStateWorkerEvent::GetBalance(
//                 rand_trits_field::<Address>(),
//                 get_balance_tx,
//             )))
//             .unwrap();
//             let value = block_on(get_balance_rx).unwrap();
//             assert!(value.is_none());
//         }
//     }
//
//     #[test]
//     fn apply_diff_on_not_found() {
//         let mut rng = rand::thread_rng();
//         let mut diff = HashMap::new();
//         let (mut tx, rx) = flume::unbounded();
//         let (_shutdown_tx, shutdown_rx) = oneshot::channel();
//
//         for _ in 0..100 {
//             diff.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
//         }
//
//         block_on(tx.send(LedgerStateWorkerEvent::ApplyDiff(diff.clone()))).unwrap();
//
//         spawn(LedgerStateWorker::new(HashMap::new(), ShutdownStream::new(shutdown_rx, rx)).run());
//
//         for (address, balance) in diff {
//             let (get_balance_tx, get_balance_rx) = oneshot::channel();
//             block_on(tx.send(LedgerStateWorkerEvent::GetBalance(address, get_balance_tx))).unwrap();
//             let value = block_on(get_balance_rx).unwrap().unwrap();
//             assert_eq!(balance as u64, value)
//         }
//     }
//
//     // TODO test LedgerStateWorkerEvent::ApplyDiff
// }
