// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a [`Packable`] trait to serialize and deserialize types.

#![no_std]
#![deny(missing_docs)]

pub mod coerce;
pub mod error;
pub mod packable;
pub mod packer;
pub mod unpacker;

pub use packable::*;
