// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod hash_cache;
mod hasher;
mod processor;

pub(crate) use hash_cache::HashCache;
pub(crate) use hasher::{HasherWorker, HasherWorkerEvent};
pub(crate) use processor::{ProcessorWorker, ProcessorWorkerEvent};

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         config::ProtocolConfig,
//         message::Message as MessagePacket,
//         protocol::Protocol,
//         tangle::{self, tangle},
//     };
//
//     use bee_common::shutdown_stream::ShutdownStream;
//     use bee_common::{bee_node::BeeNode, event::Bus, node::Node, worker::Worker};
//     use bee_crypto::ternary::Hash;
//     use bee_network::{EndpointId, NetworkConfig};
//
//     use futures::{channel::oneshot, join};
//     use tokio::{spawn, time::delay_for};
//
//     use std::{sync::Arc, time::Duration};
//
//     #[tokio::test]
//     async fn test_message_workers_with_compressed_buffer() {
//         let bee_node = Arc::new(BeeNode::new());
//         let bus = Arc::new(Bus::default());
//
//         // build network
//         let network_config = NetworkConfig::builder().finish();
//         let (network, _) = bee_network::init(network_config);
//
//         // init tangle
//         tangle::init();
//
//         // init protocol
//         let protocol_config = ProtocolConfig::build().finish();
//         Protocol::init(protocol_config, network, 0, bee_node.clone(), bus).await;
//
//         assert_eq!(tangle().len(), 0);
//
//         let (hasher_worker_sender, hasher_worker_receiver) = flume::unbounded();
//         let (hasher_worker_shutdown_sender, hasher_worker_shutdown_receiver) = oneshot::channel();
//         let (processor_worker_sender, processor_worker_receiver) = flume::unbounded();
//         let (milestone_validator_worker_sender, _milestone_validator_worker_receiver) = flume::unbounded();
//
//         let hasher_handle = HasherWorker::<BeeNode>::new(processor_worker_sender).start(
//             <HasherWorker<BeeNode> as Worker<BeeNode>>::Receiver::new(
//                 10000,
//                 ShutdownStream::new(hasher_worker_shutdown_receiver, hasher_worker_receiver),
//             ),
//             bee_node.clone(),
//             (),
//         );
//
//         let processor = ProcessorWorker::new(milestone_validator_worker_sender);
//         let processor_handle = Worker::<BeeNode>::start(processor, processor_worker_receiver, bee_node.clone(), ());
//
//         spawn(async move {
//             let message: [u8; 1024] = [0; 1024];
//             let message = MessagePacket::new(&message);
//             let peer_id = EndpointId::new();
//             let event = HasherWorkerEvent {
//                 from: peer_id,
//                 message_packet: message,
//             };
//             hasher_worker_sender.unbounded_send(event).unwrap();
//             delay_for(Duration::from_secs(5)).await;
//             hasher_worker_shutdown_sender.send(()).unwrap();
//             // TODO shutdown node to fix the test
//         });
//
//         let (hasher_result, processor_result) = join!(hasher_handle, processor_handle);
//
//         hasher_result.unwrap();
//         processor_result.unwrap();
//
//         assert_eq!(tangle().len(), 1);
//         assert_eq!(tangle().contains(&Hash::zeros()), true);
//     }
// }
