// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Core data types for messages in the tangle.

//#![deny(missing_docs, warnings)]

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

#[macro_use]
mod macros;
mod error;
mod message;
mod message_id;

/// A module that provides types and syntactic validations of addresses.
pub mod address;
/// A module containing a convenient builder for `Message` construction.
pub mod builder;
/// A module that contains constants related to messages.
pub mod constants;
/// A module that provides types and syntactic validations of inputs.
pub mod input;
/// A module that provides types and syntactic validations of outputs.
pub mod output;
/// A module that provides types and syntactic validations of parents.
pub mod parents;
/// A module that provides types and syntactic validations of payloads.
pub mod payload;
/// A prelude for the `bee-message` crate.
pub mod prelude;
/// A module that provides types and syntactic validations of signatures.
pub mod signature;
/// A module that provides types and syntactic validations of unlock blocks.
pub mod unlock;

pub use builder::MessageBuilder;
pub use error::*;
pub use message::{Message, MESSAGE_LENGTH_RANGE};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};
