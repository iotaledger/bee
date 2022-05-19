// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_storage::{
    access::{Exist, Fetch, Insert, InsertStrict, Update},
    backend,
};

use crate::{block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint};

/// A blanket-implemented helper trait for the storage layer.
pub trait StorageBackend:
    backend::StorageBackend
    + Insert<BlockId, Block>
    + Insert<(BlockId, BlockId), ()>
    + Insert<MilestoneIndex, MilestoneMetadata>
    + Insert<MilestoneId, MilestonePayload>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + InsertStrict<BlockId, BlockMetadata>
    + Exist<BlockId, Block>
    + Exist<MilestoneIndex, MilestoneMetadata>
    + Exist<MilestoneId, MilestonePayload>
    + Fetch<BlockId, Block>
    + Fetch<BlockId, BlockMetadata>
    + Fetch<BlockId, Vec<BlockId>>
    + Fetch<MilestoneIndex, MilestoneMetadata>
    + Fetch<MilestoneId, MilestonePayload>
    + Update<BlockId, BlockMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Insert<BlockId, Block>
        + Insert<(BlockId, BlockId), ()>
        + Insert<MilestoneIndex, MilestoneMetadata>
        + Insert<MilestoneId, MilestonePayload>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + InsertStrict<BlockId, BlockMetadata>
        + Exist<BlockId, Block>
        + Exist<MilestoneIndex, MilestoneMetadata>
        + Exist<MilestoneId, MilestonePayload>
        + Fetch<BlockId, Block>
        + Fetch<BlockId, BlockMetadata>
        + Fetch<BlockId, Vec<BlockId>>
        + Fetch<MilestoneIndex, MilestoneMetadata>
        + Fetch<MilestoneId, MilestonePayload>
        + Update<BlockId, BlockMetadata>
{
}
