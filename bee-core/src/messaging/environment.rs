//! Environment module.

use crate::common::Waker;
use crate::common::{Trigger, TriggerHandle};
use crate::constants::BROADCAST_BUFFER_SIZE;
use crate::errors::Error;

use super::Effect;
use super::EntityHost;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bus::Bus as Broadcaster;
use bus::BusReader as BroadcastReceiver;
use crossbeam_channel::Receiver;
use log::*;
use tokio::prelude::*;

/// An environment in the EEE model.
pub struct Environment {
    /// Name of the environment
    name: String,

    /// Entities that joined this environment
    joined_entities: Arc<Mutex<Vec<JoinedEntity>>>,

    /// Entities that affect this environment
    affecting_entities: Arc<Mutex<Vec<AffectingEntity>>>,

    /// Receiver half of the channel to the supervisor
    in_chan: Arc<Receiver<Effect>>,

    /// Sender half of the outgoing broadcast channel to send data to entities.
    out_chan: Arc<Mutex<Broadcaster<Effect>>>,

    /// A notifier that signals the end of this environment to subscribed
    /// entities
    drop_notifier: Arc<Mutex<Trigger>>,

    /// A listener for supervisor shutdown
    shutdown_listener: Arc<Mutex<TriggerHandle>>,

    /// A notifier that allows to wake this environments task/future
    waker: Waker,

    /// The number of received effects.
    num_received_effects: Arc<AtomicUsize>,
}

pub(crate) struct JoinedEntity {
    /// A waker to wake up the entity's task/future
    pub ent_waker: Waker,
}

pub(crate) struct AffectingEntity {
    /// Entity uuid
    pub ent_uuid: String,

    /// Entity effect receiver
    pub ent_rx: BroadcastReceiver<Effect>,

    /// Entity drop signal receiver
    pub ent_drop_rx: TriggerHandle,
}

impl Environment {
    /// Creates a new environment.
    pub(crate) fn new(
        name: &str,
        in_chan: Receiver<Effect>,
        shutdown_listener: TriggerHandle,
    ) -> Self {
        let waker = Waker::new();
        Self {
            name: name.into(),
            joined_entities: shared_mut!(vec![]),
            affecting_entities: shared_mut!(vec![]),
            in_chan: shared!(in_chan),
            out_chan: shared_mut!(Broadcaster::new(BROADCAST_BUFFER_SIZE)),
            drop_notifier: shared_mut!(Trigger::new()),
            shutdown_listener: shared_mut!(shutdown_listener),
            waker,
            num_received_effects: shared!(AtomicUsize::new(0)),
        }
    }

    /// Registers an entity that wants to join this evironment.
    pub(crate) fn register_joining_entity(
        &mut self,
        entity: &mut EntityHost,
    ) -> Result<(), Error> {
        //
        let env_rx = unlock!(self.out_chan).add_rx();
        let env_drop_rx = unlock!(self.drop_notifier).get_handle();

        let ent_waker = entity.join_environment(&self.name, env_rx, env_drop_rx)?;
        let joiner = JoinedEntity { ent_waker };

        unlock!(self.joined_entities).push(joiner);

        Ok(())
    }

    /// Registers and entity that wants to affect this environment.
    pub(crate) fn register_affecting_entity(
        &mut self,
        entity: &mut EntityHost,
    ) -> Result<(), Error> {
        //
        let env_waker = self.waker.clone();

        //
        let affector = entity.affect_environment(&self.name, env_waker)?;
        unlock!(self.affecting_entities).push(affector);

        Ok(())
    }

    /// Inform joined entities that this environment is going to be dropped.
    pub(crate) fn send_sig_term(&self) -> Result<(), Error> {
        unlock!(self.drop_notifier).pull()?;

        debug!("Environment '{}' sent sig_term", self.name);

        Ok(())
    }

    /// Returns the uuid of this entity.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of effects that this entity has received.
    pub fn num_received_effects(&self) -> usize {
        self.num_received_effects.load(Ordering::Relaxed)
        //*unlock!(self.num_received_effects)
    }

    /// Returns a waker that allows to wake this environments task/future.
    pub(crate) fn get_waker(&self) -> Waker {
        self.waker.clone()
    }
}

impl Future for Environment {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        self.waker.task.register();

        // As long as effects can be received go on broadcasting them
        {
            let joined = unlock!(self.joined_entities);
            let mut affecting = unlock!(self.affecting_entities);
            let mut env_tx = unlock!(self.out_chan);

            // TODO: maybe make this a for-loop with some predefined max number
            // of effects to not block other futures from making
            // progress
            let mut num_received = self.num_received_effects.load(Ordering::Acquire);

            let mut num = 0;

            // Forward incoming effects from the supervisor to all subscribed entities
            loop {
                // Try to receive a new effect from the supervisor
                match self.in_chan.try_recv() {
                    Ok(effect) => {
                        num += 1;

                        debug!(
                            "Env. {} received effect '{:?}' from supervisor ({})",
                            self.name,
                            effect,
                            num_received + num
                        );

                        // Broadcast received effect to joined entities
                        env_tx.broadcast(effect);

                        // Wake all joined entities if half of the broadcaster
                        // buffer size is full
                        if num == BROADCAST_BUFFER_SIZE / 2 {
                            for JoinedEntity { ent_waker } in joined.iter() {
                                ent_waker.task.notify();
                            }

                            num_received += num;
                            num = 0;
                        }
                    }
                    _ => break,
                }
            } // end forwarding supervisor effects

            // Wake all joined entities to process the remaining effects buffered in the
            // broadcast channel
            for JoinedEntity { ent_waker } in joined.iter() {
                ent_waker.task.notify();
            }

            num_received += num;
            num = 0;

            //
            for AffectingEntity { ent_uuid, ent_rx, ent_drop_rx: _ } in
                affecting.iter_mut()
            {
                loop {
                    match ent_rx.try_recv() {
                        Ok(effect) => {
                            num += 1;

                            debug!(
                                "Env. {} received effect '{:?}' from entity {} ({})",
                                self.name,
                                effect,
                                &ent_uuid[0..5],
                                num_received + num,
                            );
                        }
                        _ => break,
                    }
                }
            }

            self.num_received_effects.store(num_received + num, Ordering::Release);
        }

        // Check for shutdown signal
        match unlock!(self.shutdown_listener).0.poll() {
            // sig-term received
            Ok(Async::Ready(Some(is_term))) => {
                if is_term {
                    debug!("Env. {} received sig-term", self.name);
                    // End this future
                    return Ok(Async::Ready(()));
                }
            }
            _ => (),
        }

        // otherwise go to sleep
        return Ok(Async::NotReady);
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            joined_entities: Arc::clone(&self.joined_entities),
            affecting_entities: Arc::clone(&self.affecting_entities),
            in_chan: Arc::clone(&self.in_chan),
            out_chan: Arc::clone(&self.out_chan),
            drop_notifier: Arc::clone(&self.drop_notifier),
            shutdown_listener: Arc::clone(&self.shutdown_listener),
            waker: self.waker.clone(),
            num_received_effects: Arc::clone(&self.num_received_effects),
        }
    }
}
