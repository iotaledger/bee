// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the IOTA tangle.

#![deny(missing_docs)]

mod message_data;
mod tangle;
mod walker;

pub use message_data::MessageData;
pub use tangle::Tangle;
pub use walker::{TangleDfsWalker, TangleDfsWalkerBuilder, TangleWalkerStatus};
