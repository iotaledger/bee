// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    packet::{IncomingPacket, MessageType, OutgoingPacket, Packet, DISCOVERY_MSG_TYPE_RANGE, PEERING_MSG_TYPE_RANGE},
    LocalId,
};

use tokio::{net::UdpSocket, sync::mpsc};

use std::{net::SocketAddr, sync::Arc};

const READ_BUFFER_SIZE: usize = crate::packet::MAX_PACKET_SIZE;

type PacketTx = mpsc::UnboundedSender<IncomingPacket>;
type PacketRx = mpsc::UnboundedReceiver<OutgoingPacket>;

pub(crate) struct ServerConfig {
    pub bind_addr: SocketAddr,
}

impl ServerConfig {
    pub(crate) fn new(config: &AutopeeringConfig) -> Self {
        Self {
            bind_addr: config.bind_addr,
        }
    }
}

pub(crate) struct PacketTxs {
    pub(crate) discovery_tx: PacketTx,
    pub(crate) peering_tx: PacketTx,
}

pub(crate) struct Server {
    config: ServerConfig,
    local_id: LocalId,
    incoming_txs: PacketTxs,
    outgoing_rx: PacketRx,
}

impl Server {
    pub fn new(config: ServerConfig, local_id: LocalId, incoming_txs: PacketTxs, outgoing_rx: PacketRx) -> Self {
        Self {
            config,
            local_id,
            incoming_txs,
            outgoing_rx,
        }
    }

    pub async fn run(self) {
        let Server {
            config,
            local_id,
            incoming_txs,
            outgoing_rx,
        } = self;

        // Try to bind the UDP socket to the configured address.
        let socket = UdpSocket::bind(&config.bind_addr)
            .await
            .expect("error binding udp socket");

        // The Tokio docs explain that there's no split method, and that we have to arc the UdpSocket in order to share
        // it.
        let incoming_socket = Arc::new(socket);
        let outgoing_socket = Arc::clone(&incoming_socket);

        // Spawn socket handlers
        tokio::spawn(incoming_packet_handler(incoming_socket, incoming_txs));
        tokio::spawn(outgoing_packet_handler(outgoing_socket, outgoing_rx, local_id));
    }
}

async fn incoming_packet_handler(socket: Arc<UdpSocket>, incoming_txs: PacketTxs) {
    let mut packet_bytes = [0; READ_BUFFER_SIZE];

    let PacketTxs {
        discovery_tx,
        peering_tx,
    } = incoming_txs;

    loop {
        if let Ok((n, source_addr)) = socket.recv_from(&mut packet_bytes).await {
            log::debug!("Received {} bytes from {}.", n, source_addr);

            let packet = Packet::from_protobuf(&packet_bytes[..n]).expect("error decoding incoming packet");

            // Verify the packet.
            let message = packet.message();
            let signature = packet.signature();
            if !packet.public_key().verify(&signature, message) {
                log::debug!("Received packet with invalid signature");
                continue;
            }

            // Depending on the message type, forward it to the appropriate manager.
            let msg_type = packet.message_type().expect("invalid message type");
            let msg_bytes = packet.into_message();

            let packet = IncomingPacket {
                msg_type,
                msg_bytes,
                source_addr,
            };

            match msg_type as u32 {
                t if DISCOVERY_MSG_TYPE_RANGE.contains(&t) => {
                    discovery_tx.send(packet).expect("channel send error: discovery");
                }
                t if PEERING_MSG_TYPE_RANGE.contains(&t) => {
                    peering_tx.send(packet).expect("channel send error: peering");
                }
                _ => panic!("invalid message type"),
            }
        } else {
            log::error!("udp socket read error; stopping incoming packet handler");
            break;
        }
    }
}

async fn outgoing_packet_handler(socket: Arc<UdpSocket>, mut outgoing_rx: PacketRx, local_id: LocalId) {
    loop {
        if let Some(packet) = outgoing_rx.recv().await {
            let OutgoingPacket {
                msg_type,
                msg_bytes,
                target_addr,
            } = packet;

            let signature = local_id.sign(&msg_bytes);

            let packet = Packet::new(msg_type, &msg_bytes, &local_id.public_key(), signature);

            let bytes = packet.protobuf().expect("error encoding outgoing packet");
            let n = socket.send_to(&bytes, target_addr).await.expect("socket send error");

            log::debug!("Sent {} bytes to {}.", n, target_addr);
        } else {
            log::error!("outgoing message channel dropped; stopping outgoing packet handler");
            break;
        }
    }
}
