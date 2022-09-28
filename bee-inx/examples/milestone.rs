// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_inx::{client, Error};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let inx_connect_url = std::env::var("INX_CONNECT_URL").unwrap_or_else(|_| "http://localhost:9029".to_string());

    let mut inx = client::Inx::connect(&inx_connect_url).await?;
    println!("Connected via INX to node at {inx_connect_url}");

    let mut milestone_stream = inx.listen_to_confirmed_milestones((..).into()).await?;
    println!("Streaming confirmed milestones and protocol parameters... ");

    // Listen to the milestones from the node.
    while let Some(milestone_and_params) = milestone_stream.next().await {
        let milestone_and_params = milestone_and_params?;
        println!("{:?}", milestone_and_params);

        let milestone_index = milestone_and_params.milestone.milestone_info.milestone_index;
        println!("Fetching cone of milestone {milestone_index}");

        // Listen to blocks in the past cone of a milestone.
        let mut cone_stream = inx.read_milestone_cone(milestone_index.0.into()).await?;

        // Keep track of the number of blocks.
        let mut count = 0usize;

        while let Some(Ok(block_metadata)) = cone_stream.next().await {
            println!("Received block with id `{}`", block_metadata.metadata.block_id);
            count += 1;
        }

        println!("Milestone `{:?}` contained {count} blocks", milestone_index);
    }

    Ok(())
}
