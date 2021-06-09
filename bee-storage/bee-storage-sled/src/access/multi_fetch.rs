// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Multi-fetch access operations.

use crate::{storage::Storage, trees::*};

use bee_common::packable::Packable;
use bee_ledger::types::{Balance, ConsumedOutput, CreatedOutput, OutputDiff};
use bee_message::{
    address::Address,
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    Message, MessageId,
};
use bee_storage::{access::MultiFetch, backend::StorageBackend, system::System};
use bee_tangle::{metadata::MessageMetadata, solid_entry_point::SolidEntryPoint};

#[async_trait::async_trait]
impl MultiFetch<u8, System> for Storage {
    async fn multi_fetch(
        &self,
        keys: &[u8],
    ) -> Result<Vec<Result<Option<System>, <Self as StorageBackend>::Error>>, <Self as StorageBackend>::Error> {
        Ok(keys
            .iter()
            .map(|k| self.inner.get(k.pack_new()))
            .map(|r| {
                r.map(|o| o.map(|v| System::unpack_unchecked(&mut v.as_ref()).unwrap()))
                    .map_err(|e| e.into())
            })
            .collect())
    }
}

macro_rules! impl_multi_fetch {
    ($key:ty, $value:ty, $cf:expr) => {
        #[async_trait::async_trait]
        impl MultiFetch<$key, $value> for Storage {
            async fn multi_fetch(
                &self,
                keys: &[$key],
            ) -> Result<Vec<Result<Option<$value>, <Self as StorageBackend>::Error>>, <Self as StorageBackend>::Error>
            {
                let tree = self.inner.open_tree($cf)?;

                Ok(keys
                    .iter()
                    .map(|k| tree.get(k.pack_new()))
                    .map(|r| {
                        r.map(|o| o.map(|v| <$value>::unpack_unchecked(&mut v.as_ref()).unwrap()))
                            .map_err(|e| e.into())
                    })
                    .collect())
            }
        }
    };
}

impl_multi_fetch!(MessageId, Message, TREE_MESSAGE_ID_TO_MESSAGE);
impl_multi_fetch!(MessageId, MessageMetadata, TREE_MESSAGE_ID_TO_METADATA);
impl_multi_fetch!(OutputId, CreatedOutput, TREE_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_multi_fetch!(OutputId, ConsumedOutput, TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_multi_fetch!(MilestoneIndex, Milestone, TREE_MILESTONE_INDEX_TO_MILESTONE);
impl_multi_fetch!(
    SolidEntryPoint,
    MilestoneIndex,
    TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX
);
impl_multi_fetch!(MilestoneIndex, OutputDiff, TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_multi_fetch!(Address, Balance, TREE_ADDRESS_TO_BALANCE);
