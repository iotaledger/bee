// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::peer_id::PeerId;

use tokio::sync::mpsc;

#[derive(Debug)]
pub(crate) enum Command {
    SendVerificationRequest { peer_id: PeerId },
    SendDiscoveryRequest { peer_id: PeerId },
    SendVerificationRequests,
    SendDiscoveryRequests,
}

pub(crate) type CommandRx = mpsc::UnboundedReceiver<Command>;
pub(crate) type CommandTx = mpsc::UnboundedSender<Command>;

pub(crate) fn command_chan() -> (CommandTx, CommandRx) {
    mpsc::unbounded_channel::<Command>()
}
