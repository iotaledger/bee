//! Bee module.

use crate::common::shutdown::GracefulShutdown;
use crate::errors::Result;
use crate::messaging::Effect;
use crate::messaging::Environment;
use crate::messaging::Supervisor;
use crate::messaging::{Entity, EntityHost};

use std::fmt;

use log::*;
use stream_cancel::{StreamExt, Trigger, Tripwire};
use tokio::prelude::*;
use tokio::runtime::Runtime;

enum State {
    Starting,
    Started,
    Stopping,
    Stopped,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let log_text = match *self {
            State::Starting => "Starting...",
            State::Started => "Started.",
            State::Stopping => "Stopping...",
            State::Stopped => "Stopped.",
        };
        write!(f, "{}", log_text)
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
    /// The Tokio runtime for this node.
    runtime: Runtime,

    /// The current node state.
    state: State,

    /// The supervisor used for messaging.
    supervisor: Supervisor,

    /// Graceful shutdown of the supervisor, environments, and entities.
    supervisor_shutdown: GracefulShutdown,

    /// Graceful shutdown of async streams.
    stream_shutdown: (Trigger, Tripwire),
}

impl Bee {
    /// Creates a new node.
    pub fn new() -> Result<Self> {
        let state = State::Starting;
        let supervisor_shutdown = GracefulShutdown::new();
        let sd_handle = supervisor_shutdown.get_listener();

        info!("{}", state);

        Ok(Self {
            runtime: Runtime::new()?,
            state,
            supervisor: Supervisor::new(sd_handle)?,
            stream_shutdown: Tripwire::new(),
            supervisor_shutdown,
        })
    }

    /// Initializes the node.
    pub fn init(&mut self) {
        // Spawn the Supervisor onto the runtime
        self.runtime.spawn(self.supervisor.clone().map_err(|_| ()));
    }

    /// Spawns a stream on Bee's runtime.
    pub fn spawn(
        &mut self,
        fut: impl Stream<Item = (), Error = String> + Send + 'static,
    ) {
        let processing = fut
            .take_until(self.stream_shutdown.1.clone())
            .for_each(|_| Ok(()))
            .map_err(|e| panic!("error: {}", e));

        self.runtime.spawn(processing);
    }

    /// Joins an entity to one or more environments.
    pub fn join(&mut self, environments: &[&str], entity: impl Entity + 'static) {
        let mut ent = self.create_entity().expect("error creating entity");

        ent.inject_core(Box::new(entity));

        for env in environments {
            self.create_environment(env).expect("error creating environment");
        }
        self.supervisor
            .join_environments(&mut ent, environments)
            .expect("error joining environments");
    }

    /// Run the node.
    pub fn run(mut self) -> Result<()> {
        self.state = State::Started;
        info!("{}", self.state);

        self.supervisor_shutdown.wait_for_ctrl_c();
        println!();

        self.state = State::Stopping;
        info!("{}", self.state);

        // Send thke signal to make all infinite futures return
        // Ok(Async::Ready(None))
        self.supervisor_shutdown.send_sig_term()?;

        // Stop all threads/tasks
        drop(self.stream_shutdown.0);

        // Block until all futures have finished terminating
        self.runtime
            .shutdown_on_idle()
            .wait()
            .expect("error waiting for async tasks to complete");

        self.state = State::Stopped;
        info!("{}", self.state);

        Ok(())
    }

    /// Creates an environment.
    fn create_environment(&mut self, name: &str) -> Result<Environment> {
        let sd_handle = self.supervisor_shutdown.get_listener();
        let env = self.supervisor.create_environment(name, sd_handle)?;

        // Spawn the Environment future onto the Tokio runtime
        self.runtime.spawn(env.clone().map_err(|_| ()));

        Ok(env)
    }

    /// Creates an entity.
    fn create_entity(&mut self) -> Result<EntityHost> {
        let sd_handle = self.supervisor_shutdown.get_listener();
        let ent = self.supervisor.create_entity(sd_handle)?;

        // Spawn the Entity future onto the Tokio runtime
        self.runtime.spawn(ent.clone().map_err(|_| ()));

        Ok(ent)
    }

    /// Let an entity join a single or multiple environments.
    fn join_environments(
        &mut self,
        entity: &mut EntityHost,
        environments: &[&str],
    ) -> Result<()> {
        self.supervisor.join_environments(entity, environments)
    }

    /// Let an entity affect a single or multiple environments.
    pub fn affect_environments(
        &mut self,
        entity: &mut EntityHost,
        environments: Vec<&str>,
    ) -> Result<()> {
        self.supervisor.affect_environments(entity, environments)
    }

    /// Submit an effect
    pub fn submit_effect(&mut self, effect: Effect, env_name: &str) -> Result<()> {
        self.supervisor.submit_effect(effect, env_name)
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
