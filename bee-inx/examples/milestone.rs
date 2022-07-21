// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_inx::{client, Error};
use futures::StreamExt;

const INX_ADDRESS: &str = "http://localhost:9029";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut inx = client::Inx::connect(INX_ADDRESS.into()).await?;
    let mut milestone_stream = inx.listen_to_confirmed_milestones(..).await?;

    // Listen to the milestones from the node.
    while let Some(milestone) = milestone_stream.next().await {
        println!("Fetch cone of milestone {}", milestone?.milestone_info.milestone_index);

        // // Listen to messages in the past cone of a milestone.
        // let mut cone_stream = inx
        //     .read_milestone_cone(proto::MilestoneRequest::from_index(
        //         milestone.milestone_info.milestone_index,
        //     ))
        //     .await?
        //     .into_inner().map_ok(|b| bee_inx::BlockWithMetadata::try_from(b));

        // // Keep track of the number of blocks.
        // let mut count = 0usize;

        // while let Some(Ok(Ok(block_metadata))) = cone_stream.next().await {

        //     println!(
        //         "Block {}: {:#?}",
        //         block_metadata.metadata.block_id, block_metadata.block
        //     );
        //     count += 1;
        // }

        // println!(
        //     "Milestone {:?} contained {count} blocks",
        //     milestone.milestone_info.milestone_id
        // );
    }

    Ok(())
}
