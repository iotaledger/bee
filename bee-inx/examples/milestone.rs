// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::payload::milestone::MilestoneIndex;
use bee_inx::{client::Inx, Error};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let inx_connect_url = std::env::var("INX_CONNECT_URL").unwrap_or_else(|_| "http://localhost:9029".to_string());
    let milestone_stream = std::env::var("MILESTONE_STREAM").unwrap_or_else(|_| "confirmed_milestones".to_string());
    let read_cone = std::env::var("READ_CONE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap();
    let read_cone_metadata = std::env::var("READ_CONE_METADATA")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap();

    let mut inx = Inx::connect(&inx_connect_url).await?;
    println!("Connected via INX to node at {inx_connect_url}");

    match milestone_stream.as_str() {
        "confirmed_milestones" => {
            let mut milestone_stream = inx.listen_to_confirmed_milestones((..).into()).await?;
            println!("Streaming confirmed milestones and protocol parameters... ");

            while let Some(milestone_and_params) = milestone_stream.next().await {
                let milestone_and_params = milestone_and_params?;
                println!(
                    "{:?}{:?}",
                    milestone_and_params.milestone.milestone_info, milestone_and_params.current_protocol_parameters
                );

                let milestone_index = milestone_and_params.milestone.milestone_info.milestone_index;
                fetch_cone_and_metadata(&mut inx, milestone_index, read_cone, read_cone_metadata).await?;
            }
        }
        "latest_milestones" => {
            println!("Streaming latest milestones... ");

            let mut milestone_stream = inx.listen_to_latest_milestones().await?;

            while let Some(milestone) = milestone_stream.next().await {
                let milestone = milestone?;
                println!("{:?}", milestone.milestone_info);

                let milestone_index = milestone.milestone_info.milestone_index;
                fetch_cone_and_metadata(&mut inx, milestone_index, read_cone, read_cone_metadata).await?;
            }
        }
        _ => {
            panic!("unknown milestone stream variant: '{milestone_stream}'");
        }
    }

    Ok(())
}

async fn fetch_cone_and_metadata(
    inx: &mut Inx,
    milestone_index: MilestoneIndex,
    read_cone: bool,
    read_cone_metadata: bool,
) -> Result<(), Error> {
    if read_cone {
        println!("Fetching cone for {milestone_index}...");

        let mut cone_stream = inx.read_milestone_cone(milestone_index.0.into()).await?;
        let mut count = 0usize;

        while let Some(Ok(block_metadata)) = cone_stream.next().await {
            println!("\t{}", block_metadata.metadata.block_id);
            count += 1;
        }

        println!("Fetched {count} blocks in total.");
    }

    if read_cone_metadata {
        println!("Fetching cone metadata for {milestone_index}...");

        let mut cone_stream = inx.read_milestone_cone_metadata(milestone_index.0.into()).await?;
        let mut count = 0usize;

        while let Some(Ok(block_metadata)) = cone_stream.next().await {
            println!("\t{}", block_metadata.block_id);
            count += 1;
        }

        println!("Fetched {count} blocks in total.");
    }

    Ok(())
}
