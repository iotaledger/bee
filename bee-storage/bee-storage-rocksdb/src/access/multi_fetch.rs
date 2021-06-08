// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

use bee_common::packable::Packable;
use bee_ledger::types::{Balance, ConsumedOutput, CreatedOutput, OutputDiff};
use bee_message::{
    address::Address,
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    Message, MessageId,
};
use bee_storage::{access::MultiFetch, system::System};
use bee_tangle::{metadata::MessageMetadata, solid_entry_point::SolidEntryPoint};

macro_rules! impl_multi_fetch {
    ($key:ty, $value:ty, $cf:expr) => {
        #[async_trait::async_trait]
        impl MultiFetch<$key, $value> for Storage {
            async fn multi_fetch(
                &self,
                keys: &[$key],
            ) -> Result<Vec<Result<Option<$value>, <Self as StorageBackend>::Error>>, <Self as StorageBackend>::Error>
            {
                let cf = self.cf_handle($cf)?;

                Ok(self
                    .inner
                    .multi_get_cf(keys.iter().map(|k| (cf, k.pack_new())))
                    .into_iter()
                    .map(|r| {
                        r.map(|o| {
                            o.map(|v| {
                                // Unpacking from storage is fine.
                                <$value>::unpack_unchecked(&mut v.as_slice()).unwrap()
                            })
                        })
                        .map_err(|e| e.into())
                    })
                    .collect())
            }
        }
    };
}

impl_multi_fetch!(u8, System, CF_SYSTEM);
impl_multi_fetch!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
impl_multi_fetch!(MessageId, MessageMetadata, CF_MESSAGE_ID_TO_METADATA);
impl_multi_fetch!(OutputId, CreatedOutput, CF_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_multi_fetch!(OutputId, ConsumedOutput, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_multi_fetch!(MilestoneIndex, Milestone, CF_MILESTONE_INDEX_TO_MILESTONE);
impl_multi_fetch!(SolidEntryPoint, MilestoneIndex, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX);
impl_multi_fetch!(MilestoneIndex, OutputDiff, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_multi_fetch!(Address, Balance, CF_ADDRESS_TO_BALANCE);
