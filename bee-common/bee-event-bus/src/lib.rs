// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a generic, type-safe and thread-safe event bus for arbitrary event types.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Mutex,
};

type Listener<'a> = dyn Fn(&dyn Any) + Send + Sync + 'a;
type Listeners<'a, ID> = HashMap<TypeId, Vec<(Box<Listener<'a>>, ID)>>;

/// An event bus for arbitrary event types.
pub struct EventBus<'a, ID = TypeId> {
    listeners: Mutex<Listeners<'a, ID>>,
}

impl<'a, ID> Default for EventBus<'a, ID> {
    fn default() -> Self {
        Self {
            listeners: Mutex::new(HashMap::default()),
        }
    }
}

impl<'a, ID: Clone + PartialEq> EventBus<'a, ID> {
    /// Dispatch an event via this event bus.
    ///
    /// All active listeners registered for this event will be invoked.
    pub fn dispatch<E: Any>(&self, event: E) {
        if let Some(ls) = self.listeners.lock().unwrap().get_mut(&TypeId::of::<E>()) {
            ls.iter_mut().for_each(|(l, _)| l(&event))
        }
    }

    /// Add an event listener bound to a specific event type, `E`, and registered with the given ID.
    pub fn add_listener_raw<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, id: ID, handler: F) {
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

    /// Remove all event listeners registered with the given ID, dropping them in the process.
    pub fn remove_listeners_by_id(&self, id: ID) {
        self.listeners
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|(_, listeners)| listeners.retain(|(_, listener_id)| listener_id != &id));
    }
}

impl<'a> EventBus<'a, TypeId> {
    /// Add an event listener bound to a specific event type, `E`, and bound to a type `T`.
    ///
    /// This event listener will be removed when [`EventBus::remove_listeners_by_id`] is called with the `TypeId` of `T`.
    pub fn add_listener<T: Any, E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        self.add_listener_raw(TypeId::of::<T>(), handler);
    }

    /// Add an event listener bound to a specific event type, `E`, registered using a hidden type that will prevent its
    /// removal until the event bus is dropped.
    pub fn add_static_listener<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        struct Static;
        self.add_listener_raw(TypeId::of::<Static>(), handler);
    }
}
