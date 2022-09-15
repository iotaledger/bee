// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_inx::{client, Error};
use futures::StreamExt;

const INX_ADDRESS: &str = "http://localhost:9029";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut inx = client::Inx::connect(INX_ADDRESS.into()).await?;
    let mut milestone_stream = inx.listen_to_confirmed_milestones((..).into()).await?;

    // Listen to the milestones from the node.
    while let Some(milestone_and_params) = milestone_stream.next().await {
        let milestone_index = milestone_and_params?.milestone.milestone_info.milestone_index;
        println!("Fetch cone of milestone {milestone_index}");

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
