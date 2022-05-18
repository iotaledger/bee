// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::ED25519_ADDRESS_LENGTH, milestone::MilestoneIndex, payload::indexation::INDEXATION_PADDED_INDEX_LENGTH,
    MESSAGE_ID_LENGTH,
};
pub use bee_storage::{
    access::Insert,
    backend::{StorageBackend, StorageBackendExt},
    system::{StorageHealth, StorageVersion, System, SYSTEM_HEALTH_KEY, SYSTEM_VERSION_KEY},
};
use parking_lot::RwLock;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType, Env, FlushOptions, MergeOperands,
    Options, SliceTransform, DB,
};

use super::{
    column_families::*,
    config::{RocksDbConfig, RocksDbConfigBuilder, StorageConfig},
    error::Error,
};

pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion(9);

pub struct Locks {
    pub(crate) message_id_to_metadata: RwLock<()>,
}

pub struct Storage {
    pub(crate) config: StorageConfig,
    pub(crate) inner: DB,
    pub(crate) locks: Locks,
}

impl Storage {
    fn new(config: RocksDbConfig) -> Result<Self, Error> {
        let cf_system = ColumnFamilyDescriptor::new(CF_SYSTEM, Options::default());

        let cf_message_id_to_message = ColumnFamilyDescriptor::new(CF_MESSAGE_ID_TO_MESSAGE, Options::default());

        fn keep_current(_key: &[u8], existing_val: Option<&[u8]>, operands: &MergeOperands) -> Option<Vec<u8>> {
            // Keep the existing value, if the value does not exist, take the first operand
            // instead.
            existing_val.or_else(|| operands.into_iter().next()).map(|v| v.to_vec())
        }
        let mut options = Options::default();
        options.set_merge_operator_associative("keep current", keep_current);
        let cf_message_id_to_metadata = ColumnFamilyDescriptor::new(CF_MESSAGE_ID_TO_METADATA, options);

        let mut options = Options::default();
        options.set_prefix_extractor(SliceTransform::create_fixed_prefix(MESSAGE_ID_LENGTH));
        let cf_message_id_to_message_id = ColumnFamilyDescriptor::new(CF_MESSAGE_ID_TO_MESSAGE_ID, options);

        let mut options = Options::default();
        options.set_prefix_extractor(SliceTransform::create_fixed_prefix(INDEXATION_PADDED_INDEX_LENGTH));
        let cf_index_to_message_id = ColumnFamilyDescriptor::new(CF_INDEX_TO_MESSAGE_ID, options);

        let cf_output_id_to_created_output =
            ColumnFamilyDescriptor::new(CF_OUTPUT_ID_TO_CREATED_OUTPUT, Options::default());

        let cf_output_id_to_consumed_output =
            ColumnFamilyDescriptor::new(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT, Options::default());

        let cf_output_id_unspent = ColumnFamilyDescriptor::new(CF_OUTPUT_ID_UNSPENT, Options::default());

        let mut options = Options::default();
        options.set_prefix_extractor(SliceTransform::create_fixed_prefix(ED25519_ADDRESS_LENGTH));
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

        let mut options = Options::default();
        options.set_prefix_extractor(SliceTransform::create_fixed_prefix(
            std::mem::size_of::<MilestoneIndex>(),
        ));
        let cf_milestone_index_to_unreferenced_message =
            ColumnFamilyDescriptor::new(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE, options);

        let mut options = Options::default();
        options.set_prefix_extractor(SliceTransform::create_fixed_prefix(
            std::mem::size_of::<MilestoneIndex>(),
        ));
        let cf_milestone_index_to_receipt = ColumnFamilyDescriptor::new(CF_MILESTONE_INDEX_TO_RECEIPT, options);

        let mut options = Options::default();
        options.set_prefix_extractor(SliceTransform::create_fixed_prefix(std::mem::size_of::<bool>()));
        let cf_spent_to_treasury = ColumnFamilyDescriptor::new(CF_SPENT_TO_TREASURY_OUTPUT, options);

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
        opts.set_write_buffer_size(config.set_write_buffer_size);
        opts.set_db_write_buffer_size(config.set_db_write_buffer_size);
        opts.set_disable_auto_compactions(config.set_disable_auto_compactions);
        opts.set_compression_type(DBCompressionType::from(config.set_compression_type));
        opts.set_unordered_write(config.set_unordered_write);
        opts.set_use_direct_io_for_flush_and_compaction(config.set_use_direct_io_for_flush_and_compaction);

        let mut env = Env::default()?;
        env.set_background_threads(config.env.set_background_threads);
        env.set_high_priority_background_threads(config.env.set_high_priority_background_threads);
        opts.set_env(&env);

        let db = DB::open_cf_descriptors(
            &opts,
            config.path,
            vec![
                cf_system,
                cf_message_id_to_message,
                cf_message_id_to_metadata,
                cf_message_id_to_message_id,
                cf_index_to_message_id,
                cf_output_id_to_created_output,
                cf_output_id_to_consumed_output,
                cf_output_id_unspent,
                cf_ed25519_address_to_output_id,
                cf_ledger_index,
                cf_milestone_index_to_milestone,
                cf_snapshot_info,
                cf_solid_entry_point_to_milestone_index,
                cf_milestone_index_to_output_diff,
                cf_address_to_balance,
                cf_milestone_index_to_unreferenced_message,
                cf_milestone_index_to_receipt,
                cf_spent_to_treasury,
            ],
        )?;

        let mut flushopts = FlushOptions::new();
        flushopts.set_wait(true);
        db.flush_opt(&flushopts)?;
        db.flush_cf_opt(db.cf_handle(CF_SYSTEM).unwrap(), &flushopts)?;

        Ok(Storage {
            config: config.storage,
            inner: db,
            locks: Locks {
                message_id_to_metadata: RwLock::new(()),
            },
        })
    }

    pub(crate) fn cf_handle(&self, cf_str: &'static str) -> Result<&ColumnFamily, Error> {
        self.inner.cf_handle(cf_str).ok_or(Error::UnknownColumnFamily(cf_str))
    }
}

impl StorageBackend for Storage {
    type ConfigBuilder = RocksDbConfigBuilder;
    type Config = RocksDbConfig;
    type Error = Error;

    fn start(config: Self::Config) -> Result<Self, Self::Error> {
        let storage = Self::new(config)?;

        match storage.fetch::<u8, System>(&SYSTEM_VERSION_KEY)? {
            Some(System::Version(version)) => {
                if version != STORAGE_VERSION {
                    return Err(Error::VersionMismatch(version, STORAGE_VERSION));
                }
            }
            None => Insert::<u8, System>::insert_op(&storage, &SYSTEM_VERSION_KEY, &System::Version(STORAGE_VERSION))?,
            _ => panic!("Another system value was inserted on the version key."),
        }

        if let Some(health) = storage.get_health()? {
            if health != StorageHealth::Healthy {
                return Err(Self::Error::UnhealthyStorage(health));
            }
        }

        storage.set_health(StorageHealth::Idle)?;

        Ok(storage)
    }

    fn shutdown(self) -> Result<(), Self::Error> {
        self.set_health(StorageHealth::Healthy)?;

        Ok(self.inner.flush()?)
    }

    fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(Some(
            self.inner.live_files()?.iter().fold(0, |acc, file| acc + file.size),
        ))
    }

    fn get_health(&self) -> Result<Option<StorageHealth>, Self::Error> {
        Ok(match self.fetch::<u8, System>(&SYSTEM_HEALTH_KEY)? {
            Some(System::Health(health)) => Some(health),
            None => None,
            _ => panic!("Another system value was inserted on the health key."),
        })
    }

    fn set_health(&self, health: StorageHealth) -> Result<(), Self::Error> {
        Insert::<u8, System>::insert_op(self, &SYSTEM_HEALTH_KEY, &System::Health(health))
    }
}
