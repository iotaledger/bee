// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use bee_block::BlockId;
use bee_storage::access::Update;
use bee_tangle::block_metadata::BlockMetadata;
use packable::PackableExt;

use crate::{storage::Storage, trees::*};

impl Update<BlockId, BlockMetadata> for Storage {
    fn update(&self, block_id: &BlockId, mut f: impl FnMut(&mut BlockMetadata)) -> Result<(), Self::Error> {
        self.inner
            .open_tree(TREE_BLOCK_ID_TO_METADATA)?
            .fetch_and_update(block_id, move |opt_bytes| {
                opt_bytes.map(|mut bytes| {
                    // Unpacking from storage is fine.
                    let mut metadata = BlockMetadata::unpack_unverified(&mut bytes, &mut ()).unwrap();
                    f(&mut metadata);
                    metadata.pack_to_vec()
                })
            })?;

        Ok(())
    }
}
