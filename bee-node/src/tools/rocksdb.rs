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
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::*, system::System};
use bee_tangle::metadata::MessageMetadata;

use futures::{executor, stream::StreamExt};
use structopt::StructOpt;

use std::str::FromStr;

// TODO handle errors/unwraps/panics

#[derive(Debug, StructOpt)]
pub enum RocksdbCommand {
    /// Fetches a value by its key.
    Fetch { key: String },
    /// Streams a column family.
    Stream,
}

#[derive(Debug, StructOpt)]
pub struct Rocksdb {
    path: String,
    column_family: String,
    #[structopt(subcommand)]
    command: RocksdbCommand,
}

pub fn exec(tool: &Rocksdb) {
    executor::block_on(async {
        let config = RocksDBConfigBuilder::default().with_path(tool.path.clone()).finish();
        let storage = Storage::start(config).await.unwrap();

        match &tool.column_family[..] {
            CF_MESSAGE_ID_TO_MESSAGE => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = MessageId::from_str(key).unwrap();
                    let value = Fetch::<MessageId, Message>::fetch(&storage, &key).await.unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MessageId, Message>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_MESSAGE_ID_TO_METADATA => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = MessageId::from_str(key).unwrap();
                    let value = Fetch::<MessageId, MessageMetadata>::fetch(&storage, &key)
                        .await
                        .unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MessageId, MessageMetadata>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_MESSAGE_ID_TO_MESSAGE_ID => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = MessageId::from_str(key).unwrap();
                    let value = Fetch::<MessageId, Vec<MessageId>>::fetch(&storage, &key).await.unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(MessageId, MessageId), ()>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_INDEX_TO_MESSAGE_ID => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = IndexationPayload::new(key.clone(), &[]).unwrap().hash();
                    let value = Fetch::<HashedIndex, Vec<MessageId>>::fetch(&storage, &key)
                        .await
                        .unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(HashedIndex, MessageId), ()>::stream(&storage)
                        .await
                        .unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_OUTPUT_ID_TO_CREATED_OUTPUT => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = OutputId::from_str(key).unwrap();
                    let value = Fetch::<OutputId, CreatedOutput>::fetch(&storage, &key).await.unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<OutputId, CreatedOutput>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_OUTPUT_ID_TO_CONSUMED_OUTPUT => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = OutputId::from_str(key).unwrap();
                    let value = Fetch::<OutputId, ConsumedOutput>::fetch(&storage, &key).await.unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<OutputId, ConsumedOutput>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_OUTPUT_ID_UNSPENT => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = Unspent::from(OutputId::from_str(key).unwrap());
                    let value = Exist::<Unspent, ()>::exist(&storage, &key).await.unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<Unspent, ()>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_ED25519_ADDRESS_TO_OUTPUT_ID => match &tool.command {
                RocksdbCommand::Fetch { key } => {
                    let key = Ed25519Address::from_str(key).unwrap();
                    let value = Fetch::<Ed25519Address, Vec<OutputId>>::fetch(&storage, &key)
                        .await
                        .unwrap();

                    println!("Key: {:?}\nValue: {:?}\n", key, value);
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(Ed25519Address, OutputId), ()>::stream(&storage)
                        .await
                        .unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_LEDGER_INDEX => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {
                    panic!("Unhandled command");
                }
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(), LedgerIndex>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_MILESTONE_INDEX_TO_MILESTONE => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MilestoneIndex, Milestone>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_SNAPSHOT_INFO => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(), SnapshotInfo>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(&storage)
                        .await
                        .unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_MILESTONE_INDEX_TO_OUTPUT_DIFF => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MilestoneIndex, OutputDiff>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_ADDRESS_TO_BALANCE => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<Address, Balance>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            CF_SYSTEM => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<u8, System>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?}\n", key, value);
                    }
                }
            },
            _ => {
                println!("Unknown column family.");
            }
        }
    });
}
