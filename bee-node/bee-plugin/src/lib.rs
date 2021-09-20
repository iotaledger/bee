// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The bee node API for plugins.

#![deny(missing_docs)]

mod handler;
mod manager;
mod streamer;
mod grpc {
    tonic::include_proto!("plugin");
}
mod error;
mod handshake;
mod plugin;

pub mod event;
pub mod hotloader;
pub mod message;

pub use error::PluginError;
pub use handshake::PluginHandshake;
pub use manager::PluginManager;
pub use plugin::{serve_plugin, Plugin};

/// A unique identifier for each plugin.
///
/// Uniqueness is guaranteed inside a single [`PluginManager`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct PluginId(usize);

impl core::fmt::Display for PluginId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier used by an [`EventBus`](bee_event_bus::EventBus) that handles plugin callbacks.
pub type UniqueId = bee_event_bus::UniqueId<PluginId>;

// We cannot provide a `From` implementation because `UniqueId<T>` is defined outside this crate.
#[allow(clippy::from_over_into)]
impl Into<UniqueId> for PluginId {
    fn into(self) -> UniqueId {
        UniqueId::Object(self)
    }
}
