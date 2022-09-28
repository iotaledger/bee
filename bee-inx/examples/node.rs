// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_inx::{client, milestone::requests::MilestoneRequest, node::requests::NodeStatusRequest, Error};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let inx_connect_url = std::env::var("INX_CONNECT_URL").expect("missing INX_CONNECT_URL environment variable");

    let mut inx = client::Inx::connect(&inx_connect_url).await?;
    println!("Connected via INX to node at {inx_connect_url}");

    let node_status = inx.read_node_status().await?;
    println!("{:?}", node_status);

    let node_configuration = inx.read_node_configuration().await?;
    println!("{:?}", node_configuration);

    let protocol_parameters = inx
        .read_protocol_parameters(MilestoneRequest::MilestoneIndex(node_status.ledger_index))
        .await?;
    println!("{:?}", protocol_parameters);

    const COOLDOWN_MS: u32 = 5000;
    let mut node_status_stream = inx
        .listen_to_node_status(NodeStatusRequest {
            cooldown_in_milliseconds: COOLDOWN_MS,
        })
        .await?;
    println!("Streaming current node status ... ");

    while let Some(node_status) = node_status_stream.next().await {
        let node_status = node_status?;

        println!("healthy: {} | synced: {} | ledger_index: {}", node_status.is_healthy, node_status.is_synced, node_status.ledger_index);
    }

    Ok(())
}
