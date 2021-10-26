// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Core data types for messages in the Tangle.

#![no_std]
#![deny(missing_docs)]
#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

#[macro_use]
mod macros;
mod builder;
mod message;
mod message_id;
#[cfg(feature = "metadata")]
mod metadata;

pub mod address;
pub mod error;
pub mod input;
pub mod output;
pub mod parents;
pub mod payload;
pub mod signature;
pub mod unlock;
pub mod util;

pub use builder::MessageBuilder;
pub use error::{MessageUnpackError, ValidationError};
pub use message::{Message, MESSAGE_LENGTH_RANGE};
pub use message_id::MessageId;
#[cfg(feature = "metadata")]
pub use metadata::MessageMetadata;

/// The total number of IOTA tokens in circulation.
pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;
