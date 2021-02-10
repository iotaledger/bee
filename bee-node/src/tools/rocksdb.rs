// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::{
    balance::Balance,
    model::{OutputDiff, Unspent},
};
use bee_message::{
    ledger_index::LedgerIndex,
    milestone::{Milestone, MilestoneIndex},
    payload::{
        indexation::{HashedIndex, IndexationPayload},
        transaction::{Address, ConsumedOutput, CreatedOutput, Ed25519Address, OutputId},
    },
    solid_entry_point::SolidEntryPoint,
    Message, MessageId,
};
use bee_snapshot::SnapshotInfo;
use bee_storage::{
    access::{AsStream, Exist, Fetch},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, error::Error as BackendError, storage::*, system::System};
use bee_tangle::metadata::MessageMetadata;

use futures::{executor, stream::StreamExt};
use structopt::StructOpt;
use thiserror::Error;

use std::str::FromStr;

#[derive(Debug, StructOpt)]
pub enum RocksdbCommand {
    /// Fetches a value by its key.
    Fetch { key: String },
    /// Streams a column family.
    Stream,
}

#[derive(Debug, Error)]
pub enum RocksdbError {
    #[error("{0}")]
    StorageBackend(#[from] BackendError),
    #[error("{0}")]
    InvalidKey(String),
    #[error("{0}")]
    UnknownColumnFamily(String),
    #[error("Unsupported command")]
    UnsupportedCommand,
}

#[derive(Debug, StructOpt)]
pub struct RocksdbTool {
    path: String,
    column_family: String,
    #[structopt(subcommand)]
    command: RocksdbCommand,
}

async fn exec_inner(tool: &RocksdbTool) -> Result<(), RocksdbError> {
    let config = RocksDBConfigBuilder::default().with_path(tool.path.clone()).finish();
    let storage = Storage::start(config).await?;

    match &tool.column_family[..] {
        CF_SYSTEM => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = u8::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<u8, System>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<u8, System>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_MESSAGE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, Message>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<MessageId, Message>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_METADATA => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, MessageMetadata>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<MessageId, MessageMetadata>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MESSAGE_ID_TO_MESSAGE_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<MessageId, Vec<MessageId>>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<(MessageId, MessageId), ()>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_INDEX_TO_MESSAGE_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = IndexationPayload::new(key.clone(), &[])
                    .map_err(|_| RocksdbError::InvalidKey(key.clone()))?
                    .hash();
                let value = Fetch::<HashedIndex, Vec<MessageId>>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<(HashedIndex, MessageId), ()>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_TO_CREATED_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<OutputId, CreatedOutput>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<OutputId, CreatedOutput>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_TO_CONSUMED_OUTPUT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<OutputId, ConsumedOutput>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<OutputId, ConsumedOutput>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_OUTPUT_ID_UNSPENT => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = Unspent::from(OutputId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Exist::<Unspent, ()>::exist(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<Unspent, ()>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_ED25519_ADDRESS_TO_OUTPUT_ID => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = Ed25519Address::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?;
                let value = Fetch::<Ed25519Address, Vec<OutputId>>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<(Ed25519Address, OutputId), ()>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_LEDGER_INDEX => match &tool.command {
            RocksdbCommand::Fetch { key: _key } => return Err(RocksdbError::UnsupportedCommand),
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<(), LedgerIndex>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_MILESTONE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, Milestone>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<MilestoneIndex, Milestone>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_SNAPSHOT_INFO => match &tool.command {
            RocksdbCommand::Fetch { key: _key } => return Err(RocksdbError::UnsupportedCommand),
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<(), SnapshotInfo>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key =
                    SolidEntryPoint::from(MessageId::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_MILESTONE_INDEX_TO_OUTPUT_DIFF => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key = MilestoneIndex(u32::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<MilestoneIndex, OutputDiff>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<MilestoneIndex, OutputDiff>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },
        CF_ADDRESS_TO_BALANCE => match &tool.command {
            RocksdbCommand::Fetch { key } => {
                let key =
                    Address::from(Ed25519Address::from_str(key).map_err(|_| RocksdbError::InvalidKey(key.clone()))?);
                let value = Fetch::<Address, Balance>::fetch(&storage, &key).await?;

                println!("Key: {:?}\nValue: {:?}\n", key, value);
            }
            RocksdbCommand::Stream => {
                let mut stream = AsStream::<Address, Balance>::stream(&storage).await?;

                while let Some((key, value)) = stream.next().await {
                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
            }
        },

        _ => return Err(RocksdbError::UnknownColumnFamily(tool.column_family[..].to_owned())),
    }

    Ok(())
}

pub fn exec(tool: &RocksdbTool) -> Result<(), RocksdbError> {
    executor::block_on(exec_inner(tool))
}
