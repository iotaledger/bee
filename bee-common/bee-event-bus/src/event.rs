// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Type of the identifier of an event type.
pub type EventId = u64;

/// Trait describing an event type.
pub trait Event: 'static {
    /// Returns the identifier of the event type.
    fn id() -> EventId
    where
        Self: Sized;
}
