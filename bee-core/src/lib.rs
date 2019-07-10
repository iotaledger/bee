//! Core functionality

#![deny(bad_style, missing_docs, unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

pub mod constants;

use std::fmt;
use std::io::stdout;
use std::time::{Duration, Instant};

use log::info;
use stream_cancel::{StreamExt, Trigger, Tripwire};
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::Interval;
use tokio::runtime::current_thread;

enum State {
    Starting,
    Running,
    Stopping,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            State::Starting => write!(f, "Starting..."),
            State::Running => write!(f, "Running..."),
            State::Stopping => write!(f, "Stopping..."),
        }
    }
}

/// The Bee node.
///
/// # Example
/// ```
/// use bee_core::Bee;
///
/// let mut bee = Bee::new();
/// bee.init();
/// ```
pub struct Bee {
    runtime: Runtime,
    state: State,
    shutdown: (Trigger, Tripwire),
}

impl Bee {
    /// Create a new Bee node.
    pub fn new() -> Self {
        let state = State::Starting;
        info!("{}", state);

        Self {
            runtime: Runtime::new().expect("Couldn't create Tokio runtime."),
            state,
            shutdown: Tripwire::new(),
        }
    }

    /// Start the node.
    pub fn init(&mut self) {
        let processing = Interval::new(Instant::now(), Duration::from_millis(250))
                .take_until(self.shutdown.1.clone())
                .for_each(|_| {
                    print!("");
                    stdout().flush().unwrap();
                    Ok(())
                })
                .map_err(|e| panic!("error: {}", e));
        
        self.runtime.spawn(processing);
    }

    /// Run the node.
    pub fn run(mut self) {
        self.state = State::Running;
        info!("{}", self.state);

        // Block the current thread until CTRL-C is detected
        current_thread::block_on_all(tokio_signal::ctrl_c()
            .flatten_stream()
            .take(1)
            .for_each(|_| Ok(()))).expect("error waiting for CTRL-C");

        self.state = State::Stopping;
        info!("{}", self.state);

        // Stop all threads/tasks
        drop(self.shutdown.0);

        // Block until all futures have finished terminating
        self.runtime.shutdown_on_idle().wait().unwrap();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_core_init() {
        let mut bee = Bee::new();

        bee.init();
    }
}
