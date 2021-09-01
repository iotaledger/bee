// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Identifiers for each tree.
//!
//! Sled allows creating new, isolated keyspaces by adding new trees to the database.
//! Each tree can be accessed using the `sled::Db::open_tree` method with one of the identifiers found here.

/// Identifier for the `MessageId` to `Message` tree.
pub const TREE_MESSAGE_ID_TO_MESSAGE: &str = "message_id_to_message";
