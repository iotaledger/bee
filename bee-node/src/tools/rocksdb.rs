// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::model::{Diff, LedgerIndex, Output, Spent, Unspent};
use bee_message::{
    payload::{
        indexation::HashedIndex,
        transaction::{Ed25519Address, OutputId},
    },
    Message, MessageId,
};
use bee_snapshot::SnapshotInfo;
use bee_storage::{access::AsStream, backend::StorageBackend};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::*};
use bee_tangle::{
    metadata::MessageMetadata,
    milestone::{Milestone, MilestoneIndex},
    solid_entry_point::SolidEntryPoint,
};

use futures::{executor, stream::StreamExt};
use structopt::StructOpt;

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
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MessageId, Message>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_MESSAGE_ID_TO_METADATA => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MessageId, MessageMetadata>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_MESSAGE_ID_TO_MESSAGE_ID => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(MessageId, MessageId), ()>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_INDEX_TO_MESSAGE_ID => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(HashedIndex, MessageId), ()>::stream(&storage)
                        .await
                        .unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_OUTPUT_ID_TO_OUTPUT => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<OutputId, Output>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_OUTPUT_ID_TO_SPENT => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<OutputId, Spent>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_OUTPUT_ID_UNSPENT => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<Unspent, ()>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_ED25519_ADDRESS_TO_OUTPUT_ID => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(Ed25519Address, OutputId), ()>::stream(&storage)
                        .await
                        .unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_LEDGER_INDEX => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(), LedgerIndex>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_MILESTONE_INDEX_TO_MILESTONE => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MilestoneIndex, Milestone>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_SNAPSHOT_INFO => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<(), SnapshotInfo>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
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
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            CF_MILESTONE_INDEX_TO_DIFF => match &tool.command {
                RocksdbCommand::Fetch { key: _key } => {}
                RocksdbCommand::Stream => {
                    let mut stream = AsStream::<MilestoneIndex, Diff>::stream(&storage).await.unwrap();

                    while let Some((key, value)) = stream.next().await {
                        println!("Key: {:?}\nValue: {:?})\n", key, value);
                    }
                }
            },
            _ => {
                println!("Unknown column family.");
            }
        }
    });
}
