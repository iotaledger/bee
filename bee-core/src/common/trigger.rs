//! Signaling trigger events across asynchronous tasks.

use crate::errors::Error;

use tokio::sync::watch::{self, Receiver, Sender};

/// A wrapper for the receiving half of a broadcasting channel used to signal a certain
/// event (e.g. a shutdown)
pub struct TriggerHandle(pub Receiver<bool>);

/// A trigger mechanism to signal a certain event.
pub struct Trigger {
    trigger: Sender<bool>,
    handle: Receiver<bool>,
}

impl Trigger {
    /// Creates a new `Trigger`.
    pub fn new() -> Self {
        let (trigger, handle) = watch::channel(false);
        Self { trigger, handle }
    }

    /// Returns a new `TriggerHandle` that listens for the trigger to pulled.
    pub fn get_handle(&self) -> TriggerHandle {
        TriggerHandle(self.handle.clone())
    }

    /// Pulls the trigger which results in all holders of a `TriggerHandle` to get
    /// notified.
    pub fn pull(&mut self) -> Result<(), Error> {
        Ok(self.trigger.broadcast(true)?)
    }
}
