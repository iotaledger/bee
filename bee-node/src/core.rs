// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains the base types to build a Bee node.

use crate::shutdown::ShutdownTx;

use bee_runtime::{node::Node, worker::Worker};

use anymap::{any::Any as AnyMapAny, Map};
use futures::Future;
use fxhash::FxBuildHasher;
use tokio::task;

use std::{
    any::{type_name, TypeId},
    collections::{HashMap, HashSet},
    pin::Pin,
};

pub(crate) type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
pub(crate) type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;
pub(crate) type ResourceRegister<N> = dyn for<'a> FnOnce(&'a mut N);
pub(crate) type WorkerStopMap<N> = HashMap<TypeId, Box<WorkerStop<N>>>;
pub(crate) type WorkerNameMap = HashMap<TypeId, &'static str>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("storage backend operation failed: {0}")]
    StorageBackend(Box<dyn std::error::Error>),
    #[error("shutdown error")]
    Shutdown,
}

/// Combines worker, resource, task management and shutdown logic.
#[allow(clippy::type_complexity)]
pub struct Core<N: Node> {
    pub(crate) workers: Map<dyn AnyMapAny + Send + Sync>,
    pub(crate) tasks: HashMap<
        TypeId,
        Vec<(
            ShutdownTx,
            // TODO Result ?
            Box<dyn Future<Output = Result<(), task::JoinError>> + Send + Sync + Unpin>,
        )>,
    >,
    pub(crate) resources: Map<dyn AnyMapAny + Send + Sync>,
    pub(crate) worker_stops: HashMap<TypeId, Box<WorkerStop<N>>>,
    pub(crate) worker_order: Vec<TypeId>,
    pub(crate) worker_names: HashMap<TypeId, &'static str>,
    // phantom: PhantomData<(B, S)>,
}

impl<N: Node> Core<N> {
    /// Creates a new base node.
    pub(crate) fn new(worker_stops: WorkerStopMap<N>, worker_order: Vec<TypeId>, worker_names: WorkerNameMap) -> Self {
        log_topological_order(&worker_order, &worker_names);

        Self {
            workers: Map::new(),
            tasks: HashMap::new(),
            resources: Map::new(),
            worker_stops,
            worker_order,
            worker_names,
            // phantom: PhantomData,
        }
    }

    /// Adds a worker to the node.
    pub(crate) fn add_worker<W: Worker<N> + Send + Sync>(&mut self, worker: W) {
        self.workers.insert(worker);
    }

    /// Removes a worker from the node.
    pub(crate) fn remove_worker<W: Worker<N> + Send + Sync>(&mut self) -> W {
        self.workers
            .remove()
            .unwrap_or_else(|| panic!("failed to remove worker `{}`", type_name::<W>()))
    }
}

pub(crate) struct TopologicalOrder {
    graph: HashMap<TypeId, &'static [TypeId], FxBuildHasher>,
    non_visited: HashSet<TypeId, FxBuildHasher>,
    being_visited: HashSet<TypeId, FxBuildHasher>,
    order: Vec<TypeId>,
}

impl TopologicalOrder {
    pub(crate) fn visit(&mut self, id: TypeId) {
        if !self.non_visited.contains(&id) {
            return;
        }

        if !self.being_visited.insert(id) {
            panic!("Cyclic dependency detected.");
        }

        for &id in self.graph[&id] {
            self.visit(id);
        }

        self.being_visited.remove(&id);
        self.non_visited.remove(&id);
        self.order.push(id);
    }

    pub(crate) fn sort(graph: HashMap<TypeId, &'static [TypeId], FxBuildHasher>) -> Vec<TypeId> {
        let non_visited = graph.keys().copied().collect();

        let mut this = Self {
            graph,
            non_visited,
            being_visited: HashSet::default(),
            order: vec![],
        };

        while let Some(&id) = this.non_visited.iter().next() {
            this.visit(id);
        }

        this.order
    }
}

pub(crate) fn log_topological_order(worker_order: &[TypeId], worker_names: &WorkerNameMap) {
    let mut topol_order = String::with_capacity(512);

    for worker_id in worker_order.iter() {
        // Panic: unwrapping is fine since worker_id is from the list of workers.
        let worker_name = *worker_names.get(worker_id).unwrap();

        topol_order.push_str(worker_name);
        topol_order.push(' ');
    }

    log::debug!("Workers topological order:{}", topol_order);
}
