// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides a generic, type-safe and thread-safe event bus for event types.

#![warn(missing_docs)]

/// Module containing the `EventBus` type.
mod bus;
/// Module containing the `Event` trait and `EventId` type.
mod event;

pub use bus::EventBus;
pub use event::{Event, EventId};
