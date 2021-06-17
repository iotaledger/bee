// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that provides a generic, type-safe and thread-safe event bus for arbitrary event types.

#![warn(missing_docs)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Mutex,
};

type Listener<'a> = dyn Fn(&dyn Any) + Send + Sync + 'a;
type Listeners<'a> = HashMap<TypeId, Vec<(Box<Listener<'a>>, TypeId)>>;

/// An event bus for arbitrary event types.
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

// TODO unwrap

impl<'a> EventBus<'a> {
    /// Creates a new `EventBus`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an event listener bound to a specific event type `E` and registered with the given `TypeId`.
    pub fn add_listener_with_id<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F, id: TypeId) {
        self.listeners
            .lock()
            .unwrap()
            .entry(TypeId::of::<E>())
            .or_default()
            .push((
                Box::new(move |event| handler(&event.downcast_ref().expect("Invalid event"))),
                id,
            ));
    }

    /// Adds an event listener bound to a specific event type `E` and registered with the type `T`.
    pub fn add_listener<T: Any, E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        self.add_listener_with_id(handler, TypeId::of::<T>());
    }

    /// Adds an event listener bound to a specific event type `E` and registered with a hidden type that will prevent
    /// its removal until the event bus is dropped.
    pub fn add_static_listener<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        struct Static;

        self.add_listener_with_id(handler, TypeId::of::<Static>());
    }

    /// Dispatches an event via the event bus. All active listeners registered for this event will be invoked.
    pub fn dispatch<E: Any>(&self, event: E) {
        if let Some(listeners) = self.listeners.lock().unwrap().get_mut(&TypeId::of::<E>()) {
            listeners.iter_mut().for_each(|(listener, _)| listener(&event))
        }
    }

    /// Removes all event listeners registered with the given `TypeId`.
    pub fn remove_listeners_with_id(&self, id: TypeId) {
        self.listeners
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|(_, listeners)| listeners.retain(|(_, listener_id)| listener_id != &id));
    }

    /// Removes all event listeners registered with the type `T`.
    pub fn remove_listeners<T: Any>(&self) {
        self.remove_listeners_with_id(TypeId::of::<T>());
    }
}
