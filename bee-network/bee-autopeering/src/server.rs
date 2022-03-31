// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{net::SocketAddr, sync::Arc};

pub(crate) use tokio::sync::mpsc::unbounded_channel as server_chan;
use tokio::{net::UdpSocket, sync::mpsc};

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

const READ_BUFFER_SIZE: usize = crate::packet::MAX_PACKET_SIZE;
const IP_V4_FLAG: bool = false;
const IP_V6_FLAG: bool = true;

pub(crate) type ServerRx = mpsc::UnboundedReceiver<IncomingPacket>;
pub(crate) type ServerTx = mpsc::UnboundedSender<OutgoingPacket>;

type IncomingPacketTx = mpsc::UnboundedSender<IncomingPacket>;
type OutgoingPacketTx = mpsc::UnboundedSender<OutgoingPacket>;
type OutgoingPacketRx = mpsc::UnboundedReceiver<OutgoingPacket>;

pub(crate) struct ServerConfig {
    pub(crate) bind_addr_v4: Option<SocketAddr>,
    pub(crate) bind_addr_v6: Option<SocketAddr>,
}

impl ServerConfig {
    /// Note: the constructor assumes a semantically valid config, i.e. at least one bind address is set and correct IP
    /// versions are used.
    pub(crate) fn new(config: &AutopeeringConfig) -> Self {
        Self {
            bind_addr_v4: config.bind_addr_v4(),
            bind_addr_v6: config.bind_addr_v6(),
        }
    }
}

#[derive(Clone)]
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
    pub(crate) fn new(config: ServerConfig, local: Local, incoming_senders: IncomingPacketSenders) -> (Self, ServerTx) {
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

    pub(crate) async fn init<S: PeerStore>(self, task_mngr: &mut TaskManager<S>) {
        let Server {
            config,
            local,
            incoming_senders,
            outgoing_rx,
        } = self;

        let (outgoing_tx_v4, outgoing_rx_v4) = server_chan::<OutgoingPacket>();
        let (outgoing_tx_v6, outgoing_rx_v6) = server_chan::<OutgoingPacket>();

        let outgoing_packet_manager = OutgoingPacketManager {
            outgoing_rx,
            outgoing_tx_v4,
            outgoing_tx_v6,
        };

        task_mngr.run::<OutgoingPacketManager>(outgoing_packet_manager);

        let mut socket_bound = false;

        // Bind a socket to the given IPv4 address.
        if let Some(bind_addr_v4) = config.bind_addr_v4 {
            if let Ok(local_addr) = bind_socket::<_, IP_V4_FLAG>(
                bind_addr_v4,
                incoming_senders.clone(),
                local.clone(),
                outgoing_rx_v4,
                task_mngr,
            )
            .await
            {
                log::debug!("Bound IPv4 socket to {}.", local_addr);
                socket_bound = true;
            } else {
                log::warn!("Binding to the configured IPv4 address ({}) failed.", bind_addr_v4)
            }
        }

        // Bind a socket to the given IPv6 address.
        if let Some(bind_addr_v6) = config.bind_addr_v6 {
            if let Ok(local_addr) =
                bind_socket::<_, IP_V6_FLAG>(bind_addr_v6, incoming_senders, local, outgoing_rx_v6, task_mngr).await
            {
                log::debug!("Bound IPv6 socket to {}.", local_addr);
                socket_bound = true;
            } else {
                log::warn!("Binding to the configured IPv6 address ({}) failed.", bind_addr_v6)
            }
        }

        // At least one socket must be successfully bound.
        if !socket_bound {
            panic!("failed binding a UDP socket to an address");
        }
    }
}

async fn bind_socket<S: PeerStore, const USE_IP_V6: bool>(
    bind_addr: SocketAddr,
    incoming_senders: IncomingPacketSenders,
    local: Local,
    outgoing_rx: OutgoingPacketRx,
    task_mngr: &mut TaskManager<S>,
) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    // Bind the UDP socket to the configured address.
    // Panic: We don't allow any UDP socket binding to fail.
    let socket = UdpSocket::bind(bind_addr).await?;
    let local_addr = socket.local_addr()?;

    // Note: See Tokio docs for an explanation why there's no split method.
    let incoming_socket = Arc::new(socket);
    let outgoing_socket = Arc::clone(&incoming_socket);

    let incoming_packet_handler = IncomingPacketHandler {
        incoming_socket,
        incoming_senders,
        bind_addr,
    };

    let outgoing_packet_handler = OutgoingPacketHandler {
        outgoing_socket,
        outgoing_rx,
        local,
        bind_addr,
    };

    task_mngr.run::<IncomingPacketHandler<USE_IP_V6>>(incoming_packet_handler);
    task_mngr.run::<OutgoingPacketHandler<USE_IP_V6>>(outgoing_packet_handler);

    Ok(local_addr)
}

struct IncomingPacketHandler<const USE_IP_V6: bool> {
    incoming_socket: Arc<UdpSocket>,
    incoming_senders: IncomingPacketSenders,
    bind_addr: SocketAddr,
}

// Note: Invalid packets from peers are not logged as warnings because the fault is not on our side.
#[async_trait::async_trait]
impl<const USE_IP_V6: bool> Runnable for IncomingPacketHandler<USE_IP_V6> {
    const NAME: &'static str = if USE_IP_V6 {
        "IncomingIPv6PacketHandler"
    } else {
        "IncomingIPv4PacketHandler"
    };
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

struct OutgoingPacketManager {
    outgoing_rx: OutgoingPacketRx,
    outgoing_tx_v4: OutgoingPacketTx,
    outgoing_tx_v6: OutgoingPacketTx,
}

#[async_trait::async_trait]
impl Runnable for OutgoingPacketManager {
    const NAME: &'static str = "OutgoingPacketManager";
    const SHUTDOWN_PRIORITY: u8 = 4;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
        let OutgoingPacketManager {
            mut outgoing_rx,
            outgoing_tx_v4,
            outgoing_tx_v6,
        } = self;

        loop {
            tokio::select! {
            _ = &mut shutdown_rx => {
                break;
            }
            o = outgoing_rx.recv() => {
                if let Some(packet) = o {
                    let peer_addr = packet.peer_addr;
                    if peer_addr.is_ipv6() {
                        // Send with IPv6 interface.
                        if outgoing_tx_v6.send(packet).is_err() {
                            log::debug!("Trying to send to an IPv6 address without configured IPv6 interface ({}). Ignoring packet.", peer_addr);
                        }
                    } else {
                        // Send with IPv4 interface.
                        if outgoing_tx_v4.send(packet).is_err() {
                            log::debug!("Trying to send to an IPv4 address without configured IPv4 interface ({}). Ignoring packet.", peer_addr);
                        }
                    }
                }}}
        }
    }
}

struct OutgoingPacketHandler<const USE_IP_V6: bool> {
    outgoing_socket: Arc<UdpSocket>,
    outgoing_rx: OutgoingPacketRx,
    local: Local,
    bind_addr: SocketAddr,
}

#[async_trait::async_trait]
impl<const USE_IP_V6: bool> Runnable for OutgoingPacketHandler<USE_IP_V6> {
    const NAME: &'static str = if USE_IP_V6 {
        "OutgoingIPv6PacketHandler"
    } else {
        "OutgoingIPv4PacketHandler"
    };
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
