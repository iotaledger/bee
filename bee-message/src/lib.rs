// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

#[cfg(feature = "serde")]
#[macro_use]
mod serde;
mod error;
mod message;
mod message_id;

pub mod address;
pub mod constants;
pub mod input;
pub mod ledger_index;
pub mod milestone;
pub mod output;
pub mod parents;
pub mod payload;
pub mod prelude;
pub mod solid_entry_point;
pub mod unlock;

pub use error::Error;
pub use message::{Message, MessageBuilder, MESSAGE_LENGTH_MAX, MESSAGE_LENGTH_MIN};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};
pub use parents::Parents;
