// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    config::{RocksDBConfig, RocksDBConfigBuilder, StorageConfig},
    error::Error,
};

pub use bee_storage::backend::StorageBackend;

use bee_message::{
    payload::{indexation::HASHED_INDEX_LENGTH, transaction::ED25519_ADDRESS_LENGTH},
    MESSAGE_ID_LENGTH,
};

use async_trait::async_trait;
use rocksdb::{ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType, Options, SliceTransform, DB};

pub const CF_MESSAGE_ID_TO_MESSAGE: &str = "message_id_to_message";
pub const CF_MESSAGE_ID_TO_METADATA: &str = "message_id_to_metadata";
pub const CF_MESSAGE_ID_TO_MESSAGE_ID: &str = "message_id_to_message_id";
pub const CF_INDEX_TO_MESSAGE_ID: &str = "index_to_message_id";
pub const CF_OUTPUT_ID_TO_OUTPUT: &str = "output_id_to_output";
pub const CF_OUTPUT_ID_TO_SPENT: &str = "output_id_to_spent";
pub const CF_OUTPUT_ID_UNSPENT: &str = "output_id_unspent";
pub const CF_ED25519_ADDRESS_TO_OUTPUT_ID: &str = "ed25519_address_to_output_id";
pub const CF_LEDGER_INDEX: &str = "ledger_index";
pub const CF_MILESTONE_INDEX_TO_MILESTONE: &str = "milestone_index_to_milestone";
pub const CF_SNAPSHOT_INFO: &str = "snapshot_info";
pub const CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX: &str = "solid_entry_point_to_milestone_index";
pub const CF_MILESTONE_INDEX_TO_OUTPUT_DIFF: &str = "milestone_index_to_output_diff";
pub const CF_ADDRESS_TO_BALANCE: &str = "address_to_balance";

pub struct Storage {
    pub(crate) config: StorageConfig,
    pub(crate) inner: DB,
}

impl Storage {
    pub fn try_new(config: RocksDBConfig) -> Result<DB, Error> {
        let cf_message_id_to_message = ColumnFamilyDescriptor::new(CF_MESSAGE_ID_TO_MESSAGE, Options::default());

        let cf_message_id_to_metadata = ColumnFamilyDescriptor::new(CF_MESSAGE_ID_TO_METADATA, Options::default());

        let prefix_extractor = SliceTransform::create_fixed_prefix(MESSAGE_ID_LENGTH);
        let mut options = Options::default();
        options.set_prefix_extractor(prefix_extractor);
        let cf_message_id_to_message_id = ColumnFamilyDescriptor::new(CF_MESSAGE_ID_TO_MESSAGE_ID, options);

        let prefix_extractor = SliceTransform::create_fixed_prefix(HASHED_INDEX_LENGTH);
        let mut options = Options::default();
        options.set_prefix_extractor(prefix_extractor);
        let cf_index_to_message_id = ColumnFamilyDescriptor::new(CF_INDEX_TO_MESSAGE_ID, options);

        let cf_output_id_to_output = ColumnFamilyDescriptor::new(CF_OUTPUT_ID_TO_OUTPUT, Options::default());

        let cf_output_id_to_spent = ColumnFamilyDescriptor::new(CF_OUTPUT_ID_TO_SPENT, Options::default());

        let cf_output_id_unspent = ColumnFamilyDescriptor::new(CF_OUTPUT_ID_UNSPENT, Options::default());

        let prefix_extractor = SliceTransform::create_fixed_prefix(ED25519_ADDRESS_LENGTH);
        let mut options = Options::default();
        options.set_prefix_extractor(prefix_extractor);
        let cf_ed25519_address_to_output_id = ColumnFamilyDescriptor::new(CF_ED25519_ADDRESS_TO_OUTPUT_ID, options);

        let cf_ledger_index = ColumnFamilyDescriptor::new(CF_LEDGER_INDEX, Options::default());

        let cf_milestone_index_to_milestone =
            ColumnFamilyDescriptor::new(CF_MILESTONE_INDEX_TO_MILESTONE, Options::default());

        let cf_snapshot_info = ColumnFamilyDescriptor::new(CF_SNAPSHOT_INFO, Options::default());

        let cf_solid_entry_point_to_milestone_index =
            ColumnFamilyDescriptor::new(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX, Options::default());

        let cf_milestone_index_to_output_diff =
            ColumnFamilyDescriptor::new(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF, Options::default());

        let cf_address_to_balance = ColumnFamilyDescriptor::new(CF_ADDRESS_TO_BALANCE, Options::default());

        let mut opts = Options::default();

        opts.create_if_missing(config.create_if_missing);
        opts.create_missing_column_families(config.create_missing_column_families);
        if config.enable_statistics {
            opts.enable_statistics();
        }
        opts.increase_parallelism(config.increase_parallelism);
        opts.optimize_for_point_lookup(config.optimize_for_point_lookup);
        opts.optimize_level_style_compaction(config.optimize_level_style_compaction);
        opts.optimize_universal_style_compaction(config.optimize_universal_style_compaction);
        opts.set_advise_random_on_open(config.set_advise_random_on_open);
        opts.set_allow_concurrent_memtable_write(config.set_allow_concurrent_memtable_write);
        opts.set_allow_mmap_reads(config.set_allow_mmap_reads);
        opts.set_allow_mmap_writes(config.set_allow_mmap_writes);
        opts.set_atomic_flush(config.set_atomic_flush);
        opts.set_bytes_per_sync(config.set_bytes_per_sync);
        opts.set_compaction_readahead_size(config.set_compaction_readahead_size);
        opts.set_compaction_style(DBCompactionStyle::from(config.set_compaction_style));
        opts.set_max_write_buffer_number(config.set_max_write_buffer_number);
        opts.set_disable_auto_compactions(config.set_disable_auto_compactions);
        opts.set_compression_type(DBCompressionType::from(config.set_compression_type));

        let column_familes = vec![
            cf_message_id_to_message,
            cf_message_id_to_metadata,
            cf_message_id_to_message_id,
            cf_index_to_message_id,
            cf_output_id_to_output,
            cf_output_id_to_spent,
            cf_output_id_unspent,
            cf_ed25519_address_to_output_id,
            cf_ledger_index,
            cf_milestone_index_to_milestone,
            cf_snapshot_info,
            cf_solid_entry_point_to_milestone_index,
            cf_milestone_index_to_output_diff,
            cf_address_to_balance,
        ];

        Ok(DB::open_cf_descriptors(&opts, config.path, column_familes)?)
    }
}

#[async_trait]
impl StorageBackend for Storage {
    type ConfigBuilder = RocksDBConfigBuilder;
    type Config = RocksDBConfig;
    type Error = Error;

    /// It starts RocksDB instance and then initializes the required column familes.
    async fn start(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Storage {
            config: config.storage.clone(),
            inner: Self::try_new(config)?,
        })
    }

    /// It shutdown RocksDB instance.
    /// Note: the shutdown is done through flush method and then droping the storage object.
    async fn shutdown(self) -> Result<(), Self::Error> {
        Ok(self.inner.flush()?)
    }

    async fn size(&self) -> Result<Option<usize>, Self::Error> {
        let mut size = 0;
        let files = self.inner.live_files()?;

        for file in files {
            size += file.size
        }

        Ok(Some(size))
    }
}
