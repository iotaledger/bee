// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_inx::{client::Inx, Error, MilestoneRangeRequest};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let inx_connect_url = std::env::var("INX_CONNECT_URL").unwrap_or_else(|_| "http://localhost:9029".to_string());

    let mut inx = Inx::connect(&inx_connect_url).await?;
    println!("Connected via INX to node at {inx_connect_url}");

    let mut unspent_outputs = inx.read_unspent_outputs().await?;

    let mut count = 0;
    while let Some(_unspent_output) = unspent_outputs.next().await {
        count += 1;
    }
    println!("Read {count} unspent outputs.");

    let mut ledger_update_feed = inx.listen_to_ledger_updates(MilestoneRangeRequest::from(..)).await?;

    while let Some(ledger_update) = ledger_update_feed.next().await {
        let ledger_update = ledger_update?;
        println!("{:?}", ledger_update);
    }

    Ok(())
}
