// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides types to hold metrics related to other components.

pub mod node;
pub mod peer;

pub use node::NodeMetrics;
pub use peer::PeerMetrics;
