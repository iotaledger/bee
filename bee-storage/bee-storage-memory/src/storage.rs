// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! An in-memory storage backend.

use crate::table::{SingletonTable, Table, VecBinTable, VecTable};

use bee_ledger::types::{
    snapshot::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
    Unspent,
};
use bee_message::{
    address::{Address, AliasAddress, Ed25519Address, NftAddress},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::{
    access::{Fetch, Insert},
    backend::StorageBackend,
    system::{StorageHealth, StorageVersion, System, SYSTEM_HEALTH_KEY, SYSTEM_VERSION_KEY},
};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

use thiserror::Error;

use std::sync::{PoisonError, RwLock};

/// Error to be raised when a backend operation fails.
#[derive(Debug, Error)]
pub enum Error {
    /// A thread panicked while a lock was being held.
    #[error("A lock is poisoned")]
    PoisonedLock,
    /// There is a storage version mismatch between the storage folder and this version of the storage.
    #[error("Storage version mismatch, {0:?} != {1:?}, remove storage folder and restart")]
    VersionMismatch(StorageVersion, StorageVersion),
    /// The storage was not closed properly.
    #[error("Unhealthy storage: {0:?}, remove storage folder and restart")]
    UnhealthyStorage(StorageHealth),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonedLock
    }
}

pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion(0);

/// An in-memory database.
#[derive(Default)]
pub struct Storage {
    pub(crate) inner: RwLock<InnerStorage>,
}

#[derive(Default)]
pub(crate) struct InnerStorage {
    pub(crate) system: Table<u8, System>,
    pub(crate) message_id_to_message: Table<MessageId, Message>,
    pub(crate) message_id_to_metadata: Table<MessageId, MessageMetadata>,
    pub(crate) message_id_to_message_id: VecBinTable<MessageId, MessageId>,
    pub(crate) index_to_message_id: VecBinTable<PaddedIndex, MessageId>,
    pub(crate) output_id_to_created_output: Table<OutputId, CreatedOutput>,
    pub(crate) output_id_to_consumed_output: Table<OutputId, ConsumedOutput>,
    pub(crate) output_id_unspent: Table<Unspent, ()>,
    pub(crate) ed25519_address_to_output_id: VecBinTable<Ed25519Address, OutputId>,
    pub(crate) alias_address_to_output_id: VecBinTable<AliasAddress, OutputId>,
    pub(crate) nft_address_to_output_id: VecBinTable<NftAddress, OutputId>,
    pub(crate) ledger_index: SingletonTable<LedgerIndex>,
    pub(crate) milestone_index_to_milestone: Table<MilestoneIndex, Milestone>,
    pub(crate) snapshot_info: SingletonTable<SnapshotInfo>,
    pub(crate) solid_entry_point_to_milestone_index: Table<SolidEntryPoint, MilestoneIndex>,
    pub(crate) milestone_index_to_output_diff: Table<MilestoneIndex, OutputDiff>,
    pub(crate) address_to_balance: Table<Address, Balance>,
    pub(crate) milestone_index_to_unreferenced_message: VecTable<MilestoneIndex, UnreferencedMessage>,
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

        match Fetch::<u8, System>::fetch(&storage, &SYSTEM_VERSION_KEY)? {
            Some(System::Version(version)) => {
                if version != STORAGE_VERSION {
                    return Err(Error::VersionMismatch(version, STORAGE_VERSION));
                }
            }
            None => Insert::<u8, System>::insert(&storage, &SYSTEM_VERSION_KEY, &System::Version(STORAGE_VERSION))?,
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
