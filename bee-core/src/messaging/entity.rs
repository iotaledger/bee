//! Entity

use super::effect::Effect;
use super::environment::AffectingEntity;

use crate::common::Waker;
use crate::common::{Trigger, TriggerHandle};
use crate::constants::BROADCAST_BUFFER_SIZE;
use crate::errors::{Error, Result};

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bus::Bus as Broadcaster;
use bus::BusReader as BroadcastReceiver;
use log::*;
use tokio::{io, prelude::*};
use uuid::Uuid;

/// Processes effects.
pub trait Entity: Send {
    ///
    fn process_effect(&mut self, effect: Effect, environment: &str) -> Effect;
}

type Name = String;

/// An entity in the EEE model.
pub struct EntityHost {
    /// A unique identifier of this entity.
    uuid: String,

    /// The environments this entity has joined.
    joined_environments: Arc<Mutex<HashMap<Name, JoinedEnvironment>>>,

    /// The environments this entity affects.
    affected_environments: Arc<Mutex<HashMap<Name, AffectedEnvironment>>>,

    /// Sender half of the outgoing broadcast channel for affecting
    /// environments.
    out_chan: Arc<Mutex<Broadcaster<Effect>>>,

    /// A notifier that signals the end of this entity to affected environments
    drop_notifier: Arc<Mutex<Trigger>>,

    /// A handle to signal supervisor shutdown
    shutdown_listener: Arc<Mutex<TriggerHandle>>,

    /// A waker to wake up this entity's task/future
    waker: Waker,

    /// The number of received effects.
    num_received_effects: Arc<AtomicUsize>,

    /// The entity core
    entity: Arc<Mutex<Option<Box<dyn Entity>>>>,
}

struct JoinedEnvironment {
    /// Environment effect receiver
    pub env_rx: BroadcastReceiver<Effect>,

    /// Environment drop signal receiver
    pub env_drop_rx: TriggerHandle,
}

struct AffectedEnvironment {
    /// A waker to wake the affected environment's task/future
    pub env_waker: Waker,
}

impl EntityHost {
    /// Creates a new entity.
    pub(crate) fn new(shutdown_listener: TriggerHandle) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            joined_environments: shared_mut!(HashMap::new()),
            affected_environments: shared_mut!(HashMap::new()),
            out_chan: shared_mut!(Broadcaster::new(BROADCAST_BUFFER_SIZE)),
            drop_notifier: shared_mut!(Trigger::new()),
            shutdown_listener: shared_mut!(shutdown_listener),
            waker: Waker::new(),
            num_received_effects: shared!(AtomicUsize::new(0)),
            entity: shared_mut!(None),
        }
    }

    /// Injects an entity.
    pub fn inject_core(&mut self, entity: Box<dyn Entity>) {
        let mut core = unlock!(self.entity);
        core.replace(entity);
    }

    /// Registers an environment as joined by this entity.
    pub(crate) fn join_environment(
        &mut self,
        env_name: &str,
        env_rx: BroadcastReceiver<Effect>,
        env_drop_rx: TriggerHandle,
    ) -> Result<Waker> {
        //
        let mut joined = unlock!(self.joined_environments);

        if joined.contains_key(env_name) {
            return Err(Error::Node("This entity already joined that environment"));
        }

        // Store the name and an environment listener
        joined.insert(env_name.into(), JoinedEnvironment { env_rx, env_drop_rx });

        Ok(self.waker.clone())
    }

    /// Registers an environment as affected by this entity.
    pub(crate) fn affect_environment(
        &mut self,
        env_name: &str,
        env_waker: Waker,
    ) -> Result<AffectingEntity> {
        //
        let mut affected = unlock!(self.affected_environments);

        if affected.contains_key(env_name.into()) {
            return Err(Error::Node("This entity already affects that environment"));
        }
        // Store the name and the receiver handle of that environment
        affected.insert(env_name.into(), AffectedEnvironment { env_waker });

        let ent_uuid = self.uuid.clone();
        let ent_rx = unlock!(self.out_chan).add_rx();
        let ent_drop_rx = unlock!(self.drop_notifier).get_handle();

        Ok(AffectingEntity { ent_uuid, ent_rx, ent_drop_rx })
    }

    /// Notify affected environments, that this entity will be dropped.
    pub(crate) fn send_sig_term(&self) -> Result<()> {
        unlock!(self.drop_notifier).pull()?;
        debug!("Entity '{}' sent sig_term", self.uuid);
        Ok(())
    }

    /// Returns the uuid of this entity.
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Returns a list of all environments this entity has joined.
    pub fn joined_environments(&self) -> Vec<String> {
        unlock!(self.joined_environments)
            .keys()
            .map(|key| key.to_string())
            .collect::<Vec<String>>()
    }

    /// Returns a list of all environments this entity is affecting.
    pub fn affected_environments(&self) -> Vec<String> {
        unlock!(self.affected_environments)
            .keys()
            .map(|key| key.to_string())
            .collect::<Vec<String>>()
    }

    /// Returns true, if this entity has joined the specified environment,
    /// otherwise false.
    pub fn has_joined(&self, env_name: &str) -> bool {
        unlock!(self.joined_environments).contains_key(env_name)
    }

    /// Returns true, if this entity has joined the specified environment,
    /// otherwise false.
    pub fn is_affecting(&self, env_name: &str) -> bool {
        unlock!(self.affected_environments).contains_key(env_name)
    }

    /// Returns the number of joined environments.
    pub fn num_joined(&self) -> usize {
        unlock!(self.joined_environments).len()
    }

    /// Returns the number of affected environments.
    pub fn num_affected(&self) -> usize {
        unlock!(self.affected_environments).len()
    }

    /// Returns the number of effects that this entity has received.
    pub fn num_received_effects(&self) -> usize {
        self.num_received_effects.load(Ordering::Relaxed)
    }
}

impl Future for EntityHost {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        self.waker.task.register();

        // this scope will modify 'joined_environments'
        {
            let num_effects = self.num_received_effects.load(Ordering::Acquire);
            let mut num = 0;

            let mut joined = unlock!(self.joined_environments);
            let affected = unlock!(self.affected_environments);
            let mut core = unlock!(self.entity);

            let mut out_chan = unlock!(self.out_chan);
            let mut to_drop = vec![];

            'outer: loop {
                // number of dry in-channels
                let mut num_dry = 0;

                // Check each joined environment if there is a new effect
                for (env, JoinedEnvironment { env_rx, env_drop_rx: _ }) in
                    joined.iter_mut()
                {
                    // Try to receive as many effects as possible from that
                    // environment TODO: maybe make this a
                    // for-loop with an upper limit to give other
                    // futures time to progress as well
                    'inner: loop {
                        match env_rx.try_recv() {
                            Ok(effect) => {
                                num += 1;

                                debug!(
                                    "Ent. {} received effect '{:?}' from environment {} ({})",
                                    &self.uuid[0..5],
                                    effect,
                                    env,
                                    num_effects + num,
                                );

                                // Process the effect data
                                let effect = match core
                                    .as_mut()
                                    .map(|core| core.process_effect(effect, &env))
                                {
                                    Some(effect) => effect,
                                    None => Effect::Empty,
                                };

                                // Broadcast result to affected environments
                                out_chan.broadcast(effect);

                                // Wake all affected environments if half of the
                                // broadcaster buffer size is full
                                if num == BROADCAST_BUFFER_SIZE / 2 {
                                    for (_, AffectedEnvironment { env_waker }) in
                                        affected.iter()
                                    {
                                        env_waker.task.notify();
                                    }
                                }
                            }
                            _ => {
                                num_dry += 1;
                                break 'inner;
                            }
                        }
                    }
                }

                // If all channels are dry this future can finally go to sleep
                // until awakened again
                if num_dry >= joined.len() {
                    break 'outer;
                }
            }

            self.num_received_effects.store(num_effects + num, Ordering::Release);

            // Wake all affected environments to process the remaining effects buffered in
            // the broadcast channel
            for (_, AffectedEnvironment { env_waker }) in affected.iter() {
                env_waker.task.notify();
            }

            // Check if any environment sent a sig-term
            for (env, JoinedEnvironment { env_rx: _, env_drop_rx }) in joined.iter_mut() {
                match env_drop_rx.0.poll() {
                    Ok(Async::Ready(Some(is_term))) => {
                        if is_term {
                            debug!(
                                "Ent. {} received sig-term from environment '{}'",
                                &self.uuid[0..5],
                                env
                            );

                            // Remember to unsubscribe from that environment
                            to_drop.push(env.clone());
                        }
                    }
                    _ => (),
                }
            }

            // Remove all environments we received a term signal from
            for env in to_drop {
                joined.remove(&env);
                debug!(
                    "Ent. {} unsubscribed from environment '{}'",
                    &self.uuid[0..5],
                    env
                );
            }
        } // we're finished with mutating 'joined_environments'

        // Check if the supervisor is about to shutdown
        match unlock!(self.shutdown_listener).0.poll() {
            // sig-term received
            // NOTE: the 'watch' channel always yields Some!!
            Ok(Async::Ready(Some(is_term))) => {
                if is_term {
                    debug!("Ent. {} received sig-term", &self.uuid[0..5]);
                    // End this future
                    return Ok(Async::Ready(()));
                }
            }
            _ => (),
        }

        // Entity goes to sleep
        Ok(Async::NotReady)
    }
}

impl Clone for EntityHost {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid.clone(),
            joined_environments: Arc::clone(&self.joined_environments),
            affected_environments: Arc::clone(&self.affected_environments),
            out_chan: Arc::clone(&self.out_chan),
            drop_notifier: Arc::clone(&self.drop_notifier),
            shutdown_listener: Arc::clone(&self.shutdown_listener),
            waker: self.waker.clone(),
            num_received_effects: Arc::clone(&self.num_received_effects),
            entity: Arc::clone(&self.entity),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::trigger::Trigger;

    #[test]
    fn each_entity_has_uuid() {
        let shutdown_listener = Trigger::new().get_handle();

        let entity = EntityHost::new(shutdown_listener);

        assert!(!entity.uuid().is_empty())
    }
}
