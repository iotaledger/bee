// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Identifiers for each column family.
//!
//! Rocksdb allows creating new, isolated keyspaces by adding new column families to the database.
//! Each column family can be accessed using the [`cf_handle`](https://docs.rs/rocksdb/latest/rocksdb/struct.DBWithThreadMode.html#method.cf_handle)
//! method with one of the identifiers found here.

/// Identifier for the `System` column family.
pub const CF_SYSTEM: &str = "system";
/// Identifier for the `MessageId` to `Message` column family.
pub const CF_MESSAGE_ID_TO_MESSAGE: &str = "message_id_to_message";
/// Identifier for the `MessageId` to `MessageMetadata` column family.
pub const CF_MESSAGE_ID_TO_MESSAGE_METADATA: &str = "message_id_to_message_metadata";
