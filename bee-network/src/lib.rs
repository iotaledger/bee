// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Utilities to establish and maintain network connections with peers.

#![deny(missing_docs)]

mod conn;
mod consts;
mod handshake;
mod packet;
mod proto;
mod util;

#[cfg(feature = "backstage")]
pub mod backstage;
pub mod config;
pub mod event;
pub mod identity;
pub mod message;
pub mod network;
pub mod peer;
