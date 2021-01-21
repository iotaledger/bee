// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a generic, type-safe event bus for arbitrary event types.

use dashmap::DashMap;

use std::any::{Any, TypeId};

type Listener<'a> = dyn Fn(&dyn Any) + Send + Sync + 'a;

/// An event bus for arbitrary event types.
pub struct Bus<'a, ID = TypeId> {
    listeners: DashMap<TypeId, Vec<(Box<Listener<'a>>, ID)>>,
}

impl<'a, ID> Default for Bus<'a, ID> {
    fn default() -> Self {
        Self {
            listeners: DashMap::default(),
        }
    }
}

impl<'a, ID: Clone + PartialEq> Bus<'a, ID> {
    /// Dispatch an event via this event bus.
    ///
    /// All active listeners registered for this event will be invoked.
    pub fn dispatch<E: Any>(&self, event: E) {
        if let Some(mut ls) = self.listeners.get_mut(&TypeId::of::<E>()) {
            ls.iter_mut().for_each(|(l, _)| l(&event))
        }
    }

    /// Add an event listener bound to a specific event type, `E`, and registered with the given ID.
    pub fn add_listener_raw<E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, id: ID, handler: F) {
        self.listeners.entry(TypeId::of::<E>()).or_default().push((
            Box::new(move |event| handler(&event.downcast_ref().expect("Invalid event"))),
            id,
        ));
    }

    /// Remove all event listeners registered with the given ID, dropping them in the process.
    pub fn remove_listeners_by_id(&self, id: ID) {
        self.listeners
            .iter_mut()
            .for_each(|mut listeners| listeners.retain(|(_, listener_id)| listener_id != &id));
    }
}

impl<'a> Bus<'a, TypeId> {
    /// Add an event listener bound to a specific event type, `E`, and bound to a type `T`.
    ///
    /// This event listener will be removed when [`Bus::remove_listeners_by_id`] is called with the `TypeId` of `T`.
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
