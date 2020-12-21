// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::worker::Worker;

use bee_common::{event::Bus, shutdown};
use bee_storage::storage::Backend;

use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};
use log::warn;

use std::{
    any::{type_name, Any},
    collections::HashMap,
    ops::Deref,
    panic::Location,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
};

#[async_trait]
pub trait Node: Send + Sized + 'static {
    type Builder: NodeBuilder<Self>;
    type Backend: Backend;

    fn build(config: <Self::Builder as NodeBuilder<Self>>::Config) -> Self::Builder {
        Self::Builder::new(config)
    }

    async fn stop(mut self) -> Result<(), shutdown::Error>;

    fn spawn<W, G, F>(&mut self, g: G)
    where
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static;

    fn worker<W>(&self) -> Option<&W>
    where
        W: Worker<Self> + Send + Sync;

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R);

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R>;

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResHandle<R>;

    fn storage(&self) -> ResHandle<Self::Backend> {
        self.resource()
    }

    fn event_bus(&self) -> ResHandle<Bus<'static>> {
        self.resource()
    }
}

#[async_trait(?Send)]
pub trait NodeBuilder<N: Node> {
    type Error;
    type Config;

    fn new(config: Self::Config) -> Self;

    fn with_worker<W: Worker<N> + 'static>(self) -> Self
    where
        W::Config: Default;

    fn with_worker_cfg<W: Worker<N> + 'static>(self, config: W::Config) -> Self;

    fn with_resource<R: Any + Send + Sync>(self, res: R) -> Self;

    async fn finish(self) -> Result<N, Self::Error>;
}

static RES_ID: AtomicUsize = AtomicUsize::new(0);

pub struct ResHandle<R> {
    id: Option<usize>,
    inner: Arc<(R, Mutex<HashMap<usize, &'static Location<'static>>>)>,
}

impl<R> ResHandle<R> {
    pub fn new(res: R) -> Self {
        Self {
            id: None,
            inner: Arc::new((res, Mutex::new(HashMap::new()))),
        }
    }

    pub fn into_weak(self) -> WeakHandle<R> {
        let inner = Arc::downgrade(&self.inner);
        drop(self);
        WeakHandle { inner }
    }

    pub fn try_unwrap(self) -> Option<R>
    where
        R: Any,
    {
        let inner = self.inner.clone();
        drop(self);
        match Arc::try_unwrap(inner) {
            Ok((res, _)) => Some(res),
            Err(inner) => {
                let usages = inner
                    .1
                    .lock()
                    .unwrap()
                    .values()
                    .fold(String::new(), |s, loc| format!("{}\n- {}", s, loc));
                warn!(
                    "Attempted to gain ownership resource `{}` but it is still being used in the following places: {}",
                    type_name::<R>(),
                    usages,
                );
                None
            }
        }
    }
}

impl<R> Clone for ResHandle<R> {
    #[track_caller]
    fn clone(&self) -> Self {
        let new_id = RES_ID.fetch_add(1, Ordering::Relaxed);
        self.inner.1.lock().unwrap().insert(new_id, Location::caller());
        Self {
            id: Some(new_id),
            inner: self.inner.clone(),
        }
    }
}

impl<R> Deref for ResHandle<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

impl<R> Drop for ResHandle<R> {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.inner.1.lock().unwrap().remove(&id);
        }
    }
}

pub struct WeakHandle<R> {
    inner: Weak<(R, Mutex<HashMap<usize, &'static Location<'static>>>)>,
}

impl<R> WeakHandle<R> {
    #[track_caller]
    pub fn upgrade(&self) -> Option<ResHandle<R>> {
        let new_id = RES_ID.fetch_add(1, Ordering::Relaxed);
        let inner = self.inner.upgrade()?;
        inner.1.lock().unwrap().insert(new_id, Location::caller());
        Some(ResHandle {
            id: Some(new_id),
            inner,
        })
    }
}

impl<R> Clone for WeakHandle<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
