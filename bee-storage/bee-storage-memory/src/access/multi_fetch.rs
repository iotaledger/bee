// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Multi-fetch access operations.

use std::{iter::Map, vec::IntoIter};

use bee_block::{
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{ConsumedOutput, CreatedOutput, OutputDiff};
use bee_storage::{access::MultiFetch, backend::StorageBackend, system::System};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
};

use crate::storage::Storage;

macro_rules! impl_multi_fetch {
    ($key:ty, $value:ty, $field:ident) => {
        impl<'a> MultiFetch<'a, $key, $value> for Storage {
            type Iter = Map<
                IntoIter<Option<$value>>,
                fn(Option<$value>) -> Result<Option<$value>, <Self as StorageBackend>::Error>,
            >;

            fn multi_fetch(&'a self, keys: &'a [$key]) -> Result<Self::Iter, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.multi_fetch(keys))
            }
        }
    };
}

impl_multi_fetch!(u8, System, system);
impl_multi_fetch!(BlockId, Block, block_id_to_block);
impl_multi_fetch!(BlockId, BlockMetadata, block_id_to_metadata);
impl_multi_fetch!(OutputId, CreatedOutput, output_id_to_created_output);
impl_multi_fetch!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_multi_fetch!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_multi_fetch!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_multi_fetch!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_multi_fetch!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
