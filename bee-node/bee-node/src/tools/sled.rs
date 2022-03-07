// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_message::{
    address::Ed25519Address,
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    Message, MessageId,
};
use bee_storage::{
    access::{AsIterator, Exist, Fetch},
    backend::StorageBackend,
};
use bee_storage_sled::{
    config::SledConfigBuilder,
    storage::{Error as BackendError, Storage},
    trees::*,
};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

use structopt::StructOpt;
use thiserror::Error;

use std::str::FromStr;

#[derive(Clone, Debug, StructOpt)]
pub enum SledCommand {
    /// Fetches a value by its key.
    Fetch { key: String },
    /// Iterates a tree.
    Iterator,
}

#[derive(Debug, Error)]
pub enum SledError {
    #[error("Storage backend error: {0}")]
    StorageBackend(#[from] BackendError),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Unknown tree: {0}")]
    UnknownTree(String),
    #[error("Unsupported command")]
    UnsupportedCommand,
}

#[derive(Clone, Debug, StructOpt)]
pub struct SledTool {
    path: String,
    tree: String,
    #[structopt(subcommand)]
    command: SledCommand,
}

fn exec_inner(tool: &SledTool, storage: &Storage) -> Result<(), SledError> {
    match &tool.tree[..] {
        TREE_MESSAGE_ID_TO_MESSAGE => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, Message>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<MessageId, Message>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_MESSAGE_ID_TO_METADATA => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, MessageMetadata>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<MessageId, MessageMetadata>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_MESSAGE_ID_TO_MESSAGE_ID => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<(MessageId, MessageId), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_OUTPUT_ID_TO_CREATED_OUTPUT => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?;
                let value = Fetch::<OutputId, CreatedOutput>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<OutputId, CreatedOutput>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?;
                let value = Fetch::<OutputId, ConsumedOutput>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<OutputId, ConsumedOutput>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_OUTPUT_ID_UNSPENT => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = Unspent::from(OutputId::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?);
                let value = Exist::<Unspent, ()>::exist(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<Unspent, ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_ED25519_ADDRESS_TO_OUTPUT_ID => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = Ed25519Address::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?;
                let value = Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<(Ed25519Address, OutputId), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_LEDGER_INDEX => match &tool.command {
            SledCommand::Fetch { key: _key } => return Err(SledError::UnsupportedCommand),
            SledCommand::Iterator => {
                let iterator = AsIterator::<(), LedgerIndex>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_MILESTONE_INDEX_TO_MILESTONE => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, Milestone>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<MilestoneIndex, Milestone>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_SNAPSHOT_INFO => match &tool.command {
            SledCommand::Fetch { key: _key } => return Err(SledError::UnsupportedCommand),
            SledCommand::Iterator => {
                let iterator = AsIterator::<(), SnapshotInfo>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX => match &tool.command {
            SledCommand::Fetch { key } => {
                let key =
                    SolidEntryPoint::from(MessageId::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?);
                let value = Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<SolidEntryPoint, MilestoneIndex>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<MilestoneIndex, OutputDiff>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<(MilestoneIndex, UnreferencedMessage), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_MILESTONE_INDEX_TO_RECEIPT => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<(MilestoneIndex, Receipt), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        TREE_SPENT_TO_TREASURY_OUTPUT => match &tool.command {
            SledCommand::Fetch { key } => {
                let key = bool::from_str(key).map_err(|_| SledError::InvalidKey(key.clone()))?;
                let value = Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &key)?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            SledCommand::Iterator => {
                let iterator = AsIterator::<(bool, TreasuryOutput), ()>::iter(storage)?;

                for result in iterator {
                    let (key, value) = result?;
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },

        _ => return Err(SledError::UnknownTree(tool.tree[..].to_owned())),
    }

    Ok(())
}

pub fn exec(tool: &SledTool) -> Result<(), SledError> {
    let storage = Storage::start(SledConfigBuilder::default().with_path(tool.path.clone()).finish())?;
    let res = exec_inner(tool, &storage);

    storage.shutdown()?;

    res
}
