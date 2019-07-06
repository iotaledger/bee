//! Core functionality

#![deny(bad_style, missing_docs, unsafe_code)]
#![cfg_attr(release, deny(warnings))]

pub mod constants;

use constants::{NAME, VERSION};

use std::fmt;
use std::io::stdout;
use std::time::{Duration, Instant};

use log::info;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::Interval;

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
}

impl Bee {
    /// Create a new Bee node.
    pub fn new() -> Self {
        let state = State::Starting;
        info!("{}", state);

        Self { runtime: Runtime::new().expect("Couldn't create Tokio runtime."), state }
    }

    /// Start the node.
    pub fn init(&mut self) {
        let processing = Interval::new(Instant::now(), Duration::from_millis(250));
        self.runtime.spawn(
            processing
                .for_each(|_| {
                    print!(".");
                    stdout().flush().unwrap();
                    Ok(())
                })
                .map_err(|_| {}),
        );
    }

    /// Run the node.
    pub fn run(mut self) {
        self.state = State::Running;
        info!("{}", self.state);

        self.runtime.shutdown_on_idle().wait().unwrap();

        self.state = State::Stopping;
        info!("{}", self.state);
    }
}

/// Returns a nice greeting.
pub fn get_name() -> String {
    format!("{} v{}", NAME, VERSION)
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