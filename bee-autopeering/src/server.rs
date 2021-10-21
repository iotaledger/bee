// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    identity::PeerId,
    local::Local,
    packet::{IncomingPacket, OutgoingPacket, Packet, DISCOVERY_MSG_TYPE_RANGE, PEERING_MSG_TYPE_RANGE},
    shutdown::ShutdownRx,
};

use tokio::{
    net::UdpSocket,
    sync::mpsc::{self, error::SendError},
};

use std::{net::SocketAddr, sync::Arc};

pub(crate) use tokio::sync::mpsc::unbounded_channel as server_chan;

const READ_BUFFER_SIZE: usize = crate::packet::MAX_PACKET_SIZE;

pub(crate) type ServerRx = mpsc::UnboundedReceiver<IncomingPacket>;
pub(crate) type ServerTx = mpsc::UnboundedSender<OutgoingPacket>;

type IncomingPacketTx = mpsc::UnboundedSender<IncomingPacket>;
type OutgoingPacketRx = mpsc::UnboundedReceiver<OutgoingPacket>;

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

pub(crate) struct IncomingPacketSenders {
    pub(crate) discover_tx: IncomingPacketTx,
    pub(crate) peering_tx: IncomingPacketTx,
}

pub(crate) struct Server {
    config: ServerConfig,
    local: Local,
    incoming_senders: IncomingPacketSenders,
    outgoing_rx: OutgoingPacketRx,
    incoming_shutdown_rx: ShutdownRx,
    outgoing_shutdown_rx: ShutdownRx,
}

impl Server {
    pub fn new(
        config: ServerConfig,
        local: Local,
        incoming_senders: IncomingPacketSenders,
        incoming_shutdown_rx: ShutdownRx,
        outgoing_shutdown_rx: ShutdownRx,
    ) -> (Self, ServerTx) {
        let (outgoing_tx, outgoing_rx) = server_chan::<OutgoingPacket>();

        (
            Self {
                config,
                local,
                incoming_senders,
                outgoing_rx,
                incoming_shutdown_rx,
                outgoing_shutdown_rx,
            },
            outgoing_tx,
        )
    }

    pub async fn run(self) {
        let Server {
            config,
            local: local_id,
            incoming_senders,
            outgoing_rx,
            incoming_shutdown_rx,
            outgoing_shutdown_rx,
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
        tokio::spawn(incoming_packet_handler(
            incoming_socket,
            incoming_senders,
            incoming_shutdown_rx,
        ));
        tokio::spawn(outgoing_packet_handler(
            outgoing_socket,
            outgoing_rx,
            local_id,
            outgoing_shutdown_rx,
        ));
    }
}

async fn incoming_packet_handler(
    socket: Arc<UdpSocket>,
    incoming_senders: IncomingPacketSenders,
    mut incoming_shutdown_rx: ShutdownRx,
) {
    let mut packet_bytes = [0; READ_BUFFER_SIZE];

    let IncomingPacketSenders {
        discover_tx,
        peering_tx,
    } = incoming_senders;

    loop {
        tokio::select! {
            _ = &mut incoming_shutdown_rx => {
                break;
            }
            r = socket.recv_from(&mut packet_bytes) => {
                if let Ok((n, source_addr)) = r {
                    log::debug!("Received {} bytes from {}.", n, source_addr);

                    let packet = Packet::from_protobuf(&packet_bytes[..n]).expect("error decoding incoming packet");

                    // Restore the peer id.
                    let peer_id = PeerId::from_public_key(packet.public_key());
                    log::debug!("PeerId: {}.", peer_id);

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
                        peer_id,
                    };

                    match msg_type as u32 {
                        t if DISCOVERY_MSG_TYPE_RANGE.contains(&t) => {
                            discover_tx.send(packet).expect("channel send error: discovery");
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
    }
}

async fn outgoing_packet_handler(
    socket: Arc<UdpSocket>,
    mut outgoing_rx: OutgoingPacketRx,
    local_id: Local,
    mut outgoing_shutdown_rx: ShutdownRx,
) {
    loop {
        tokio::select! {
            _ = &mut outgoing_shutdown_rx => {
                break;
            }
            o = outgoing_rx.recv() => {
                if let Some(packet) = o {
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
    }
}

pub(crate) struct ServerSocket {
    pub(crate) server_rx: ServerRx,
    pub(crate) server_tx: ServerTx,
}

impl ServerSocket {
    pub fn new(rx: ServerRx, tx: ServerTx) -> Self {
        Self {
            server_rx: rx,
            server_tx: tx,
        }
    }
}