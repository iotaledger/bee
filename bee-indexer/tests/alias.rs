// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_indexer::{Error, Indexer};
use bee_ledger::{types::CreatedOutput,workers::event::OutputCreated};
use bee_message::{milestone::MilestoneIndex, output::Output};
use bee_test::rand::{
    number::rand_number,
    message::rand_message_id,
    output::{rand_alias_output, rand_output_id}, milestone::rand_milestone_index,
};

use serde_json::json;

use packable::PackableExt;

#[tokio::test]
async fn get_id() -> Result<(), Error> {
    let indexer = Indexer::new_in_memory().await?;
    indexer.update_status(MilestoneIndex(42)).await?;

    let expected_output_id = rand_output_id();
    let alias = rand_alias_output();
    let alias_id = alias.alias_id().clone();
    let created = OutputCreated {
        output_id: expected_output_id.clone(),
        output: CreatedOutput::new(
            rand_message_id(),
            rand_milestone_index(),
            rand_number(),
            alias.into())
    };


    indexer.process_created_output(&created).await?;

    let res = indexer.get_output_id_for_alias_id(hex::encode(alias_id)).await?;

    assert_eq!(res.unwrap(), expected_output_id.to_string());

    Ok(())
}

#[tokio::test]
async fn with_state_controller() -> Result<(), Error> {
    let indexer = Indexer::new_in_memory().await?;
    indexer.update_status(MilestoneIndex(42)).await?;

    let expected_output_id = rand_output_id();
    let alias = rand_alias_output();
    let state_controller = alias.state_controller().clone();
    let created = OutputCreated {
        output_id: expected_output_id.clone(),
        output: CreatedOutput::new(
            rand_message_id(),
            rand_milestone_index(),
            rand_number(),
            Output::Alias(alias),)
    };

    let created2 = OutputCreated {
        output_id: rand_output_id(),
        output: CreatedOutput::new(
            rand_message_id(),
            rand_milestone_index(),
            rand_number(),
            Output::Alias(rand_alias_output()),)
    };

    indexer.process_created_output(&created).await?;
    indexer.process_created_output(&created2).await?;

    let options = json!({ "stateController": state_controller.to_bech32("atoi") }).to_string();

    let res = indexer
        .alias_outputs_with_filters(serde_json::from_str(&options).unwrap())
        .await?;

    assert_eq!(res.items, &[hex::encode(expected_output_id.pack_to_vec())]);

    Ok(())
}

#[tokio::test]
async fn created_after_before() -> Result<(), Error> {
    let indexer = Indexer::new_in_memory().await?;
    indexer.update_status(MilestoneIndex(42)).await?;

    let created = OutputCreated {
        output_id: rand_output_id(),
        output: CreatedOutput::new(
            rand_message_id(),
            rand_milestone_index(),
            41,
            Output::Alias(rand_alias_output()),)
        
    };

    let created2 = OutputCreated {
        output_id: rand_output_id(),
        output: CreatedOutput::new(
            rand_message_id(),
            rand_milestone_index(),
            43,
            Output::Alias(rand_alias_output()),)
    };

    indexer.process_created_output(&created).await?;
    indexer.process_created_output(&created2).await?;

    let options = json!({ "createdBefore": 42u32 }).to_string();

    let res = indexer
        .alias_outputs_with_filters(serde_json::from_str(&options).unwrap())
        .await?;

    assert_eq!(res.items, &[hex::encode(created.output_id.pack_to_vec())]);

    let options = json!({ "createdAfter": 42u32 }).to_string();

    let res = indexer
        .alias_outputs_with_filters(serde_json::from_str(&options).unwrap())
        .await?;

    assert_eq!(res.items, &[hex::encode(created2.output_id.pack_to_vec())]);

    Ok(())
}
