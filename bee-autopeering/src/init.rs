// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::config::AutopeeringConfig;
use crate::manager::AutopeeringManager;
use crate::packets::{IncomingPacket, OutgoingPacket};

use tokio::{net::UdpSocket, sync::mpsc};

use std::{error, net::SocketAddr, sync::Arc};

const READ_BUFFER_SIZE: usize = 1024;

type Tx = mpsc::UnboundedSender<IncomingPacket>;
type Rx = mpsc::UnboundedReceiver<OutgoingPacket>;

pub async fn init(config: AutopeeringConfig) -> Result<(), Box<dyn error::Error>> {
    // Create 2 channels for the communication with the UDP socket
    let (incoming_send, incoming_recv) = mpsc::unbounded_channel::<IncomingPacket>();
    let (outgoing_send, outgoing_recv) = mpsc::unbounded_channel::<OutgoingPacket>();

    // Try to bind the UDP socket
    let socket = UdpSocket::bind(&config.bind_addr).await?;

    // The Tokio docs explain that there's no split method, and that we have to arc the UdpSocket in order to share it.
    let incoming_socket = Arc::new(socket);
    let outgoing_socket = Arc::clone(&incoming_socket);

    // Spawn socket handlers
    tokio::spawn(incoming_msg_handler(incoming_socket, incoming_send));
    tokio::spawn(outgoing_msg_handler(outgoing_socket, outgoing_recv));

    // Spawn the autopeering manager
    let mngr = AutopeeringManager::new(incoming_recv, outgoing_send, config);
    tokio::spawn(mngr.run());

    Ok(())
}

async fn incoming_msg_handler(socket: Arc<UdpSocket>, tx: Tx) {
    let mut buf = [0; READ_BUFFER_SIZE];

    loop {
        if let Ok((len, from_peer)) = socket.recv_from(&mut buf).await {
            let msg = IncomingPacket {
                bytes: (&buf[..len]).to_vec(),
                source: from_peer,
            };

            tx.send(msg).expect("channel send error");
        } else {
            log::error!("udp socket read error; stopping incoming message handler");
            break;
        }
    }
}

async fn outgoing_msg_handler(socket: Arc<UdpSocket>, mut rx: Rx) {
    loop {
        if let Some(msg) = rx.recv().await {
            let OutgoingPacket { bytes, target } = msg;
            let len = socket.send_to(&bytes, target).await.expect("socket send error");

            log::debug!("Sent {} bytes", len);
        } else {
            log::error!("outgoing message channel dropped; stopping outgoing message handler");
            break;
        }
    }
}
