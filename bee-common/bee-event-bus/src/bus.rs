// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Event, EventId};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Mutex,
};

type Listener<'a> = dyn Fn(&dyn Any) + Send + Sync + 'a;
type Listeners<'a> = HashMap<EventId, Vec<(Box<Listener<'a>>, TypeId)>>;

/// A generic, type-safe and thread-safe event bus for event types.
pub struct EventBus<'a> {
    listeners: Mutex<Listeners<'a>>,
}

impl<'a> Default for EventBus<'a> {
    fn default() -> Self {
        Self {
            listeners: Mutex::new(HashMap::default()),
        }
    }
}

impl<'a> EventBus<'a> {
    /// Creates a new `EventBus`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an event listener bound to a specific event type `E` and registered with the given `TypeId`.
    pub fn add_listener_with_id<E: Event, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F, id: TypeId) {
        self.listeners
            .lock()
            // We unwrap() to assert that we are not expecting threads to ever fail while holding the lock.
            .unwrap()
            .entry(E::id())
            .or_default()
            .push((
                Box::new(move |event| handler(&event.downcast_ref().expect("Invalid event"))),
                id,
            ));
    }

    /// Adds an event listener bound to a specific event type `E` and registered with the type `T`.
    pub fn add_listener<T: Any, E: Event, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        self.add_listener_with_id(handler, TypeId::of::<T>());
    }

    /// Adds an event listener bound to a specific event type `E` and registered with a hidden type that will prevent
    /// its removal until the event bus is dropped.
    pub fn add_static_listener<E: Event, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        struct Static;

        self.add_listener_with_id(handler, TypeId::of::<Static>());
    }

    /// Dispatches an event via the event bus. All active listeners registered for this event will be invoked.
    pub fn dispatch<E: Event>(&self, event: E) {
        // We unwrap() to assert that we are not expecting threads to ever fail while holding the lock.
        if let Some(listeners) = self.listeners.lock().unwrap().get_mut(&E::id()) {
            listeners.iter_mut().for_each(|(listener, _)| listener(&event))
        }
    }

    /// Removes all event listeners registered with the given `TypeId`.
    pub fn remove_listeners_with_id(&self, id: TypeId) {
        self.listeners
            .lock()
            // We unwrap() to assert that we are not expecting threads to ever fail while holding the lock.
            .unwrap()
            .iter_mut()
            .for_each(|(_, listeners)| listeners.retain(|(_, listener_id)| listener_id != &id));
    }

    /// Removes all event listeners registered with the type `T`.
    pub fn remove_listeners<T: Any>(&self) {
        self.remove_listeners_with_id(TypeId::of::<T>());
    }
}
