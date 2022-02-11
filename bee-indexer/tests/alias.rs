// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_indexer::{Error, Indexer};
use bee_ledger::workers::event::OutputCreated;
use bee_message::{milestone::MilestoneIndex, output::Output};
use bee_test::rand::{
    message::rand_message_id,
    output::{rand_alias_output, rand_output_id},
};

use serde_json::json;

use packable::PackableExt;

#[tokio::test]
async fn with_state_controller() -> Result<(), Error> {
    let indexer = Indexer::new().await?;
    indexer.update_status(MilestoneIndex(42)).await?;

    let expected_output_id = rand_output_id();
    let alias = rand_alias_output();
    let state_controller = alias.state_controller().clone();
    let created = OutputCreated {
        message_id: rand_message_id(),
        output_id: expected_output_id.clone(),
        output: Output::Alias(alias),
    };

    let created2 = OutputCreated {
        message_id: rand_message_id(),
        output_id: rand_output_id(),
        output: Output::Alias(rand_alias_output()),
    };

    indexer.process_created_output(&created).await?;
    indexer.process_created_output(&created2).await?;

    let options = json!({ "stateController": hex::encode(state_controller.pack_to_vec()) }).to_string();

    let res = indexer
        .alias_outputs_with_filters(serde_json::from_str(&options).unwrap())
        .await?;

    assert_eq!(res.output_ids, &[expected_output_id]);

    Ok(())
}
