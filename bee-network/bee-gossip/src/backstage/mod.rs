// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A Bee gossip layer API using [backstage](https://github.com/iotaledger/backstage.git).

mod gossip;
mod peer;

pub use gossip::{GossipActor, GossipEvent};
pub use peer::{GossipReaderActor, GossipWriterActor, GossipWriterEvent};
