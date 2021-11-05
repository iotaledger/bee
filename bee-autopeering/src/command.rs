// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::peer_id::PeerId;

use tokio::sync::mpsc;

// TODO: revisit dead code
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Command {
    // Send a verfication request to that peer.
    Verify { peer_id: PeerId },
    // Send a discovery request to that peer.
    Query { peer_id: PeerId },
    // Send a peering request to that peer.
    Peer { peer_id: PeerId },
    // Send a drop-peering request to that peer.
    Drop { peer_id: PeerId },
}

pub(crate) type CommandRx = mpsc::UnboundedReceiver<Command>;
pub(crate) type CommandTx = mpsc::UnboundedSender<Command>;

pub(crate) fn command_chan() -> (CommandTx, CommandRx) {
    mpsc::unbounded_channel::<Command>()
}
