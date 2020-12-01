// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod banned;
mod errors;
mod list;
mod manager;

pub use banned::*;
pub use errors::Error;
pub use list::*;
pub use manager::*;

pub type DataSender = flume::Sender<Vec<u8>>;
pub type DataReceiver = flume::Receiver<Vec<u8>>;

pub fn channel() -> (DataSender, DataReceiver) {
    flume::unbounded()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerRelation {
    Known,
    Unknown,
    Discovered,
}

impl PeerRelation {
    pub fn is_known(&self) -> bool {
        *self == PeerRelation::Known
    }

    pub fn is_unknown(&self) -> bool {
        *self == PeerRelation::Unknown
    }

    pub fn is_discovered(&self) -> bool {
        *self == PeerRelation::Discovered
    }
}
