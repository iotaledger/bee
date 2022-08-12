// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::BlockId;
use bee_storage::access::Update;
use bee_tangle::block_metadata::BlockMetadata;
use packable::PackableExt;

use crate::{column_families::*, storage::Storage};

impl Update<BlockId, BlockMetadata> for Storage {
    fn update(&self, block_id: &BlockId, mut f: impl FnMut(&mut BlockMetadata)) -> Result<(), Self::Error> {
        let cf_handle = self.cf_handle(CF_BLOCK_ID_TO_METADATA)?;

        let guard = self.locks.block_id_to_metadata.write();

        if let Some(v) = self.inner.get_pinned_cf(cf_handle, block_id)? {
            // Unpacking from storage is fine.
            let mut metadata = BlockMetadata::unpack_unverified(&mut &*v, &()).unwrap();

            f(&mut metadata);

            self.inner.put_cf(cf_handle, block_id, metadata.pack_to_vec())?;
        }

        drop(guard);

        Ok(())
    }
}
