// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Identifiers for each tree.
//!
//! Sled allows creating new, isolated keyspaces by adding new trees to the database.
//! Each tree can be accessed using the `sled::Db::open_tree` method with one of the identifiers found here.

/// Identifier for the `BlockId` to `Block` tree.
pub const TREE_MESSAGE_ID_TO_MESSAGE: &str = "message_id_to_message";
/// Identifier for the `BlockId` to `BlockMetadata` tree.
pub const TREE_MESSAGE_ID_TO_METADATA: &str = "message_id_to_metadata";
/// Identifier for the `BlockId` to `Vec<BlockId>` tree.
pub const TREE_MESSAGE_ID_TO_MESSAGE_ID: &str = "message_id_to_message_id";
/// Identifier for the `OutputId` to `CreatedOutput` tree.
pub const TREE_OUTPUT_ID_TO_CREATED_OUTPUT: &str = "output_id_to_created_output";
/// Identifier for the `OutputId` to `ConsumedOutput` tree.
pub const TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT: &str = "output_id_to_consumed_output";
/// Identifier for the `Unspent` tree.
pub const TREE_OUTPUT_ID_UNSPENT: &str = "output_id_unspent";
/// Identifier for the `Ed25519Address` to `OutputId` tree.
pub const TREE_ED25519_ADDRESS_TO_OUTPUT_ID: &str = "ed25519_address_to_output_id";
/// Identifier for the `LedgerIndex` tree.
pub const TREE_LEDGER_INDEX: &str = "ledger_index";
/// Identifier for the `MilestoneIndex` to `MilestoneMetadata` tree.
pub const TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA: &str = "milestone_index_to_milestone_metadata";
/// Identifier for the `MilestoneId` to `MilestonePayload` tree.
pub const TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD: &str = "milestone_id_to_milestone_payload";
/// Identifier for the `SnapshotInfo` tree.
pub const TREE_SNAPSHOT_INFO: &str = "snapshot_info";
/// Identifier for the `SolidEntryPoint` to `MilestoneIndex` tree.
pub const TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX: &str = "solid_entry_point_to_milestone_index";
/// Identifier for the `MilestoneIndex` to `OutputDiff` tree.
pub const TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF: &str = "milestone_index_to_output_diff";
/// Identifier for the `MilestoneIndex` to `Vec<UnreferencedBlock>` tree.
pub const TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE: &str = "milestone_index_to_unreferenced_block";
/// Identifier for the `MilestoneIndex` to `Vec<Receipt>` tree.
pub const TREE_MILESTONE_INDEX_TO_RECEIPT: &str = "milestone_index_to_receipt";
/// Identifier for the `bool` to `Vec<TreasuryOutput>` tree.
pub const TREE_SPENT_TO_TREASURY_OUTPUT: &str = "spent_to_treasury_output";
