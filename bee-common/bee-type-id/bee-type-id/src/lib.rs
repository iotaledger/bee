// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides a type to uniquely identify types.

pub use bee_type_id_derive::TypeId;

pub const TYPE_ID_LENGTH: usize = 32;

pub trait TypeId {
    const TYPE_ID: [u8; TYPE_ID_LENGTH];

    fn type_id() -> [u8; TYPE_ID_LENGTH] {
        Self::TYPE_ID
    }
}
