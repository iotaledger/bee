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

/// A module that provides types and syntactic validations of addresses.
pub mod address;
/// A module that provides error types for validation and packing/unpacking.
pub mod error;
/// A module that provides types and syntactic validations of inputs.
pub mod input;
/// A module containing a convenient builder for [`Message`] construction.
pub mod message_builder;
/// A module that provides types and syntactic validations of outputs.
pub mod output;
/// A module that provides types and syntactic validations of parents.
pub mod parents;
/// A module that provides types and syntactic validations of payloads.
pub mod payload;
/// A module that provides types and syntactic validations of signatures.
pub mod signature;
/// A module that provides types and syntactic validations of unlock blocks.
pub mod unlock;
/// A module that provides utilities.
pub mod util;

pub use error::{MessagePackError, MessageUnpackError, ValidationError};
pub use message::{Message, MESSAGE_LENGTH_RANGE};
pub use message_builder::MessageBuilder;
pub use message_id::MessageId;

/// The total number of IOTA tokens in circulation.
pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;
