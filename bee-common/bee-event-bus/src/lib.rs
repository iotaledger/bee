// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides a generic, type-safe and thread-safe event bus for arbitrary event types.

#![deny(missing_docs, warnings)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Mutex,
};

type Listener<'a> = dyn Fn(&dyn Any) + Send + Sync + 'a;
type Listeners<'a, T> = HashMap<TypeId, Vec<(Box<Listener<'a>>, T)>>;

/// A unique identifier for [`EventBus`] callbacks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UniqueId<T> {
    /// Identifier for types.
    Type(TypeId),
    /// Identifier for an object.
    Object(T),
}

impl<T> From<TypeId> for UniqueId<T> {
    fn from(id: TypeId) -> Self {
        Self::Type(id)
    }
}

/// An event bus for arbitrary event types.
pub struct EventBus<'a, T = TypeId> {
    listeners: Mutex<Listeners<'a, T>>,
}

impl<'a, T> Default for EventBus<'a, T> {
    fn default() -> Self {
        Self {
            listeners: Mutex::new(HashMap::default()),
        }
    }
}

impl<'a, T: PartialEq> EventBus<'a, T> {
    /// Creates a new [`EventBus`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an event listener bound to a specific event type `E` and registered with the given
    /// identifier.
    pub fn add_listener_with_id<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F, id: T) {
        self.listeners
            .lock()
            // We unwrap() to assert that we are not expecting threads to ever fail while holding the lock.
            .unwrap()
            .entry(TypeId::of::<E>())
            .or_default()
            .push((
                Box::new(move |event| handler(event.downcast_ref().expect("Invalid event"))),
                id,
            ));
    }

    /// Dispatches an event via the event bus. All active listeners registered for this event will be invoked.
    pub fn dispatch<E: Any>(&self, event: E) {
        // We unwrap() to assert that we are not expecting threads to ever fail while holding the lock.
        if let Some(listeners) = self.listeners.lock().unwrap().get(&TypeId::of::<E>()) {
            listeners.iter().for_each(|(listener, _)| listener(&event))
        }
    }

    /// Removes all event listeners registered with the given [`TypeId`].
    pub fn remove_listeners_with_id(&self, id: T) {
        self.listeners
            .lock()
            // We unwrap() to assert that we are not expecting threads to ever fail while holding the lock.
            .unwrap()
            .iter_mut()
            .for_each(|(_, listeners)| listeners.retain(|(_, listener_id)| listener_id != &id));
    }
}

impl<'a, T: From<TypeId> + PartialEq> EventBus<'a, T> {
    /// Adds an event listener bound to a specific event type `E` and registered with the type `V`.
    pub fn add_listener<V: Any, E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        self.add_listener_with_id(handler, TypeId::of::<V>().into());
    }

    /// Adds an event listener bound to a specific event type `E` and registered with a hidden type that will prevent
    /// its removal until the event bus is dropped.
    pub fn add_static_listener<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        struct Static;

        self.add_listener_with_id(handler, TypeId::of::<Static>().into());
    }

    /// Removes all event listeners registered with the type `V`.
    pub fn remove_listeners<V: Any>(&self) {
        self.remove_listeners_with_id(TypeId::of::<V>().into());
    }
}
