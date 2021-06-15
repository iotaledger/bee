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

#[allow(clippy::type_complexity)]
impl<'a> MultiFetch<'a, u8, System> for Storage {
    fn multi_fetch(
        &'a self,
        keys: &'a [u8],
    ) -> Result<
        Box<dyn Iterator<Item = Result<Option<System>, <Self as StorageBackend>::Error>> + 'a>,
        <Self as StorageBackend>::Error,
    > {
        Ok(Box::new(keys.iter().map(move |k| self.inner.get(k.pack_new())).map(
            |r| {
                r.map(|o| o.map(|v| System::unpack_unchecked(&mut v.as_ref()).unwrap()))
                    .map_err(|e| e.into())
            },
        )))
    }
}

macro_rules! impl_multi_fetch {
    ($key:ty, $value:ty, $cf:expr) => {
        impl<'a> MultiFetch<'a, $key, $value> for Storage {
            fn multi_fetch(
                &'a self,
                keys: &'a [$key],
            ) -> Result<
                Box<dyn Iterator<Item = Result<Option<$value>, <Self as StorageBackend>::Error>> + 'a>,
                <Self as StorageBackend>::Error,
            > {
                let tree = self.inner.open_tree($cf)?;

                Ok(Box::new(keys.iter().map(move |k| tree.get(k.pack_new())).map(
                    |r| {
                        r.map(|o| o.map(|v| <$value>::unpack_unchecked(&mut v.as_ref()).unwrap()))
                            .map_err(|e| e.into())
                    },
                )))
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
