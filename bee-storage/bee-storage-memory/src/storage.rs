// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! An in-memory storage backend.

use std::sync::{PoisonError, RwLock};

use bee_block::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_storage::{
    access::{Fetch, Insert},
    backend::StorageBackend,
    system::{StorageHealth, StorageVersion, System, SYSTEM_HEALTH_KEY},
};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use thiserror::Error;

use crate::table::{SingletonTable, Table, VecBinTable, VecTable};

/// Error to be raised when a backend operation fails.
#[derive(Debug, Error)]
pub enum Error {
    /// A thread panicked while a lock was being held.
    #[error("a lock is poisoned")]
    PoisonedLock,
    /// There is a storage version mismatch between the storage folder and this version of the storage.
    #[error("storage version mismatch, {0:?} != {1:?}, remove storage folder and restart")]
    VersionMismatch(StorageVersion, StorageVersion),
    /// The storage was not closed properly.
    #[error("unhealthy storage: {0:?}, remove storage folder and restart")]
    UnhealthyStorage(StorageHealth),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonedLock
    }
}

/// An in-memory database.
#[derive(Default)]
pub struct Storage {
    pub(crate) inner: RwLock<InnerStorage>,
}

#[derive(Default)]
pub(crate) struct InnerStorage {
    pub(crate) system: Table<u8, System>,
    pub(crate) block_id_to_block: Table<BlockId, Block>,
    pub(crate) block_id_to_metadata: Table<BlockId, BlockMetadata>,
    pub(crate) block_id_to_block_id: VecBinTable<BlockId, BlockId>,
    pub(crate) output_id_to_created_output: Table<OutputId, CreatedOutput>,
    pub(crate) output_id_to_consumed_output: Table<OutputId, ConsumedOutput>,
    pub(crate) output_id_unspent: Table<Unspent, ()>,
    pub(crate) ed25519_address_to_output_id: VecBinTable<Ed25519Address, OutputId>,
    pub(crate) ledger_index: SingletonTable<LedgerIndex>,
    pub(crate) milestone_index_to_milestone_metadata: Table<MilestoneIndex, MilestoneMetadata>,
    pub(crate) milestone_id_to_milestone_payload: Table<MilestoneId, MilestonePayload>,
    pub(crate) snapshot_info: SingletonTable<SnapshotInfo>,
    pub(crate) solid_entry_point_to_milestone_index: Table<SolidEntryPoint, MilestoneIndex>,
    pub(crate) milestone_index_to_output_diff: Table<MilestoneIndex, OutputDiff>,
    pub(crate) milestone_index_to_unreferenced_block: VecTable<MilestoneIndex, UnreferencedBlock>,
    pub(crate) milestone_index_to_receipt: VecTable<MilestoneIndex, Receipt>,
    pub(crate) spent_to_treasury_output: VecTable<bool, TreasuryOutput>,
}

impl Storage {
    /// Create a new database.
    pub fn new() -> Self {
        Default::default()
    }
}

impl StorageBackend for Storage {
    type ConfigBuilder = ();
    type Config = ();
    type Error = Error;

    fn start(_: Self::Config) -> Result<Self, Self::Error> {
        let storage = Self::new();

        storage.set_health(StorageHealth::Idle)?;

        Ok(storage)
    }

    fn shutdown(self) -> Result<(), Self::Error> {
        self.set_health(StorageHealth::Healthy)?;
        Ok(())
    }

    fn size(&self) -> Result<Option<usize>, Self::Error> {
        todo!()
    }

    fn get_health(&self) -> Result<Option<StorageHealth>, Self::Error> {
        Ok(match Fetch::<u8, System>::fetch(self, &SYSTEM_HEALTH_KEY)? {
            Some(System::Health(health)) => Some(health),
            None => None,
            _ => panic!("Another system value was inserted on the health key."),
        })
    }

    fn set_health(&self, health: StorageHealth) -> Result<(), Self::Error> {
        Insert::<u8, System>::insert(self, &SYSTEM_HEALTH_KEY, &System::Health(health))
    }
}
