// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a generic, type-safe and thread-safe event bus for arbitrary event types.

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

impl<'a> EventBus<'a> {
    /// Creates a new `EventBus`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Dispatch an event via this event bus.
    ///
    /// All active listeners registered for this event will be invoked.
    pub fn dispatch<E: Any>(&self, event: E) {
        if let Some(ls) = self.listeners.lock().unwrap().get_mut(&TypeId::of::<E>()) {
            ls.iter_mut().for_each(|(l, _)| l(&event))
        }
    }

    /// Add an event listener bound to a specific event type, `E`, and registered with the given type ID.
    pub fn add_listener_raw<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, id: TypeId, handler: F) {
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

    /// Add an event listener bound to a specific event type, `E`, and bound to a type `T`.
    ///
    /// This event listener will be removed when [`EventBus::remove_listeners_by_id`] is called with the `TypeId` of
    /// `T`.
    pub fn add_listener<T: Any, E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        self.add_listener_raw(TypeId::of::<T>(), handler);
    }

    /// Add an event listener bound to a specific event type, `E`, registered using a hidden type that will prevent its
    /// removal until the event bus is dropped.
    pub fn add_static_listener<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        struct Static;

        self.add_listener_raw(TypeId::of::<Static>(), handler);
    }

    ///
    pub fn remove_listeners<T: Any>(&self) {
        self.remove_listeners_by_id(TypeId::of::<T>());
    }

    /// Remove all event listeners registered with the given ID, dropping them in the process.
    pub fn remove_listeners_by_id(&self, id: TypeId) {
        self.listeners
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|(_, listeners)| listeners.retain(|(_, listener_id)| listener_id != &id));
    }
}
