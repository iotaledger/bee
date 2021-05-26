// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{StorageConfig, StorageConfigBuilder},
    trees::*,
};

use bee_storage::{backend::StorageBackend, health::StorageHealth};

use async_trait::async_trait;
use sled;

type Error = sled::Error;

pub struct Storage {
    pub(crate) inner: sled::Db,
    pub(crate) config: StorageConfig,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self, Error> {
        let inner = sled::open("./storage/mainnet")?;

        inner.open_tree(TREE_SYSTEM)?;
        inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?;
        inner.open_tree(TREE_MESSAGE_ID_TO_METADATA)?;
        inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE_ID)?;
        inner.open_tree(TREE_INDEX_TO_MESSAGE_ID)?;
        inner.open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?;
        inner.open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?;
        inner.open_tree(TREE_OUTPUT_ID_UNSPENT)?;
        inner.open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?;
        inner.open_tree(TREE_LEDGER_INDEX)?;
        inner.open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE)?;
        inner.open_tree(TREE_SNAPSHOT_INFO)?;
        inner.open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?;
        inner.open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?;
        inner.open_tree(TREE_ADDRESS_TO_BALANCE)?;
        inner.open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?;
        inner.open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?;
        inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?;

        Ok(Self { inner, config })
    }
}

#[async_trait]
impl StorageBackend for Storage {
    type ConfigBuilder = StorageConfigBuilder;
    type Config = StorageConfig;
    type Error = Error;

    async fn start(config: Self::Config) -> Result<Self, Self::Error> {
        Self::new(config)
    }

    async fn shutdown(self) -> Result<(), Self::Error> {
        self.inner.flush()?;
        Ok(())
    }

    async fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(Some(self.inner.size_on_disk()? as usize))
    }

    async fn get_health(&self) -> Result<Option<StorageHealth>, Self::Error> {
        Ok(None)
    }

    async fn set_health(&self, health: StorageHealth) -> Result<(), Self::Error> {
        Ok(())
    }
}
