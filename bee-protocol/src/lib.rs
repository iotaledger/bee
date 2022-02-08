// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides types and workers enabling the IOTA protocol.

// TODO
// #![deny(missing_docs, warnings)]

#![cfg_attr(doc_cfg, feature(doc_cfg))]

/// Protocol version currently used by the network.
pub const PROTOCOL_VERSION: u8 = 0;

pub mod types;
#[cfg(feature = "workers")]
pub mod workers;
