//! **bee-e3** or **e3** for short is an asynchronous pub/sub messaging system based on EEE
//! (Entity-Environment-Effect) which is a concept introduced by the IOTA Foundation, and Tokio for
//! IOTAs node framework with the working title **Bee**, that can be used to send messages between
//! so-called **environments** and **entities**. The messages in this system are called **effects**.
//! Environments are passive components that forward effects received from affecting entities to all
//! joined entities. Entities are active components that can create, modify and receive data streams
//! in the form of effects, and affect and/or join one or multiple environments. They are
//! customizable by implementing the `Entity` trait.
//!
//! # Example
//! `e3` needs to be initialized early and shutdown and at the end of your program.
//!
//! ```
//! e3::init();
//! // OTHER CODE
//! e3::shutdown();
//! ```
//! Anywhere in your code base you can add environments and/or entities, and connect them to
//! eachother like so:
//! ```
//! # use e3::prelude::*;
//! struct MyAwesomeEntity;
//!
//! impl Entity for MyAwesomeEntity {
//!     fn process(&mut self, _: Effect, _: Context) -> Effect { Effect::Empty }
//! }
//!
//! # e3::init();
//! e3::create_environment("X");
//! e3::create_environment("Y");
//!
//! let m = e3::register_entity(MyAwesomeEntity);
//!
//! e3::join_environment(&m, "X");
//! e3::affect_environment(&m, "Y");
//!
//! # e3::shutdown();
//! ```

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

#[macro_use]
mod common;
mod constants;
mod effect;
mod entity;
mod environment;
mod supervisor;

use crate::common::shutdown::GracefulShutdown;
use crate::constants::messages::*;
use crate::supervisor::Supervisor;

use std::sync::{Arc, Mutex};

use futures::future::Future;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;

pub use effect::Effect;
pub use entity::{Context, Entity, EntityInfo};
pub use environment::EnvironmentInfo;

/// Export of commonly used types in E3.
pub mod prelude {
    pub use crate::effect::Effect;
    pub use crate::entity::{Context, Entity, EntityInfo};
    pub use crate::environment::EnvironmentInfo;
}

lazy_static! {
    static ref GRACEFUL_SHUTDOWN: Arc<Mutex<Option<GracefulShutdown>>> = share_mut!(None);
    static ref SUPERVISOR: Arc<Mutex<Option<Supervisor>>> = share_mut!(None);
    static ref RUNTIME: Arc<Mutex<Option<Runtime>>> = share_mut!(None);
}

/// Initializes the messaging system.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor, shutdown manager, or the Tokio runtime.
///
/// # Examples
/// Initializes the messaging system.
/// ```
/// e3::init();
/// # e3::shutdown();
/// ```
pub fn init() {
    #[cfg(debug_assertions)]
    println!("initializing e3 messaging system...");

    let graceful_shutdown = GracefulShutdown::new();
    let shutdown_listener = graceful_shutdown.add_rx();
    let supervisor = Supervisor::new(shutdown_listener);
    let mut runtime = Runtime::new().expect(RUNTIME_START_ERROR);

    runtime.spawn(
        supervisor
            .clone()
            .map_err(|_| panic!(SUPERVISOR_RUNTIME_ERROR)),
    );

    unlock!(GRACEFUL_SHUTDOWN).replace(graceful_shutdown);
    unlock!(SUPERVISOR).replace(supervisor);
    unlock!(RUNTIME).replace(runtime);
}

/// Immediatedly shuts down the messaging system.
///
/// # Panics
/// Panics if it can't get a lock to the shutdown manager, or the Tokio runtime.
///
/// # Examples
/// Shuts down the messaging system.
/// ```
/// # e3::init();
/// e3::shutdown();
/// ```
pub fn shutdown() {
    #[cfg(debug_assertions)]
    println!("shutting down e3 messaging system...");

    let mut shutdown = unlock_msg!(GRACEFUL_SHUTDOWN, UNLOCK_SHUTDOWN_HANDLER_ERROR);
    let shutdown = shutdown.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    shutdown.send_termination_signal();

    let runtime = unlock_msg!(RUNTIME, UNLOCK_RUNTIME_ERROR)
        .take()
        .expect(SYSTEM_NOT_INITIALIZED_ERROR);

    runtime
        .shutdown_on_idle()
        .wait()
        .expect(RUNTIME_SHUTDOWN_ERROR);
}

/// Initiates shutdown only after it received termination signal CTRL-C from the console.
///
/// # Panics
/// Panics if it can't get a lock to the shutdown manager, or the Tokio runtime.
pub fn manual_shutdown() {
    #[cfg(debug_assertions)]
    println!("waiting for shutdown signal ...");

    let mut shutdown = unlock_msg!(GRACEFUL_SHUTDOWN, UNLOCK_SHUTDOWN_HANDLER_ERROR);
    let shutdown = shutdown.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    shutdown.wait_for_ctrl_c();

    #[cfg(debug_assertions)]
    println!("Shutting down e3 messaging system...");

    shutdown.send_termination_signal();

    let runtime = unlock_msg!(RUNTIME, UNLOCK_RUNTIME_ERROR)
        .take()
        .expect(SYSTEM_NOT_INITIALIZED_ERROR);

    runtime
        .shutdown_on_idle()
        .wait()
        .expect(RUNTIME_SHUTDOWN_ERROR);
}

/// Creates a new environment with the specified `name`, unless that environment already exists.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor, shutdown manager, or the Tokio runtime.
///
/// # Examples
/// You can create an environment anywhere in the code base as long as the given name isn't used
/// already.
/// ```
/// # e3::init();
/// # assert_eq!(0, e3::num_environments());
/// e3::create_environment("X");
/// # assert_eq!(1, e3::num_environments());
/// # e3::shutdown();
/// ```
/// This function returns the name of the created environment if the environment could successfully
/// be created.
/// ```
/// # e3::init();
/// let y = &e3::create_environment("Y");
/// # assert_eq!("Y", y);
/// # e3::shutdown();
/// ```
pub fn create_environment(name: &str) -> String {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let shutdown = unlock_msg!(GRACEFUL_SHUTDOWN, UNLOCK_SHUTDOWN_HANDLER_ERROR);
    let mut runtime = unlock_msg!(RUNTIME, UNLOCK_RUNTIME_ERROR);

    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);
    let shutdown = shutdown.as_ref().expect(SYSTEM_NOT_INITIALIZED_ERROR);
    let runtime = runtime.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    let environment = supervisor.create_environment(name, shutdown.add_rx());
    let name = String::from(environment.name());

    runtime.spawn(environment.map_err(|_| panic!(ENVIRONMENT_RUNTIME_ERROR)));

    name
}

/// Deletes the environment with the specified `name` if it exists and notifies all joined entities
/// about it.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
///
/// # Examples
/// Creates an environment and immediatedly deletes it again.
/// ```
/// # e3::init();
/// let x = &e3::create_environment("X");
/// assert_eq!(1, e3::num_environments());
///
/// e3::delete_environment(x);
/// assert_eq!(0, e3::num_environments());
/// # e3::shutdown();
/// ```
pub fn delete_environment(name: &str) {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.delete_environment(name);
}

/// Returns the number of environments in the system.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
///
/// # Examples
/// Creates two different environments, and prints the current number of environments in the system.
/// ```
/// # e3::init();
/// # assert_eq!(0, e3::num_environments());
/// e3::create_environment("X");
/// # assert_eq!(1, e3::num_environments());
/// e3::create_environment("Y");
///
/// assert_eq!(2, e3::num_environments());
/// # e3::shutdown();
/// ```
pub fn num_environments() -> usize {
    let supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_ref().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.num_environments()
}

/// Returns whether an environment with the specified `name` exists in the system.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
///
/// # Examples
/// Creates an environment and checks whether it exists. Additionally makes sure that another
/// environment doesn't exist.
/// ```
/// # e3::init();
/// let x = &e3::create_environment("X");
///
/// assert_eq!(true, e3::environment_exists(x));
/// assert_eq!(false, e3::environment_exists("Y"));
/// # e3::shutdown();
/// ```
pub fn environment_exists(name: &str) -> bool {
    let supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_ref().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.environment_exists(name)
}

/// Registers an entity with the supervisor and returns its ID.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor, shutdown manager, or the Tokio runtime.
///
/// # Examples
/// Defines an entity and registers it with the e3 messaging system.
/// ```
/// # use e3::prelude::*;
///
/// struct EmptyEntity;
/// impl Entity for EmptyEntity {
///     fn process(&mut self, _: Effect, _: Context) -> Effect { Effect::Empty }
/// }
/// # e3::init();
/// # assert_eq!(0, e3::num_entities());
///
/// let a = &e3::register_entity(EmptyEntity);
/// assert_eq!(1, e3::num_entities());
///
/// # e3::shutdown();
/// ```
pub fn register_entity(entity: impl Entity + 'static) -> String {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let shutdown = unlock_msg!(GRACEFUL_SHUTDOWN, UNLOCK_SHUTDOWN_HANDLER_ERROR);
    let mut runtime = unlock_msg!(RUNTIME, UNLOCK_RUNTIME_ERROR);

    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);
    let shutdown = shutdown.as_ref().expect(SYSTEM_NOT_INITIALIZED_ERROR);
    let runtime = runtime.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    let entity = supervisor.register_entity(entity, shutdown.add_rx());
    let id = String::from(entity.id());

    runtime.spawn(entity.map_err(|_| panic!(ENTITY_RUNTIME_ERROR)));

    id
}

/// Deregisters entity with the specified `id` if it exists in the system.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
///
/// # Examples
/// Defines an entity and registers it with the e3 messaging system, then immediatedly deregisters
/// it.
/// ```
/// # use e3::prelude::*;
/// #
/// struct EmptyEntity;
/// impl Entity for EmptyEntity {
///     fn process(&mut self, _: Effect, _: Context) -> Effect { Effect::Empty }
/// }
/// # e3::init();
///
/// # let a = &e3::register_entity(EmptyEntity);
/// # assert_eq!(1, e3::num_entities());
///
/// e3::deregister_entity(a);
/// assert_eq!(0, e3::num_entities());
///
/// # e3::shutdown();
/// ```
pub fn deregister_entity(id: &str) {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.deregister_entity(id);
}

/// Returns the number of entities in the system.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
pub fn num_entities() -> usize {
    let supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_ref().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.num_entities()
}

/// Returns whether an entity exists in the system.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
///
/// # Examples
/// Tests for two entities whether they exist or not.
/// ```
/// # use e3::prelude::*;
/// #
/// struct EmptyEntity;
/// impl Entity for EmptyEntity {
///     fn process(&mut self, _: Effect, _: Context) -> Effect { Effect::Empty }
/// }
/// # e3::init();
/// let a = &e3::register_entity(EmptyEntity);
///
/// assert!(e3::entity_exists(a));
/// assert!(!e3::entity_exists("abcdefghijklmnopqrstuvwxyz"));
/// ```
pub fn entity_exists(id: &str) -> bool {
    let supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_ref().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.entity_exists(id)
}

/// Lets the specified entity affect the specified environment.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
///
/// # Examples
/// Tests for two entities whether they exist or not.
/// ```
/// # use e3::prelude::*;
/// #
/// struct EmptyEntity;
/// impl Entity for EmptyEntity {
///     fn process(&mut self, _: Effect, _: Context) -> Effect { Effect::Empty }
/// }
///
/// # e3::init();
/// let x = &e3::create_environment("X");
/// let a = &e3::register_entity(EmptyEntity);
///
/// assert!(e3::affect_environment(a, x));
/// assert!(!e3::affect_environment(a, "Y"));
/// ```
pub fn affect_environment(entity_id: &str, environment_name: &str) -> bool {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.affect_environment(entity_id, environment_name)
}

/// Lets the specified entity ignore the specified environment. This is only useful for entities
/// that were previously affecting that environment.
///
/// # Panics
/// Panics if it can't get a lock to the supervisor.
///
/// # Examples
pub fn ignore_environment(entity_id: &str, environment_name: &str) -> bool {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.ignore_environment(entity_id, environment_name)
}

/// Lets the specified entity join the specified environment.
pub fn join_environment(entity_id: &str, environment_name: &str) -> bool {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.join_environment(entity_id, environment_name)
}

/// Lets the specified entity leave the specified environment. This is only useful for entities that
/// had previously joined that environment.
pub fn leave_environment(entity_id: &str, environment_name: &str) -> bool {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.leave_environment(entity_id, environment_name)
}

/// Sends an effect to the specified environment.
pub fn send_effect(effect: Effect, environment_name: &str) -> bool {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.send_effect(effect, environment_name)
}

/// Returns information about an environment.
pub fn get_environment_info(environment_name: &str) -> Option<EnvironmentInfo> {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.get_environment_info(environment_name)
}

/// Returns information about an entity.
pub fn get_entity_info(entity_id: &str) -> Option<EntityInfo> {
    let mut supervisor = unlock_msg!(SUPERVISOR, UNLOCK_SUPERVISOR_ERROR);
    let supervisor = supervisor.as_mut().expect(SYSTEM_NOT_INITIALIZED_ERROR);

    supervisor.get_entity_info(entity_id)
}
