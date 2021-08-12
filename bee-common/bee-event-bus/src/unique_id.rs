// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::any::TypeId;

/// A unique identifier for [`EventBus`] callbacks.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum UniqueId<T> {
    /// Identifier for types.
    Type(TypeId),
    /// Identifier for objects.
    Object(T),
}

impl<T> From<TypeId> for UniqueId<T> {
    fn from(id: TypeId) -> Self {
        Self::Type(id)
    }
}
