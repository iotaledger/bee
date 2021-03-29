// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Functionality for performing an FPC vote.

use crate::{
    context::{ObjectType, VoteContext},
    error::Error,
    events::{Event, OpinionEvent, RoundStats},
    opinion::{Opinion, OpinionGiver, Opinions, QueriedOpinions, QueryIds},
};

use flume::Sender;
use rand::prelude::*;
use tokio::{sync::RwLock, time::timeout};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    default::Default,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub const DEFAULT_SAMPLE_SIZE: u32 = 21;

/// Stores `VoteContext`s in a queue, and provides a HashSet for quick lookup.
#[derive(Debug)]
struct Queue {
    /// Queue of all `VoteContext`s
    queue: VecDeque<VoteContext>,
    /// `HashSet` of IDs, for quick lookup.
    queue_set: HashSet<String>,
}

impl Queue {
    /// Construct a new, empty `Queue`.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            queue_set: HashSet::new(),
        }
    }

    /// Look up a `VoteContext` ID and determine if it is in the queue.
    pub fn contains(&self, value: &str) -> bool {
        self.queue_set.contains(value)
    }

    /// Push a new `VoteContext` to the end of the queue.
    pub fn push(&mut self, context: VoteContext) {
        self.queue_set.insert(context.id());
        self.queue.push_back(context);
    }

    /// Pop a `VoteContext` from the front of the queue.
    pub fn pop(&mut self) -> Option<VoteContext> {
        let context = self.queue.pop_front()?;
        self.queue_set.remove(&context.id());

        Some(context)
    }
}

/// Builder pattern struct for instantiating `Fpc`s.
pub struct FpcBuilder<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    tx: Option<Sender<Event>>,
    opinion_giver_fn: Option<F>,
    first_round_lower_bound: f64,
    first_round_upper_bound: f64,
    subsequent_rounds_lower_bound: f64,
    subsequent_rounds_upper_bound: f64,
    query_sample_size: u32,
    finalization_threshold: u32,
    cooling_off_period: u32,
    max_rounds_per_vote_context: u32,
    query_timeout: Duration,
}

impl<F> Default for FpcBuilder<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    /// Initialise with default parameters.
    /// Note that the `tx` and `opinion_giver_fn` fields still need to be set before building.
    fn default() -> Self {
        Self {
            tx: None,
            opinion_giver_fn: None,
            first_round_lower_bound: 0.67,
            first_round_upper_bound: 0.67,
            subsequent_rounds_lower_bound: 0.5,
            subsequent_rounds_upper_bound: 0.67,
            query_sample_size: DEFAULT_SAMPLE_SIZE,
            finalization_threshold: 10,
            cooling_off_period: 0,
            max_rounds_per_vote_context: 100,
            query_timeout: Duration::from_millis(6500),
        }
    }
}

impl<F> FpcBuilder<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    /// Provide a `Sender<Event>` to the builder, so that the user may receive events as the voting proceeds.
    pub fn with_tx(mut self, tx: Sender<Event>) -> Self {
        self.tx = Some(tx);
        self
    }

    /// Provide a closure to the builder that describes the `OpinionGivers` that will be used for voting.
    pub fn with_opinion_giver_fn(mut self, opinion_giver_fn: F) -> Self {
        self.opinion_giver_fn = Some(opinion_giver_fn);
        self
    }

    /// Provide upper and lower bounds for random opinion forming threshold, used on the first voting round.
    /// These bounds will be used to determine whether a `VoteContext` likes or dislikes a voting object.
    pub fn with_first_round_bounds(mut self, lower: f64, upper: f64) -> Self {
        self.first_round_lower_bound = lower;
        self.first_round_upper_bound = upper;
        self
    }

    /// Provide upper and lower bounds for random opinion forming threshold, used on subsequent voting rounds.
    /// These bounds will be used to determine whether a `VoteContext` likes or dislikes a voting object.
    pub fn with_subsequent_rounds_bounds(mut self, lower: f64, upper: f64) -> Self {
        self.subsequent_rounds_lower_bound = lower;
        self.subsequent_rounds_upper_bound = upper;
        self
    }

    /// Provide a query sample size.
    /// This is used to define the number of `Opinion`s to query on each voting round.
    pub fn with_query_sample_size(mut self, sample_size: u32) -> Self {
        self.query_sample_size = sample_size;
        self
    }

    /// Provide a finalization threshold.
    /// This is used to define the number of voting rounds in which a `VoteContext`s opinion must stay constant for.
    pub fn with_finalization_threshold(mut self, threshold: u32) -> Self {
        self.finalization_threshold = threshold;
        self
    }

    /// Provide a cool-off period.
    /// This is used to define the number of voting rounds in which to skip any finalization checks.
    pub fn with_cooling_off_period(mut self, period: u32) -> Self {
        self.cooling_off_period = period;
        self
    }

    /// Define the maximum number of rounds to execute before aborting the vote (if not finalized).
    pub fn with_max_rounds(mut self, max: u32) -> Self {
        self.max_rounds_per_vote_context = max;
        self
    }

    /// Instantiate a new `Fpc` struct using parameters given by the `FpcBuilder`.
    /// Note: this will panic if `tx` or `opinion_giver_fn` are not defined.
    pub fn build(self) -> Result<Fpc<F>, Error> {
        Ok(Fpc {
            tx: self.tx.ok_or(Error::FpcNoSender)?,
            opinion_giver_fn: self.opinion_giver_fn.ok_or(Error::FpcNoOpinionGiverFn)?,
            queue: RwLock::new(Queue::new()),
            contexts: RwLock::new(HashMap::new()),
            last_round_successful: AtomicBool::new(false),
            first_round_lower_bound: self.first_round_lower_bound,
            first_round_upper_bound: self.first_round_lower_bound,
            subsequent_rounds_lower_bound: self.subsequent_rounds_lower_bound,
            subsequent_rounds_upper_bound: self.subsequent_rounds_upper_bound,
            query_sample_size: self.query_sample_size,
            finalization_threshold: self.finalization_threshold,
            cooling_off_period: self.cooling_off_period,
            max_rounds_per_vote_context: self.max_rounds_per_vote_context,
            query_timeout: Duration::from_millis(6500),
        })
    }
}

/// Contains all instance information about a vote, including all `VoteContext`s, a queue of contexts
/// to be added to the vote in the next round, and RNG paramaters.
#[derive(Debug)]
pub struct Fpc<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    /// `Sender` for transmitting voting events through a channel.
    tx: Sender<Event>,
    /// Closure that describes the `OpinionGiver`s used in the vote.
    opinion_giver_fn: F,
    /// `Queue` of `VoteContext`s to be added to the next voting round.
    queue: RwLock<Queue>,
    /// Map of `VoteContext` IDs to contexts.
    /// Contains all `VoteContext`s that are participating in this voting round.
    contexts: RwLock<HashMap<String, VoteContext>>,
    /// Indicates whether the last round completed without error or other failure.
    /// These will be indicated through `Error` or `Failed` events.
    last_round_successful: AtomicBool,
    /// Lower bound for random opinion forming threshold, used on the first voting round.
    /// These bounds will be used to determine whether a `VoteContext` likes or dislikes a voting object.
    first_round_lower_bound: f64,
    /// Upper bound for random opinion forming threshold, used on the first voting round.
    first_round_upper_bound: f64,
    /// Lower bound for random opinion forming threshold, used on subsequent voting rounds.
    subsequent_rounds_lower_bound: f64,
    /// Upper bound for random opinion forming threshold, used on subsequent voting rounds.
    subsequent_rounds_upper_bound: f64,
    /// Number of `Opinion`s to query on each voting round.
    query_sample_size: u32,
    /// Number of voting rounds in which a `VoteContext`s opinion must stay constant for.
    finalization_threshold: u32,
    /// Number of voting rounds in which to skip any finalization checks.
    cooling_off_period: u32,
    /// Maximum number of rounds to execute before aborting the vote (if not finalized).
    max_rounds_per_vote_context: u32,
    /// Maximum time before aborting a query.
    query_timeout: Duration,
}

impl<F> Fpc<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    /// Add a `VoteContext` to the queue for the next round, providing a vote ID, `ObjectType` and an initial opinion
    /// of the context.
    /// This can fail if there is already a vote ongoing for this ID.
    pub async fn vote(&self, id: String, object_type: ObjectType, initial_opinion: Opinion) -> Result<(), Error> {
        let mut queue_guard = self.queue.write().await;
        let context_guard = self.contexts.read().await;

        if queue_guard.contains(&id) {
            return Err(Error::VoteOngoing(id));
        }

        if context_guard.contains_key(&id) {
            return Err(Error::VoteOngoing(id));
        }

        queue_guard.push(VoteContext::new(id, object_type, initial_opinion));
        Ok(())
    }

    /// Return the most recent opinion on the given ID. If a `VoteContext` with the ID does not exist, returns None.
    pub async fn intermediate_opinion(&self, id: String) -> Option<Opinion> {
        if let Some(context) = self.contexts.read().await.get(&id) {
            context.last_opinion()
        } else {
            Some(Opinion::Unknown)
        }
    }

    /// Add a `VoteContext` to the queue, to participate on the voting for the next round.
    async fn enqueue(&self) {
        let mut queue_guard = self.queue.write().await;
        let mut context_guard = self.contexts.write().await;

        while let Some(context) = queue_guard.pop() {
            context_guard.insert(context.id(), context);
        }
    }

    /// Loop through all `VoteContext`s that are participating, and have them form an opinion on the voting object.
    async fn form_opinions(&self, rand: f64) {
        let mut context_guard = self.contexts.write().await;

        for context in context_guard.values_mut() {
            if context.is_new() {
                continue;
            }

            let (lower_bound, upper_bound) = if context.had_first_round() {
                (self.first_round_lower_bound, self.first_round_upper_bound)
            } else {
                (self.subsequent_rounds_lower_bound, self.subsequent_rounds_upper_bound)
            };

            if context.liked() >= self.rand_uniform_threshold(rand, lower_bound, upper_bound) {
                context.add_opinion(Opinion::Like);
            } else {
                context.add_opinion(Opinion::Dislike);
            }
        }
    }

    /// Check if any `VoteContext`s have finalized opinions.
    /// If a context has finalized on an opinion, send an event down the channel and remove it from the voting pool.
    async fn finalize_opinions(&self) -> Result<(), Error> {
        let context_guard = self.contexts.read().await;
        let mut to_remove = vec![];

        for (id, context) in context_guard.iter() {
            if context.finalized(self.cooling_off_period, self.finalization_threshold) {
                self.tx
                    .send(Event::Finalized(OpinionEvent {
                        id: id.clone(),
                        opinion: context.last_opinion().ok_or(Error::Unknown("No opinions found"))?,
                        context: context.clone(),
                    }))
                    .or(Err(Error::SendError))?;

                to_remove.push(id.clone());
                continue;
            }

            if context.rounds() >= self.max_rounds_per_vote_context {
                self.tx
                    .send(Event::Failed(OpinionEvent {
                        id: id.clone(),
                        opinion: context.last_opinion().ok_or(Error::Unknown("No opinions found"))?,
                        context: context.clone(),
                    }))
                    .or(Err(Error::SendError))?;

                to_remove.push(id.clone());
            }
        }
        drop(context_guard);

        let mut context_guard = self.contexts.write().await;

        for id in to_remove {
            context_guard.remove(&id);
        }

        Ok(())
    }

    /// Perform the voting round, with a given threshold (between 0 and 1).
    /// This threshold is used to generate opinions on the voting object.
    ///
    /// For each `VoteContext` in the voting pool, a random number is generated within the range
    /// given on initialisation of the `Fpc` struct, and compared to the threshold to generate a
    /// `Like` or `Dislike` opinion.
    pub async fn do_round(&self, rand: f64) -> Result<(), Error> {
        let start = Instant::now();
        self.enqueue().await;

        if self.last_round_successful.load(Ordering::Relaxed) {
            self.form_opinions(rand).await;
            self.finalize_opinions().await?;
        }

        let queried_opinions = self.query_opinions().await?;
        self.last_round_successful.store(true, Ordering::Relaxed);

        let round_stats = RoundStats {
            duration: start.elapsed(),
            rand_used: rand,
            vote_contexts: self.contexts.read().await.clone(),
            queried_opinions,
        };

        self.tx
            .send(Event::RoundExecuted(round_stats))
            .or(Err(Error::SendError))?;

        Ok(())
    }

    /// Select a number of `OpinionGiver`s and query them for opinions.
    async fn query_opinions(&self) -> Result<Vec<QueriedOpinions>, Error> {
        let mut rng = thread_rng();
        let query_ids = self.vote_context_ids().await;

        if query_ids.conflict_ids.is_empty() && query_ids.timestamp_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut opinion_givers = (self.opinion_giver_fn)()?;

        if opinion_givers.is_empty() {
            return Err(Error::NoOpinionGivers);
        }

        let dist = rand::distributions::Uniform::new(0, opinion_givers.len());
        let mut queries = vec![0u32; opinion_givers.len()];

        for _ in 0..self.query_sample_size {
            let index = rng.sample(dist);

            if let Some(selected_count) = queries.get_mut(index) {
                *selected_count += 1;
            }
        }

        let vote_map = Arc::new(RwLock::new(HashMap::<String, Opinions>::new()));
        let all_queried_opinions = Arc::new(RwLock::new(Vec::<QueriedOpinions>::new()));

        let mut futures = vec![];

        for (i, opinion_giver) in opinion_givers.iter_mut().enumerate() {
            // This should never panic, since `queries.len()` == `opinion_givers.len()`
            let selected_count = queries.get(i).unwrap();

            if *selected_count > 0 {
                futures.push(timeout(
                    self.query_timeout,
                    Self::do_query(
                        &query_ids,
                        vote_map.clone(),
                        all_queried_opinions.clone(),
                        opinion_giver,
                        *selected_count,
                    ),
                ));
            }
        }

        futures::future::join_all(futures).await;

        let mut contexts_guard = self.contexts.write().await;
        let votes_guard = vote_map.read().await;

        for (id, votes) in votes_guard.iter() {
            let mut liked_sum = 0.0;
            let mut voted_count = votes.len() as f64;

            for vote in votes.iter() {
                match vote {
                    Opinion::Unknown => voted_count -= 1.0,
                    Opinion::Like => liked_sum += 1.0,
                    _ => {}
                }
            }

            // This should never happen – there should always be a context for a given vote.
            contexts_guard.get_mut(id).unwrap().round_completed();

            if voted_count == 0.0 {
                continue;
            }

            contexts_guard.get_mut(id).unwrap().set_liked(liked_sum / voted_count);
        }

        // This should never fail – all futures are completed, so only one reference remains.
        Ok(Arc::try_unwrap(all_queried_opinions).unwrap().into_inner())
    }

    /// Run a query on a given `OpinionGiver`, to generate opinions on the voting object.
    async fn do_query(
        query_ids: &QueryIds,
        vote_map: Arc<RwLock<HashMap<String, Opinions>>>,
        all_queried_opinions: Arc<RwLock<Vec<QueriedOpinions>>>,
        opinion_giver: &mut Box<dyn OpinionGiver>,
        selected_count: u32,
    ) {
        let opinions = opinion_giver.query(query_ids);

        let opinions = if let Ok(opinions) = opinions {
            if opinions.len() != query_ids.conflict_ids.len() + query_ids.timestamp_ids.len() {
                return;
            } else {
                opinions
            }
        } else {
            return;
        };

        let mut queried_opinions = QueriedOpinions {
            opinion_giver_id: opinion_giver.id().to_string(),
            opinions: HashMap::new(),
            times_counted: selected_count,
        };

        let mut vote_map_guard = vote_map.write().await;

        for (i, id) in query_ids.conflict_ids.iter().enumerate() {
            let mut votes = vote_map_guard.get(id).map_or(Opinions::new(vec![]), |opt| opt.clone());

            for _ in 0..selected_count {
                votes.push(opinions[i]);
            }

            queried_opinions.opinions.insert(id.to_string(), opinions[i]);

            if vote_map_guard.contains_key(id) {
                // This will never fail – the key exists.
                *vote_map_guard.get_mut(id).unwrap() = votes;
            } else {
                vote_map_guard.insert(id.to_string(), votes);
            }
        }

        for (i, id) in query_ids.timestamp_ids.iter().enumerate() {
            let mut votes = vote_map_guard.get(id).map_or(Opinions::new(vec![]), |opt| opt.clone());

            for _ in 0..selected_count {
                votes.push(opinions[i]);
            }

            queried_opinions.opinions.insert(id.to_string(), opinions[i]);

            if vote_map_guard.contains_key(id) {
                // This will never fail - the key exists.
                *vote_map_guard.get_mut(id).unwrap() = votes;
            } else {
                vote_map_guard.insert(id.to_string(), votes);
            }
        }

        all_queried_opinions.write().await.push(queried_opinions);
    }

    /// Get the IDs of all `VoteContext`s currently in the voting pool.
    async fn vote_context_ids(&self) -> QueryIds {
        let context_guard = self.contexts.read().await;
        let mut conflict_ids = vec![];
        let mut timestamp_ids = vec![];

        for (id, context) in context_guard.iter() {
            match context.object_type() {
                ObjectType::Conflict => {
                    conflict_ids.push(id.clone());
                }
                ObjectType::Timestamp => {
                    timestamp_ids.push(id.clone());
                }
            }
        }

        QueryIds {
            conflict_ids,
            timestamp_ids,
        }
    }

    fn rand_uniform_threshold(&self, rand: f64, lower_bound: f64, upper_bound: f64) -> f64 {
        lower_bound + rand * (upper_bound - lower_bound)
    }
}
