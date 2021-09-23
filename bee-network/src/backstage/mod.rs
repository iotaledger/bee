// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A bee network layer API using [backstage](https://github.com/iotaledger/backstage.git).

mod network;
mod peer;

pub use network::{GossipActor, NetworkEvent};
pub use peer::{PeerReaderActor, PeerWriterActor, PeerWriterEvent};
