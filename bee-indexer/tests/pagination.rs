// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0


use bee_indexer::{Error, Indexer};
use bee_ledger::{types::CreatedOutput, workers::event::OutputCreated};
use bee_message::{milestone::MilestoneIndex, output::Output};
use bee_test::rand::{
    message::rand_message_id,
    milestone::rand_milestone_index,
    number::{rand_number, rand_number_range},
    output::{rand_alias_output, rand_output_id},
};

use serde_json::json;

use std::collections::HashSet;

fn random_created_alias() -> OutputCreated {
    OutputCreated {
        output_id: rand_output_id(),
        output: CreatedOutput::new(
            rand_message_id(),
            rand_milestone_index(),
            rand_number(),
            Output::Alias(rand_alias_output()),
        ),
    }
}

#[tokio::test]
async fn pagination() -> Result<(), Error> {
    // TODO: Test edge cases: num_outputs: 0; page_size: 0;
    let num_outputs = rand_number_range(1..=100);
    let page_size = rand_number_range(1..=100);

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
            .alias_outputs_with_filters(Default::default())
            .await?
            .output_ids
            .len(),
        num_outputs
    );

    // TODO: Use Vec + correct ordering and check for equality!
    let mut paginated_output_ids = HashSet::new();

    let mut page_index = 0;

    let options = json!({ "pageSize": page_size }).to_string();

    let mut page = indexer
        .alias_outputs_with_filters(serde_json::from_str(&options).unwrap())
        .await?;

    while let Some(cursor) = page.cursor {
        assert_eq!(page.output_ids.len(), page_size as usize, "Page {}", page_index);
        for output_id in page.output_ids {
            paginated_output_ids.insert(output_id);
        }

        let options = json!({ "pageSize": page_size, "cursor": cursor }).to_string();

        page = indexer
            .alias_outputs_with_filters(serde_json::from_str(&options).unwrap())
            .await?;

        page_index += 1;
    }

    // Last page
    for output_id in page.output_ids {
        paginated_output_ids.insert(output_id);
    }

    let expected_page_count = (num_outputs as u64 + page_size - 1) / page_size;

    assert_eq!(page.cursor, None, "The cursor should be no cursor in the end.");
    assert_eq!(
        page_index + 1,
        expected_page_count,
        "Number of outputs: {}, Page size: {}",
        num_outputs,
        page_size
    );
    assert_eq!(
        paginated_output_ids.len(),
        num_outputs,
        "Number of outputs: {}, Page size: {}",
        num_outputs,
        page_size
    );

    Ok(())
}
