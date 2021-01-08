// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types that allow access to and management of node resources.

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

static RESOURCE_ID: AtomicUsize = AtomicUsize::new(0);

/// An owning handle to a node resource.
pub struct ResourceHandle<R> {
    id: Option<usize>,
    inner: Arc<(R, Mutex<HashMap<usize, &'static Location<'static>>>)>,
}

impl<R> ResourceHandle<R> {
    /// Wrap the given resource into a [`ResourceHandle`], providing shared immutable access to it.
    pub fn new(res: R) -> Self {
        Self {
            id: None,
            inner: Arc::new((res, Mutex::new(HashMap::new()))),
        }
    }

    /// Turn this owned resource handle into a weak non-owning handle.
    pub fn into_weak(self) -> WeakHandle<R> {
        let inner = Arc::downgrade(&self.inner);
        drop(self);
        WeakHandle { inner }
    }

    /// Attempt to gain ownership over the resource, returning `None` if the resource is still in use.
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
                    "Attempted to gain ownership resource `{}` but it is still being used. This is not, by itself, a \
                    problem but may indicate that a node task or event listener is not being stopped at the \
                    appropriate time during the shutdown process. Using arcane magic, we determined that the resource \
                    is still being used in the following places: {}",
                    type_name::<R>(),
                    usages,
                );
                None
            }
        }
    }
}

impl<R> Clone for ResourceHandle<R> {
    #[track_caller]
    fn clone(&self) -> Self {
        let new_id = RESOURCE_ID.fetch_add(1, Ordering::Relaxed);
        self.inner.1.lock().unwrap().insert(new_id, Location::caller());
        Self {
            id: Some(new_id),
            inner: self.inner.clone(),
        }
    }
}

impl<R> Deref for ResourceHandle<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

impl<R> Drop for ResourceHandle<R> {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.inner.1.lock().unwrap().remove(&id);
        }
    }
}

/// An non-owning handle to a node resource.
pub struct WeakHandle<R> {
    inner: Weak<(R, Mutex<HashMap<usize, &'static Location<'static>>>)>,
}

impl<R> WeakHandle<R> {
    /// Attempt to turn this owned resource handle into a weak non-owning handle, if it still exists.
    #[track_caller]
    pub fn upgrade(&self) -> Option<ResourceHandle<R>> {
        let new_id = RESOURCE_ID.fetch_add(1, Ordering::Relaxed);
        let inner = self.inner.upgrade()?;
        inner.1.lock().unwrap().insert(new_id, Location::caller());
        Some(ResourceHandle {
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
