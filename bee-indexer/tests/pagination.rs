// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_indexer::{AliasFilterOptions, Indexer, IndexerError};
use bee_ledger::workers::event::OutputCreated;
use bee_message::{milestone::MilestoneIndex, output::Output};
use bee_test::rand::{
    message::rand_message_id,
    output::{rand_alias_output, rand_output_id},
};

fn random_created_alias() -> OutputCreated {
    OutputCreated {
        message_id: rand_message_id(),
        output_id: rand_output_id(),
        output: Output::Alias(rand_alias_output()),
    }
}

#[tokio::test]
async fn pagination() -> Result<(), IndexerError> {
    let num_outputs = 10;
    let page_size = 4;

    let outputs = std::iter::repeat_with(|| random_created_alias())
        .take(num_outputs)
        .collect::<Vec<_>>();

    let indexer = Indexer::new().await?;
    indexer.update_status(MilestoneIndex(42)).await?;

    for created in outputs.iter() {
        indexer.process_created_output(&created).await?;
    }

    assert_eq!(
        indexer
            .alias_outputs_with_filters(AliasFilterOptions::default())
            .await?
            .output_ids
            .len(),
        num_outputs
    );

    let page_1 = indexer
        .alias_outputs_with_filters(AliasFilterOptions {
            page_size,
            ..Default::default()
        })
        .await?;

    assert_eq!(page_1.output_ids.len(), page_size as usize, "Page 1");
    assert!(page_1.cursor.is_some());

    let page_2 = indexer
        .alias_outputs_with_filters(AliasFilterOptions {
            page_size,
            cursor: page_1.cursor,
            ..Default::default()
        })
        .await?;

    assert_eq!(page_2.output_ids.len(), page_size as usize, "Page 2");
    assert!(page_2.cursor.is_some());

    let page_3 = indexer
        .alias_outputs_with_filters(AliasFilterOptions {
            page_size,
            cursor: page_2.cursor,
            ..Default::default()
        })
        .await?;

    assert_eq!(page_3.output_ids.len(), num_outputs % page_size as usize, "Page 3");
    assert!(page_3.cursor.is_none());

    Ok(())
}
