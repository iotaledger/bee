// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod handler;
mod manager;
mod streamer;
mod grpc {
    tonic::include_proto!("plugin");
}
mod error;
pub mod server;

pub use error::PluginError;
pub use grpc::EventId;
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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct PluginId(usize);
