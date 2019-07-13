//! Graceful shutdown

use crate::errors::Error;

use super::trigger::{Trigger, TriggerHandle};

use tokio::prelude::*;
use tokio::runtime::current_thread;
use tokio_signal::ctrl_c;

/// A graceful shutdown abstraction.
pub struct GracefulShutdown {
    trigger: Trigger,
}

impl GracefulShutdown {
    /// Creates a new graceful shutdown mechanism.
    pub fn new() -> Self {
        Self { trigger: Trigger::new() }
    }

    /// Blocks the current thread until CTRL-C is observed
    pub fn wait_for_ctrl_c(&self) {
        // Create a future, that completes when the first CTRL-C is observed
        let ctrl_c = ctrl_c().flatten_stream().take(1).for_each(|_| Ok(()));

        // Block the current thread until the 'ctrl_c' future completes
        current_thread::block_on_all(ctrl_c).expect("error waiting for CTRL-C");
    }

    /// Sends a termination signal to all holders of a handle.
    pub fn send_sig_term(&mut self) -> Result<(), Error> {
        self.trigger.pull()
    }

    /// Returns a shutdown listener.
    pub fn get_listener(&self) -> TriggerHandle {
        self.trigger.get_handle()
    }
}
