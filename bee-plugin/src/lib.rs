// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod handler;
pub mod hotloading;
mod manager;
mod streamer;
mod grpc {
    tonic::include_proto!("plugin");
}
mod error;
pub mod event;
pub mod plugin;

pub use error::PluginError;
pub use manager::PluginManager;

use std::any::TypeId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UniqueId {
    Type(TypeId),
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PluginId(usize);
