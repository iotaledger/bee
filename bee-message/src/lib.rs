// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// #![no_std]

#[macro_use]
extern crate alloc;

#[macro_use]
mod serde;
mod error;
mod message;
mod message_id;
mod vertex;

pub mod payload;
pub mod prelude;

pub use error::Error;
pub use message::{Message, MessageBuilder};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};
pub use vertex::Vertex;
