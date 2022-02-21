// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_indexer::{Indexer, Error};
use bee_message::milestone::MilestoneIndex;

#[tokio::test]
async fn update_status() -> Result<(), Error> {
    let indexer = Indexer::new_in_memory().await?;
    assert_eq!(indexer.current_status().await?, MilestoneIndex(0));
    indexer.update_status(MilestoneIndex(42)).await?;
    assert_eq!(indexer.current_status().await?, MilestoneIndex(42));
    Ok(())
}
