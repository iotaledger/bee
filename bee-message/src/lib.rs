// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Core data types for messages in the tangle.

#![deny(missing_docs, warnings)]

extern crate alloc;

#[macro_use]
mod r#macro;
mod error;
mod message;
mod message_id;

/// A module that provides types and syntactic validations of addresses.
pub mod address;
/// A module that contains constants related to messages.
pub mod constant;
/// A module that provides types and syntactic validations of inputs.
pub mod input;
/// A module that provides types and syntactic validations of milestones.
pub mod milestone;
/// A module that provides types and syntactic validations of outputs.
pub mod output;
/// A module that provides types and syntactic validations of parents.
pub mod parent;
/// A module that provides types and syntactic validations of payloads.
pub mod payload;
/// A module that provides types and syntactic validations of signatures.
pub mod signature;
/// A module that provides types and syntactic validations of unlock blocks.
pub mod unlock_block;
/// A module that provides utilities.
pub mod util;

pub use error::Error;
pub use message::{Message, MessageBuilder};
pub use message_id::MessageId;
