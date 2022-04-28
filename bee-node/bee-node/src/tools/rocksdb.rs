// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_message::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Message, MessageId,
};
use bee_storage::{
    access::{AsIterator, Exist, Fetch},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{
    column_families::*,
    config::RocksDbConfigBuilder,
    error::Error as BackendError,
    storage::{Storage, System},
};
use bee_tangle::{
    message_metadata::MessageMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_message::UnreferencedMessage,
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
                let value = Fetch::<u8, System>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<u8, System>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_MESSAGE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, Message>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MessageId, Message>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_METADATA => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, MessageMetadata>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MessageId, MessageMetadata>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_MESSAGE_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(MessageId, MessageId), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_TO_CREATED_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<OutputId, CreatedOutput>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<OutputId, CreatedOutput>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_TO_CONSUMED_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<OutputId, ConsumedOutput>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<OutputId, ConsumedOutput>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_UNSPENT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = Unspent::from(OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Exist::<Unspent, ()>::exist(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<Unspent, ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_ED25519_ADDRESS_TO_OUTPUT_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = Ed25519Address::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(Ed25519Address, OutputId), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_LEDGER_INDEX => match &tool.command {
            RocksdbCommand::Fetch { key: _key } => return Err(RocksdbError::UnsupportedCommand),
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(), LedgerIndex>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_MILESTONE_METADATA => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<MilestoneId, MilestonePayload>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MilestoneId, MilestonePayload>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, MilestoneMetadata>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MilestoneIndex, MilestoneMetadata>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_SNAPSHOT_INFO => match &tool.command {
            RocksdbCommand::Fetch { key: _key } => return Err(RocksdbError::UnsupportedCommand),
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(), SnapshotInfo>::iter(storage)?;

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
                let value = Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<SolidEntryPoint, MilestoneIndex>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_OUTPUT_DIFF => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<MilestoneIndex, OutputDiff>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(MilestoneIndex, UnreferencedMessage), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_RECEIPT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(MilestoneIndex, Receipt), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_SPENT_TO_TREASURY_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = bool::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Iterator => {
                let iterator = AsIterator::<(bool, TreasuryOutput), ()>::iter(storage)?;

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
