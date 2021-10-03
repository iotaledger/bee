// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Core data types for messages in the Tangle.

#![no_std]
#![deny(missing_docs, warnings)]

extern crate alloc;

#[macro_use]
mod macros;
mod message;
mod message_builder;
mod message_id;

pub mod address;
pub mod error;
pub mod input;
pub mod output;
pub mod parents;
pub mod payload;
pub mod signature;
pub mod unlock;
pub mod util;

pub use error::{MessageUnpackError, ValidationError};
pub use message::{Message, MESSAGE_LENGTH_RANGE, MESSAGE_PUBLIC_KEY_LENGTH, MESSAGE_SIGNATURE_LENGTH};
pub use message_builder::MessageBuilder;
pub use message_id::MessageId;
pub use parents::{PREFIXED_PARENTS_LENGTH_MIN, PREFIXED_PARENTS_LENGTH_MAX};

/// The total number of IOTA tokens in circulation.
pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;
