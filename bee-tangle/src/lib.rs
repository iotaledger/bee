// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the IOTA tangle.

#![deny(missing_docs)]

mod config;
mod message_data;
mod tangle;

pub mod walkers;

pub use config::{TangleConfig, TangleConfigBuilder};
pub use message_data::MessageData;
pub use tangle::Tangle;
