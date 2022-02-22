// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_indexer::{FilterOptionsDto, AliasFilterOptionsDto, Indexer};
use bee_ledger::{types::CreatedOutput, workers::event::OutputCreated};
use bee_message::output::Output;
use bee_test::rand::{
    message::rand_message_id,
    milestone::rand_milestone_index,
    number::rand_number,
    output::{rand_alias_output, rand_output_id},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use packable::PackableExt;
use tokio::runtime::Runtime;

fn indexation_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("indexation");
    for num_samples in [10, 100, 1_000, 10_000, 100_000, 1_000_000] {
        let indexer = rt.block_on(Indexer::new_in_memory()).unwrap();

        let mut target_state_controller = None;
        let mut target_output_id = None;

        for i in 0..num_samples {
            let created_output = OutputCreated {
                output_id: rand_output_id(),
                output: CreatedOutput::new(
                    rand_message_id(),
                    rand_milestone_index(),
                    rand_number(),
                    Output::Alias(rand_alias_output()),
                ),
            };

            // We take some element towards the back of the `OutputCreated`s.
            if i == (num_samples / 4) * 3 {
                let inner = created_output.output.inner();
                match inner {
                    Output::Alias(alias) => {
                        target_state_controller = Some(alias.state_controller().clone());
                        target_output_id = Some(created_output.output_id);
                    }
                    _ => unimplemented!(),
                }
            }

            rt.block_on(indexer.process_created_output(&created_output)).unwrap();
        }

        let state_controller_enc = target_state_controller.unwrap().to_bech32("atoi");
        let output_id_enc = target_output_id.unwrap().to_string();

        group.bench_with_input(
            BenchmarkId::from_parameter(num_samples),
            &num_samples,
            |b, &_num_samples| {
                b.to_async(&rt).iter(|| async {
                    assert_eq!(
                        indexer
                            .alias_outputs_with_filters(FilterOptionsDto {
                                inner: AliasFilterOptionsDto {
                                    state_controller: black_box(Some(state_controller_enc.clone())),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .await
                            .ok()
                            .unwrap()
                            .items
                            .first()
                            .unwrap(),
                        &output_id_enc
                    )
                })
            },
        );
    }
}

criterion_group!(benches, indexation_benchmark);
criterion_main!(benches);
