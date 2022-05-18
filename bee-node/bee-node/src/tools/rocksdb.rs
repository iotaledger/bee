// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use bee_ledger::types::{
    snapshot::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
    Unspent,
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::{IndexationPayload, PaddedIndex},
    Message, MessageId,
};
use bee_storage::{
    access::{AsIterator},
    backend::{StorageBackend, StorageBackendExt},
};
use bee_storage_rocksdb::{
    column_families::*,
    config::RocksDbConfigBuilder,
    error::Error as BackendError,
    storage::{Storage, System},
};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Clone, Debug, StructOpt)]
pub enum RocksdbCommand {
    /// Fetches a value by its key.
    Fetch { key: String },
    /// Iterates a column family.
    Iterator,
}

#[derive(Debug, Error)]
pub enum RocksdbError {
    #[error("Storage backend error: {0}")]
    StorageBackend(#[from] BackendError),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Unknown column family: {0}")]
    UnknownColumnFamily(String),
    #[error("Unsupported command")]
    UnsupportedCommand,
}

#[derive(Clone, Debug, StructOpt)]
pub struct RocksdbTool {
    path: String,
    column_family: String,
    #[structopt(subcommand)]
    command: RocksdbCommand,
}

fn exec_inner(tool: &RocksdbTool, storage: &Storage) -> Result<(), RocksdbError> {
    match &tool.column_family[..] {
        CF_SYSTEM => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = u8::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<u8, System>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<u8, System>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_MESSAGE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<MessageId, Message>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MessageId, Message>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_METADATA => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<MessageId, MessageMetadata>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MessageId, MessageMetadata>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_MESSAGE_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<MessageId, Vec<MessageId>>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(MessageId, MessageId), ()>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_INDEX_TO_MESSAGE_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = IndexationPayload::new(
                    &hex::decode(key.clone()).map_err(|_| RocksdbError::InvalidKey(key.clone()))?,
                    &[],
                )
                .map_err(|_| RocksdbError::InvalidKey(key.clone()))?
                .padded_index();
                let value = storage.fetch::<PaddedIndex, Vec<MessageId>>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(PaddedIndex, MessageId), ()>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_TO_CREATED_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<OutputId, CreatedOutput>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<OutputId, CreatedOutput>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_TO_CONSUMED_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<OutputId, ConsumedOutput>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<OutputId, ConsumedOutput>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_UNSPENT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = Unspent::from(OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = storage.exist::<Unspent, ()>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<Unspent, ()>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_ED25519_ADDRESS_TO_OUTPUT_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = Ed25519Address::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<Ed25519Address, Vec<OutputId>>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(Ed25519Address, OutputId), ()>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_LEDGER_INDEX => match &tool.command {
            RocksdbCommand::Fetch { key: _key } => return Err(RocksdbError::UnsupportedCommand),
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(), LedgerIndex>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_MILESTONE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = storage.fetch::<MilestoneIndex, Milestone>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MilestoneIndex, Milestone>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_SNAPSHOT_INFO => match &tool.command {
            RocksdbCommand::Fetch { key: _key } => return Err(RocksdbError::UnsupportedCommand),
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(), SnapshotInfo>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key =
                    SolidEntryPoint::from(MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = storage.fetch::<SolidEntryPoint, MilestoneIndex>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<SolidEntryPoint, MilestoneIndex>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_OUTPUT_DIFF => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = storage.fetch::<MilestoneIndex, OutputDiff>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MilestoneIndex, OutputDiff>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_ADDRESS_TO_BALANCE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key =
                    Address::from(Ed25519Address::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = storage.fetch::<Address, Balance>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<Address, Balance>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = storage.fetch::<MilestoneIndex, Vec<UnreferencedMessage>>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(MilestoneIndex, UnreferencedMessage), ()>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_RECEIPT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = storage.fetch::<MilestoneIndex, Vec<Receipt>>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(MilestoneIndex, Receipt), ()>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_SPENT_TO_TREASURY_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = bool::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = storage.fetch::<bool, Vec<TreasuryOutput>>(&key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(bool, TreasuryOutput), ()>::iter_op(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },

        _ => return Err(RocksdbError::UnknownColumnFamily(tool.column_family[..].to_owned())),
    }

    Ok(())
}

pub fn exec(tool: &RocksdbTool) -> Result<(), RocksdbError> {
    let storage = Storage::start(RocksDbConfigBuilder::default().with_path(tool.path.clone()).finish())?;
    let res = exec_inner(tool, &storage);

    storage.shutdown()?;

    res
}
