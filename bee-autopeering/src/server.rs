// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::config::AutopeeringConfig;
use crate::packets::{IncomingPacket, OutgoingPacket};

use tokio::{net::UdpSocket, sync::mpsc};

use std::{error, net::SocketAddr, sync::Arc};

const READ_BUFFER_SIZE: usize = crate::packets::MAX_PACKET_SIZE;

type PacketTx = mpsc::UnboundedSender<IncomingPacket>;
type PacketRx = mpsc::UnboundedReceiver<OutgoingPacket>;

pub(crate) struct AutopeeringServer {
    config: AutopeeringConfig,
    incoming_send: PacketTx,
    outgoing_recv: PacketRx,
}

impl AutopeeringServer {
    pub fn new(incoming_send: PacketTx, outgoing_recv: PacketRx, config: AutopeeringConfig) -> Self {
        Self {
            config,
            incoming_send,
            outgoing_recv,
        }
    }

    pub async fn run(self) {
        let AutopeeringServer {
            config,
            incoming_send,
            outgoing_recv,
        } = self;

        // Try to bind the UDP socket
        let socket = UdpSocket::bind(&config.bind_addr)
            .await
            .expect("error binding udp socket");

        // The Tokio docs explain that there's no split method, and that we have to arc the UdpSocket in order to share it.
        let incoming_socket = Arc::new(socket);
        let outgoing_socket = Arc::clone(&incoming_socket);

        // Spawn socket handlers
        tokio::spawn(incoming_packet_handler(incoming_socket, incoming_send));
        tokio::spawn(outgoing_packet_handler(outgoing_socket, outgoing_recv));
    }
}

async fn incoming_packet_handler(socket: Arc<UdpSocket>, tx: PacketTx) {
    let mut buf = [0; READ_BUFFER_SIZE];

    loop {
        if let Ok((len, from_peer)) = socket.recv_from(&mut buf).await {
            let packet = IncomingPacket {
                bytes: (&buf[..len]).to_vec(),
                source: from_peer,
            };

            tx.send(packet).expect("channel send error");
        } else {
            log::error!("udp socket read error; stopping incoming packet handler");
            break;
        }
    }
}

async fn outgoing_packet_handler(socket: Arc<UdpSocket>, mut rx: PacketRx) {
    loop {
        if let Some(packet) = rx.recv().await {
            let OutgoingPacket { bytes, target } = packet;
            let len = socket.send_to(&bytes, target).await.expect("socket send error");

            log::debug!("Sent {} bytes", len);
        } else {
            log::error!("outgoing message channel dropped; stopping outgoing packet handler");
            break;
        }
    }
}
