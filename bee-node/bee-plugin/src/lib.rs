// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The bee node API for plugins.

#![deny(missing_docs, warnings)]

mod handler;
mod manager;
mod streamer;
mod grpc {
    tonic::include_proto!("plugin");
}
mod error;
mod plugin;

pub mod event;
pub mod handshake;
pub mod hotloading;

pub use error::PluginError;
pub use manager::PluginManager;
pub use plugin::{serve_plugin, Plugin};

use std::any::TypeId;

/// A unique identifier for `EventBus` callbacks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UniqueId {
    /// Identifier for types.
    Type(TypeId),
    /// Identifier for plugins.
    Plugin(PluginId),
}

impl From<TypeId> for UniqueId {
    fn from(id: TypeId) -> Self {
        Self::Type(id)
    }
}

impl From<PluginId> for UniqueId {
    fn from(id: PluginId) -> Self {
        Self::Plugin(id)
    }
}

/// A unique identifier for each plugin.
///
/// Uniqueness is guaranteed inside a single `PluginManager`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PluginId(usize);
