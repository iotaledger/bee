// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides a generic, type-safe and thread-safe event bus for arbitrary event types.

#![deny(missing_docs, warnings)]

mod event_bus;
mod unique_id;

pub use event_bus::EventBus;
pub use unique_id::UniqueId;
