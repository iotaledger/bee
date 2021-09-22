// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A bee network layer API using backstage.

mod network;
mod peer;

pub use network::{NetworkActor, NetworkEvent};
pub use peer::{PeerReaderActor, PeerWriterActor, PeerWriterEvent};
