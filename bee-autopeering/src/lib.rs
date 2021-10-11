// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering implementation for the Bee framework.

#![warn(missing_docs)]

mod config;
mod identity;
mod init;
mod manager;
mod message;
mod multiaddr;
mod packet;
mod peer;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}
mod store;
