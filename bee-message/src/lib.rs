// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

#[cfg(feature = "serde")]
#[macro_use]
mod serde;
mod error;
mod message;
mod message_id;

pub mod ledger_index;
pub mod milestone;
pub mod parents;
pub mod payload;
pub mod prelude;
pub mod solid_entry_point;

pub use error::Error;
pub use message::{Message, MessageBuilder, MESSAGE_LENGTH_MAX, MESSAGE_LENGTH_MIN};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};
pub use parents::Parents;
