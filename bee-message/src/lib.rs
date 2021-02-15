// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

#[macro_use]
mod serde;
mod error;
mod message;
mod message_id;

pub mod ledger_index;
pub mod milestone;
pub mod payload;
pub mod prelude;
pub mod solid_entry_point;

pub use error::Error;
pub use message::{Message, MessageBuilder, MESSAGE_LENGTH_MAX, MESSAGE_LENGTH_MIN, MESSAGE_PARENTS_RANGE};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};
