// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Core data types for messages in the tangle.

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

#[macro_use]
mod macros;
mod message;
mod message_id;

pub mod address;
pub mod error;
pub mod input;
pub mod message_builder;
pub mod output;
pub mod parents;
pub mod payload;
pub mod signature;
pub mod unlock;
pub mod util;

pub use error::{MessagePackError, MessageUnpackError, ValidationError};
pub use message::{Message, MESSAGE_LENGTH_RANGE};
pub use message_builder::MessageBuilder;
pub use message_id::MessageId;

/// The total number of IOTA tokens in circulation.
pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;
