// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    local::Local,
    packet::{
        IncomingPacket, MessageType, OutgoingPacket, Packet, DISCOVERY_MSG_TYPE_RANGE, MAX_PACKET_SIZE,
        PEERING_MSG_TYPE_RANGE,
    },
    peer::{peer_id::PeerId, PeerStore},
    task::{Runnable, ShutdownRx, TaskManager},
};

use tokio::{net::UdpSocket, sync::mpsc};

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
            bind_addr: config.bind_addr(),
        }
    }
}

pub(crate) struct IncomingPacketSenders {
    pub(crate) discovery_tx: IncomingPacketTx,
    pub(crate) peering_tx: IncomingPacketTx,
}

pub(crate) struct Server {
    config: ServerConfig,
    local: Local,
    incoming_senders: IncomingPacketSenders,
    outgoing_rx: OutgoingPacketRx,
}

impl Server {
    pub fn new(config: ServerConfig, local: Local, incoming_senders: IncomingPacketSenders) -> (Self, ServerTx) {
        let (outgoing_tx, outgoing_rx) = server_chan::<OutgoingPacket>();

        (
            Self {
                config,
                local,
                incoming_senders,
                outgoing_rx,
            },
            outgoing_tx,
        )
    }

    pub async fn init<S: PeerStore, const N: usize>(self, task_mngr: &mut TaskManager<S, N>) {
        let Server {
            config,
            local,
            incoming_senders,
            outgoing_rx,
        } = self;

        // Bind the UDP socket to the configured address.
        // Panic: We don't allow UDP socket binding to fail.
        let socket = UdpSocket::bind(&config.bind_addr)
            .await
            .expect("error binding udp socket");

        // The Tokio docs explain that there's no split method, and that we have to arc the UdpSocket in order to share
        // it.
        let incoming_socket = Arc::new(socket);
        let outgoing_socket = Arc::clone(&incoming_socket);

        let incoming_packet_handler = IncomingPacketHandler {
            incoming_socket,
            incoming_senders,
            bind_addr: config.bind_addr,
        };

        let outgoing_packet_handler = OutgoingPacketHandler {
            outgoing_socket,
            outgoing_rx,
            local,
            bind_addr: config.bind_addr,
        };

        task_mngr.run::<IncomingPacketHandler>(incoming_packet_handler);
        task_mngr.run::<OutgoingPacketHandler>(outgoing_packet_handler);
    }
}

struct IncomingPacketHandler {
    incoming_socket: Arc<UdpSocket>,
    incoming_senders: IncomingPacketSenders,
    bind_addr: SocketAddr,
}

struct OutgoingPacketHandler {
    outgoing_socket: Arc<UdpSocket>,
    outgoing_rx: OutgoingPacketRx,
    local: Local,
    bind_addr: SocketAddr,
}

// Note: Invalid packets from peers are not logged as warnings because the fault is not on our side.
#[async_trait::async_trait]
impl Runnable for IncomingPacketHandler {
    const NAME: &'static str = "IncomingPacketHandler";
    const SHUTDOWN_PRIORITY: u8 = 2;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
        let IncomingPacketHandler {
            incoming_socket,
            incoming_senders,
            bind_addr,
        } = self;

        let mut packet_bytes = [0; READ_BUFFER_SIZE];

        let IncomingPacketSenders {
            discovery_tx,
            peering_tx,
        } = incoming_senders;

        'recv: loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                r = incoming_socket.recv_from(&mut packet_bytes) => {
                    match r {
                        Ok((n, peer_addr)) => {
                            if peer_addr == bind_addr {
                                log::trace!("Received bytes from own bind address {}. Ignoring packet.", peer_addr);
                                continue 'recv;
                            }

                            if n > MAX_PACKET_SIZE {
                                log::trace!("Received too many bytes from {}. Ignoring packet.", peer_addr);
                                continue 'recv;
                            }

                            log::trace!("Received {} bytes from {}.", n, peer_addr);

                            // Decode the packet.
                            let packet = match Packet::from_protobuf(&packet_bytes[..n]) {
                                Ok(packet) => packet,
                                Err(e) => {
                                    log::trace!("Error decoding incoming packet from {}. {:?}. Ignoring packet.", peer_addr, e);
                                    continue 'recv;
                                }
                            };

                            // Unmarshal the message.
                            let (msg_type, msg_bytes) = match unmarshal(packet.msg_bytes()) {
                                Ok((msg_type, msg_bytes)) => (msg_type, msg_bytes),
                                Err(e) => {
                                    log::trace!("Error unmarshalling incoming message from {}. {:?}. Ignoring packet.", peer_addr, e);
                                    continue 'recv;
                                }
                            };

                            // Restore the peer id.
                            let peer_id = PeerId::from_public_key(*packet.public_key());

                            // Verify the packet.
                            let message = packet.msg_bytes();
                            let signature = packet.signature();
                            if !packet.public_key().verify(signature, message) {
                                log::trace!("Received packet with invalid signature");
                                continue 'recv;
                            }

                            let packet = IncomingPacket {
                                msg_type,
                                msg_bytes,
                                peer_addr,
                                peer_id,
                            };

                            // Depending on the message type, forward it to the appropriate manager.
                            match msg_type as u8 {
                                t if DISCOVERY_MSG_TYPE_RANGE.contains(&t) => {
                                    // Panic: We don't allow channel send failures.
                                    discovery_tx.send(packet).expect("channel send error: discovery");
                                }
                                t if PEERING_MSG_TYPE_RANGE.contains(&t) => {
                                    // Panic: We don't allow channel send failures.
                                    peering_tx.send(packet).expect("channel send error: peering");
                                }
                                _ => log::trace!("Received invalid message type. Ignoring packet."),
                            }
                        }
                        Err(e) => {
                            log::error!("UDP socket read error; stopping incoming packet handler. Cause: {}", e);
                            // TODO: initiate graceful shutdown
                            break 'recv;
                        }
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Runnable for OutgoingPacketHandler {
    const NAME: &'static str = "OutgoingPacketHandler";
    const SHUTDOWN_PRIORITY: u8 = 3;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
        let OutgoingPacketHandler {
            outgoing_socket,
            mut outgoing_rx,
            local,
            bind_addr,
        } = self;

        'recv: loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                o = outgoing_rx.recv() => {
                    if let Some(packet) = o {
                        let OutgoingPacket {
                            msg_type,
                            msg_bytes,
                            peer_addr,
                        } = packet;

                        // TODO: support ipv6
                        if peer_addr.is_ipv6() {
                            log::warn!("Trying to send to an IPv6 address ({}), which is not yet supported. Ignoring packet.", peer_addr);
                            continue 'recv;
                        }

                        if peer_addr == bind_addr {
                            log::warn!("Trying to send to own bind address: {}. Ignoring packet.", peer_addr);
                            continue 'recv;
                        }

                        let marshalled_bytes = marshal(msg_type, &msg_bytes);

                        let signature = local.sign(&marshalled_bytes);
                        let packet = Packet::new(msg_type, &marshalled_bytes, local.public_key(), signature);

                        let bytes = packet.to_protobuf();

                        if bytes.len() > MAX_PACKET_SIZE {
                            log::warn!("Trying to send too many bytes to {}. Ignoring packet.", peer_addr);
                            continue 'recv;
                        }

                        let n = outgoing_socket.send_to(&bytes, peer_addr).await.expect("socket send error");

                        log::trace!("Sent {} bytes to {}.", n, peer_addr);
                    } else {
                        // All `outgoing_tx` message senders were dropped.
                        break 'recv;
                    }
                }
            }
        }
    }
}

// TODO: @pvdrz wants to optimize this.
pub(crate) fn marshal(msg_type: MessageType, msg_bytes: &[u8]) -> Vec<u8> {
    let mut marshalled_bytes = vec![0u8; msg_bytes.len() + 1];
    marshalled_bytes[0] = msg_type as u8;
    marshalled_bytes[1..].copy_from_slice(msg_bytes);
    marshalled_bytes
}

// TODO: @pvdrz wants to optimize this.
pub(crate) fn unmarshal(marshalled_bytes: &[u8]) -> Result<(MessageType, Vec<u8>), ()> {
    let msg_type = num::FromPrimitive::from_u8(marshalled_bytes[0]).ok_or(())?;

    let mut msg_bytes = vec![0u8; marshalled_bytes.len() - 1];
    msg_bytes[..].copy_from_slice(&marshalled_bytes[1..]);

    Ok((msg_type, msg_bytes))
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
