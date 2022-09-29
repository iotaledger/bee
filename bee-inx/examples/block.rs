// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::BlockId;
use bee_inx::{client::Inx, Error};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let inx_connect_url = std::env::var("INX_CONNECT_URL").unwrap_or_else(|_| "http://localhost:9029".to_string());
    let block_stream = std::env::var("BLOCK_STREAM").unwrap_or_else(|_| "blocks".to_string());
    let read_block = std::env::var("READ_BLOCK")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap();
    let read_block_metadata = std::env::var("READ_BLOCK_METADATA")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap();

    let mut inx = Inx::connect(&inx_connect_url).await?;
    println!("Connected via INX to node at {inx_connect_url}");

    match block_stream.as_str() {
        "blocks" => {
            let mut block_stream = inx.listen_to_blocks().await?;
            println!("Streaming blocks ... ");

            while let Some(block) = block_stream.next().await {
                let block = block?;
                println!("{}", block.block_id);

                fetch_block_and_metadata(&mut inx, block.block_id, read_block, read_block_metadata).await?;
            }
        }
        "solid_blocks" => {
            let mut block_stream = inx.listen_to_solid_blocks().await?;
            println!("Streaming solid blocks ... ");

            while let Some(block_metadata) = block_stream.next().await {
                let block_metadata = block_metadata?;
                println!("{}", block_metadata.block_id);

                fetch_block_and_metadata(&mut inx, block_metadata.block_id, read_block, read_block_metadata).await?;
            }
        }
        "referenced_blocks" => {
            let mut block_stream = inx.listen_to_referenced_blocks().await?;
            println!("Streaming referenced blocks ... ");

            while let Some(block_metadata) = block_stream.next().await {
                let block_metadata = block_metadata?;
                println!("{}", block_metadata.block_id);

                fetch_block_and_metadata(&mut inx, block_metadata.block_id, read_block, read_block_metadata).await?;
            }
        }
        _ => {
            panic!("unknown block stream variant: '{block_stream}'");
        }
    }

    Ok(())
}

async fn fetch_block_and_metadata(
    inx: &mut Inx,
    block_id: BlockId,
    read_block: bool,
    read_block_metadata: bool,
) -> Result<(), Error> {
    if read_block {
        let raw_block = inx.read_block(block_id).await?;
        println!("{:?}", raw_block.inner_unverified());
    }

    if read_block_metadata {
        let block_metadata = inx.read_block_metadata(block_id).await?;
        println!("{:?}", block_metadata);
    }

    Ok(())
}
