// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_indexer::{AliasFilterOptions, Indexer, IndexerError};
use bee_ledger::workers::event::OutputCreated;
use bee_message::{milestone::MilestoneIndex, output::Output};
use bee_test::rand::{
    bytes::rand_bytes_array,
    message::rand_message_id,
    milestone::rand_milestone_index,
    number::rand_number,
    output::{rand_alias_output, rand_output_id},
};

#[tokio::test]
async fn update_status() -> Result<(), IndexerError> {
    let indexer = Indexer::new().await?;
    indexer.update_status(MilestoneIndex(42)).await?;

    let alias = rand_alias_output();
    let state_controller = alias.state_controller().clone();
    let created = OutputCreated {
        message_id: rand_message_id(),
        output_id: rand_output_id(),
        output: Output::Alias(alias),
    };

    let created2 = OutputCreated {
        message_id: rand_message_id(),
        output_id: rand_output_id(),
        output: Output::Alias(rand_alias_output()),
    };

    indexer.process_created_output(&created).await?;
    indexer.process_created_output(&created2).await?;

    let res = indexer
        .alias_outputs_with_filters(AliasFilterOptions {
            state_controller: Some(state_controller),
            ..Default::default()
        })
        .await?;

    println!("{:#?}", res);
    assert!(false);

    Ok(())
}
