// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Generic access tests for key/value storage tables.

#![deny(missing_docs)]

mod message_id_to_message;
mod message_id_to_message_metadata;
mod system;

pub use message_id_to_message::message_id_to_message_access;
pub use message_id_to_message_metadata::message_id_to_message_metadata_access;
pub use system::system_access;
