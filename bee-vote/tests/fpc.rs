// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod mock;

use bee_vote::{
    error::Error,
    events::Event,
    fpc::{self, FpcBuilder},
    opinion::{Opinion, OpinionGiver, Opinions},
    VoteObject,
};

#[tokio::test]
async fn prohibit_multiple_votes() {
    let opinion_giver_fn = || Err(Error::NoOpinionGivers);
    let (tx, _) = flume::unbounded();

    let voter = FpcBuilder::default()
        .with_opinion_giver_fn(opinion_giver_fn)
        .with_tx(tx)
        .build()
        .unwrap();

    let tx_id = mock::rand_transaction_id();
    assert!(voter
        .vote(VoteObject::Conflict(tx_id), Opinion::Like)
        .await
        .is_ok());
    assert!(matches!(
        voter.vote(VoteObject::Conflict(tx_id), Opinion::Like).await,
        Err(Error::VoteOngoing(_))
    ));

    assert!(voter.vote(VoteObject::Conflict(mock::rand_transaction_id()), Opinion::Like).await.is_ok());
}

#[tokio::test]
async fn finalized_event() {
    let mock = mock::MockOpinionGiver {
        id: mock::random_id_string(),
        round: 0,
        round_replies: vec![Opinions::new(vec![Opinion::Like]); 4],
    };

    let opinion_giver_fn = || -> Result<Vec<Box<dyn OpinionGiver>>, Error> { Ok(vec![Box::new(mock.clone())]) };
    let id = String::from("test");

    let (tx, rx) = flume::unbounded();
    let voter = FpcBuilder::default()
        .with_opinion_giver_fn(opinion_giver_fn)
        .with_tx(tx)
        .with_finalization_threshold(2)
        .with_cooling_off_period(2)
        .with_query_sample_size(1)
        .build()
        .unwrap();

    assert!(voter.vote(VoteObject::Conflict(mock::rand_transaction_id()), Opinion::Like).await.is_ok());

    for _ in 0..5 {
        futures::executor::block_on(voter.do_round(0.5)).unwrap();
    }

    let mut event = None;

    let mut iter = rx.try_iter();
    while let Some(ev) = iter.next() {
        if let Event::Finalized(_) = ev {
            event = Some(ev);
        }
    }

    assert!(matches!(event, Some(Event::Finalized(_))));
}

#[tokio::test]
async fn failed_event() {
    let mock = mock::MockOpinionGiver {
        id: mock::random_id_string(),
        round: 0,
        round_replies: vec![Opinions::new(vec![Opinion::Dislike])],
    };

    let opinion_giver_fn = || -> Result<Vec<Box<dyn OpinionGiver>>, Error> { Ok(vec![Box::new(mock.clone())]) };
    let id = String::from("test");

    let (tx, rx) = flume::unbounded();
    let voter = FpcBuilder::default()
        .with_opinion_giver_fn(opinion_giver_fn)
        .with_tx(tx)
        .with_finalization_threshold(4)
        .with_cooling_off_period(0)
        .with_query_sample_size(1)
        .with_max_rounds(3)
        .build()
        .unwrap();

    assert!(voter.vote(VoteObject::Conflict(mock::rand_transaction_id()), Opinion::Like).await.is_ok());

    for _ in 0..4 {
        futures::executor::block_on(voter.do_round(0.5)).unwrap();
    }

    let mut event = None;

    let mut iter = rx.try_iter();
    while let Some(ev) = iter.next() {
        if let Event::Failed(_) = ev {
            event = Some(ev);
        }
    }

    assert!(matches!(event, Some(Event::Failed(_))));
}

#[tokio::test]
async fn multiple_opinion_givers() {
    let init_opinions = vec![Opinion::Like, Opinion::Dislike];
    let expected_opinions = vec![Opinion::Like, Opinion::Dislike];
    let num_tests = 2;

    for i in 0..num_tests {
        let opinion_giver_fn = || -> Result<Vec<Box<dyn OpinionGiver>>, Error> {
            let mut opinion_givers: Vec<Box<dyn OpinionGiver>> = vec![];

            for _ in 0..fpc::DEFAULT_SAMPLE_SIZE {
                opinion_givers.push(Box::new(mock::MockOpinionGiver {
                    id: mock::random_id_string(),
                    round: 0,
                    round_replies: vec![Opinions::new(vec![init_opinions[i]])],
                }));
            }

            Ok(opinion_givers)
        };

        let (tx, rx) = flume::unbounded();
        let voter = FpcBuilder::default()
            .with_opinion_giver_fn(opinion_giver_fn)
            .with_tx(tx)
            .with_finalization_threshold(2)
            .with_cooling_off_period(2)
            .build()
            .unwrap();

        assert!(voter
            .vote(VoteObject::Conflict(mock::rand_transaction_id()), init_opinions[i])
            .await
            .is_ok());

        let mut rounds = 0u32;

        let final_opinion = loop {
            assert!(voter.do_round(0.7f64).await.is_ok());
            rounds += 1;

            let mut iter = rx.try_iter();
            let mut final_opinion = None;

            while let Some(ev) = iter.next() {
                if let Event::Finalized(opinion_event) = ev {
                    final_opinion = Some(opinion_event.opinion);
                }
            }

            if let Some(opinion) = final_opinion {
                break opinion;
            }
        };

        assert_eq!(rounds, 5);
        assert_eq!(final_opinion, expected_opinions[i]);
    }
}
